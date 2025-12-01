// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod crypt;
mod store;
use dialog::DialogBox;
use secrecy::SecretString;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

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
    // let answer = app
    //     .dialog()
    //     .message("Tauri is Awesome")
    //     .title("Tauri is Awesome")
    //     .blocking_show();
    // tauri::WebviewWindowBuilder::new(
    //     &app,
    //     "vault-unlock",
    //     WebviewUrl::App(PathBuf::from("prompt.html")),
    // )
    // .always_on_top(true)
    // .title("unlock vault")
    // .build()
    // .expect("should have opened idk whats wrong");
    let pass = dialog::Password::new("enter your vault password")
        .title("vault auth")
        .show()
        .expect("couldn't ask for password")
        .unwrap_or("".to_string());
    return SecretString::from(pass);
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
            is_first_open,
            store::fetch_keys,
            store::load_vault,
            store::create_vault,
            store::vault_exists,
            crypt::generate_keypair,
            crypt::encrypt_text,
        ])
        .manage(Mutex::new(AppState {
            vault: None,
            index,
            first_open,
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
