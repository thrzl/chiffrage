use age::{
    secrecy::{zeroize::Zeroizing, ExposeSecret},
    DecryptError, EncryptError, Identity, Recipient,
};
use age_core::format::{FileKey, Stanza};
use base64::prelude::{Engine, BASE64_STANDARD_NO_PAD};
use bip39::{rand::RngCore, rand_core::OsRng};
use chacha20poly1305::{
    aead::{AeadMut, KeyInit},
    ChaCha20Poly1305,
};
use hkdf::Hkdf;
use libcrux_ml_kem::mlkem1024::{self as mlkem, MlKem1024Ciphertext};
use secrecy::zeroize::Zeroize;
use sha2::Sha512;
use std::collections::HashSet;
use tokio_util::bytes::Buf;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519Secret};

const RECIPIENT_TAG: &str = "x25519-mlkem1024";

fn hybrid_combiner(
    x25519_ss: &[u8; 32],
    mlkem_ss: &[u8; 32],
    x25519_epk: &[u8; 32],
    mlkem_ct: &[u8; 1568],
) -> [u8; 32] {
    // Concatenate shared secrets (IKM)
    let mut ikm = Vec::with_capacity(64);
    ikm.extend_from_slice(x25519_ss);
    ikm.extend_from_slice(mlkem_ss);

    // Concatenate public inputs (salt)
    let mut salt = Vec::with_capacity(1600);
    salt.extend_from_slice(x25519_epk);
    salt.extend_from_slice(mlkem_ct);

    // HKDF-SHA512 with domain separation
    let mut output = [0u8; 32];
    Hkdf::<Sha512>::new(Some(&salt.as_slice()), &ikm)
        .expand(b"age-encryption/v1/hybrid-x25519-mlkem1024", &mut output)
        .expect("failed to derive hybrid key");
    output
}

pub struct HybridRecipient {
    pub x25519_pub: age::x25519::Recipient,
    pub mlkem_pub: mlkem::MlKem1024PublicKey,
}

impl Recipient for HybridRecipient {
    fn wrap_file_key(
        &self,
        file_key: &FileKey,
    ) -> Result<(Vec<Stanza>, HashSet<String>), EncryptError> {
        let mut x25519_pubkey_bytes = [0u8; 32];
        self.x25519_pub
            .to_string()
            .as_bytes()
            .copy_to_slice(&mut x25519_pubkey_bytes);

        // make the X25519 ephemeral key
        let x25519_ephemeral = X25519Secret::random_from_rng(OsRng);
        let x25519_shared_secret =
            x25519_ephemeral.diffie_hellman(&X25519PublicKey::from(x25519_pubkey_bytes));

        // encapsulate it with mlkem1024
        let mut randomness = [0u8; 32];
        OsRng.fill_bytes(&mut randomness);
        let (mlkem_ciphertext, mlkem_shared_secret) =
            mlkem::encapsulate(&self.mlkem_pub, randomness);

        let x25519_public_key = X25519PublicKey::from(&x25519_ephemeral);
        // combine them with sha512 hkdf
        let hybrid_key = Zeroizing::new(hybrid_combiner(
            x25519_shared_secret.as_bytes(),
            &mlkem_shared_secret,
            x25519_public_key.as_bytes(),
            mlkem_ciphertext.as_slice(),
        ));

        // Return ciphertext containing both ephemeral X25519 pub and Kyber ciphertext
        //
        let mut ciphertext = x25519_public_key.as_bytes().to_vec();
        ciphertext.extend(mlkem_ciphertext.as_slice());

        let mut cipher = ChaCha20Poly1305::new(hybrid_key.as_slice().into());
        let wrapped_key = cipher
            .encrypt(&[0u8; 12].into(), file_key.expose_secret().as_slice())
            .expect("failed to wrap file key");

        let mut labels = HashSet::new();
        labels.insert("postquantum".to_string());

        Ok((
            vec![Stanza {
                tag: RECIPIENT_TAG.to_string(),
                args: vec![
                    BASE64_STANDARD_NO_PAD.encode(&x25519_public_key.as_bytes()),
                    BASE64_STANDARD_NO_PAD.encode(&mlkem_ciphertext.as_slice()),
                ],
                body: wrapped_key,
            }],
            labels,
        ))
    }
}

// --- Identity ---
pub struct HybridIdentity {
    pub x25519_priv: age::x25519::Identity,
    pub mlkem_keypair: mlkem::MlKem1024KeyPair,
}

impl HybridIdentity {
    pub fn new(x25519_identity: age::x25519::Identity) -> Self {
        let mut output = [0u8; 64];
        Hkdf::<Sha512>::new(
            Some(b"age-encryption/v1/hybrid-x25519-mlkem1024"),
            &x25519_identity.to_string().expose_secret().as_bytes(),
        )
        .expand(b"mlkem-seed", &mut output)
        .expect("failed to derive hybrid key");

        Self {
            x25519_priv: x25519_identity,
            mlkem_keypair: mlkem::generate_key_pair(output),
        }
    }

    pub fn to_public(&self) -> HybridRecipient {
        HybridRecipient {
            x25519_pub: self.x25519_priv.to_public(),
            mlkem_pub: *self.mlkem_keypair.public_key(),
        }
    }
}

impl Identity for HybridIdentity {
    fn unwrap_stanza(&self, stanza: &Stanza) -> Option<Result<FileKey, DecryptError>> {
        if RECIPIENT_TAG.to_string() != stanza.tag {
            return None;
        }
        if stanza.args.len() != 2 {
            return Some(Err(DecryptError::InvalidHeader));
        }

        let x25519_public_key = match BASE64_STANDARD_NO_PAD.decode(&stanza.args[0]) {
            Ok(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return Some(Err(DecryptError::InvalidHeader)),
        };

        // Parse ML-KEM ciphertext
        let mlkem_ciphertext = match BASE64_STANDARD_NO_PAD.decode(&stanza.args[1]) {
            Ok(bytes) if bytes.len() == 1568 => {
                let mut arr = [0u8; 1568];
                arr.copy_from_slice(&bytes);
                arr
            }
            _ => return Some(Err(DecryptError::InvalidHeader)),
        };

        let mut x25519_pk_bytes = [0u8; 32];
        self.x25519_priv
            .to_string()
            .expose_secret()
            .as_bytes()
            .copy_to_slice(&mut x25519_pk_bytes);
        let x25519_priv = X25519Secret::from(x25519_pk_bytes);
        let x25519_ss = x25519_priv.diffie_hellman(&X25519PublicKey::from(x25519_public_key));

        // decapsulate key
        let mlkem_ss = mlkem::decapsulate(
            &self.mlkem_keypair.private_key(),
            &MlKem1024Ciphertext::from(mlkem_ciphertext),
        );

        let hybrid_key = Zeroizing::new(hybrid_combiner(
            x25519_ss.as_bytes(),
            &mlkem_ss,
            &x25519_public_key,
            &mlkem_ciphertext,
        ));

        let mut cipher = ChaCha20Poly1305::new(hybrid_key.as_slice().into());
        let mut plaintext = match cipher.decrypt(&[0u8; 12].into(), stanza.body.as_slice()) {
            Ok(file_key) => file_key,
            Err(_) => return Some(Err(DecryptError::NoMatchingKeys)),
        };

        Some(Ok(FileKey::init_with_mut(|file_key| {
            file_key.copy_from_slice(&plaintext);
            plaintext.zeroize();
        })))
    }
}
