use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Maximum number of history entries per window key.
/// Increased from 7 to 50 to match the configurable turn limit slider max (5-50).
/// Memory impact is negligible (~100KB per window at 50 entries).
pub const MAX_HISTORY_PER_WINDOW: usize = 50;

/// Maximum number of tracked windows in the history map.
/// When exceeded, the window with the oldest most-recent entry is evicted.
pub const MAX_TRACKED_WINDOWS: usize = 50;

/// A snapshot of terminal context at the time a query was made.
/// Stored alongside each history entry for potential AI follow-up context.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TerminalContextSnapshot {
    pub cwd: Option<String>,
    pub shell_type: Option<String>,
    pub visible_output: Option<String>,
}

/// A single history entry representing one query-response pair.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryEntry {
    /// The user's query text
    pub query: String,
    /// Full AI response text (explanation + command + warnings)
    pub response: String,
    /// Unix milliseconds when the entry was created
    pub timestamp: u64,
    /// Terminal context (CWD, shell, output) at time of query
    pub terminal_context: Option<TerminalContextSnapshot>,
    /// Whether the AI request failed/errored
    pub is_error: bool,
}

impl HistoryEntry {
    pub fn new(
        query: String,
        response: String,
        terminal_context: Option<TerminalContextSnapshot>,
        is_error: bool,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Self {
            query,
            response,
            timestamp,
            terminal_context,
            is_error,
        }
    }
}

/// Application state shared across Tauri commands.
///
/// All fields are Mutex-wrapped for thread-safe access from async command handlers
/// and the global hotkey callback which may fire on a different thread.
pub struct AppState {
    /// The currently registered hotkey string, e.g. "Super+K"
    pub hotkey: Mutex<String>,
    /// Timestamp of last hotkey trigger -- used for 200ms debounce
    /// to work around Tauri double-fire bug #10025
    pub last_hotkey_trigger: Mutex<Option<Instant>>,
    /// Current visibility state of the overlay window
    pub overlay_visible: Mutex<bool>,
    /// PID of the frontmost app captured BEFORE showing overlay.
    /// Populated in hotkey handler before show_and_make_key().
    /// Used by get_terminal_context to detect which terminal was active.
    pub previous_app_pid: Mutex<Option<i32>>,
    /// Pre-captured AX text from the frontmost app, read BEFORE the overlay
    /// steals focus. Written by the hotkey handler, consumed (`.take()`) by
    /// `get_app_context` so it is used at most once.
    pub pre_captured_text: Mutex<Option<String>>,
    /// The window key for the current overlay invocation.
    /// Format: "bundle_id:shell_pid" for terminals/IDEs, "bundle_id:app_pid" for other apps.
    /// Set synchronously in the hotkey handler before toggle_overlay().
    pub current_window_key: Mutex<Option<String>>,
    /// Pre-captured CWD from the focused terminal tab (AX-derived).
    /// Set in the hotkey handler for IDEs with integrated terminals before the overlay
    /// steals focus. Consumed by compute_window_key to disambiguate multi-tab shell PIDs.
    pub pre_captured_focused_cwd: Mutex<Option<String>>,
    /// Per-window query history. Key is the window key, value is a bounded deque of entries.
    /// Capped at MAX_HISTORY_PER_WINDOW entries per window, MAX_TRACKED_WINDOWS total windows.
    pub history: Mutex<HashMap<String, VecDeque<HistoryEntry>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            hotkey: Mutex::new("Super+K".to_string()),
            last_hotkey_trigger: Mutex::new(None),
            overlay_visible: Mutex::new(false),
            previous_app_pid: Mutex::new(None),
            pre_captured_text: Mutex::new(None),
            current_window_key: Mutex::new(None),
            pre_captured_focused_cwd: Mutex::new(None),
            history: Mutex::new(HashMap::new()),
        }
    }
}
