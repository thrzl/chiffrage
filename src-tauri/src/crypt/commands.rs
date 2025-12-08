use crate::crypt;
use crate::store::{fetch_keys, KeyMetadata, KeyType};
use crate::AppState;
use chacha20poly1305::aead::Key;
use secrecy::{ExposeSecret, SecretString};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::reveal_item_in_dir;

#[tauri::command]
pub fn encrypt_file_cmd(
    public_keys: Vec<String>,
    app_handle: tauri::AppHandle,
    state: tauri::State<Mutex<AppState>>,
) {
    let state = state.lock().expect("failed to get lock on state");
    let vault = state.vault.as_ref().expect("vault not initialized");

    let key_contents = public_keys
        .iter()
        .map(|key| {
            vault
                .load_secret(key.to_owned())
                .unwrap()
                .unwrap()
                .expose_secret()
                .to_string() // what a mess
        })
        .collect::<Vec<String>>();
    println!("{:?}", key_contents);
    app_handle.dialog().file().pick_file(|file| {
        let file_path = file.expect("user did not pick a file");
        let output_path = crypt::encrypt_file(
            key_contents,
            file_path
                .clone()
                .into_path()
                .expect("failed to get file as PathBuf"),
        )
        .expect("failed to encrypt file");
        reveal_item_in_dir(output_path.as_path()).expect("failed to reveal item");
    })
    // let file_path = Dialog::file().blocking_pick_file();
}

#[tauri::command]
pub fn generate_keypair(id: String, state: tauri::State<Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault = state.vault.as_mut().expect("failed to load vault");
    let keypair = crypt::generate_key();
    vault.put_secret(format!("priv:{}", id), keypair.private_key)?;
    vault.put_secret(
        format!("pub:{}", id),
        SecretString::from(keypair.public_key),
    )?;
    state
        .index
        .insert(
            format!("pub:{}", id),
            serde_cbor::to_vec(&KeyMetadata {
                name: format!("pub:{}", id),
                key_type: KeyType::Public,
                date_created: std::time::SystemTime::now(),
            })
            .expect("failed to serialize key metadata"),
        )
        .expect("failed to insert pulic key");
    state
        .index
        .insert(
            format!("priv:{}", id),
            serde_cbor::to_vec(&KeyMetadata {
                name: format!("priv:{}", id),
                key_type: KeyType::Public,
                date_created: std::time::SystemTime::now(),
            })
            .expect("failed to serialize key metadata"),
        )
        .expect("failed to insert private key");
    Ok(())
}
