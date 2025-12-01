use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use secrecy::SecretString;
use std::sync::Mutex;

#[tauri::command]
pub fn vault_exists() -> bool {
    dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb")
        .exists()
}

#[tauri::command]
pub fn load_vault(state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_location = dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb");
    let vault_load = Vault::load_vault(vault_location.to_str().unwrap());
    if let Err(error) = vault_load {
        return Err(error);
    }
    state.vault = Some(vault_load.unwrap());
    Ok(())
}

#[tauri::command]
pub fn create_vault(password: String) -> Result<(), String> {
    let password = SecretString::from(password);
    let vault_path = dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb");

    let vault_location = vault_path.to_str().unwrap();
    let vault = Vault::create_vault(vault_location, &password);
    vault.save_vault(&vault.file);
    Ok(())
}

#[tauri::command]
pub fn fetch_keys(state: tauri::State<Mutex<AppState>>) -> Vec<KeyMetadata> {
    let state = state.lock().unwrap();
    let index = &state.index;

    let items: Vec<KeyMetadata> = index
        .iter()
        .map(|entry| match entry {
            Ok(data) => Some(serde_cbor::from_slice::<KeyMetadata>(&data.1).unwrap()),
            Err(_) => None,
        })
        .filter(|entry| entry.is_some())
        .map(|entry| entry.unwrap())
        .collect();

    return items;
}
