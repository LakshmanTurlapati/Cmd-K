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
    let state = app.try_state::<AppState>();
    if state.is_none() {
        eprintln!("[terminal] AppState not found");
        return None;
    }
    let state = state.unwrap();
    let guard = state.previous_app_pid.lock();
    if guard.is_err() {
        eprintln!("[terminal] previous_app_pid mutex poisoned");
        return None;
    }
    let pid_opt = *guard.unwrap();
    eprintln!("[terminal] get_terminal_context called, previous_app_pid={:?}", pid_opt);

    let pid = pid_opt?;
    let result = terminal::detect(pid);
    eprintln!("[terminal] detect({}) returned: {:?}", pid, result.as_ref().map(|c| (&c.shell_type, &c.cwd)));
    result
}
