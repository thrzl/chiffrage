// higher-level age functions to be called from the frontend

use age::x25519::{Identity, Recipient};
use secrecy::{ExposeSecret, SecretString};
use std::io::{BufReader, Write};
use std::path::Path;

pub struct Keypair {
    pub private_key: SecretString,
    pub public_key: String,
}

pub fn generate_key() -> Keypair {
    let key = Identity::generate();
    return Keypair {
        private_key: SecretString::from(key.to_string().expose_secret().to_string()),
        public_key: key.to_public().to_string(),
    };
}

#[tauri::command]
pub fn encrypt_file(public_keys: Vec<String>, file_path: &Path) -> Vec<u8> {
    todo!()
}

pub fn encrypt_bytes(public_keys: Vec<String>, bytes: &[u8]) -> Vec<u8> {
    // TODO need to make error handling not be terrible here. you dont want to encrypt something to nobody
    let recipients = public_keys
        .iter()
        .map(|key| -> Result<Recipient, String> {
            Ok(key
                .parse::<Recipient>()
                .expect(&format!("could not parse recipient from key: {}", key)))
        })
        .filter_map(|recipient: Result<Recipient, _>| recipient.ok())
        .collect::<Vec<Recipient>>();

    let encryptor =
        age::Encryptor::with_recipients(recipients.iter().map(|recipient| recipient as _))
            .expect("encryptor initialization failed");

    let mut encrypted_output = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted_output)
        .expect("failed to initialize writer");

    writer.write_all(bytes).expect("failed to write bytes");
    writer.finish().expect("failed to write final chunk");

    return encrypted_output;
}

#[tauri::command]
pub fn encrypt_text(public_keys: Vec<String>, text: String) -> Vec<u8> {
    return encrypt_bytes(public_keys, text.as_bytes());
}
