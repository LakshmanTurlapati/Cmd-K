use keyring::Entry;

const SERVICE: &str = "com.lakshmanturlapati.cmd-k";
const ACCOUNT: &str = "xai_api_key";

#[tauri::command]
pub fn save_api_key(key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read from Keychain: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete from Keychain: {}", e))
}
