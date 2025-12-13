// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#![feature(path_add_extension)]
mod crypto;
mod store;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Manager;

// im ngl idk what im doin
pub fn set_timeout<F>(delay_ms: u64, f: F)
where
    F: FnOnce() + Send + 'static,
{
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        f();
    });
}

struct AppState {
    vault: Option<Arc<Mutex<store::Vault>>>,
    first_open: bool,
}

#[tauri::command]
fn is_first_open(state: tauri::State<Mutex<AppState>>) -> bool {
    return state.lock().unwrap().first_open;
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            is_first_open,
            store::fetch_keys,
            store::load_vault,
            store::create_vault,
            store::vault_exists,
            crypto::generate_keypair,
            crypto::encrypt_text,
            crypto::encrypt_file_cmd,
            crypto::decrypt_file_cmd,
            store::export_key,
            store::import_key
        ])
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("could not find app data directory");
            let first_open = !app_data_dir.join("vault.cb").exists();
            if !app_data_dir.exists() {
                std::fs::create_dir(app_data_dir).expect("failed to create app data directory")
            }
            app.manage(Mutex::new(AppState {
                vault: None,
                first_open,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
