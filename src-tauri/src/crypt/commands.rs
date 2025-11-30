use crate::crypt;
use crate::store::{KeyMetadata, KeyType};
use crate::AppState;
use secrecy::SecretString;
use std::sync::Mutex;

#[tauri::command]
pub fn generate_keypair(id: String, state: tauri::State<Mutex<AppState>>, app: tauri::AppHandle) {
    let mut state = state.lock().unwrap();
    let vault = state.vault.as_mut().expect("failed to load vault");
    let keypair = crypt::generate_key();
    let password = crate::prompt_password(app);
    vault.put_secret(format!("priv:{}", id), keypair.private_key, &password);
    vault.put_secret(
        format!("pub:{}", id),
        SecretString::from(keypair.public_key),
        &password,
    );
    state.index.insert(
        format!("pub:{}", id),
        serde_cbor::to_vec(&KeyMetadata {
            name: format!("pub:{}", id),
            key_type: KeyType::Public,
            date_created: std::time::SystemTime::now(),
        })
        .expect("failed to serialize key metadata"),
    );
    state.index.insert(
        format!("priv:{}", id),
        serde_cbor::to_vec(&KeyMetadata {
            name: format!("priv:{}", id),
            key_type: KeyType::Public,
            date_created: std::time::SystemTime::now(),
        })
        .expect("failed to serialize key metadata"),
    );
}
