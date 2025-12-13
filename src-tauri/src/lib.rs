// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#![feature(path_add_extension)]
mod crypt;
mod store;
use serde::Serialize;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

use tauri_plugin_store::StoreExt;

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
    vault: Option<store::Vault>,
    first_open: bool,
}

#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
enum FileProcessingEvent {
    Progress { percent: usize },
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
            crypt::generate_keypair,
            crypt::encrypt_text,
            crypt::encrypt_file_cmd,
            crypt::decrypt_file_cmd,
            store::export_key
        ])
        .setup(|app| {
            let store = app
                .store_builder("index.json")
                .auto_save(Duration::from_millis(100))
                .build()?;
            let first_open = !app.path().app_data_dir()?.join("vault.cb").exists();
            app.manage(Mutex::new(AppState {
                vault: None,
                first_open,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
