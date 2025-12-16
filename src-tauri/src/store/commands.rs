use crate::store::{KeyMetadata, Vault};
use crate::AppState;
use age::x25519::{Identity, Recipient};
use secrecy::ExposeSecret;
use secrecy::SecretString;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{Listener, Manager};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;

#[tauri::command]
pub fn vault_exists(app_handle: tauri::AppHandle) -> bool {
    let res = app_handle
        .path()
        .app_data_dir()
        .unwrap()
        .join("vault.cb")
        .exists();
    println!("{:?}", res);
    res
}

#[tauri::command]
pub fn vault_unlocked(state: tauri::State<Mutex<AppState>>) -> bool {
    let state = state.lock().unwrap_or_else(|poisoned| {
        state.clear_poison();
        poisoned.into_inner()
    });
    let vault = state
        .vault
        .as_ref()
        .expect("no vault")
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    vault.key.is_some()
}

#[tauri::command]
pub fn load_vault(
    state: tauri::State<Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    let vault_location = app_handle.path().app_data_dir().unwrap().join("vault.cb");
    let vault_load = Vault::load_vault(vault_location.to_str().unwrap());
    if let Err(error) = vault_load {
        return Err(error);
    }
    state.vault = Some(Arc::new(Mutex::new(vault_load.unwrap())));
    Ok(())
}

#[tauri::command]
pub async fn create_vault(password: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let password = SecretString::from(password);
    let vault_path = app_handle.path().app_data_dir().unwrap().join("vault.cb");

    let vault_location = vault_path.to_str().unwrap();
    let vault = Vault::create_vault(vault_location, &password);
    vault.save_vault();
    Ok(())
}

#[tauri::command]
pub fn fetch_keys(state: tauri::State<Mutex<AppState>>) -> Vec<KeyMetadata> {
    let items = {
        let state = state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let vault_handle = state.vault.as_ref().expect("vault not initialized");
        let vault = vault_handle.lock().unwrap();
        vault
            .file
            .secrets
            .values()
            .cloned()
            .map(|key| {
                key.redacted() // we dont need to send private
            })
            .collect::<Vec<KeyMetadata>>()
    };

    return items;
}

#[tauri::command]
pub fn fetch_key(name: String, state: tauri::State<Mutex<AppState>>) -> Option<KeyMetadata> {
    let state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let vault_handle = state.vault.as_ref().expect("vault not initialized");
    let vault = vault_handle
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    vault.get_key(&name).cloned()
}

#[tauri::command]
pub fn delete_key(id: String, state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let state = state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let vault_handle = state.vault.as_ref().expect("vault not initialized");
    let mut vault = vault_handle.lock().unwrap();
    vault.delete_key(id);
    vault.save_vault();
    Ok(())
}

#[tauri::command]
pub async fn export_key(
    key: String,
    path: String,
    key_type: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let key_content = {
        let state = match state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(), // idc gangalang
        };
        let vault_handle = state.vault.as_ref().expect("vault not initialized").clone();
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };

        let key_meta = vault.get_key(&key).expect("could not load key");
        let key_contents = key_meta.contents.clone();
        match key_type.as_str() {
            "public" => key_contents.public,
            "private" => vault
                .decrypt_secret(&key_contents.private.expect("no private key!"))
                .unwrap()
                .expose_secret()
                .to_string(), // should be guarded against in frontend
            &_ => return Err("invalid key type".to_string()),
        }
    };

    let mut key_file = File::create(path).await.expect("failed to open key file");
    key_file
        .write_all(key_content.as_bytes())
        .await
        .expect("failed to write file");
    key_file.flush().await.expect("failed to flush file buffer");
    Ok(())
}

#[tauri::command]
pub async fn import_key(
    name: String,
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    println!("running key import");
    let mut key_file = File::open(path).await.expect("failed to open key file");
    let mut key_content = String::new();

    key_file
        .read_to_string(&mut key_content)
        .await
        .expect("failed to read key file");

    let is_private = key_content.starts_with("AGE-SECRET-KEY");

    let vault_handle = {
        let state = match state.lock() {
            Ok(state) => state,
            Err(poisoned) => poisoned.into_inner(),
        };
        state.vault.as_ref().expect("vault not initialized").clone()
    };
    {
        let mut vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        if is_private {
            let identity =
                Identity::from_str(key_content.clone().as_str()).expect("failed to parse key");
            let key = vault.new_key(
                name,
                identity.to_public().to_string(),
                Some(SecretString::from(identity.to_string())),
            );
            vault.put_key(key)?;
        } else {
            let key = vault.new_key(
                name,
                Recipient::from_str(key_content.clone().as_str())
                    .expect("failed to parse public key")
                    .to_string(),
                None,
            );
            vault.put_key(key)?;
        }
    }
    tauri::async_runtime::spawn_blocking(move || {
        let vault = match vault_handle.lock() {
            Ok(vault) => vault,
            Err(poisoned) => poisoned.into_inner(),
        };
        vault.save_vault();
        println!("saved vault")
    })
    .await
    .expect("failed to save vault");
    Ok("key import complete".to_string())
}

#[tauri::command]
pub async fn authenticate(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let webview = tauri::WebviewWindowBuilder::new(
        &app_handle,
        "vault-unlock",
        tauri::WebviewUrl::App("unlock".into()),
    )
    .build()
    .expect("failed to open auth window");
    let (tx, rx) = oneshot::channel();
    let tx = Mutex::new(Some(tx));
    webview.listen("authenticate", move |event| {
        if let Some(tx) = tx
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        // wtf bro
        {
            let _ = tx.send(event.payload().to_string());
        }
    });
    let password = SecretString::from(match rx.await {
        Ok(password) => serde_json::from_str::<String>(&password).unwrap(),
        Err(error) => return Err(error.to_string()),
    });
    println!("{:?}", password.expose_secret());
    let _ = webview.close();
    let state = state.lock().unwrap_or_else(|p| p.into_inner());
    let mut vault = state
        .vault
        .as_ref()
        .clone()
        .expect("vault not initialized")
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    vault.set_vault_key(password).expect("incorrect vault key");

    Ok(())
}
