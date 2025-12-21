use crate::crypto::{self, WildcardIdentity, WildcardRecipient};
use crate::AppState;
use futures_util::future::join_all;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use tauri_plugin_opener::reveal_items_in_dir;
use tokio::fs::metadata;

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

#[tauri::command]
pub async fn encrypt_file(
    recipient: EncryptionMethod,
    reader: tauri::ipc::Channel<serde_json::Value>,
    files: Vec<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
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
    let total_bytes: u64 = join_all(files.iter().map(async |path| {
        metadata(path)
            .await
            .expect("failed to get file metadata")
            .len()
    }))
    .await
    .into_iter()
    .sum();
    let total_read_bytes = Arc::new(AtomicUsize::new(0));
    let reader = Arc::new(reader);
    let mut output_paths = Vec::new();

    for file in files {
        let total_read_bytes = total_read_bytes.clone();
        let path = PathBuf::from(file);
        let reader = reader.clone();
        let output_path =
            crypto::encrypt_file(&recipients, &path.clone(), move |processed_bytes| {
                total_read_bytes.fetch_add(processed_bytes, std::sync::atomic::Ordering::SeqCst);
                let _ = reader.send(
                    json!({ // its okay if it doesnt send i'd rather the files just encrypt
                        "read_bytes": total_read_bytes,
                        "total_bytes": total_bytes,
                        "current_file": path.file_name().unwrap().to_str().unwrap()
                    }),
                );
            })
            .await
            .expect("failed to encrypt file");
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
    let total_read_bytes = Arc::new(AtomicUsize::new(0));
    let reader_ptr = Arc::new(reader);
    let mut output_paths = Vec::new();
    for file in files {
        let total_read_bytes = total_read_bytes.clone();
        let reader = reader_ptr.clone();
        let path = PathBuf::from(file.clone());
        let output_path = crypto::decrypt_file(&identity, &path.clone(), move |processed_bytes| {
            total_read_bytes.fetch_add(processed_bytes, std::sync::atomic::Ordering::SeqCst);
            let _ = reader.send(json!({
                "read_bytes": total_read_bytes,
                "total_bytes": total_bytes,
                "current_file": path.file_name().unwrap().to_str().unwrap()
            }));
        })
        .await?;

        let reader = reader_ptr.clone();
        let _ = reader.send(json!({
            "read_bytes": file_sizes.get(&file).unwrap(),
            "total_bytes": total_bytes,
            "current_file": PathBuf::from(file).file_name().unwrap().to_str().unwrap()
        })); // ensure that it "completes" on the frontend
        output_paths.push(output_path)
    }
    reveal_items_in_dir(output_paths).expect("failed to reveal item");
    Ok(())
}

#[tauri::command]
pub async fn generate_keypair(
    name: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
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
