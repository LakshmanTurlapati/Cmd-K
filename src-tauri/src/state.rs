use std::sync::Mutex;
use std::time::Instant;

/// Application state shared across Tauri commands.
///
/// All fields are Mutex-wrapped for thread-safe access from async command handlers
/// and the global hotkey callback which may fire on a different thread.
pub struct AppState {
    /// The currently registered hotkey string, e.g. "Super+K"
    pub hotkey: Mutex<String>,
    /// Timestamp of last hotkey trigger -- used for 200ms debounce
    /// to work around Tauri double-fire bug #10025
    pub last_hotkey_trigger: Mutex<Option<Instant>>,
    /// Current visibility state of the overlay window
    pub overlay_visible: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkey: Mutex::new("Super+K".to_string()),
            last_hotkey_trigger: Mutex::new(None),
            overlay_visible: Mutex::new(false),
        }
    }
}
