use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use age::x25519::Identity;
use secrecy::{ExposeSecret, SecretString};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri_plugin_store::StoreExt;
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
pub fn fetch_keys(app_handle: tauri::AppHandle) -> Vec<KeyMetadata> {
    let index = app_handle.store("index.json").expect("failed to get store");

    let items: Vec<KeyMetadata> = index
        .values()
        .iter()
        .map(|value| {
            serde_json::from_value::<KeyMetadata>(value.clone())
                .expect("failed to deserialize data")
        })
        .collect();

    return items;
}

#[tauri::command]
pub async fn export_key(
    key: String,
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let mut key_file = File::create(path).await.expect("failed to open key file");
    let key_content = {
        let state = match state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(), // idc gangalang
        };
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };

        vault.load_secret(key).expect("could not load key")
    };
    key_file
        .write_all(key_content.expose_secret().as_bytes())
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
) -> Result<(), String> {
    let mut key_file = File::open(path).await.expect("failed to open key file");
    let mut key_content = String::new();

    key_file
        .read_to_string(&mut key_content)
        .await
        .expect("failed to read key file");

    let is_private = key_content.starts_with("AGE-SECRET-KEY");

    let state = match state.lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    };
    let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
    {
        let mut vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault
            .put_secret(name.clone(), SecretString::from(key_content.clone()))
            .expect("failed to add secret to vault");
        if is_private {
            vault
                .put_secret(
                    name.clone(),
                    SecretString::from(
                        Identity::from_str(key_content.clone().as_str())
                            .expect("failed to parse key")
                            .to_public()
                            .to_string(),
                    ),
                )
                .expect("failed to add secret to vault");
        }
    }
    tauri::async_runtime::spawn_blocking(move || vault_handle.lock().unwrap().save_vault());
    Ok(())
}
