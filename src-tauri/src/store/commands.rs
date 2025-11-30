use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use std::sync::Mutex;

#[tauri::command]
pub fn unlock_vault(password: String, state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_location = dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb");
    let vault_load =
        Vault::load_vault(vault_location.to_str().unwrap()).unwrap_or(Vault::create_vault(
            vault_location.to_str().unwrap(),
            &secrecy::SecretString::from(password),
        ));
    state.vault = Some(vault_load);
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
