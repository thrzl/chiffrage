use crate::crypt;
use crate::store::{KeyMetadata, KeyType};
use crate::AppState;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri_plugin_opener::reveal_item_in_dir;
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub async fn encrypt_file_cmd(
    public_keys: Vec<String>,
    reader: tauri::ipc::Channel<f64>,
    file: String,
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
            .map(|key| {
                vault
                    .load_secret(key.to_owned())
                    .unwrap()
                    .expose_secret()
                    .to_string() // what a mess
            })
            .collect::<Vec<String>>()
    };
    let path = PathBuf::from(file);
    let output_path = crypt::encrypt_file(key_contents, path, reader)
        .await
        .expect("failed to encrypt file");
    reveal_item_in_dir(output_path.as_path()).expect("failed to reveal item");
    Ok(())
    // let file_path = Dialog::file().blocking_pick_file();
}

#[tauri::command]
pub async fn decrypt_file_cmd(
    private_key: String,
    reader: tauri::ipc::Channel<f64>,
    file: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), ()> {
    let key_content = {
        let state = state.lock().expect("failed to get lock on state");
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault.load_secret(private_key).unwrap().clone()
    };
    let output_path =
        crypt::decrypt_file(key_content.expose_secret().to_string(), PathBuf::from(file))
            .await
            .expect("failed to encrypt file");
    reveal_item_in_dir(output_path.as_path()).expect("failed to reveal item");
    Ok(())
}

#[tauri::command]
pub async fn generate_keypair(
    id: String,
    state: tauri::State<'_, Mutex<AppState>>,
    app_handle: tauri::AppHandle,
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
        let keypair = crypt::generate_key();
        vault.put_secret(format!("priv:{}", id), keypair.private_key)?;
        vault.put_secret(
            format!("pub:{}", id),
            SecretString::from(keypair.public_key),
        )?;
    }
    tauri::async_runtime::spawn_blocking(move || {
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault.save_vault();
    });

    let index = app_handle
        .store("index.json")
        .expect("failed to open key index");
    index.set(
        format!("pub:{}", id),
        json!(&KeyMetadata {
            name: format!("pub:{}", id),
            key_type: KeyType::Public,
            date_created: std::time::SystemTime::now(),
        }),
    );
    index.set(
        format!("priv:{}", id),
        json!(&KeyMetadata {
            name: format!("priv:{}", id),
            key_type: KeyType::Public,
            date_created: std::time::SystemTime::now(),
        }),
    );
    Ok(())
}
