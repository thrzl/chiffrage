use crate::crypto::hybrid::HybridIdentity;
use crate::crypto::WildcardIdentity;
use crate::store::{KeyMetadata, Vault, VaultStatusUpdate};
use crate::AppState;
use age::x25519::{Identity, Recipient};
use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Listener, Manager};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;

#[tauri::command]
#[specta::specta]
pub fn vault_exists(app_handle: tauri::AppHandle) -> bool {
    let res = app_handle
        .path()
        .app_data_dir()
        .unwrap()
        .join("vault.cb")
        .exists();
    res
}

#[tauri::command]
#[specta::specta]
pub fn vault_unlocked(state: tauri::State<AppState>) -> bool {
    let vault_status = state.with_vault(|vault| vault.key.is_some());

    vault_status.unwrap_or(false)
}

#[tauri::command]
#[specta::specta]
pub fn load_vault(
    state: tauri::State<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let vault_location = app_handle.path().app_data_dir().unwrap().join("vault.cb");
    let vault_load = Vault::load_vault(vault_location.to_str().unwrap());
    if let Err(error) = vault_load {
        return Err(error);
    }
    let mut vault = state.get_vault();
    *vault = Some(vault_load.unwrap());
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn create_vault(password: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let password = SecretString::from(password);
    let vault_path = app_handle.path().app_data_dir().unwrap().join("vault.cb");

    let vault_location = vault_path.to_str().unwrap();
    let mut vault = Vault::create_vault(vault_location, &password)?;
    tauri::async_runtime::spawn_blocking(move || vault.save_vault())
        .await
        .map_err(|e| e.to_string())??; // bro wdf lmaoooo
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn fetch_keys(state: tauri::State<AppState>) -> Vec<KeyMetadata> {
    let items = state.with_vault(|vault| {
        vault
            .file
            .secrets
            .values()
            .cloned()
            .map(|key: KeyMetadata| {
                key.redacted() // we dont need to send private
            })
            .collect::<Vec<KeyMetadata>>()
    });

    return items.unwrap_or(vec![]);
}

#[tauri::command]
#[specta::specta]
pub fn fetch_key(name: String, state: tauri::State<AppState>) -> Option<KeyMetadata> {
    state
        .with_vault(|vault| vault.get_key(&name).cloned())
        .unwrap_or(None)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_key(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.with_vault(|vault| vault.delete_key(id))?;
    state.save_vault().await?;
    Ok(())
}

#[derive(specta::Type, Deserialize)]
pub enum KeyExportMode {
    PostQuantum,
    X25519,
}

#[tauri::command]
#[specta::specta]
pub async fn export_key(
    key: String,
    path: String,
    mode: KeyExportMode,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let key_content = {
        let raw_key_content = state.with_vault(|vault| {
            let key_meta = vault.get_key(&key).expect("could not load key");
            let key_contents = key_meta.contents.clone();
            SecretString::from(
                vault
                    .decrypt_secret(&key_contents.private.expect("no private key!"))
                    .unwrap()
                    .expose_secret()
                    .to_string(),
            )
        })?;

        let key_is_pq = raw_key_content
            .expose_secret()
            .starts_with("AGE-SECRET-KEY-PQ-");
        if matches!(mode, KeyExportMode::PostQuantum) {
            // if we're supposed to be exporting a postquantum key...
            if key_is_pq {
                // and the key IS postquantum...
                raw_key_content // then return the key content! all good
            } else {
                // otherwise
                return Err("cannot export x25519 key as postquantum".to_string());
            }
        } else {
            // otherwise, if we're exporting x25519 keys...
            if key_is_pq {
                // and the key is postquantum...
                // then we need to convert it
                HybridIdentity::from_string(raw_key_content)?
                    .to_x25519()
                    .to_string()
            } else {
                // or if it's already x25519...
                // just leave it
                raw_key_content
            }
        }
    };

    let mut key_file = File::create(path).await.expect("failed to open key file");
    key_file
        .write_all(key_content.expose_secret().as_bytes())
        .await
        .expect("failed to write file");
    key_file.flush().await.expect("failed to flush file buffer");
    Ok(())
}

/// returns `true` if the key is private
#[tauri::command]
#[specta::specta]
pub async fn check_keyfile_type(path: String) -> Result<bool, String> {
    let mut key_file = File::open(path).await.expect("failed to open key file");
    let mut key_content = String::new();
    if let Err(error) = key_file.read_to_string(&mut key_content).await {
        return Err(error.to_string());
    };
    return Ok(key_content.starts_with("AGE-SECRET-KEY"));
}

/// command to regenerate the public keys of all identities.
#[tauri::command]
#[specta::specta]
pub async fn regenerate_public_identities(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let mut vault = state.get_vault();
    let vault = vault.as_mut().ok_or("vault not initialized".to_string())?;
    vault.file.secrets = vault
        .file
        .secrets
        .iter()
        .filter(|(_, key)| key.contents.private.is_some())
        .map(|(name, key)| {
            let key_content = vault
                .decrypt_secret(&key.contents.private.as_ref().unwrap())
                .expect("decrypting should not fail");
            let identity = if key_content
                .expose_secret()
                .starts_with("AGE-SECRET-KEY-PQ-")
            {
                WildcardIdentity::Hybrid(
                    HybridIdentity::from_string(key_content)
                        .expect("making an object from this key should not fail"),
                )
            } else {
                WildcardIdentity::X25519(
                    age::x25519::Identity::from_str(key_content.expose_secret())
                        .expect("making an object from this key should not fail"),
                )
            };
            let mut key = key.clone();
            key.contents = vault
                .keypair_from(identity)
                .expect("keypair generation should not fail");
            (name.clone(), key)
        })
        .collect();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn import_key_text(
    name: String,
    key_content: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let is_private = key_content.starts_with("AGE-SECRET-KEY");
    let key = if is_private {
        let identity = if key_content.starts_with("AGE-SECRET-KEY-PQ-") {
            WildcardIdentity::Hybrid(HybridIdentity::from_string(SecretString::from(
                key_content,
            ))?)
        } else {
            WildcardIdentity::X25519(
                Identity::from_str(key_content.clone().as_str()).map_err(|e| e.to_string())?,
            )
        };
        state.with_vault(|vault| {
            vault.new_key(
                name,
                identity.to_public().unwrap().to_string().unwrap(),
                Some(identity.to_string().unwrap()),
            )
        })?
    } else {
        state.with_vault(|vault| {
            vault.new_key(
                name,
                Recipient::from_str(key_content.clone().as_str())
                    .expect("failed to parse public key")
                    .to_string(),
                None,
            )
        })?
    }?;
    state.with_vault(|vault| vault.put_key(key))??;
    state.save_vault().await?;
    Ok("key import complete".to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn import_key(
    name: String,
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    if name.len() == 0 {
        return Err("no name set".to_string());
    }
    let mut key_file = File::open(path).await.expect("failed to open key file");
    let mut key_content = String::new();

    // theres genuinely no case in which a key file should be greater than 10 kb
    // this is extremely generous
    if key_file.metadata().await.map_err(|e| e.to_string())?.len() > (1024 * 10) {
        return Err("key file too large".to_string());
    };

    key_file
        .read_to_string(&mut key_content)
        .await
        .expect("failed to read key file");

    return import_key_text(name, key_content, state).await;
}

#[tauri::command]
#[specta::specta]
pub async fn authenticate(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<VaultStatusUpdate, String> {
    let webview = app_handle.get_webview_window("main").unwrap();
    webview.emit("auth-start", ()).map_err(|e| e.to_string())?;
    let mut integrity_check_fail = false;
    loop {
        let (tx, rx) = oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        let tx2 = tx.clone();
        webview.once("auth-cancel", move |_| {
            if let Some(tx) = tx
                .clone()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
            // wtf bro
            {
                let _ = tx.send("".to_string());
            }
        });
        webview.once("authenticate", move |event| {
            if let Some(tx) = tx2
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
            // wtf bro
            {
                let _ = tx.send(event.payload().to_string());
            }
        });

        let password = SecretString::from(match rx.await {
            Ok(message) => {
                if message.len() == 0 {
                    return Ok(VaultStatusUpdate::AuthenticationCancel);
                }
                let raw_unwrap = serde_json::from_str::<String>(&message).unwrap();
                raw_unwrap
            }
            Err(error) => return Err(error.to_string()),
        });
        let unlock_attempt = state.with_vault(|vault| vault.set_vault_key(password))?;
        if let Err(error) = unlock_attempt {
            if error.as_str() == "integrity check failed" {
                integrity_check_fail = true;
                break;
            }
            let _ = webview.emit("auth-response", false);
        } else {
            break;
        };
    }
    if integrity_check_fail {
        let _ = regenerate_public_identities(state).await;
    }
    let result = if integrity_check_fail {
        VaultStatusUpdate::VerificationFail
    } else {
        VaultStatusUpdate::Unlocked
    };
    let _ = app_handle.emit("vault-status-update", &result);

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub async fn lock_vault(
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    state.with_vault(|vault| {
        vault.delete_vault_key();
    })?;
    let _ = app_handle.emit("vault-status-update", VaultStatusUpdate::Locked);
    Ok(())
}
