use crate::crypto;
use crate::AppState;
use futures_util::future::join_all;
use secrecy::ExposeSecret;
use serde_json::json;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};
use tauri_plugin_opener::reveal_items_in_dir;
use tokio::fs::metadata;

#[tauri::command]
pub async fn encrypt_file_cmd(
    public_keys: Vec<String>,
    reader: tauri::ipc::Channel<serde_json::Value>,
    files: Vec<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ()> {
    let key_contents = {
        let state = state.lock().expect("failed to get lock on state");
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        public_keys
            .iter()
            .map(|key| vault.get_key(key).unwrap().contents.public.clone())
            .collect::<Vec<String>>()
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
            crypto::encrypt_file(&key_contents, &path.clone(), move |processed_bytes| {
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
pub async fn decrypt_file_cmd(
    private_key: String,
    reader: tauri::ipc::Channel<serde_json::Value>,
    files: Vec<String>,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ()> {
    let key_content = {
        let state = state.lock().expect("failed to get lock on state");
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        let key_content = vault.get_key(&private_key).unwrap();
        vault
            .decrypt_secret(&key_content.contents.private.as_ref().unwrap())
            .unwrap()
            .clone()
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
        let reader = reader.clone();
        let path = PathBuf::from(file);
        let output_path = crypto::decrypt_file(
            key_content.expose_secret().to_string(),
            &path.clone(),
            move |processed_bytes| {
                total_read_bytes.fetch_add(processed_bytes, std::sync::atomic::Ordering::SeqCst);
                let _ = reader.send(json!({
                    "read_bytes": total_read_bytes,
                    "total_bytes": total_bytes,
                    "current_file": path.file_name().unwrap().to_str().unwrap()
                }));
            },
        )
        .await
        .expect("failed to decrypt file");
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
