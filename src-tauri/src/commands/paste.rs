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
        fn CGEventKeyboardSetUnicodeString(
            event: CGEventRef,
            string_length: u64,
            unicode_string: *const u16,
        );
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

    // macOS silently truncates unicode strings longer than 20 UTF-16 code units
    const MAX_UTF16_PER_EVENT: usize = 20;
    const INTER_CHUNK_DELAY_MS: u64 = 5;

    /// Post a single chunk of UTF-16 code units as a key-down + key-up pair.
    unsafe fn post_unicode_chunk(utf16: &[u16]) -> Result<(), String> {
        let down = CGEventCreateKeyboardEvent(std::ptr::null(), 0, true);
        let up = CGEventCreateKeyboardEvent(std::ptr::null(), 0, false);
        if down.is_null() || up.is_null() {
            if !down.is_null() { CFRelease(down); }
            if !up.is_null() { CFRelease(up); }
            return Err(
                "CGEventCreateKeyboardEvent returned null -- Accessibility permission likely not granted".to_string()
            );
        }
        CGEventKeyboardSetUnicodeString(down, utf16.len() as u64, utf16.as_ptr());
        CGEventKeyboardSetUnicodeString(up, utf16.len() as u64, utf16.as_ptr());
        CGEventPost(K_CG_HID_EVENT_TAP, down);
        CGEventPost(K_CG_HID_EVENT_TAP, up);
        CFRelease(down);
        CFRelease(up);
        Ok(())
    }

    /// Encode text as UTF-16 and post it in chunks of <= MAX_UTF16_PER_EVENT.
    unsafe fn post_unicode_string(text: &str) -> Result<(), String> {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        for chunk in utf16.chunks(MAX_UTF16_PER_EVENT) {
            post_unicode_chunk(chunk)?;
            if chunk.len() == MAX_UTF16_PER_EVENT {
                std::thread::sleep(std::time::Duration::from_millis(INTER_CHUNK_DELAY_MS));
            }
        }
        Ok(())
    }

    /// Type text into the frontmost terminal via CGEvent unicode keystrokes.
    /// Sends Ctrl+U first to clear any existing input, then types the text
    /// character-by-character so the terminal sees typed input (not a paste),
    /// avoiding bracketed paste highlighting.
    pub fn type_text(text: &str) -> Result<(), String> {
        unsafe {
            post_key(KVK_U, K_CG_EVENT_FLAG_MASK_CONTROL)?;
        }
        eprintln!("[cg_keys] posted Ctrl+U (pre-type clear)");
        std::thread::sleep(std::time::Duration::from_millis(50));
        unsafe {
            post_unicode_string(text)?;
        }
        eprintln!("[cg_keys] typed {} chars via CGEvent unicode", text.len());
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
        "com.apple.Terminal" => {
            // Terminal.app: type text via CGEvent unicode keystrokes to avoid
            // bracketed paste highlighting that Cmd+V triggers in zsh.
            #[cfg(target_os = "macos")]
            ensure_accessibility()?;

            let activate = build_activate_script(&bundle_id);
            let _ = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&activate)
                .output();
            std::thread::sleep(std::time::Duration::from_millis(150));

            #[cfg(target_os = "macos")]
            {
                if let Err(e) = cg_keys::type_text(&command) {
                    eprintln!(
                        "[paste] Terminal.app type_text failed ({}), falling back to Cmd+V",
                        e
                    );
                    cg_keys::post_ctrl_u()?;
                    std::thread::sleep(std::time::Duration::from_millis(120));
                    cg_keys::post_cmd_v()?;
                } else {
                    eprintln!("[paste] Terminal.app type_text succeeded");
                }
            }
        }
        _ => {
            // All other terminals: type text via CGEvent unicode keystrokes
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
                if let Err(e) = cg_keys::type_text(&command) {
                    eprintln!(
                        "[paste] type_text failed for {} ({}), falling back to Cmd+V",
                        bundle_id, e
                    );
                    cg_keys::post_ctrl_u()?;
                    std::thread::sleep(std::time::Duration::from_millis(120));
                    cg_keys::post_cmd_v()?;
                } else {
                    eprintln!(
                        "[paste] type_text succeeded | bundle={} | pid={} | chars={}",
                        bundle_id, pid, command.len()
                    );
                }
            }
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
