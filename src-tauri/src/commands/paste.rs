use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use tauri_nspanel::ManagerExt;

#[cfg(target_os = "windows")]
use crate::commands::hotkey::restore_focus;

use crate::state::AppState;
#[cfg(target_os = "macos")]
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
#[cfg(target_os = "macos")]
fn build_activate_script(bundle_id: &str) -> String {
    format!(
        r#"tell application id "{bundle_id}"
    activate
end tell"#
    )
}

/// Write command to system clipboard.
/// On macOS uses pbcopy, on other platforms this is a no-op stub.
#[cfg(target_os = "macos")]
fn write_to_clipboard(command: &str) {
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

/// Write command to system clipboard on Windows via arboard.
#[cfg(target_os = "windows")]
fn write_to_clipboard(command: &str) {
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(command) {
                Ok(()) => eprintln!("[paste] clipboard written via arboard"),
                Err(e) => eprintln!("[paste] arboard set_text failed (non-fatal): {}", e),
            }
        }
        Err(e) => eprintln!("[paste] arboard Clipboard::new failed (non-fatal): {}", e),
    }
}

/// Non-macOS, non-Windows stub.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn write_to_clipboard(_command: &str) {
    // Stub for other platforms
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
    // Windows: early path — does not need bundle_id (uses HWND-based approach)
    #[cfg(target_os = "windows")]
    {
        eprintln!("[paste] paste_to_terminal called (Windows): command={:?}", command);
        paste_to_terminal_windows(&app, &command)?;
    }

    // macOS: needs PID and bundle_id for osascript
    #[cfg(target_os = "macos")]
    {
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

        write_to_clipboard(&command);

        if let Ok(panel) = app.get_webview_panel("main") {
            panel.resign_key_window();
            eprintln!("[paste] overlay panel resigned key window");
        }

        paste_to_terminal_macos(&app, &bundle_id, &command, pid)?;
    }

    // Other platforms: not yet implemented
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = &command;
        return Err("paste not yet implemented for this platform".to_string());
    }

    // Wait for keystrokes to be fully processed by the terminal before
    // re-acquiring key window. Without this delay, make_key_window() can
    // intercept pending keystrokes from the event queue.
    #[cfg(target_os = "macos")]
    {
        std::thread::sleep(std::time::Duration::from_millis(150));

        // Re-acquire key window so the overlay can detect blur for
        // click-outside dismissal. For a nonactivating NSPanel this does NOT
        // steal focus from the terminal -- it only marks our panel as key
        // within our Accessory app.
        if let Ok(panel) = app.get_webview_panel("main") {
            panel.make_key_window();
            eprintln!("[paste] overlay panel re-acquired key window");
        }
    }

    eprintln!(
        "[paste] paste succeeded | chars={}",
        command.len()
    );
    Ok(())
}

/// macOS-specific paste implementation using osascript and CGEventPost.
#[cfg(target_os = "macos")]
fn paste_to_terminal_macos(
    _app: &AppHandle,
    bundle_id: &str,
    command: &str,
    pid: i32,
) -> Result<(), String> {
    match bundle_id {
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
                ensure_accessibility()?;

                let activate = build_activate_script(bundle_id);
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&activate)
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(100));
                cg_keys::post_ctrl_u()?;
                std::thread::sleep(std::time::Duration::from_millis(120));
                cg_keys::post_cmd_v()?;
            }
        }
        "com.apple.Terminal" => {
            // Terminal.app: type text via CGEvent unicode keystrokes to avoid
            // bracketed paste highlighting that Cmd+V triggers in zsh.
            ensure_accessibility()?;

            let activate = build_activate_script(bundle_id);
            let activate_output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&activate)
                .output();
            match &activate_output {
                Ok(o) if o.status.success() => {
                    eprintln!("[paste] Terminal.app activate succeeded");
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    eprintln!("[paste] Terminal.app activate FAILED: {}", stderr.trim());
                }
                Err(e) => {
                    eprintln!("[paste] Terminal.app osascript spawn FAILED: {}", e);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(150));

            // Clear current line via System Events Ctrl+U
            let _ = std::process::Command::new("osascript")
                .arg("-e")
                .arg(r#"tell application "System Events" to keystroke "u" using control down"#)
                .output();
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Type text via System Events keystroke
            let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
            let script = format!(
                r#"tell application "System Events" to keystroke "{escaped}""#
            );
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output();
            match &output {
                Ok(o) if o.status.success() => {
                    eprintln!("[paste] Terminal.app keystroke succeeded | chars={}", command.len());
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    eprintln!("[paste] Terminal.app keystroke failed: {}", stderr.trim());
                }
                Err(e) => {
                    eprintln!("[paste] Terminal.app osascript spawn failed: {}", e);
                }
            }
        }
        _ => {
            // All other terminals: type text via CGEvent unicode keystrokes
            ensure_accessibility()?;

            let activate = build_activate_script(bundle_id);
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

            // Clear current line first
            cg_keys::post_ctrl_u()?;
            std::thread::sleep(std::time::Duration::from_millis(50));

            // Type text via System Events keystroke (streaming char effect)
            let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
            let script = format!(
                r#"tell application "System Events" to keystroke "{escaped}""#
            );
            let output = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output();
            match &output {
                Ok(o) if o.status.success() => {
                    eprintln!(
                        "[paste] keystroke succeeded | bundle={} | pid={} | chars={}",
                        bundle_id, pid, command.len()
                    );
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    eprintln!(
                        "[paste] keystroke failed ({}), falling back to Cmd+V",
                        stderr.trim()
                    );
                    cg_keys::post_cmd_v()?;
                }
                Err(e) => {
                    eprintln!(
                        "[paste] osascript spawn failed ({}), falling back to Cmd+V",
                        e
                    );
                    cg_keys::post_cmd_v()?;
                }
            }
        }
    }

    Ok(())
}

/// Windows: paste command into the terminal via clipboard + Ctrl+V.
///
/// 1. Write command to clipboard via arboard
/// 2. Read previous_hwnd from AppState
/// 3. Check if target process is elevated (elevated → warn user)
/// 4. Restore focus to terminal via SetForegroundWindow
/// 5. Wait 100ms for window activation settle
/// 6. SendInput: Ctrl down → V down → V up → Ctrl up
#[cfg(target_os = "windows")]
fn paste_to_terminal_windows(app: &AppHandle, command: &str) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "AppState not found".to_string())?;

    // Get the HWND we captured before showing the overlay
    let prev_hwnd = {
        let guard = state
            .previous_hwnd
            .lock()
            .map_err(|_| "previous_hwnd mutex poisoned".to_string())?;
        (*guard).ok_or_else(|| "no previous HWND recorded".to_string())?
    };

    // Check if target process is elevated
    let target_pid = crate::terminal::detect_windows::get_pid_from_hwnd(prev_hwnd);
    if let Some(pid) = target_pid {
        if is_elevated_process(pid) {
            return Err(
                "Terminal is running as Administrator — paste may fail. Please paste manually (Ctrl+V)."
                    .to_string(),
            );
        }
    }

    // Write command to clipboard
    write_to_clipboard(command);

    // Restore focus to the terminal
    let restored = restore_focus(prev_hwnd);
    eprintln!("[paste] Windows focus restored to HWND {}: {}", prev_hwnd, restored);

    // Wait for window activation to settle
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Send Ctrl+V via SendInput
    send_ctrl_v()?;

    eprintln!("[paste] Windows paste succeeded | hwnd={} | chars={}", prev_hwnd, command.len());
    Ok(())
}

/// Check if a process is running with elevated (Administrator) privileges.
#[cfg(target_os = "windows")]
fn is_elevated_process(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::OpenProcessToken;
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return false; // Can't open process, assume non-elevated
        }

        let mut token: HANDLE = std::ptr::null_mut();
        let ok = OpenProcessToken(process, TOKEN_QUERY, &mut token);
        CloseHandle(process);
        if ok == 0 || token.is_null() {
            return false;
        }

        let mut elevation: TOKEN_ELEVATION = std::mem::zeroed();
        let mut return_length: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut return_length,
        );
        CloseHandle(token);
        if ok == 0 {
            return false;
        }

        elevation.TokenIsElevated != 0
    }
}

/// Helper: build an INPUT struct for a keyboard event.
#[cfg(target_os = "windows")]
fn make_keyboard_input(vk: u16, flags: u32) -> windows_sys::Win32::UI::Input::KeyboardAndMouse::INPUT {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT};
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Send Ctrl+V keystroke via SendInput (Windows).
#[cfg(target_os = "windows")]
fn send_ctrl_v() -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };

    unsafe {
        let inputs: [INPUT; 4] = [
            make_keyboard_input(VK_CONTROL, 0),
            make_keyboard_input(VK_V, 0),
            make_keyboard_input(VK_V, KEYEVENTF_KEYUP),
            make_keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
        ];

        let sent = SendInput(4, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
        if sent != 4 {
            return Err(format!("SendInput Ctrl+V failed: only {} of 4 inputs sent", sent));
        }
        eprintln!("[paste] SendInput Ctrl+V succeeded");
        Ok(())
    }
}

/// Send Enter/Return keystroke via SendInput (Windows).
#[cfg(target_os = "windows")]
fn send_return() -> Result<(), String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, KEYEVENTF_KEYUP, VK_RETURN,
    };

    unsafe {
        let inputs: [INPUT; 2] = [
            make_keyboard_input(VK_RETURN, 0),
            make_keyboard_input(VK_RETURN, KEYEVENTF_KEYUP),
        ];

        let sent = SendInput(2, inputs.as_ptr(), std::mem::size_of::<INPUT>() as i32);
        if sent != 2 {
            return Err(format!("SendInput Return failed: only {} of 2 inputs sent", sent));
        }
        eprintln!("[paste] SendInput Return succeeded");
        Ok(())
    }
}

/// Send a Return keystroke to the terminal that was frontmost before the overlay
/// opened, executing whatever command is currently on the shell input line.
///
/// - iTerm2: uses `write text ""` (sends newline via direct AppleEvent)
/// - All others: activates terminal, then CGEventPost Return
#[tauri::command]
pub fn confirm_terminal_command(app: AppHandle) -> Result<(), String> {
    // Windows: early path — does not need bundle_id
    #[cfg(target_os = "windows")]
    {
        eprintln!("[paste] confirm_terminal_command called (Windows)");
        confirm_command_windows(&app)?;
    }

    // macOS: needs PID and bundle_id for osascript
    #[cfg(target_os = "macos")]
    {
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

        if let Ok(panel) = app.get_webview_panel("main") {
            panel.resign_key_window();
        }

        confirm_terminal_command_macos(&bundle_id, pid)?;
    }

    // Other platforms: not yet implemented
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        return Err("confirm not yet implemented for this platform".to_string());
    }

    #[allow(unreachable_code)]
    Ok(())
}

/// Windows-specific confirm implementation: restore focus + SendInput Return.
#[cfg(target_os = "windows")]
fn confirm_command_windows(app: &AppHandle) -> Result<(), String> {
    let state = app
        .try_state::<AppState>()
        .ok_or_else(|| "AppState not found".to_string())?;

    let prev_hwnd = {
        let guard = state
            .previous_hwnd
            .lock()
            .map_err(|_| "previous_hwnd mutex poisoned".to_string())?;
        (*guard).ok_or_else(|| "no previous HWND recorded".to_string())?
    };

    // Restore focus to terminal
    restore_focus(prev_hwnd);
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Send Return keystroke
    send_return()?;

    eprintln!("[paste] Windows confirm succeeded | hwnd={}", prev_hwnd);
    Ok(())
}

/// macOS-specific confirm implementation.
#[cfg(target_os = "macos")]
fn confirm_terminal_command_macos(bundle_id: &str, _pid: i32) -> Result<(), String> {
    match bundle_id {
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
                ensure_accessibility()?;

                let activate = build_activate_script(bundle_id);
                let _ = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(&activate)
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(100));
                cg_keys::post_return()?;
            }
        }
        _ => {
            // All other terminals: activate + CGEventPost Return
            ensure_accessibility()?;

            let activate = build_activate_script(bundle_id);
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

            cg_keys::post_return()?;

            eprintln!("[paste] confirm succeeded for {} (CGEventPost Return)", bundle_id);
        }
    }

    Ok(())
}
