// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod crypt;
mod store;
use secrecy::SecretString;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};
use tauri::{Manager, WebviewUrl};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

use crate::store::KeyMetadata;

struct AppState {
    vault: Option<store::Vault>,
    first_open: bool,
    index: sled::Tree,
}

#[tauri::command]
fn is_first_open(state: tauri::State<Mutex<AppState>>) -> bool {
    return state.lock().unwrap().first_open;
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn prompt_password(app: tauri::AppHandle) -> SecretString {
    let answer = app
        .dialog()
        .message("Tauri is Awesome")
        .title("Tauri is Awesome")
        .blocking_show();
    tauri::WebviewWindowBuilder::new(
        &app,
        "vault-unlock",
        WebviewUrl::App(PathBuf::from("prompt.html")),
    )
    .always_on_top(true)
    .title("unlock vault")
    .build()
    .expect("should have opened idk whats wrong");
    return SecretString::from("miracle baby");
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            store::fetch_keys,
            store::unlock_vault,
            is_first_open,
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
