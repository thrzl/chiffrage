// higher-level age functions to be called from the frontend

use age::x25519::{Identity, Recipient};
use age::Encryptor;
use secrecy::{ExposeSecret, SecretString};
use std::io::{Read, Write};

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
pub fn encrypt_text(public_keys: Vec<String>, text: String) -> Vec<u8> {
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
    let mut encrypted_text = vec![];
    let mut writer = encryptor
        .wrap_output(&mut encrypted_text)
        .expect("failed to initialize writer");
    writer.write_all(text.as_bytes());
    writer.finish();

    return encrypted_text;
}
