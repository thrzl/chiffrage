// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod crypto;
mod store;
use specta_typescript::{BigIntExportBehavior, Typescript};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tauri::Manager;
use tauri_specta::{collect_commands, collect_events};

use crate::store::Vault;

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
#[specta::specta]
fn is_first_open(state: tauri::State<Mutex<AppState>>) -> bool {
    return state.lock().unwrap().first_open;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let command_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            is_first_open,
            store::fetch_keys,
            store::load_vault,
            store::create_vault,
            store::vault_exists,
            crypto::generate_keypair,
            crypto::commands::encrypt_file,
            crypto::commands::decrypt_file,
            crypto::generate_passphrase,
            store::export_key,
            store::import_key,
            store::delete_key,
            store::fetch_key,
            store::authenticate,
            store::vault_unlocked,
            store::import_key_text,
            store::check_keyfile_type,
            store::lock_vault,
            crypto::commands::validate_key_file,
            crypto::commands::validate_key_text,
            crypto::commands::armor_check_text,
            crypto::commands::decrypt_text,
            crypto::commands::encrypt_text
        ])
        .events(collect_events![store::VaultStatusUpdate]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    command_builder
        .export(
            Typescript::new().bigint(BigIntExportBehavior::Number),
            "../src/lib/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            app.get_webview_window("main").unwrap().set_focus().unwrap();
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(command_builder.invoke_handler())
        .setup(move |app| {
            command_builder.mount_events(app);

            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("could not find app data directory");
            let vault_path = app_data_dir.join("vault.cb");
            let first_open = !vault_path.exists();
            if first_open && !app_data_dir.exists() {
                std::fs::create_dir(app_data_dir).expect("failed to create app data directory")
            }
            app.manage(Mutex::new(AppState {
                vault: if first_open {
                    None
                } else {
                    Some(Arc::new(Mutex::new(
                        Vault::load_vault(vault_path.to_str().unwrap())
                            .expect("failed to initialize vault"),
                    )))
                },
                first_open,
            }));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
