use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::commands::window::toggle_overlay;
use crate::state::AppState;

/// Register (or re-register) a global hotkey that toggles the CMD+K overlay.
///
/// This command:
/// 1. Unregisters all existing shortcuts to avoid duplicates
/// 2. Parses and registers the new shortcut string
/// 3. The handler debounces within 200ms to work around Tauri double-fire bug #10025
/// 4. Updates AppState with the new hotkey string
///
/// Returns an error string if registration fails (e.g. hotkey conflict with
/// another application), which the frontend can surface to the user.
#[tauri::command]
pub fn register_hotkey(app: AppHandle, shortcut_str: String) -> Result<(), String> {
    // Unregister all previously registered shortcuts
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())?;

    let shortcut: tauri_plugin_global_shortcut::Shortcut = shortcut_str
        .parse()
        .map_err(|_| format!("Invalid shortcut format: '{}'", shortcut_str))?;

    let app_handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            // Only respond to key press, not key release
            if event.state() != ShortcutState::Pressed {
                return;
            }

            // Debounce: skip if fired within 200ms of the previous trigger
            // This works around the known Tauri double-fire bug (#10025)
            let should_fire = if let Some(state) = app_handle.try_state::<AppState>() {
                if let Ok(mut last_trigger) = state.last_hotkey_trigger.lock() {
                    let now = Instant::now();
                    let fire = match *last_trigger {
                        None => true,
                        Some(prev) => now.duration_since(prev) >= Duration::from_millis(200),
                    };
                    if fire {
                        *last_trigger = Some(now);
                    }
                    fire
                } else {
                    true
                }
            } else {
                true
            };

            if should_fire {
                toggle_overlay(&app_handle);
            }
        })
        .map_err(|e| format!("Failed to register hotkey '{}': {}", shortcut_str, e))?;

    // Update stored hotkey in AppState
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut hotkey) = state.hotkey.lock() {
            *hotkey = shortcut_str;
        }
    }

    Ok(())
}
