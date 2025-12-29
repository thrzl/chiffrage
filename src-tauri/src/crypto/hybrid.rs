use age::{DecryptError, EncryptError, Identity, Recipient};
use age_core::format::{FileKey, Stanza};
use base64::prelude::{Engine, BASE64_STANDARD_NO_PAD};
use bech32::{primitives::decode::UncheckedHrpstring, Bech32, ByteIterExt, Checksum, Fe32IterExt};
use bip39::{rand::RngCore, rand_core::OsRng};
use hpke_rs::{hpke_types, Hpke, HpkePrivateKey, HpkePublicKey};
use hpke_rs_libcrux::HpkeLibcrux;
use libcrux_ml_kem::mlkem768 as mlkem;
use secrecy::{zeroize::Zeroize, ExposeSecret, SecretBox, SecretString};
use sha3::digest::{ExtendableOutput, Update, XofReader};
use std::{array::TryFromSliceError, collections::HashSet, str::FromStr};
use x25519_dalek::X25519_BASEPOINT_BYTES;

const RECIPIENT_TAG: &str = "mlkem768x25519";
const KEM_N_SEED: usize = 64;
const GROUP_N_SEED: usize = 32;

/// an implementation of SHAKE256 from [FIPS 202](https://doi.org/10.6028/NIST.FIPS.202).
pub fn shake256<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut hasher = sha3::Shake256::default();
    hasher.update(input);
    let mut reader = hasher.finalize_xof();
    let mut output = [0u8; N];
    XofReader::read(&mut reader, &mut output);
    output
}

fn bech32_decode(string: &String) -> Result<Vec<u8>, String> {
    if !string.starts_with(&"age1pq".to_string()) {
        return Err("not a valid recipient".to_string());
    }

    let decoded = UncheckedHrpstring::new(string.as_str()).map_err(|e| e.to_string())?;

    if decoded.data_part_ascii().len() < Bech32::CODE_LENGTH {
        return Err("failed to parse string as Bech32".to_string());
    }

    {
        // this is simply an implementation of UncheckedHrpString::validate_checksum
        // it is reimplemented to exclude the length check
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
    Ok(decoded
        .iter()
        .map(|char| bech32::Fe32::from_char_unchecked(*char))
        .fes_to_bytes()
        .collect::<Vec<u8>>())
}

pub struct HybridRecipient {
    pub encapsulation_key: [u8; 1216],
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

        // HPKE.SealBase(...)
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
            .expect("nothing here should fail");

        let mut labels = HashSet::new();
        labels.insert("postquantum".to_string());

        // i know this is weird
        // age_core::Stanza encodes this as
        //
        // -> mlkem768x25519 <base64(enc)>
        //
        // and this is what it takes to get it to properly do it
        Ok((
            vec![Stanza {
                tag: RECIPIENT_TAG.to_string(),
                args: vec![BASE64_STANDARD_NO_PAD.encode(wrapped_key)],
                body: ciphertext.to_vec(),
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

    pub fn to_x25519(&self) -> age::x25519::Recipient {
        let hrp = bech32::Hrp::parse_unchecked("age");
        let encoded_chars = self.encapsulation_key[1184..1216] // only the x25519 bit
            .iter()
            .copied()
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars();

        let age_key: String = encoded_chars.collect();
        age::x25519::Recipient::from_str(age_key.as_str())
            .expect("this should produce a valid x25519 recipient")
    }

    pub fn from_string(string: &String) -> Result<Self, String> {
        let decoded_bytes = bech32_decode(string)?;
        Ok(Self {
            encapsulation_key: decoded_bytes.try_into().unwrap(),
        })
    }
}

pub struct HybridIdentity {
    seed: SecretBox<[u8; 32]>,
}

impl HybridIdentity {
    /// generates a new ML-KEM768x25519 seed with `OsRng`.
    pub fn generate() -> Self {
        let mut seed = [0u8; 32];
        OsRng::default().fill_bytes(&mut seed);
        let identity = Self {
            seed: SecretBox::new(Box::new(seed)),
        };
        seed.zeroize();
        identity
    }

    /// the `expandKey` function from https://filippo.io/hpke-pq. expands an identity seed into an ML-KEM768 and x25519 keypair.
    fn expand_key(
        seed: &[u8; 32],
    ) -> (
        [u8; 1184],
        [u8; 32],
        SecretBox<[u8; 2400]>,
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
        let ek_t = x25519_dalek::x25519(dk_t, X25519_BASEPOINT_BYTES); // Group.Exp

        let ek_pq: &[u8; 1184] = ek_pq.as_ref().try_into().unwrap();
        let dk_pq: &[u8; 2400] = dk_pq.as_ref().try_into().unwrap();

        (
            *ek_pq,
            ek_t,
            SecretBox::new(Box::new(*dk_pq)),
            SecretBox::new(Box::new(dk_t)),
        )
    }

    pub fn to_public(&self) -> HybridRecipient {
        let (ek_pq, ek_t, _, _) = HybridIdentity::expand_key(&self.seed.expose_secret());
        let mut ek = Vec::with_capacity(1216);
        ek.extend_from_slice(&ek_pq[..]);
        ek.extend_from_slice(&ek_t[..]);
        HybridRecipient {
            encapsulation_key: ek.as_slice().try_into().unwrap(),
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
                .copied() // for each cloned byte..
                .bytes_to_fes() // convert it to an Fe...
                .with_checksum::<Bech32>(&hrp) // for a Bech32 address
                .chars() // as an ascii character
                .collect::<String>()
                .to_ascii_uppercase() // make it all uppercase cus its supposed to be
                .into(),
        )
    }

    /// parse an identity from a Bech32-encoded string
    pub fn from_string(string: SecretString) -> Result<Self, String> {
        if !string.expose_secret().starts_with("AGE-SECRET-KEY-PQ-") {
            return Err("not a valid secret key".to_string());
        }
        let decoded = bech32_decode(&string.expose_secret().to_string())?;
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
        let hrp = bech32::Hrp::parse_unchecked("age1pq");
        let encoded_chars = dk_t
            .expose_secret()
            .iter()
            .copied()
            .bytes_to_fes()
            .with_checksum::<Bech32>(&hrp)
            .chars();

        let identity_string: String = encoded_chars.collect::<String>().to_ascii_uppercase();
        age::x25519::Identity::from_str(identity_string.as_str())
            .expect("x25519 private key was not valid")
    }
}

impl Identity for HybridIdentity {
    fn unwrap_stanza(&self, stanza: &Stanza) -> Option<Result<FileKey, DecryptError>> {
        // from age-encryption.org/v1:
        if stanza.args.len() > 0 && RECIPIENT_TAG.to_string() != stanza.tag {
            // "The identity implementation MUST ignore any stanza that does not have mlkem768x25519 as the first argument"
            return None;
        }
        if stanza.args.len() != 1 {
            // "and MUST otherwise reject any stanza that has more or less than two arguments"
            return Some(Err(DecryptError::InvalidHeader));
        }
        let enc = match BASE64_STANDARD_NO_PAD.decode(stanza.args[0].clone()) {
            Ok(vec) => {
                if vec.len() != 1120 {
                    // "or where the second argument is not a canonical base64 encoding of a 1120-byte value"
                    return Some(Err(DecryptError::InvalidHeader));
                }
                vec
            }
            Err(_) => return Some(Err(DecryptError::InvalidHeader)),
        };
        let ct = stanza.body.as_slice();
        if ct.len() != 32 {
            // "It MUST check that the body length is exactly 32 bytes before attempting to decrypt it, to mitigate partitioning oracle attacks."
            return Some(Err(DecryptError::DecryptionFailed));
        }

        let hpke = Hpke::<HpkeLibcrux>::new(
            hpke_rs::Mode::Base,
            hpke_types::KemAlgorithm::XWingDraft06,
            hpke_types::KdfAlgorithm::HkdfSha256,
            hpke_types::AeadAlgorithm::ChaCha20Poly1305,
        );

        // HPKE.OpenBase(...)
        let mut file_key: Vec<u8> = match hpke.open(
            &enc.as_slice()[..],
            &HpkePrivateKey::from(&self.seed.expose_secret()[..]),
            b"age-encryption.org/mlkem768x25519",
            &[],
            ct,
            None,
            None,
            None,
        ) {
            Ok(file_key) => file_key,
            Err(_) => return Some(Err(DecryptError::DecryptionFailed)),
        };

        Some(Ok(FileKey::init_with_mut(|inner| {
            inner.copy_from_slice(&file_key);
            file_key.zeroize();
        })))
    }
}
