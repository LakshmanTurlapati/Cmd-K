/// IPC commands for per-window query history.
///
/// Three commands:
/// - get_window_key: returns the current window key from AppState
/// - get_window_history: returns all history entries for a given window key
/// - add_history_entry: adds a new entry, enforcing per-window and total-window caps

/// Returns the current window key stored in AppState.
///
/// The window key is computed synchronously in the hotkey handler before
/// toggle_overlay() is called, so it reflects the app/terminal that was
/// frontmost when the user pressed the hotkey.
#[tauri::command]
pub fn get_window_key(app: tauri::AppHandle) -> Option<String> {
    use tauri::Manager;
    let state = app.try_state::<crate::state::AppState>()?;
    let key = state.current_window_key.lock().ok()?.clone();
    key
}

/// Returns all history entries for the given window key.
///
/// Returns an empty Vec if the key has no history or if state access fails.
/// Entries are ordered oldest-first (front of deque = oldest).
#[tauri::command]
pub fn get_window_history(
    app: tauri::AppHandle,
    window_key: String,
) -> Vec<crate::state::HistoryEntry> {
    use tauri::Manager;
    let state = match app.try_state::<crate::state::AppState>() {
        Some(s) => s,
        None => return Vec::new(),
    };
    let history = match state.history.lock() {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    history
        .get(&window_key)
        .map(|deque| deque.iter().cloned().collect())
        .unwrap_or_default()
}

/// Adds a new history entry for the given window key.
///
/// Enforces two caps:
/// - MAX_HISTORY_PER_WINDOW (7): oldest entry evicted when full
/// - MAX_TRACKED_WINDOWS (50): when adding a new key, evicts the window
///   whose most-recent entry has the oldest timestamp
///
/// The timestamp is auto-generated server-side via HistoryEntry::new().
/// Frontend passes raw data (query, response, context, is_error) -- no need
/// to construct the exact Rust struct shape or generate timestamps.
#[tauri::command]
pub fn add_history_entry(
    app: tauri::AppHandle,
    window_key: String,
    query: String,
    response: String,
    terminal_context: Option<crate::state::TerminalContextSnapshot>,
    is_error: bool,
) -> Result<(), String> {
    use crate::state::{HistoryEntry, MAX_HISTORY_PER_WINDOW, MAX_TRACKED_WINDOWS};
    use tauri::Manager;

    let state = app
        .try_state::<crate::state::AppState>()
        .ok_or("AppState not found")?;
    let mut history = state
        .history
        .lock()
        .map_err(|_| "History mutex poisoned".to_string())?;

    // Enforce MAX_TRACKED_WINDOWS limit before adding a new key
    if !history.contains_key(&window_key) && history.len() >= MAX_TRACKED_WINDOWS {
        // Evict window with oldest most-recent entry
        let oldest_key = history
            .iter()
            .filter_map(|(k, v)| v.back().map(|e| (k.clone(), e.timestamp)))
            .min_by_key(|(_, ts)| *ts)
            .map(|(k, _)| k);
        if let Some(key) = oldest_key {
            eprintln!("[history] evicting oldest window: {}", key);
            history.remove(&key);
        }
    }

    let entries = history
        .entry(window_key.clone())
        .or_insert_with(std::collections::VecDeque::new);
    if entries.len() >= MAX_HISTORY_PER_WINDOW {
        entries.pop_front(); // evict oldest entry
    }

    let entry = HistoryEntry::new(query, response, terminal_context, is_error);
    entries.push_back(entry);
    eprintln!(
        "[history] added entry for window '{}', total entries: {}",
        window_key,
        entries.len()
    );

    Ok(())
}
