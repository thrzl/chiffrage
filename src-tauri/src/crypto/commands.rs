use crate::crypto::{self, WildcardIdentity, WildcardRecipient};
use crate::AppState;
use futures_util::future::join_all;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use tauri_plugin_opener::reveal_items_in_dir;
use tokio::fs::metadata;
use tokio::io::AsyncReadExt;

#[derive(Deserialize)]
#[serde(untagged)]
pub enum EncryptionMethod {
    X25519(Vec<String>),
    Scrypt(String),
}

#[derive(Deserialize)]
pub enum DecryptionMethod {
    X25519,
    Scrypt,
}

#[derive(Serialize)]
pub enum AgeFileType {
    Identity,
    Recipient,
    EncryptedIdentity,
    ArmoredFile,
    BinaryFile,
}

#[tauri::command]
pub async fn get_file_type(path: String) -> Result<AgeFileType, String> {
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|err| format!("could not open file: {err}"))?;
    let mut buf = [0u8; 32];
    file.read(&mut buf)
        .await
        .map_err(|err| format!("could not read file: {err}"))?;

    if buf.starts_with(b"age-encryption.org/v1") {
        return Ok(AgeFileType::BinaryFile);
    }

    let file_text = String::from_utf8(buf.to_vec()).map_err(|e| e.to_string())?;

    if file_text.starts_with("-----BEGIN AGE ENCRYPTED FILE-----") {
        return Ok(AgeFileType::ArmoredFile);
    } else if file_text.starts_with("AGE-SECRET-KEY") {
        return Ok(AgeFileType::Identity);
    } else if file_text.starts_with("age") {
        return Ok(AgeFileType::Recipient);
    }

    Err("failed to detect file type".to_string())
}

#[tauri::command]
pub async fn validate_key_text(text: String) -> Result<(), String> {
    match bech32::decode(&text) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("this is not a valid age key. {err}")),
    }
}

#[tauri::command]
pub fn armor_check_text(text: String) -> bool {
    text.starts_with("-----BEGIN AGE ENCRYPTED FILE-----")
}

pub async fn armor_check_file(path: &String) -> Result<bool, String> {
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|err| format!("could not open file: {err}"))?;
    let mut buf = [0u8; 34];
    let bytes = file
        .read(&mut buf)
        .await
        .map_err(|err| format!("could not read file: {err}"))?;
    let key_text = String::from_utf8(buf[..bytes].to_vec())
        .map_err(|err| format!("could not decode text content: {err}"))?;
    Ok(armor_check_text(key_text))
}

#[tauri::command]
pub async fn validate_key_file(path: String) -> Result<(), String> {
    let mut file = tokio::fs::File::open(&path)
        .await
        .map_err(|err| format!("could not open file: {err}"))?;
    let mut buf = [0u8; 100];
    let bytes = file
        .read(&mut buf)
        .await
        .map_err(|err| format!("could not read file: {err}"))?;
    let key_text = String::from_utf8(buf[..bytes].to_vec())
        .map_err(|err| format!("could not decode text content: {err}"))?;
    validate_key_text(key_text).await
}

#[tauri::command]
pub async fn encrypt_text(
    recipient: EncryptionMethod,
    text: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let recipients: Vec<Box<WildcardRecipient>> = match recipient {
        EncryptionMethod::X25519(public_keys) => {
            let state = state.lock().expect("failed to get lock on state");
            let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
            let vault = match vault_handle.lock() {
                Ok(vault) => vault,
                Err(poisoned) => poisoned.into_inner(),
            };
            let key_contents = public_keys
                .iter()
                .map(|key| vault.get_key(key).unwrap().contents.public.clone())
                .collect::<Vec<String>>();
            crypto::keys_to_x25519_recipients(&key_contents)?
                .into_iter()
                .map(|recipient| Box::new(recipient) as Box<WildcardRecipient>)
                .collect()
        }
        EncryptionMethod::Scrypt(password) => {
            vec![
                Box::new(age::scrypt::Recipient::new(SecretString::from(password)))
                    as Box<WildcardRecipient>,
            ]
        }
    };
    return crypto::encrypt_armored_text(&recipients, text).await;
}

#[tauri::command]
pub async fn decrypt_text(
    private_key: String,
    text: String,
    method: DecryptionMethod,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let identity = match method {
        DecryptionMethod::X25519 => {
            let state = state.lock().expect("failed to get lock on state");
            let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
            let vault = match vault_handle.lock() {
                Ok(vault) => vault,
                Err(poisoned) => poisoned.into_inner(),
            };
            let key_metadata = vault.get_key(&private_key).unwrap();
            let key_content = vault
                .decrypt_secret(&key_metadata.contents.private.as_ref().unwrap())
                .unwrap()
                .clone();
            Box::new(
                key_content
                    .expose_secret()
                    .parse::<age::x25519::Identity>()?,
            ) as Box<WildcardIdentity>
        }
        DecryptionMethod::Scrypt => {
            Box::new(age::scrypt::Identity::new(SecretString::from(private_key)))
                as Box<WildcardIdentity>
        }
    };

    return crypto::decrypt_armored_text(&identity, text).await;
}

#[tauri::command]
pub async fn encrypt_file(
    recipient: EncryptionMethod,
    reader: tauri::ipc::Channel<serde_json::Value>,
    files: Vec<String>,
    state: tauri::State<'_, Mutex<AppState>>,
    armor: Option<bool>,
) -> Result<(), String> {
    let armor = armor.unwrap_or(false);
    let recipients: Vec<Box<WildcardRecipient>> = match recipient {
        EncryptionMethod::X25519(public_keys) => {
            let state = state.lock().expect("failed to get lock on state");
            let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
            let vault = match vault_handle.lock() {
                Ok(vault) => vault,
                Err(poisoned) => poisoned.into_inner(),
            };
            let key_contents = public_keys
                .iter()
                .map(|key| vault.get_key(key).unwrap().contents.public.clone())
                .collect::<Vec<String>>();
            crypto::keys_to_x25519_recipients(&key_contents)?
                .into_iter()
                .map(|recipient| Box::new(recipient) as Box<WildcardRecipient>)
                .collect()
        }
        EncryptionMethod::Scrypt(password) => {
            vec![
                Box::new(age::scrypt::Recipient::new(SecretString::from(password)))
                    as Box<WildcardRecipient>,
            ]
        }
    };
    let file_sizes: HashMap<String, u64> = files
        .clone()
        .into_iter()
        .zip(
            join_all(files.clone().into_iter().map(async |path| {
                metadata(path)
                    .await
                    .expect("failed to get file metadata")
                    .len()
            }))
            .await
            .into_iter(),
        )
        .collect();
    let total_bytes: u64 = file_sizes.values().sum();
    let total_read_bytes_ptr = Arc::new(AtomicUsize::new(0));
    let reader_ptr = Arc::new(reader);
    let mut output_paths = Vec::new();

    let timer = Arc::new(timer::Timer::new());
    for file in files {
        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = PathBuf::from(file.clone());
        let reader = reader_ptr.clone();
        let timer = timer.clone();
        let _guard = timer.schedule_repeating(chrono::Duration::milliseconds(100), move || {
            let _ = reader.send(
                json!({ // its okay if it doesnt send i'd rather the files just encrypt
                    "read_bytes": total_read_bytes,
                    "total_bytes": total_bytes,
                    "current_file": path.file_name().unwrap().to_str().unwrap()
                }),
            );
        });

        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = PathBuf::from(file.clone());
        let output_path =
            crypto::encrypt_file(&recipients, &path.clone(), armor, move |processed_bytes| {
                total_read_bytes.fetch_add(processed_bytes, std::sync::atomic::Ordering::SeqCst);
            })
            .await
            .expect("failed to encrypt file");
        drop(_guard);
        let _ = reader_ptr.clone().send(
            json!({ // its okay if it doesnt send i'd rather the files just encrypt
                "read_bytes": file_sizes.get(&file).unwrap(),
                "total_bytes": total_bytes,
                "current_file": path.file_name().unwrap().to_str().unwrap()
            }),
        );
        output_paths.push(output_path)
    }
    reveal_items_in_dir(output_paths).expect("failed to reveal item");
    Ok(())
}

#[tauri::command]
pub async fn decrypt_file(
    private_key: String,
    reader: tauri::ipc::Channel<serde_json::Value>,
    files: Vec<String>,
    method: DecryptionMethod,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let identity = match method {
        DecryptionMethod::X25519 => {
            let state = state.lock().expect("failed to get lock on state");
            let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
            let vault = match vault_handle.lock() {
                Ok(vault) => vault,
                Err(poisoned) => poisoned.into_inner(),
            };
            let key_metadata = vault.get_key(&private_key).unwrap();
            let key_content = vault
                .decrypt_secret(&key_metadata.contents.private.as_ref().unwrap())
                .unwrap()
                .clone();
            Box::new(
                key_content
                    .expose_secret()
                    .parse::<age::x25519::Identity>()?,
            ) as Box<WildcardIdentity>
        }
        DecryptionMethod::Scrypt => {
            Box::new(age::scrypt::Identity::new(SecretString::from(private_key)))
                as Box<WildcardIdentity>
        }
    };
    let file_sizes: HashMap<String, u64> = files
        .clone()
        .into_iter()
        .zip(
            join_all(files.clone().into_iter().map(async |path| {
                metadata(path)
                    .await
                    .expect("failed to get file metadata")
                    .len()
            }))
            .await
            .into_iter(),
        )
        .collect();
    let total_bytes: u64 = file_sizes.values().sum();
    let total_read_bytes_ptr = Arc::new(AtomicUsize::new(0));
    let reader_ptr = Arc::new(reader);
    let mut output_paths = Vec::new();
    let timer = Arc::new(timer::Timer::new());
    for file in files {
        let total_read_bytes = total_read_bytes_ptr.clone();
        let reader = reader_ptr.clone();
        let timer = timer.clone();
        let path_ptr = Arc::new(PathBuf::from_str(&file).unwrap());
        let path = path_ptr.clone();
        let _guard = timer.schedule_repeating(chrono::Duration::milliseconds(100), move || {
            let _ = reader.send(
                json!({ // its okay if it doesnt send i'd rather the files just encrypt
                    "read_bytes": &total_read_bytes,
                    "total_bytes": &total_bytes,
                    "current_file": path.file_name().unwrap().to_str().unwrap()
                }),
            );
        });
        let is_armored = armor_check_file(&file).await?;
        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = path_ptr;
        let output_path =
            crypto::decrypt_file(&identity, &path, is_armored, move |processed_bytes| {
                total_read_bytes.fetch_add(processed_bytes, std::sync::atomic::Ordering::SeqCst);
            })
            .await?;
        drop(_guard);
        let reader = reader_ptr.clone();
        let _ = reader.send(json!({
            "read_bytes": file_sizes.get(&file).unwrap(),
            "total_bytes": total_bytes,
            "current_file": path.file_name().unwrap().to_str().unwrap()
        })); // ensure that it "completes" on the frontend
        output_paths.push(output_path)
    }
    let _ = reader_ptr.send(json!({
        "read_bytes": total_bytes,
        "total_bytes": total_bytes,
        "current_file": ""
    })); // ensure that it "completes" on the frontend
    reveal_items_in_dir(output_paths).expect("failed to reveal item");
    Ok(())
}

#[tauri::command]
pub async fn generate_keypair(
    name: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    if name.len() == 0 {
        return Err("no name set".to_string());
    }
    let vault_handle = {
        let state = state.lock().unwrap();
        state.vault.as_ref().expect("failed to load vault").clone()
    };
    {
        let mut vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        let keypair = vault.generate_key(name);
        vault.put_key(keypair)?;
    }
    tauri::async_runtime::spawn_blocking(move || {
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault.save_vault();
    });
    Ok(())
}

#[tauri::command]
pub async fn generate_passphrase() -> String {
    bip39::Mnemonic::generate(12)
        .expect("failed to generate mnemonic")
        .to_string()
        .replace(" ", "-")
}
