// the secure store for private keys.
// use the Vault struct to interface with it
// you can open (or create) a vault with:
// Vault::load_vault()
mod commands;
use age::x25519::{Identity, Recipient};
pub use commands::*;

use argon2::{password_hash::rand_core::RngCore, Argon2};
use chacha20poly1305::{
    aead::{AeadMut, OsRng},
    AeadCore, KeyInit, XChaCha20Poly1305, XNonce,
};

use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, io::Write, path::PathBuf, str::FromStr, time::SystemTime}; // how terrifying

#[derive(Serialize, Deserialize, Debug)]
pub enum KeyType {
    Public,
    Private,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyMetadata {
    pub name: String,
    pub key_type: KeyType,
    pub date_created: SystemTime,
}

impl KeyMetadata {
    pub fn new(name: String, key_type: KeyType) -> KeyMetadata {
        KeyMetadata {
            name,
            key_type,
            date_created: SystemTime::now(),
        }
    }
}

pub fn derive_key(password: &SecretString, salt: &[u8]) -> SecretBox<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.expose_secret().as_bytes(), salt, &mut key)
        .expect("failed to hash password into key");

    SecretBox::new(Box::new(key))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EncryptedSecret {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultFile {
    salt: Vec<u8>,
    hello: EncryptedSecret,
    secrets: HashMap<String, EncryptedSecret>,
}

pub struct Vault {
    file: VaultFile,
    path: PathBuf,
    key: SecretBox<[u8; 32]>,
}

impl Vault {
    fn decrypt_secret(&self, encrypted_secret: &EncryptedSecret) -> Result<SecretString, String> {
        let mut cipher = XChaCha20Poly1305::new(self.key.expose_secret().into());
        let nonce = XNonce::from_slice(&encrypted_secret.nonce);

        let decrypted_bytes = cipher.decrypt(nonce, encrypted_secret.ciphertext.as_ref());
        if decrypted_bytes.is_err() {
            return Err("failed to decrypt secret.".to_string());
        }

        return Ok(SecretString::from(
            String::from_utf8(decrypted_bytes.unwrap()).expect("failed to decode as utf-8"),
        ));
    }

    fn encrypt_secret(key: &SecretBox<[u8; 32]>, secret: SecretString) -> EncryptedSecret {
        let mut cipher = XChaCha20Poly1305::new(key.expose_secret().into());
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let ciphertext = cipher
            .encrypt(&nonce, secret.expose_secret().as_bytes())
            .expect("failed to encrypt secret");

        EncryptedSecret {
            nonce: nonce.to_vec(),
            ciphertext,
        }
    }

    #[allow(dead_code)] // shhhh we got it bro
    pub fn load_secret(&self, id: String) -> Option<SecretString> {
        let encrypted_secret = self.file.secrets.get(&id);
        if let Some(secret) = encrypted_secret {
            return Some(
                self.decrypt_secret(secret)
                    .expect("failed to decrypt secret"),
            );
        } else {
            return None;
        }
    }

    pub fn put_secret(&mut self, id: String, secret: SecretString) -> Result<(), String> {
        let encrypted_secret = Vault::encrypt_secret(&self.key, secret);
        self.file.secrets.insert(id, encrypted_secret);
        Ok(())
    }

    pub fn load_vault(path: &str, password: SecretString) -> Result<Self, String> {
        if !std::path::Path::new(path).exists() {
            return Err("vault does not exist".to_string());
        }
        let data = fs::read(path).expect("could not read vault");
        let vault_file: VaultFile = serde_cbor::from_slice(&data).expect("could not parse vault");
        let key = derive_key(&password, &vault_file.salt);

        let vault = Vault {
            file: vault_file,
            path: PathBuf::from_str(path).expect("invalid path"),
            key,
        };

        let hello = vault.decrypt_secret(&vault.file.hello);
        if hello.is_ok() {
            let raw_hello = hello.unwrap().expose_secret().to_string();
            if raw_hello != "hello" {
                return Err("authentication failed".to_string());
            }
        } else {
            return Err("authentication failed".to_string());
        };

        Ok(vault)
    }

    pub fn create_vault(path: &str, password: &SecretString) -> Vault {
        let mut salt = [0u8; 16];
        OsRng.fill_bytes(&mut salt);

        let key = derive_key(password, &salt);

        let vault_file = VaultFile {
            salt: salt.to_vec(),
            hello: Vault::encrypt_secret(&key, SecretString::from("hello")),
            secrets: HashMap::new(),
        };
        Vault {
            file: vault_file,
            path: PathBuf::from_str(path).expect("invalid path"),
            key,
        }
    }

    pub fn save_vault(&self) {
        let data = serde_cbor::to_vec(&self.file).expect("failed to serialize vault");
        let path = self.path.clone();
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).expect("failed to create parent directories");
        }

        std::fs::write(path, &data).expect("failed to init vault file");
        // file.write_all(&data)
        //     .expect("failed to write vault to disk");
        // file.flush().expect("failed to flush buffers");
    }
}
