use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

use crate::commands::window::toggle_overlay;
use crate::state::AppState;
use crate::terminal;

#[cfg(target_os = "macos")]
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

/// Capture the HWND of the foreground window using Win32 GetForegroundWindow.
///
/// This MUST be called BEFORE showing the overlay because after the overlay
/// appears, GetForegroundWindow returns our own overlay HWND.
#[cfg(target_os = "windows")]
fn get_foreground_hwnd() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == 0 {
        None
    } else {
        Some(hwnd)
    }
}

#[cfg(not(target_os = "windows"))]
fn get_foreground_hwnd() -> Option<isize> {
    None
}

/// Restore focus to the previously captured HWND using the
/// AttachThreadInput + SetForegroundWindow workaround.
///
/// Windows restricts which processes can call SetForegroundWindow.
/// The workaround: attach our thread's input to the target window's
/// thread, call SetForegroundWindow, then detach.
///
/// Returns true if focus was successfully restored.
#[cfg(target_os = "windows")]
pub fn restore_focus(target_hwnd: isize) -> bool {
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    unsafe {
        // Validate the HWND is still a valid window (might have been closed)
        if IsWindow(target_hwnd) == 0 {
            eprintln!("[focus] target HWND {} is no longer valid", target_hwnd);
            return false;
        }

        let target_thread = GetWindowThreadProcessId(target_hwnd, std::ptr::null_mut());
        let our_thread = GetCurrentThreadId();

        // Attach our thread's input queue to the target's thread
        let attached = if target_thread != 0 && target_thread != our_thread {
            AttachThreadInput(our_thread, target_thread, 1) != 0 // TRUE = attach
        } else {
            false
        };

        let result = SetForegroundWindow(target_hwnd);

        // Always detach if we attached
        if attached {
            AttachThreadInput(our_thread, target_thread, 0); // FALSE = detach
        }

        if result != 0 {
            eprintln!(
                "[focus] SetForegroundWindow succeeded for HWND {}",
                target_hwnd
            );
            true
        } else {
            eprintln!(
                "[focus] SetForegroundWindow failed for HWND {}, trying fallback",
                target_hwnd
            );
            // Fallback: AllowSetForegroundWindow then retry
            // This handles edge cases where AttachThreadInput alone is insufficient
            let mut target_pid: u32 = 0;
            GetWindowThreadProcessId(target_hwnd, &mut target_pid);
            AllowSetForegroundWindow(target_pid);
            let retry = SetForegroundWindow(target_hwnd);
            eprintln!(
                "[focus] Fallback SetForegroundWindow result: {}",
                retry != 0
            );
            retry != 0
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn restore_focus(_target_hwnd: isize) -> bool {
    false
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
///
/// `focused_cwd`: AX-derived CWD from the focused terminal tab. Used by
/// `find_shell_pid` to disambiguate between multiple candidate shells in
/// Electron IDEs with multiple terminal tabs.
fn compute_window_key(pid: i32, focused_cwd: Option<String>) -> String {
    let bundle_id = terminal::detect::get_bundle_id(pid);
    let bundle_str = bundle_id.as_deref().unwrap_or("unknown");
    let is_terminal = terminal::detect::is_known_terminal(bundle_str);
    let is_ide = terminal::detect::is_ide_with_terminal(bundle_str);

    let key = if is_terminal || is_ide {
        match terminal::process::find_shell_pid(pid, focused_cwd.as_deref()) {
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

                        // Windows: capture HWND of foreground window before overlay steals focus
                        #[cfg(target_os = "windows")]
                        {
                            let hwnd = get_foreground_hwnd();
                            eprintln!(
                                "[hotkey] Windows: capturing foreground HWND: {:?}",
                                hwnd
                            );
                            if let Some(state) = app_handle.try_state::<AppState>() {
                                if let Ok(mut prev) = state.previous_hwnd.lock() {
                                    *prev = hwnd;
                                }
                            }
                        }

                        // Pre-capture AX text BEFORE toggle_overlay steals focus.
                        #[cfg(target_os = "macos")]
                        {
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
                        }
                        // Non-macOS: skip AX pre-capture (not available)
                        #[cfg(not(target_os = "macos"))]
                        {
                            if let Some(state) = app_handle.try_state::<AppState>() {
                                if let Ok(mut pt) = state.pre_captured_text.lock() {
                                    *pt = None;
                                }
                            }
                        }

                        // Pre-capture focused terminal tab CWD for IDEs with
                        // multiple terminal tabs. This MUST happen BEFORE
                        // toggle_overlay() because after the overlay steals focus,
                        // AXFocusedUIElement will point to the overlay, not the
                        // IDE's terminal tab.
                        let bundle_id = terminal::detect::get_bundle_id(pid);
                        let bundle_str = bundle_id.as_deref().unwrap_or("unknown");

                        #[cfg(target_os = "macos")]
                        let focused_cwd = if terminal::detect::is_ide_with_terminal(bundle_str) {
                            let cwd = ax_reader::get_focused_terminal_cwd(pid);
                            eprintln!("[hotkey] IDE focused tab CWD: {:?}", &cwd);
                            cwd
                        } else {
                            None
                        };
                        // Non-macOS: no AX-based CWD capture
                        #[cfg(not(target_os = "macos"))]
                        let focused_cwd: Option<String> = None;

                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut fc) = state.pre_captured_focused_cwd.lock() {
                                *fc = focused_cwd.clone();
                            }
                        }

                        // Compute window key synchronously BEFORE toggle_overlay.
                        // This captures the shell PID while the terminal is still frontmost.
                        // Pass the focused CWD so find_shell_pid can match the
                        // correct tab's shell in multi-tab IDE scenarios.
                        let window_key = compute_window_key(pid, focused_cwd);
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut wk) = state.current_window_key.lock() {
                                *wk = Some(window_key);
                            }
                        }

                    }
                } else {
                    eprintln!("[hotkey] overlay visible, hiding (no PID capture)");
                    // Clear pre-captured focused CWD when hiding the overlay.
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        if let Ok(mut fc) = state.pre_captured_focused_cwd.lock() {
                            *fc = None;
                        }
                    }
                    // Windows: clear previous HWND on hide
                    #[cfg(target_os = "windows")]
                    {
                        if let Some(state) = app_handle.try_state::<AppState>() {
                            if let Ok(mut prev) = state.previous_hwnd.lock() {
                                *prev = None;
                            }
                        }
                    }
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
