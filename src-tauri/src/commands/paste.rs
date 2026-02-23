use tauri::{AppHandle, Manager};

use crate::state::AppState;
use crate::terminal::detect::get_bundle_id;

/// Build an AppleScript that types `command` into the active session of the
/// given terminal without pressing Return.
///
/// Escapes `\` -> `\\` and `"` -> `\"` so the command string is safe to
/// embed inside AppleScript double-quoted string literals.
///
/// Supported terminals:
/// - `com.googlecode.iterm2` (iTerm2): uses `write text … newline NO`
/// - `com.apple.Terminal` (Terminal.app): sends Ctrl+U to clear the line first,
///   then types the text using `keystroke` (does NOT execute the command)
///
/// Returns `Err` for any other bundle ID.
fn build_paste_script(bundle_id: &str, command: &str) -> Result<String, String> {
    // Escape backslashes first, then double-quotes, for safe AppleScript string interpolation.
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");

    match bundle_id {
        "com.googlecode.iterm2" => Ok(format!(
            r#"tell application "iTerm2"
    activate
    tell current window
        tell current session
            write text "{escaped}" newline NO
        end tell
    end tell
end tell"#
        )),

        "com.apple.Terminal" => Ok(format!(
            r#"tell application "Terminal"
    activate
    tell application "System Events"
        tell process "Terminal"
            keystroke "u" using control down
            keystroke "{escaped}"
        end tell
    end tell
end tell"#
        )),

        other => Err(format!("unsupported terminal: {}", other)),
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

    // Build the AppleScript for this terminal type
    let script = build_paste_script(&bundle_id, &command)?;

    // Execute the script via osascript
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("failed to spawn osascript: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("osascript failed: {}", stderr.trim()))
    }
}
