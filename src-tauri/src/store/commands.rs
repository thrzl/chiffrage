use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use age::x25519::{Identity, Recipient};
use secrecy::SecretString;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tauri::command]
pub fn vault_exists(app_handle: tauri::AppHandle) -> bool {
    app_handle
        .path()
        .app_data_dir()
        .unwrap()
        .join("vault.cb")
        .exists()
}

#[tauri::command]
pub fn load_vault(
    state: tauri::State<Mutex<AppState>>,
    password: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_location = app_handle.path().app_data_dir().unwrap().join("vault.cb");
    let vault_load = Vault::load_vault(
        vault_location.to_str().unwrap(),
        SecretString::from(password),
    );
    if let Err(error) = vault_load {
        return Err(error);
    }
    state.vault = Some(Arc::new(Mutex::new(vault_load.unwrap())));
    Ok(())
}

#[tauri::command]
pub async fn create_vault(password: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let password = SecretString::from(password);
    let vault_path = app_handle.path().app_data_dir().unwrap().join("vault.cb");

    let vault_location = vault_path.to_str().unwrap();
    let vault = Vault::create_vault(vault_location, &password);
    vault.save_vault();
    Ok(())
}

#[tauri::command]
pub fn fetch_keys(state: tauri::State<Mutex<AppState>>) -> Vec<KeyMetadata> {
    let items = {
        let state = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let vault_handle = state.vault.as_ref().expect("vault not initialized");
        let vault = vault_handle.lock().unwrap();
        vault
            .file
            .secrets
            .values()
            .cloned()
            .map(|mut key| {
                key.redact(); // we dont need to send private
                key
            })
            .collect::<Vec<KeyMetadata>>()
    };

    return items;
}

#[tauri::command]
pub fn fetch_key(name: String, state: tauri::State<Mutex<AppState>>) -> Option<KeyMetadata> {
    let state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let vault_handle = state.vault.as_ref().expect("vault not initialized");
    let vault = vault_handle.lock().unwrap();
    vault.get_key(name).cloned()
}

#[tauri::command]
pub fn delete_key(name: String, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let vault_handle = state.vault.as_ref().expect("vault not initialized");
    let mut vault = vault_handle.lock().unwrap();
    vault.delete_key(name);
    vault.save_vault();
    Ok(())
}

#[tauri::command]
pub async fn export_key(
    key: String,
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut key_file = File::create(path).await.expect("failed to open key file");
    let key = {
        let state = match state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(), // idc gangalang
        };
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };

        let key_meta = vault.get_key(key).clone().expect("could not load key");
        key_meta.contents.public.clone()
    };
    // ! this needs to be rewritten to allow choosing between public and private
    key_file
        .write_all(key.as_bytes())
        .await
        .expect("failed to write file");
    key_file.flush().await.expect("failed to flush file buffer");
    Ok(())
}

#[tauri::command]
pub async fn import_key(
    name: String,
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("running key import");
    let mut key_file = File::open(path).await.expect("failed to open key file");
    let mut key_content = String::new();

    key_file
        .read_to_string(&mut key_content)
        .await
        .expect("failed to read key file");

    let is_private = key_content.starts_with("AGE-SECRET-KEY");

    let vault_handle = {
        let state = match state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.vault.as_ref().expect("vault not initialized").clone()
    };
    {
        let mut vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        if is_private {
            let identity =
                Identity::from_str(key_content.clone().as_str()).expect("failed to parse key");
            let key = vault.new_key(
                name,
                identity.to_public().to_string(),
                Some(SecretString::from(identity.to_string())),
            );
            vault.put_key(key)?;
        } else {
            let key = vault.new_key(
                name,
                Recipient::from_str(key_content.clone().as_str())
                    .expect("failed to parse public key")
                    .to_string(),
                None,
            );
            vault.put_key(key)?;
        }
    }
    tauri::async_runtime::spawn_blocking(move || {
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault.save_vault();
        println!("saved vault")
    })
    .await
    .expect("failed to save vault");
    Ok("key import complete".to_string())
}
