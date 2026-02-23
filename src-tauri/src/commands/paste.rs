use tauri::{AppHandle, Manager};
use tauri_nspanel::ManagerExt;

use crate::state::AppState;
use crate::terminal::detect::get_bundle_id;

/// Build an AppleScript that types `command` into the active session of the
/// given terminal without pressing Return.
///
/// Escapes `\` -> `\\` and `"` -> `\"` so the command string is safe to
/// embed inside AppleScript double-quoted string literals.
///
/// Supported terminals:
/// - `com.googlecode.iterm2` (iTerm2): uses `write text ... newline NO`
/// - `com.apple.Terminal` (Terminal.app): sends Ctrl+U to clear the line first,
///   then types the text using `keystroke` (does NOT execute the command)
/// - All other apps: universal fallback that activates the app and types the
///   text via System Events `keystroke` (bypasses bracketed paste mode)
fn build_paste_script(bundle_id: &str, pid: i32, command: &str) -> Result<String, String> {
    // Escape backslashes first, then double-quotes, for safe AppleScript string interpolation.
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");

    // Batch size for typewriter effect: 7 chars per 16ms delay = ~437 chars/sec.
    // Matches the frontend reveal speed (7 chars per 16ms setInterval tick).
    // AppleScript delay resolution is ~16ms, so we batch to hit target speed.

    match bundle_id {
        "com.googlecode.iterm2" => Ok(format!(
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
        )),

        "com.apple.Terminal" => Ok(format!(
            r#"tell application "Terminal"
    activate
end tell
delay 0.15
tell application "System Events"
    tell (first process whose unix id is {pid})
        keystroke "u" using control down
        set cmd to "{escaped}"
        set cmdLen to count cmd
        set i to 1
        repeat
            if i > cmdLen then exit repeat
            set j to i + 6
            if j > cmdLen then set j to cmdLen
            keystroke (text i thru j of cmd)
            set i to j + 1
            if i > cmdLen then exit repeat
            delay 0.016
        end repeat
    end tell
end tell"#
        )),

        // Universal fallback: activate the target app and type via keystroke.
        // Targets the process by PID to ensure keystrokes reach the correct app
        // even when the overlay panel retains focus.
        // Batched at 7 chars per 16ms to match overlay reveal speed.
        _ => {
            eprintln!("[paste] using universal keystroke fallback for bundle: {}", bundle_id);
            Ok(format!(
                r#"tell application id "{bundle_id}"
    activate
end tell
delay 0.15
tell application "System Events"
    tell (first process whose unix id is {pid})
        keystroke "u" using control down
        set cmd to "{escaped}"
        set cmdLen to count cmd
        set i to 1
        repeat
            if i > cmdLen then exit repeat
            set j to i + 6
            if j > cmdLen then set j to cmdLen
            keystroke (text i thru j of cmd)
            set i to j + 1
            if i > cmdLen then exit repeat
            delay 0.016
        end repeat
    end tell
end tell"#
            ))
        }
    }
}

/// Paste `command` into the terminal that was frontmost before the overlay opened.
///
/// Reads `previous_app_pid` from `AppState`, resolves the bundle ID via
/// `terminal::detect::get_bundle_id`, builds the appropriate AppleScript, and
/// executes it with `osascript`.
///
/// Returns `Ok(())` on success, `Err(message)` if any step fails (PID not set,
/// bundle ID unresolvable, unsupported terminal, or osascript failure).
#[tauri::command]
pub fn paste_to_terminal(app: AppHandle, command: String) -> Result<(), String> {
    // Read previous frontmost app PID from AppState (same pattern as commands/terminal.rs)
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

    // Resolve the bundle ID of the previous app
    let bundle_id = get_bundle_id(pid)
        .ok_or_else(|| format!("could not resolve bundle ID for pid {}", pid))?;

    eprintln!(
        "[paste] paste_to_terminal called: pid={}, bundle_id={}, command={:?}",
        pid, bundle_id, command
    );

    // Write command to system clipboard via pbcopy.
    // navigator.clipboard.writeText() in the webview may fail silently outside
    // a user gesture context, so we ensure the clipboard has the text from Rust.
    // Essential for the Cmd+V fallback; also a safety net for click-to-copy.
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
    // Without this, the NSPanel retains key window status and may intercept
    // keystrokes that System Events sends to the target terminal process.
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.resign_key_window();
        eprintln!("[paste] overlay panel resigned key window");
    }

    // Build the AppleScript for this terminal type (PID used for targeted keystroke delivery)
    let script = build_paste_script(&bundle_id, pid, &command)?;

    // Execute the script via osascript
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("failed to spawn osascript: {}", e))?;

    // Wait for the last batch of keystrokes to be fully processed by the
    // terminal before re-acquiring key window. Without this delay,
    // make_key_window() can intercept pending keystrokes from the event queue,
    // causing the end of the command to be truncated.
    std::thread::sleep(std::time::Duration::from_millis(150));

    // Re-acquire key window after paste so the overlay can detect blur
    // when the user later clicks outside (triggering click-outside dismissal).
    // For a nonactivating NSPanel, make_key_window does NOT steal focus from
    // the terminal -- it only marks our panel as key within our Accessory app.
    if let Ok(panel) = app.get_webview_panel("main") {
        panel.make_key_window();
        eprintln!("[paste] overlay panel re-acquired key window");
    }

    if output.status.success() {
        eprintln!("[paste] paste succeeded for bundle: {}", bundle_id);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[paste] paste failed for bundle {}: {}", bundle_id, stderr.trim());
        Err(format!("osascript failed: {}", stderr.trim()))
    }
}

/// Send a Return keystroke to the terminal that was frontmost before the overlay
/// opened, executing whatever command is currently on the shell input line.
///
/// Uses the same PID-based targeting as `paste_to_terminal` to ensure the
/// keystroke reaches the correct process.
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

    let script = match bundle_id.as_str() {
        "com.googlecode.iterm2" => format!(
            r#"tell application "iTerm2"
    activate
    tell current window
        tell current session
            write text ""
        end tell
    end tell
end tell"#
        ),
        _ => format!(
            r#"tell application id "{bundle_id}"
    activate
end tell
delay 0.1
tell application "System Events"
    tell (first process whose unix id is {pid})
        keystroke return
    end tell
end tell"#
        ),
    };

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("failed to spawn osascript: {}", e))?;

    if output.status.success() {
        eprintln!("[paste] confirm succeeded for bundle: {}", bundle_id);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[paste] confirm failed for bundle {}: {}", bundle_id, stderr.trim());
        Err(format!("osascript confirm failed: {}", stderr.trim()))
    }
}
