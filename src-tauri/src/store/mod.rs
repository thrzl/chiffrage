// the secure store for private keys.
// use the Vault struct to interface with it
// you can open (or create) a vault with:
// Vault::load_vault()
mod commands;
use age::secrecy::zeroize::Zeroize;
use age::x25519::{Identity, Recipient};
pub use commands::*;
use region::{alloc, lock, LockGuard, Protection};

use argon2::{password_hash::rand_core::RngCore, Argon2};
use chacha20poly1305::{
    aead::{AeadMut, OsRng},
    AeadCore, KeyInit, XChaCha20Poly1305, XNonce,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use cuid2::create_id;
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{fs, path::PathBuf, str::FromStr, time::SystemTime};

use crate::crypto::WildcardIdentity;
use age_xwing::HybridIdentity;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub enum KeyType {
    Public,
    Private,
}

/// representation of a key object. id is a cuid2
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct KeyMetadata {
    pub id: String,
    pub name: String,
    pub key_type: KeyType,
    pub date_created: SystemTime,
    pub contents: KeyPair,
}

impl KeyMetadata {
    pub fn from_keypair(name: String, keypair: KeyPair) -> KeyMetadata {
        let key_type = match keypair.private {
            Some(_) => KeyType::Private,
            None => KeyType::Public,
        };
        KeyMetadata {
            id: create_id(),
            name,
            key_type,
            date_created: SystemTime::now(),
            contents: keypair,
        }
    }

    /// removes the private key without changing `key_type`
    pub fn redacted(mut self) -> KeyMetadata {
        self.contents.redact();
        self
    }
}

/// an object storing the actual key contents.
///
/// `public` contains the public key in plaintext
///
/// `private` is `Option<EncryptedSecret>`, being an object containing a `nonce` and `ciphertext` (both `Vec<u8>`)
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct KeyPair {
    pub public: String,
    pub private: Option<EncryptedSecret>,
}

impl KeyPair {
    #[allow(dead_code)]
    pub fn redact(&mut self) {
        self.private = None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]
#[serde(rename_all = "camelCase")]
pub enum VaultStatusUpdate {
    Unlocked,
    VerificationFail,
    AuthenticationCancel,
    Locked,
}

impl From<Recipient> for KeyPair {
    fn from(recipient: Recipient) -> KeyPair {
        KeyPair {
            public: recipient.to_string(),
            private: None,
        }
    }
}

/// a type storing an XChaCha20Poly1305 `ciphertext` and `nonce`. both are of type `Vec<u8>`.
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct EncryptedSecret {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

/// derive a 256-bit key from a password and salt, using argon2.
pub fn derive_key(
    password: &SecretString,
    salt: &[u8],
) -> Result<(LockGuard, SecretBox<[u8; 32]>), String> {
    let argon2 = Argon2::default();
    let key = alloc(32, Protection::READ_WRITE).map_err(|e| e.to_string())?;
    let _guard = lock(key.as_ptr::<u8>(), 32).map_err(|e| e.to_string())?;
    let key_slice = unsafe { &mut *(key.as_ptr::<u8>() as *mut [u8; 32]) };
    argon2
        .hash_password_into(password.expose_secret().as_bytes(), salt, key_slice)
        .expect("failed to hash password into key");

    Ok((_guard, SecretBox::new(Box::new(*key_slice))))
}

/// an abstraction for the contents of the vault file. contains the `salt`, a `hello` value used to validate passwords, and a map of `secrets`.
#[derive(Serialize, Deserialize, Debug)]
pub struct VaultFile {
    salt: Vec<u8>,
    hello: EncryptedSecret,
    secrets: BTreeMap<String, KeyMetadata>,
    hmac: Option<Vec<u8>>,
}

pub struct Vault {
    file: VaultFile,
    path: PathBuf,
    key: Option<SecretBox<[u8; 32]>>,
    _key_guard: Option<LockGuard>,
}

type HmacSha256 = Hmac<Sha256>;

impl Vault {
    pub fn set_vault_key(&mut self, password: SecretString) -> Result<(), String> {
        let (_guard, key) = derive_key(&password, &self.file.salt)?;
        let hello = &self.file.hello;
        let mut cipher = XChaCha20Poly1305::new(key.expose_secret().into());
        let nonce = XNonce::from_slice(hello.nonce.as_slice());

        let decrypted_bytes = cipher.decrypt(nonce, hello.ciphertext.as_ref());
        if decrypted_bytes.is_err() {
            return Err("password is incorrect".to_string());
        };
        self.key = Some(key);
        self._key_guard = Some(_guard);
        if !self.verify_integrity() {
            return Err("integrity check failed".to_string());
        };
        Ok(())
    }
    pub fn get_vault_key(&self) -> Result<&SecretBox<[u8; 32]>, String> {
        self.key.as_ref().ok_or("vault is locked".to_string())
    }
    pub fn delete_vault_key(&mut self) {
        self.key = None;
        self._key_guard = None;
    }
    pub fn new_key(
        &self,
        name: String,
        public: String,
        private: Option<SecretString>,
    ) -> Result<KeyMetadata, String> {
        let private = match private {
            Some(private_key) => Some(Vault::encrypt_secret(self.get_vault_key()?, private_key)?),
            None => None,
        };
        Ok(KeyMetadata::from_keypair(name, KeyPair { public, private }))
    }

    pub fn keypair_from(&self, identity: WildcardIdentity) -> Result<KeyPair, String> {
        let identity_text = identity
            .to_string()
            .expect("string conversion should not fail");
        let private = Vault::encrypt_secret(self.get_vault_key()?, identity_text)
            .expect("encryption should not fail if vault key is set");

        Ok(KeyPair {
            public: identity.to_public()?.to_string()?,
            private: Some(private),
        })
    }

    pub fn delete_key(&mut self, id: String) {
        let _ = self.file.secrets.remove(&id);
    }

    /// generate an mlkem768x25519 identity
    pub fn generate_keypair(&self, name: String) -> Result<KeyMetadata, String> {
        let identity = HybridIdentity::generate();
        let keypair = KeyPair {
            public: identity.to_public().to_string(),
            private: Some(Vault::encrypt_secret(
                self.get_vault_key()?,
                identity.to_string(),
            )?),
        };
        Ok(KeyMetadata {
            id: create_id(),
            name,
            key_type: KeyType::Private,
            date_created: SystemTime::now(),
            contents: keypair,
        })
    }

    /// generate an x25519 identity
    pub fn generate_x25519_keypair(&self, name: String) -> Result<KeyMetadata, String> {
        let identity = Identity::generate();
        let keypair = KeyPair {
            public: identity.to_public().to_string(),
            private: Some(Vault::encrypt_secret(
                self.get_vault_key()?,
                SecretString::from(identity.to_string()),
            )?),
        };
        Ok(KeyMetadata {
            id: create_id(),
            name,
            key_type: KeyType::Private,
            date_created: SystemTime::now(),
            contents: keypair,
        })
    }

    pub fn decrypt_secret(
        &self,
        encrypted_secret: &EncryptedSecret,
    ) -> Result<SecretString, String> {
        let mut cipher = XChaCha20Poly1305::new(self.get_vault_key()?.expose_secret().into());
        let nonce = XNonce::from_slice(&encrypted_secret.nonce);

        let decrypted_bytes = cipher.decrypt(nonce, encrypted_secret.ciphertext.as_ref());
        if decrypted_bytes.is_err() {
            return Err("failed to decrypt secret.".to_string());
        }

        return Ok(SecretString::from(
            String::from_utf8(decrypted_bytes.unwrap()).expect("failed to decode as utf-8"),
        ));
    }

    fn encrypt_secret(
        key: &SecretBox<[u8; 32]>,
        secret: SecretString,
    ) -> Result<EncryptedSecret, String> {
        let key_ptr = alloc(32, Protection::READ_WRITE).map_err(|e| e.to_string())?;
        let _guard = lock(key_ptr.as_ptr::<u8>(), 32);

        let key_slice = unsafe { &mut *(key_ptr.as_ptr::<u8>() as *mut [u8; 32]) };

        key_slice.copy_from_slice(key.expose_secret());

        let mut cipher = XChaCha20Poly1305::new(&(*key_slice).into());
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        key_slice.zeroize();
        drop(_guard);

        let ciphertext = cipher
            .encrypt(&nonce, secret.expose_secret().as_bytes())
            .map_err(|e| e.to_string())?;

        Ok(EncryptedSecret {
            nonce: nonce.to_vec(),
            ciphertext,
        })
    }

    pub fn get_key(&self, id: &str) -> Option<&KeyMetadata> {
        self.file.secrets.get(id)
    }

    pub fn put_key(&mut self, key: KeyMetadata) -> Result<(), String> {
        self.file.secrets.insert(key.id.clone(), key);
        Ok(())
    }

    pub fn load_vault(path: &str) -> Result<Self, String> {
        if !std::path::Path::new(path).exists() {
            return Err("vault does not exist".to_string());
        }
        let data = fs::read(path).expect("could not read vault");
        let vault_file: VaultFile = serde_cbor::from_slice(&data).expect("could not parse vault");

        let vault = Vault {
            file: vault_file,
            path: PathBuf::from_str(path).expect("invalid path"),
            key: None,
            _key_guard: None,
        };

        Ok(vault)
    }

    pub fn create_vault(path: &str, password: &SecretString) -> Result<Vault, String> {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let (_guard, key) = derive_key(password, &salt)?;

        let vault_file = VaultFile {
            salt: salt.to_vec(),
            hello: Vault::encrypt_secret(&key, SecretString::from("hello"))?,
            secrets: BTreeMap::new(),
            hmac: None,
        };
        Ok(Vault {
            file: vault_file,
            path: PathBuf::from_str(path).expect("invalid path"),
            key: Some(key),
            _key_guard: Some(_guard),
        })
    }

    pub fn verify_integrity(&self) -> bool {
        let calculated_mac = self.vault_hmac();
        return self
            .file
            .hmac
            .as_ref()
            .is_some_and(|hmac| calculated_mac == *hmac);
    }

    fn vault_hmac(&self) -> Vec<u8> {
        let secrets_bytes =
            serde_cbor::to_vec(&self.file.secrets).expect("failed to serialize vault");
        let mut mac =
            <HmacSha256 as Mac>::new_from_slice(&self.key.as_ref().unwrap().expose_secret()[..])
                .expect("key should be set");
        mac.update(secrets_bytes.as_slice());
        mac.finalize().into_bytes().to_vec()
    }
    pub fn save_vault(&mut self) -> Result<(), String> {
        let path = self.path.clone();
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create parent directories");
        }

        if self.key.is_none() {
            return Err("key is not set".to_string());
        }

        self.file.hmac = Some(self.vault_hmac());

        let data = serde_cbor::to_vec(&self.file).expect("failed to serialize vault");
        std::fs::write(path, &data).expect("failed to init vault file");

        Ok(())
    }
}
