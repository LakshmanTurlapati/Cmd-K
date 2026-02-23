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
///
/// Kept for backward compatibility. New callers should use get_app_context.
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

/// Get full app context (app name, terminal context, browser console state).
///
/// Returns AppContext for ANY frontmost app, not just terminals. Includes:
/// - Cleaned app display name (e.g., "Chrome", "Code", "Terminal")
/// - Terminal context if a shell was found in the frontmost app's process tree
/// - Browser console detection state if the app is a known browser
///
/// Called by the frontend when the overlay is shown. Returns None only if
/// the detection timeout (500ms) expires before context is gathered.
#[tauri::command]
pub fn get_app_context(app: AppHandle) -> Option<terminal::AppContext> {
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
    eprintln!("[terminal] get_app_context called, previous_app_pid={:?}", pid_opt);

    let pid = pid_opt?;

    // Consume pre-captured AX text (one-time use via .take() to prevent stale data)
    let pre_captured = state
        .pre_captured_text
        .lock()
        .ok()
        .and_then(|mut pt| pt.take());

    let result = terminal::detect_full(pid, pre_captured);
    eprintln!(
        "[terminal] detect_full({}) returned: {:?}",
        pid,
        result.as_ref().map(|c| (&c.app_name, c.console_detected))
    );
    result
}
