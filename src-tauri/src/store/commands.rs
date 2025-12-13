use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Mutex;
use tauri_plugin_store::StoreExt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tauri::command]
pub fn vault_exists() -> bool {
    dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb")
        .exists()
}

#[tauri::command]
pub fn load_vault(state: tauri::State<Mutex<AppState>>, password: String) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_location = dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb");
    let vault_load = Vault::load_vault(
        vault_location.to_str().unwrap(),
        SecretString::from(password),
    );
    if let Err(error) = vault_load {
        return Err(error);
    }
    state.vault = Some(vault_load.unwrap());
    Ok(())
}

#[tauri::command]
pub async fn create_vault(password: String) -> Result<(), String> {
    let password = SecretString::from(password);
    let vault_path = dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb");

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
        let vault = state.vault.as_ref().expect("vault not initialized");

        vault.load_secret(key).expect("could not load key")
    };
    key_file
        .write_all(key_content.expose_secret().as_bytes())
        .await
        .expect("failed to write file");
    key_file.flush().await.expect("failed to flush file buffer");
    Ok(())
}
