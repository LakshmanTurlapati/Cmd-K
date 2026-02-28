use tauri::{AppHandle, Manager};
use tauri_nspanel::ManagerExt;

use crate::state::AppState;
use crate::terminal::detect::get_bundle_id;

// ---------------------------------------------------------------------------
// CGEventPost FFI -- post synthetic keyboard events via Accessibility permission
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
mod cg_keys {
    use std::ffi::c_void;

    // CGEventRef is an opaque pointer (CFTypeRef)
    type CGEventRef = *const c_void;

    // CGEventSource -- we pass NULL to use the default HID source
    type CGEventSourceRef = *const c_void;

    // CGEventTapLocation
    const K_CG_HID_EVENT_TAP: u32 = 0;

    // CGEventFlags bitmasks
    const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 1 << 20;
    const K_CG_EVENT_FLAG_MASK_CONTROL: u64 = 1 << 18;

    // Virtual key codes (macOS HID usage / Inside Macintosh)
    const KVK_V: u16 = 0x09;
    const KVK_U: u16 = 0x20;
    const KVK_RETURN: u16 = 0x24;

    extern "C" {
        fn CGEventCreateKeyboardEvent(
            source: CGEventSourceRef,
            virtual_key: u16,
            key_down: bool,
        ) -> CGEventRef;
        fn CGEventSetFlags(event: CGEventRef, flags: u64);
        fn CGEventPost(tap: u32, event: CGEventRef);
        fn CFRelease(cf: *const c_void);
    }

    /// Post a single key-down + key-up pair with optional modifier flags.
    /// Returns an error if CGEventCreateKeyboardEvent returns null.
    unsafe fn post_key(virtual_key: u16, flags: u64) -> Result<(), String> {
        let down = CGEventCreateKeyboardEvent(std::ptr::null(), virtual_key, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null(), virtual_key, false);
        if down.is_null() || up.is_null() {
            if !down.is_null() { CFRelease(down); }
            if !up.is_null() { CFRelease(up); }
            return Err(format!(
                "CGEventCreateKeyboardEvent returned null for key 0x{:02x} -- Accessibility permission likely not granted",
                virtual_key
            ));
        }
        if flags != 0 {
            CGEventSetFlags(down, flags);
            CGEventSetFlags(up, flags);
        }
        CGEventPost(K_CG_HID_EVENT_TAP, down);
        CGEventPost(K_CG_HID_EVENT_TAP, up);
        CFRelease(down);
        CFRelease(up);
        Ok(())
    }

    /// Ctrl+U -- clear the current line in most shells
    pub fn post_ctrl_u() -> Result<(), String> {
        unsafe { post_key(KVK_U, K_CG_EVENT_FLAG_MASK_CONTROL)?; }
        eprintln!("[cg_keys] posted Ctrl+U");
        Ok(())
    }

    /// Cmd+V -- paste from clipboard
    pub fn post_cmd_v() -> Result<(), String> {
        unsafe { post_key(KVK_V, K_CG_EVENT_FLAG_MASK_COMMAND)?; }
        eprintln!("[cg_keys] posted Cmd+V");
        Ok(())
    }

    /// Return -- execute the command
    pub fn post_return() -> Result<(), String> {
        unsafe { post_key(KVK_RETURN, 0)?; }
        eprintln!("[cg_keys] posted Return");
        Ok(())
    }
}

/// Pre-flight check: verify Accessibility permission is granted before
/// attempting CGEventPost calls (which silently fail without it).
#[cfg(target_os = "macos")]
fn ensure_accessibility() -> Result<(), String> {
    let trusted = unsafe { accessibility_sys::AXIsProcessTrusted() };
    eprintln!("[paste] AXIsProcessTrusted() = {}", trusted);
    if !trusted {
        return Err(
            "Accessibility permission not granted. Please enable it in System Settings > Privacy & Security > Accessibility.".to_string()
        );
    }
    Ok(())
}

/// Build a minimal AppleScript that ONLY activates the terminal app.
/// No System Events, no keystroke -- just a basic app activation Apple Event.
fn build_activate_script(bundle_id: &str) -> String {
    format!(
        r#"tell application id "{bundle_id}"
    activate
end tell"#
    )
}

/// Paste `command` into the terminal that was frontmost before the overlay opened.
///
/// - iTerm2: uses native `write text` via osascript (direct AppleEvent, no System Events).
///   Falls back to CGEventPost Cmd+V if write text fails.
/// - All other terminals: activates via simplified AppleScript, then CGEventPost
///   Ctrl+U (clear line) + Cmd+V (paste from clipboard).
///
/// Returns `Ok(())` on success, `Err(message)` if any step fails.
#[tauri::command]
pub fn paste_to_terminal(app: AppHandle, command: String) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "AppState not found".to_string())?;

    let pid = {
        let guard = state
            .previous_app_pid
            .lock()
            .map_err(|_| "previous_app_pid mutex poisoned".to_string())?;
        (*guard).ok_or_else(|| "no previous app PID recorded".to_string())?
    };

    let bundle_id = get_bundle_id(pid)
        .ok_or_else(|| format!("could not resolve bundle ID for pid {}", pid))?;

    eprintln!(
        "[paste] paste_to_terminal called: pid={}, bundle_id={}, command={:?}",
        pid, bundle_id, command
    );

    // Write command to system clipboard via pbcopy.
    {
        use std::io::Write;
        match std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(command.as_bytes());
                }
                let _ = child.wait();
                eprintln!("[paste] clipboard written via pbcopy");
            }
            Err(e) => {
                eprintln!("[paste] pbcopy failed (non-fatal): {}", e);
            }
        }
    }

    // Resign key window on the overlay panel before pasting.
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.resign_key_window();
        eprintln!("[paste] overlay panel resigned key window");
    }

    match bundle_id.as_str() {
        "com.googlecode.iterm2" => {
            // iTerm2: try native `write text` via direct AppleEvent (no System Events)
            let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
            let script = format!(
                r#"tell application "iTerm2"
    activate
    tell current window
        tell current session
            write text (ASCII character 21) newline NO
            set cmd to "{escaped}"
            set cmdLen to count cmd
            set i to 1
            repeat
                if i > cmdLen then exit repeat
                set j to i + 6
                if j > cmdLen then set j to cmdLen
                write text (text i thru j of cmd) newline NO
                set i to j + 1
                if i > cmdLen then exit repeat
                delay 0.016
            end repeat
        end tell
    end tell
end tell"#
            );

            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("failed to spawn osascript: {}", e))?;

            if output.status.success() {
                eprintln!("[paste] iTerm2 write text succeeded");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "[paste] iTerm2 write text failed ({}), falling back to CGEventPost Cmd+V",
                    stderr.trim()
                );
                // Fallback: activate iTerm2 + Cmd+V via CGEventPost
                #[cfg(target_os = "macos")]
                ensure_accessibility()?;

                let activate = build_activate_script(&bundle_id);
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&activate)
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(100));
                #[cfg(target_os = "macos")]
                {
                    cg_keys::post_ctrl_u()?;
                    std::thread::sleep(std::time::Duration::from_millis(120));
                    cg_keys::post_cmd_v()?;
                }
            }
        }
        _ => {
            // All other terminals: activate + CGEventPost Ctrl+U + Cmd+V
            #[cfg(target_os = "macos")]
            ensure_accessibility()?;

            let activate = build_activate_script(&bundle_id);
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&activate)
                .output()
                .map_err(|e| format!("failed to spawn osascript for activate: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[paste] activate failed for {}: {}", bundle_id, stderr.trim());
            }

            // Small delay for the terminal to become frontmost
            std::thread::sleep(std::time::Duration::from_millis(150));

            #[cfg(target_os = "macos")]
            {
                cg_keys::post_ctrl_u()?;
                std::thread::sleep(std::time::Duration::from_millis(120));
                cg_keys::post_cmd_v()?;
            }

            eprintln!(
                "[paste] CGEventPost Ctrl+U + Cmd+V sent | bundle={} | pid={} | chars={}",
                bundle_id, pid, command.len()
            );
        }
    }

    // Wait for keystrokes to be fully processed by the terminal before
    // re-acquiring key window. Without this delay, make_key_window() can
    // intercept pending keystrokes from the event queue.
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Re-acquire key window so the overlay can detect blur for
    // click-outside dismissal. For a nonactivating NSPanel this does NOT
    // steal focus from the terminal -- it only marks our panel as key
    // within our Accessory app.
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.make_key_window();
        eprintln!("[paste] overlay panel re-acquired key window");
    }

    eprintln!(
        "[paste] paste succeeded | bundle={} | pid={} | chars={}",
        bundle_id, pid, command.len()
    );
    Ok(())
}

/// Send a Return keystroke to the terminal that was frontmost before the overlay
/// opened, executing whatever command is currently on the shell input line.
///
/// - iTerm2: uses `write text ""` (sends newline via direct AppleEvent)
/// - All others: activates terminal, then CGEventPost Return
#[tauri::command]
pub fn confirm_terminal_command(app: AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "AppState not found".to_string())?;

    let pid = {
        let guard = state
            .previous_app_pid
            .lock()
            .map_err(|_| "previous_app_pid mutex poisoned".to_string())?;
        (*guard).ok_or_else(|| "no previous app PID recorded".to_string())?
    };

    let bundle_id = get_bundle_id(pid)
        .ok_or_else(|| format!("could not resolve bundle ID for pid {}", pid))?;

    eprintln!("[paste] confirm_terminal_command: pid={}, bundle_id={}", pid, bundle_id);

    // Resign key window so the terminal receives the keystroke
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.resign_key_window();
    }

    match bundle_id.as_str() {
        "com.googlecode.iterm2" => {
            let script = r#"tell application "iTerm2"
    activate
    tell current window
        tell current session
            write text ""
        end tell
    end tell
end tell"#;

            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(script)
                .output()
                .map_err(|e| format!("failed to spawn osascript: {}", e))?;

            if output.status.success() {
                eprintln!("[paste] confirm succeeded for iTerm2 (write text)");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!(
                    "[paste] iTerm2 write text confirm failed ({}), falling back to CGEventPost Return",
                    stderr.trim()
                );
                #[cfg(target_os = "macos")]
                ensure_accessibility()?;

                let activate = build_activate_script(&bundle_id);
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&activate)
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(100));
                #[cfg(target_os = "macos")]
                cg_keys::post_return()?;
            }
        }
        _ => {
            // All other terminals: activate + CGEventPost Return
            #[cfg(target_os = "macos")]
            ensure_accessibility()?;

            let activate = build_activate_script(&bundle_id);
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&activate)
                .output()
                .map_err(|e| format!("failed to spawn osascript for activate: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[paste] activate failed for {}: {}", bundle_id, stderr.trim());
            }

            std::thread::sleep(std::time::Duration::from_millis(100));

            #[cfg(target_os = "macos")]
            cg_keys::post_return()?;

            eprintln!("[paste] confirm succeeded for {} (CGEventPost Return)", bundle_id);
        }
    }

    Ok(())
}
