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
    /// PID of the frontmost app captured BEFORE showing overlay.
    /// Populated in hotkey handler before show_and_make_key().
    /// Used by get_terminal_context to detect which terminal was active.
    pub previous_app_pid: Mutex<Option<i32>>,
    /// Pre-captured AX text from the frontmost app, read BEFORE the overlay
    /// steals focus. Written by the hotkey handler, consumed (`.take()`) by
    /// `get_app_context` so it is used at most once.
    pub pre_captured_text: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkey: Mutex::new("Super+K".to_string()),
            last_hotkey_trigger: Mutex::new(None),
            overlay_visible: Mutex::new(false),
            previous_app_pid: Mutex::new(None),
            pre_captured_text: Mutex::new(None),
        }
    }
}
