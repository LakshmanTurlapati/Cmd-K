use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::commands::window::toggle_overlay;
use crate::state::AppState;
use crate::terminal;
use crate::terminal::ax_reader;

/// Capture the PID of the frontmost application using NSWorkspace via ObjC FFI.
///
/// This MUST be called BEFORE show_and_make_key() because after the panel is shown,
/// NSWorkspace.frontmostApplication returns our own app (Pitfall 1 from research).
#[cfg(target_os = "macos")]
fn get_frontmost_pid() -> Option<i32> {
    use std::ffi::c_void;
    extern "C" {
        fn objc_getClass(name: *const u8) -> *mut c_void;
        fn sel_registerName(name: *const u8) -> *mut c_void;
        fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }
    // SAFETY: These are stable macOS ObjC runtime functions used to call
    // [NSWorkspace sharedWorkspace].frontmostApplication.processIdentifier.
    // All pointers are checked for null before dereferencing.
    unsafe {
        let workspace_class = objc_getClass(b"NSWorkspace\0".as_ptr());
        if workspace_class.is_null() {
            return None;
        }

        let shared_sel = sel_registerName(b"sharedWorkspace\0".as_ptr());
        let workspace = objc_msgSend(workspace_class, shared_sel);
        if workspace.is_null() {
            return None;
        }

        let front_sel = sel_registerName(b"frontmostApplication\0".as_ptr());
        let front_app = objc_msgSend(workspace, front_sel);
        if front_app.is_null() {
            return None;
        }

        let pid_sel = sel_registerName(b"processIdentifier\0".as_ptr());
        let pid = objc_msgSend(front_app, pid_sel) as i32;
        if pid > 0 {
            Some(pid)
        } else {
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn get_frontmost_pid() -> Option<i32> {
    None
}

/// Compute a stable window key for the frontmost application identified by PID.
///
/// Key format:
/// - Terminals and IDEs with integrated terminals: "bundle_id:shell_pid"
///   (gives each terminal tab its own history bucket)
/// - Other apps (Finder, Safari, etc.): "bundle_id:app_pid"
///   (gives each app its own per-process history bucket)
///
/// Falls back to "bundle_id:app_pid" if shell PID cannot be resolved.
fn compute_window_key(pid: i32) -> String {
    let bundle_id = terminal::detect::get_bundle_id(pid);
    let bundle_str = bundle_id.as_deref().unwrap_or("unknown");
    let is_terminal = terminal::detect::is_known_terminal(bundle_str);
    let is_ide = terminal::detect::is_ide_with_terminal(bundle_str);

    let key = if is_terminal || is_ide {
        match terminal::process::find_shell_pid(pid) {
            Some(shell_pid) => format!("{}:{}", bundle_str, shell_pid),
            None => format!("{}:{}", bundle_str, pid),
        }
    } else {
        format!("{}:{}", bundle_str, pid)
    };

    eprintln!("[hotkey] computed window_key: {}", &key);
    key
}

/// Register (or re-register) a global hotkey that toggles the CMD+K overlay.
///
/// This command:
/// 1. Unregisters all existing shortcuts to avoid duplicates
/// 2. Parses and registers the new shortcut string
/// 3. The handler debounces within 200ms to work around Tauri double-fire bug #10025
/// 4. Captures the frontmost app PID BEFORE toggling the overlay (for terminal context)
/// 5. Updates AppState with the new hotkey string
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
                // Determine if the overlay is currently hidden (about to show).
                // If it is, capture the frontmost app PID NOW -- before toggle_overlay()
                // calls show_and_make_key(), which would change frontmostApplication to us.
                let is_currently_visible = app_handle
                    .try_state::<AppState>()
                    .and_then(|state| state.overlay_visible.lock().ok().map(|v| *v))
                    .unwrap_or(false);

                // Only capture PID when about to show (not when hiding)
                if !is_currently_visible {
                    let pid = get_frontmost_pid();
                    eprintln!("[hotkey] overlay hidden, capturing frontmost PID: {:?}", pid);
                    if let Some(pid) = pid {
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut prev) = state.previous_app_pid.lock() {
                                *prev = Some(pid);
                            }
                        }

                        // Pre-capture AX text BEFORE toggle_overlay steals focus.
                        let pre_text = ax_reader::read_focused_text_fast(pid);
                        if let Some(ref text) = pre_text {
                            eprintln!(
                                "[hotkey] pre-captured {} bytes of AX text from pid {}",
                                text.len(),
                                pid
                            );
                        }
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut pt) = state.pre_captured_text.lock() {
                                *pt = pre_text;
                            }
                        }

                        // Compute window key synchronously BEFORE toggle_overlay.
                        // This captures the shell PID while the terminal is still frontmost.
                        let window_key = compute_window_key(pid);
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut wk) = state.current_window_key.lock() {
                                *wk = Some(window_key);
                            }
                        }

                    }
                } else {
                    eprintln!("[hotkey] overlay visible, hiding (no PID capture)");
                }

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
