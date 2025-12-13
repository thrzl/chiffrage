use crate::crypto;
use crate::AppState;
use secrecy::ExposeSecret;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri_plugin_opener::reveal_item_in_dir;

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
                    .get_key(key.to_owned())
                    .unwrap()
                    .contents
                    .public
                    .clone()
            })
            .collect::<Vec<String>>()
    };
    let path = PathBuf::from(file);
    let output_path = crypto::encrypt_file(key_contents, path, reader)
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
        let key_content = vault.get_key(private_key).unwrap();
        vault
            .decrypt_secret(&key_content.contents.private.as_ref().unwrap())
            .unwrap()
            .clone()
    };
    let output_path = crypto::decrypt_file(
        key_content.expose_secret().to_string(),
        PathBuf::from(file),
        reader,
    )
    .await
    .expect("failed to decrypt file");
    reveal_item_in_dir(output_path.as_path()).expect("failed to reveal item");
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
