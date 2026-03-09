use keyring::Entry;

use super::providers::Provider;

const SERVICE: &str = "com.lakshmanturlapati.cmd-k";

#[tauri::command]
pub fn save_api_key(provider: Provider, key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}

#[tauri::command]
pub fn get_api_key(provider: Provider) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read from Keychain: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key(provider: Provider) -> Result<(), String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete from Keychain: {}", e))
}
