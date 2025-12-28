use age::{DecryptError, EncryptError, Identity, Recipient};
use age_core::format::{FileKey, Stanza};
use base64::prelude::{Engine, BASE64_STANDARD_NO_PAD};
use bech32::{primitives::decode::UncheckedHrpstring, Bech32, ByteIterExt, Checksum, Fe32IterExt};
use bip39::{rand::RngCore, rand_core::OsRng};
use hpke_rs::{hpke_types, libcrux::HpkeLibcrux, Hpke, HpkePrivateKey, HpkePublicKey};
use libcrux_ml_kem::mlkem1024::{self as mlkem};
use secrecy::{zeroize::Zeroize, ExposeSecret, SecretBox, SecretString};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use std::{array::TryFromSliceError, collections::HashSet, str::FromStr};
use x25519_dalek::X25519_BASEPOINT_BYTES;

const RECIPIENT_TAG: &str = "mlkem768x25519";
const KEM_N_SEED: usize = 64;
const GROUP_N_SEED: usize = 32;

pub fn shake256<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut hasher = sha3::Shake256::default();
    hasher.update(input);
    let mut reader = hasher.finalize_xof();
    let mut output = [0u8; N];
    reader.read(&mut output);
    output
}

pub struct HybridRecipient {
    pub encapsulation_key: [u8; 1600],
}

impl Recipient for HybridRecipient {
    fn wrap_file_key(
        &self,
        file_key: &FileKey,
    ) -> Result<(Vec<Stanza>, HashSet<String>), EncryptError> {
        let mut hpke = Hpke::<HpkeLibcrux>::new(
            hpke_rs::Mode::Base,
            hpke_types::KemAlgorithm::XWingDraft06,
            hpke_types::KdfAlgorithm::HkdfSha256,
            hpke_types::AeadAlgorithm::ChaCha20Poly1305,
        );

        let (wrapped_key, ciphertext) = hpke
            .seal(
                &HpkePublicKey::from(&self.encapsulation_key[..]),
                b"age-encryption.org/mlkem768x25519",
                &[],
                file_key.expose_secret(),
                None,
                None,
                None,
            )
            .expect("failed to wrap file key");

        let mut labels = HashSet::new();
        labels.insert("postquantum".to_string());

        Ok((
            vec![Stanza {
                tag: "".to_string(),
                args: vec![
                    RECIPIENT_TAG.to_string(),
                    BASE64_STANDARD_NO_PAD.encode(&wrapped_key.as_slice()),
                ],
                body: ciphertext,
            }],
            labels,
        ))
    }
}

impl HybridRecipient {
    pub fn to_string(&self) -> String {
        let hrp = bech32::Hrp::parse_unchecked("age1pq");
        let encoded_chars = self
            .encapsulation_key
            .iter()
            .copied()
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars();

        encoded_chars.collect()
    }

    pub fn from_string(string: &String) -> Result<Self, String> {
        if !string.starts_with(&"age1pq".to_string()) {
            return Err("not a valid recipient".to_string());
        }

        let decoded = UncheckedHrpstring::new(string.as_str()).map_err(|e| e.to_string())?;

        if decoded.data_part_ascii().len() < Bech32::CODE_LENGTH {
            return Err("failed to parse string as Bech32".to_string());
        }

        {
            // this is simply an implementation of UncheckedHrpString::validate_checksum
            let mut checksum_eng = bech32::primitives::checksum::Engine::<Bech32>::new();
            checksum_eng.input_hrp(decoded.hrp());

            for fe in decoded
                .data_part_ascii()
                .iter()
                .map(|&b| bech32::Fe32::from_char_unchecked(b))
            {
                checksum_eng.input_fe(fe);
            }

            if checksum_eng.residue() != &Bech32::TARGET_RESIDUE {
                return Err("failed to decode as Bech32".to_string());
            }
        }

        // strip checksum from data
        let decoded =
            &decoded.data_part_ascii()[..decoded.data_part_ascii().len() - Bech32::CHECKSUM_LENGTH];
        let decoded_bytes = decoded
            .iter()
            .map(|char| bech32::Fe32::from_char_unchecked(*char))
            .fes_to_bytes()
            .collect::<Vec<u8>>();

        match decoded_bytes.try_into() {
            Ok(ek) => Ok(Self {
                encapsulation_key: ek,
            }),
            Err(_) => return Err("failed to convert decoded byte array to slice".to_string()),
        }
    }
}

pub struct HybridIdentity {
    seed: SecretBox<[u8; 32]>,
}

impl HybridIdentity {
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng::default()
            .try_fill_bytes(&mut seed)
            .expect("failed to generate random seed");
        let identity = Self {
            seed: SecretBox::new(Box::new(seed)),
        };
        seed.zeroize();
        identity
    }

    fn expand_key(
        seed: &[u8; 32],
    ) -> (
        [u8; 1568],
        [u8; 32],
        SecretBox<[u8; 3168]>,
        SecretBox<[u8; 32]>,
    ) {
        let seed_full = shake256::<{ KEM_N_SEED + GROUP_N_SEED }>(seed); // KEM.Nseed + Group.Nseed

        // split the seed into its constituent parts: the ml-kem768 (pq) seed, and x25519 (t) seed
        let seed_pq: [u8; KEM_N_SEED] = seed_full[0..KEM_N_SEED].try_into().expect("wrong length");
        let seed_t: [u8; GROUP_N_SEED] = seed_full[KEM_N_SEED..KEM_N_SEED + GROUP_N_SEED]
            .try_into()
            .expect("wrong length");

        let (dk_pq, ek_pq) = mlkem::generate_key_pair(seed_pq).into_parts();
        let dk_t = seed_t; // Group.RandomScalar is just the identity
        let ek_t = x25519_dalek::x25519(X25519_BASEPOINT_BYTES, dk_t);

        (
            ek_pq.as_slice().clone(),
            ek_t,
            SecretBox::new(Box::new(dk_pq.as_slice().clone())),
            SecretBox::new(Box::new(dk_t)),
        )
    }

    pub fn to_public(&self) -> HybridRecipient {
        let (ek_pq, ek_t, _, _) = HybridIdentity::expand_key(&self.seed.expose_secret());
        HybridRecipient {
            encapsulation_key: [&ek_pq[..], &ek_t[..]]
                .concat()
                .as_slice()
                .try_into()
                .unwrap(),
        }
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            seed: SecretBox::new(Box::new(seed)),
        }
    }

    pub fn to_string(&self) -> SecretString {
        let hrp = bech32::Hrp::parse_unchecked("AGE-SECRET-KEY-PQ-");
        SecretString::new(
            self.seed
                .expose_secret()
                .iter()
                .copied()
                .bytes_to_fes()
                .with_checksum::<Bech32>(&hrp)
                .chars()
                .collect(),
        )
    }

    /// parse an identity from a Bech32-encoded string
    pub fn from_string(text: SecretString) -> Result<Self, String> {
        let (hrp, decoded) = bech32::decode(text.expose_secret()).map_err(|e| e.to_string())?;
        if hrp.as_str() != "AGE-SECRET-KEY-PQ-" {
            return Err("not a valid secret key".to_string());
        }
        Ok(Self {
            seed: SecretBox::from(Box::new(
                decoded
                    .as_slice()
                    .try_into()
                    .map_err(|e: TryFromSliceError| e.to_string())?,
            )),
        })
    }

    pub fn to_x25519(&self) -> age::x25519::Identity {
        let (_, _, _, dk_t) = HybridIdentity::expand_key(&self.seed.expose_secret());
        let hrp = bech32::Hrp::parse_unchecked("AGE-SECRET-KEY-");
        age::x25519::Identity::from_str(
            bech32::encode::<bech32::Bech32>(hrp, dk_t.expose_secret())
                .expect("failed to encode x25519 private key")
                .as_str(),
        )
        .expect("x25519 private key was not valid")
    }
}

impl Identity for HybridIdentity {
    fn unwrap_stanza(&self, stanza: &Stanza) -> Option<Result<FileKey, DecryptError>> {
        if stanza.args.len() > 0 && RECIPIENT_TAG.to_string() != stanza.args[0] {
            return None;
        }
        if stanza.args.len() != 2 {
            return Some(Err(DecryptError::InvalidHeader));
        }
        let body = stanza.body.as_slice();
        if body.len() != 32 {
            return Some(Err(DecryptError::DecryptionFailed));
        }

        let enc = match BASE64_STANDARD_NO_PAD.decode(stanza.args[1].clone()) {
            Ok(vec) => {
                if vec.len() != 1120 {
                    return Some(Err(DecryptError::InvalidHeader));
                }
                vec
            }
            Err(_) => return Some(Err(DecryptError::InvalidHeader)),
        };
        let hpke = Hpke::<HpkeLibcrux>::new(
            hpke_rs::Mode::Base,
            hpke_types::KemAlgorithm::XWingDraft06,
            hpke_types::KdfAlgorithm::HkdfSha256,
            hpke_types::AeadAlgorithm::ChaCha20Poly1305,
        );

        let mut file_key = match hpke.open(
            &enc,
            &HpkePrivateKey::from(&self.seed.expose_secret()[..]),
            b"age-encryption.org/mlkem768x25519",
            &[],
            body,
            None,
            None,
            None,
        ) {
            Ok(file_key) => file_key,
            Err(_) => return Some(Err(DecryptError::NoMatchingKeys)),
        };

        Some(Ok(FileKey::init_with_mut(|inner| {
            inner.copy_from_slice(&file_key);
            file_key.zeroize();
        })))
    }
}
