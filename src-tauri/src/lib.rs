// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod crypt;
mod store;
use secrecy::SecretString;
use std::sync::Mutex;

use crate::store::KeyMetadata;

struct AppState {
    vault: Option<store::Vault>,
    first_open: bool,
    index: sled::Tree,
}

#[tauri::command]
fn generate_keypair(id: String, state: tauri::State<Mutex<AppState>>) {
    let mut state = state.lock().unwrap();
    let vault = state.vault.as_mut().expect("failed to load vault");
    let keypair = crypt::generate_key();
    vault.put_secret(format!("priv:{}", id), keypair.private_key);
    vault.put_secret(
        format!("pub:{}", id),
        SecretString::from(keypair.public_key),
    );
    state.index.insert(
        format!("pub:{}", id),
        serde_cbor::to_vec(&KeyMetadata {
            name: format!("pub:{}", id),
            key_type: store::KeyType::Public,
            date_created: std::time::SystemTime::now(),
        })
        .expect("failed to serialize key metadata"),
    );
    state.index.insert(
        format!("priv:{}", id),
        serde_cbor::to_vec(&KeyMetadata {
            name: format!("priv:{}", id),
            key_type: store::KeyType::Public,
            date_created: std::time::SystemTime::now(),
        })
        .expect("failed to serialize key metadata"),
    );
}

#[tauri::command]
fn is_first_open(state: tauri::State<Mutex<AppState>>) -> bool {
    return state.lock().unwrap().first_open;
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn unlock_vault(password: String, state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_load = store::Vault::load_vault(
        dirs::data_dir()
            .expect("could not find app data directory")
            .join("chiffrage/vault.cb")
            .to_str()
            .unwrap(),
        &secrecy::SecretString::from(password),
    );
    match vault_load {
        Ok(vault) => {
            println!("{:?}", &vault.list_keys());
            state.vault = Some(vault);
        }
        Err(value) => return Err(value),
    };
    Ok(())
}

#[tauri::command]
fn fetch_keys(state: tauri::State<Mutex<AppState>>) -> Vec<store::KeyMetadata> {
    let state = state.lock().unwrap();
    let index = &state.index;

    let items: Vec<store::KeyMetadata> = index
        .iter()
        .map(|entry| match entry {
            Ok(data) => Some(serde_cbor::from_slice::<store::KeyMetadata>(&data.1).unwrap()),
            Err(_) => None,
        })
        .filter(|entry| entry.is_some())
        .map(|entry| entry.unwrap())
        .collect();

    return items;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let first_open = !dirs::data_dir()
        .expect("could not find app data directory")
        .join("chiffrage/vault.cb")
        .exists();
    let db_path = dirs::data_dir().unwrap().join("chiffrage/data.sled");
    let db = sled::open(dirs::data_dir().unwrap().join("chiffrage/data.sled"))
        .expect("failed to open sled metadata store");
    let index = db.open_tree("keys").expect("failed to open sled tree");
    println!("first open: {:?}", first_open);
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            fetch_keys,
            is_first_open,
            unlock_vault,
            generate_keypair,
            crypt::encrypt_text
        ])
        .manage(Mutex::new(AppState {
            vault: None,
            index,
            first_open,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
