// higher-level age functions to be called from the frontend

mod commands;
use age::x25519::{Identity, Recipient};
use age::Decryptor;
pub use commands::*;
use secrecy::{ExposeSecret, SecretString};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

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

pub fn encrypt_file(public_keys: Vec<String>, file_path: PathBuf) -> Result<PathBuf, String> {
    let file = File::open(&file_path).expect("failed to open file");
    let mut reader = BufReader::new(file);

    let mut encrypted_output = file_path.clone();
    encrypted_output.add_extension("age");
    let output = File::create(&encrypted_output).expect("failed to get handle on output file");
    let mut file_writer = BufWriter::new(output);

    let encryptor = age::Encryptor::with_recipients(
        keys_to_recipients(public_keys)
            .iter()
            .map(|recipient| recipient as _),
    )
    .expect("encryptor initialization failed");

    let mut writer = encryptor
        .wrap_output(&mut file_writer)
        .expect("failed to initialize writer");

    let mut buffer = [0u8; 8_192]; // 8 kb buffer

    loop {
        let n = reader.read(&mut buffer).expect("failed to read file");
        if n == 0 {
            break;
        }
        writer.write_all(&buffer[..n]).expect("failed to write"); // only write the new bytes
    }

    writer.finish().expect("failed to write final chunk");
    Ok(encrypted_output)
}

pub fn decrypt_file(private_key: String, file_path: PathBuf) -> Result<PathBuf, String> {
    let file = File::open(&file_path).expect("failed to open file");
    let decryptor =
        Decryptor::new_buffered(BufReader::new(file)).expect("failed to initialize decryptor");

    let decrypted_output = file_path.with_extension("");
    let output = File::create(&decrypted_output).expect("failed to get handle on output file");
    let mut file_writer = BufWriter::new(output);

    let mut decrypted_reader = decryptor
        .decrypt(std::iter::once(
            &age::x25519::Identity::from_str(private_key.as_str()).unwrap() as &dyn age::Identity,
        ))
        .expect("failed to decrypt contents");

    let mut buffer = [0u8; 8_192]; // 8 kb buffer

    loop {
        let n = decrypted_reader
            .read(&mut buffer)
            .expect("failed to read file");
        if n == 0 {
            break;
        }
        file_writer
            .write_all(&buffer[..n])
            .expect("failed to write"); // only write the new bytes
    }

    Ok(decrypted_output)
}

pub fn keys_to_recipients(public_keys: Vec<String>) -> Vec<Recipient> {
    return public_keys
        .iter()
        .map(|key| -> Result<Recipient, String> {
            Ok(key
                .parse::<Recipient>()
                .expect(&format!("could not parse recipient from key: {}", key)))
        })
        .filter_map(|recipient: Result<Recipient, _>| recipient.ok())
        .collect::<Vec<Recipient>>();
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
