use tauri::{AppHandle, Manager};

use crate::state::AppState;
use crate::terminal;

/// Get terminal context (CWD, shell type, running process, visible output).
///
/// Called by the frontend when the overlay is shown (in response to the
/// "overlay-shown" event). Returns null/None if the app that was frontmost
/// before the overlay is not a known terminal emulator.
///
/// The previous frontmost app PID must have been captured by the hotkey handler
/// BEFORE toggle_overlay was called (see commands/hotkey.rs).
#[tauri::command]
pub fn get_terminal_context(app: AppHandle) -> Option<terminal::TerminalContext> {
    // Read the previously captured frontmost app PID from AppState
    let pid = (*app
        .try_state::<AppState>()?
        .previous_app_pid
        .lock()
        .ok()?)?;

    terminal::detect(pid)
}
