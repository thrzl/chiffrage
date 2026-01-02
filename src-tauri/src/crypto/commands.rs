use crate::crypto::hybrid::{HybridIdentity, HybridRecipient};
use crate::crypto::{self, WildcardIdentity, WildcardRecipient};
use crate::AppState;
use futures_util::future::join_all;
use rand::seq::IndexedRandom;
use secrecy::zeroize::Zeroizing;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri_plugin_opener::reveal_items_in_dir;
use tokio::fs::metadata;
use tokio::io::AsyncReadExt;
use tokio::time;

const WORDLIST: &str = include_str!("wordlists/eff_large_wordlist.txt");

#[derive(serde::Serialize, specta::Type)]
pub struct FileOperationProgress {
    read_bytes: u64,
    total_bytes: u64,
    current_file: String,
}

#[derive(Deserialize, specta::Type)]
#[serde(untagged)]
pub enum EncryptionMethod {
    X25519(Vec<String>),
    Scrypt(String),
}

#[derive(Deserialize, specta::Type)]
pub enum DecryptionMethod {
    X25519,
    Scrypt,
}

#[tauri::command]
#[specta::specta]
pub async fn validate_key_text(text: String) -> Result<(), String> {
    match bech32::decode(&text) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("this is not a valid age key. {err}")),
    }
}

#[tauri::command]
#[specta::specta]
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
#[specta::specta]
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
#[specta::specta]
pub async fn encrypt_text(
    recipient: EncryptionMethod,
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let recipients: Vec<WildcardRecipient> = match recipient {
        EncryptionMethod::X25519(public_keys) => {
            let key_contents = state.with_vault(|vault| {
                public_keys
                    .iter()
                    .map(|key| vault.get_key(key).unwrap().contents.public.clone())
                    .collect::<Vec<String>>()
            })?;
            let should_encrypt_pq = key_contents.iter().all(|key| key.starts_with("age1pq"));
            let mut recipients: Vec<WildcardRecipient> = Vec::with_capacity(key_contents.len());
            for key in key_contents {
                if key.starts_with("age1pq") {
                    let hybrid_recipient =
                        HybridRecipient::from_string(&key).expect("key should be valid");
                    let recipient = if should_encrypt_pq {
                        WildcardRecipient::Hybrid(hybrid_recipient)
                    } else {
                        WildcardRecipient::X25519(hybrid_recipient.to_x25519())
                    };
                    recipients.push(recipient);
                } else {
                    recipients.push(WildcardRecipient::X25519(
                        age::x25519::Recipient::from_str(key.as_str())
                            .expect("key should be valid"),
                    ))
                }
            }
            recipients
        }
        EncryptionMethod::Scrypt(password) => {
            vec![WildcardRecipient::Scrypt(age::scrypt::Recipient::new(
                SecretString::from(password),
            ))]
        }
    };
    return crypto::encrypt_armored_text(&recipients, text).await;
}

#[tauri::command]
#[specta::specta]
pub async fn decrypt_text(
    private_key: String,
    text: String,
    method: DecryptionMethod,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let identity = match method {
        DecryptionMethod::X25519 => {
            let key_content = state.with_vault(|vault| {
                let key_metadata = vault.get_key(&private_key).unwrap();
                vault
                    .decrypt_secret(&key_metadata.contents.private.as_ref().unwrap())
                    .expect("should be able to decrypt secret")
                    .clone()
            })?;

            if key_content
                .expose_secret()
                .starts_with("AGE-SECRET-KEY-PQ-")
            {
                WildcardIdentity::Hybrid(HybridIdentity::from_string(key_content)?)
            } else {
                WildcardIdentity::X25519(
                    key_content
                        .expose_secret()
                        .parse::<age::x25519::Identity>()?,
                )
            }
        }
        DecryptionMethod::Scrypt => {
            WildcardIdentity::Scrypt(age::scrypt::Identity::new(SecretString::from(private_key)))
        }
    };

    return crypto::decrypt_armored_text(&identity, text).await;
}

#[tauri::command]
#[specta::specta]
pub async fn encrypt_file(
    recipient: EncryptionMethod,
    reader: tauri::ipc::Channel<FileOperationProgress>,
    files: Vec<String>,
    state: tauri::State<'_, AppState>,
    armor: Option<bool>,
) -> Result<(), String> {
    let armor = armor.unwrap_or(false);
    let recipients: Vec<WildcardRecipient> = match recipient {
        EncryptionMethod::X25519(public_keys) => {
            let key_contents = state.with_vault(|vault| {
                public_keys
                    .iter()
                    .map(|key| vault.get_key(key).unwrap().contents.public.clone())
                    .collect::<Vec<String>>()
            })?;
            let should_encrypt_pq = key_contents.iter().all(|key| key.starts_with("age1pq"));
            let mut recipients: Vec<WildcardRecipient> = Vec::with_capacity(key_contents.len());
            for key in key_contents {
                if key.starts_with("age1pq") {
                    let hybrid_recipient =
                        HybridRecipient::from_string(&key).expect("key should be valid");
                    let recipient = if should_encrypt_pq {
                        WildcardRecipient::Hybrid(hybrid_recipient)
                    } else {
                        WildcardRecipient::X25519(hybrid_recipient.to_x25519())
                    };
                    recipients.push(recipient);
                } else {
                    recipients.push(WildcardRecipient::X25519(
                        age::x25519::Recipient::from_str(key.as_str())
                            .expect("key should be valid"),
                    ))
                }
            }
            recipients
        }
        EncryptionMethod::Scrypt(password) => {
            vec![WildcardRecipient::Scrypt(age::scrypt::Recipient::new(
                SecretString::from(password),
            ))]
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
    let total_read_bytes_ptr = Arc::new(AtomicU64::new(0));
    let reader_ptr = Arc::new(reader);
    let mut output_paths = Vec::new();
    let cooldown = time::Duration::from_millis(100);
    for file in files {
        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = PathBuf::from(file.clone());
        let reader = reader_ptr.clone();
        let mut progress_interval = time::interval(cooldown);
        let progress_task = tauri::async_runtime::spawn(async move {
            loop {
                let _ = reader.send(FileOperationProgress {
                    // its okay if it doesnt send i'd rather the files just encrypt
                    read_bytes: total_read_bytes.load(Ordering::SeqCst),
                    total_bytes: total_bytes,
                    current_file: path.file_name().unwrap().to_str().unwrap().to_string(),
                });
                progress_interval.tick().await;
            }
        });

        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = PathBuf::from(file.clone());
        let output_path =
            crypto::encrypt_file(&recipients, &path.clone(), armor, move |processed_bytes| {
                total_read_bytes
                    .fetch_add(processed_bytes as u64, std::sync::atomic::Ordering::SeqCst);
            })
            .await
            .expect("failed to encrypt file");
        progress_task.abort();
        let _ = reader_ptr.clone().send(FileOperationProgress {
            // its okay if it doesnt send i'd rather the files just encrypt
            read_bytes: *file_sizes.get(&file).unwrap(),
            total_bytes: total_bytes,
            current_file: path.file_name().unwrap().to_str().unwrap().to_string(),
        });
        output_paths.push(output_path)
    }
    reveal_items_in_dir(output_paths).expect("failed to reveal item");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn decrypt_file(
    private_key: String,
    reader: tauri::ipc::Channel<FileOperationProgress>,
    files: Vec<String>,
    method: DecryptionMethod,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let identity = match method {
        DecryptionMethod::X25519 => {
            let key_content = state.with_vault(|vault| {
                let key_metadata = vault.get_key(&private_key).unwrap();
                vault
                    .decrypt_secret(&key_metadata.contents.private.as_ref().unwrap())
                    .expect("should be able to decrypt secret")
                    .clone()
            })?;

            if key_content
                .expose_secret()
                .starts_with("AGE-SECRET-KEY-PQ-")
            {
                WildcardIdentity::Hybrid(HybridIdentity::from_string(key_content)?)
            } else {
                WildcardIdentity::X25519(
                    key_content
                        .expose_secret()
                        .parse::<age::x25519::Identity>()?,
                )
            }
        }
        DecryptionMethod::Scrypt => {
            WildcardIdentity::Scrypt(age::scrypt::Identity::new(SecretString::from(private_key)))
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
    let total_read_bytes_ptr = Arc::new(AtomicU64::new(0));
    let reader_ptr = Arc::new(reader);
    let mut output_paths = Vec::new();
    let cooldown = time::Duration::from_millis(100);
    for file in files {
        let total_read_bytes = total_read_bytes_ptr.clone();
        let reader = reader_ptr.clone();
        let path_ptr = Arc::new(PathBuf::from_str(&file).unwrap());
        let path = path_ptr.clone();
        let mut progress_interval = time::interval(cooldown);
        let progress_task = tauri::async_runtime::spawn(async move {
            loop {
                progress_interval.tick().await;
                let _ = reader.send(FileOperationProgress {
                    // its okay if it doesnt send i'd rather the files just encrypt
                    read_bytes: total_read_bytes.load(Ordering::SeqCst),
                    total_bytes: total_bytes,
                    current_file: path.file_name().unwrap().to_str().unwrap().to_string(),
                });
            }
        });
        let is_armored = armor_check_file(&file).await?;
        let total_read_bytes = total_read_bytes_ptr.clone();
        let path = path_ptr;
        let output_path =
            crypto::decrypt_file(&identity, &path, is_armored, move |processed_bytes| {
                total_read_bytes.fetch_add(processed_bytes as u64, Ordering::SeqCst);
            })
            .await?;
        progress_task.abort();
        let reader = reader_ptr.clone();
        let _ = reader.send(FileOperationProgress {
            read_bytes: *file_sizes.get(&file).unwrap(),
            total_bytes: total_bytes,
            current_file: path.file_name().unwrap().to_str().unwrap().to_string(),
        }); // ensure that it "completes" on the frontend
        output_paths.push(output_path)
    }
    let _ = reader_ptr.send(FileOperationProgress {
        read_bytes: total_bytes,
        total_bytes: total_bytes,
        current_file: "".to_string(),
    }); // ensure that it "completes" on the frontend
    reveal_items_in_dir(output_paths).expect("failed to reveal item");
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub enum KeyFormat {
    X25519,
    PostQuantum,
}

#[tauri::command]
#[specta::specta]
pub async fn generate_keypair(
    name: String,
    format: Option<KeyFormat>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if name.len() == 0 {
        return Err("no name set".to_string());
    }
    state.with_vault(|vault| {
        let keypair = match format {
            Some(KeyFormat::X25519) => vault.generate_x25519_keypair(name),
            _ => vault.generate_keypair(name), // if none or if PostQuantum
        }?;
        vault.put_key(keypair)?;
        Ok::<(), String>(())
    })??;
    let _ = state.save_vault().await?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn downgrade_hybrid_public_key(public_key: String) -> Result<String, String> {
    let recipient = HybridRecipient::from_string(&public_key)?.to_x25519();
    Ok(recipient.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn generate_passphrase() -> String {
    let words: Vec<&str> = WORDLIST
        .lines()
        .map(|line| line.split_whitespace().last().unwrap())
        .collect();

    let passphrase = Zeroizing::new(
        words
            .choose_multiple(&mut rand::rng(), 12)
            .map(|str| *str)
            .collect::<Vec<&str>>()
            .join("-"),
    );

    passphrase.to_string()
}
