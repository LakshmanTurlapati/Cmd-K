use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Update lifecycle status, drives tray menu text.
#[derive(Debug, Clone)]
pub enum UpdateStatus {
    /// No update activity -- menu shows "Check for Updates..."
    Idle,
    /// Actively checking for updates -- menu shows "Checking for Updates..."
    Checking,
    /// Update found -- menu shows "Update Available (vX.Y.Z)"
    Available(String),
    /// Downloading update -- menu shows "Downloading vX.Y.Z..."
    Downloading(String),
    /// Downloaded and ready -- menu shows "Update Ready (restart to apply)"
    Ready(String),
    /// Auto-check disabled by user -- menu shows "Check for Updates..."
    Disabled,
}

/// Managed state for the auto-updater, separate from AppState because
/// `tauri_plugin_updater::Update` is not `Default`.
pub struct UpdateState {
    /// Current update lifecycle status
    pub status: Mutex<UpdateStatus>,
    /// The pending update object (needed for install)
    pub pending_update: Mutex<Option<tauri_plugin_updater::Update>>,
    /// Downloaded update bytes (needed for install)
    pub pending_bytes: Mutex<Option<Vec<u8>>>,
    /// Reference to the tray menu item for text updates
    pub menu_item: Mutex<Option<tauri::menu::MenuItem<tauri::Wry>>>,
}

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

/// Token usage returned by streaming adapters after each query.
/// Fields are Option because some providers may not return usage data.
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

/// Accumulated token counts for a single provider+model pair.
#[derive(Debug, Clone, Default)]
pub struct UsageEntry {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub query_count: u32,
}

/// Per-query metadata stored for later cost calculation in usage.rs.
#[derive(Debug, Clone)]
pub struct QueryRecord {
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Session-scoped accumulator for token usage across all provider+model pairs.
/// Key is (provider_display_name, model_id) using Strings for decoupling.
#[derive(Debug, Default)]
pub struct UsageAccumulator {
    entries: HashMap<(String, String), UsageEntry>,
    query_history: Vec<QueryRecord>,
}

impl UsageAccumulator {
    /// Record token usage for a provider+model pair.
    /// Only adds non-None token values to the running totals.
    pub fn record(&mut self, provider: &str, model: &str, usage: &TokenUsage) {
        let key = (provider.to_string(), model.to_string());
        let entry = self.entries.entry(key).or_default();
        let input = usage.input_tokens.unwrap_or(0);
        let output = usage.output_tokens.unwrap_or(0);
        if usage.input_tokens.is_some() {
            entry.total_input_tokens += input;
        }
        if usage.output_tokens.is_some() {
            entry.total_output_tokens += output;
        }
        entry.query_count += 1;

        // Track per-query metadata for sparkline cost calculation
        if usage.input_tokens.is_some() || usage.output_tokens.is_some() {
            self.query_history.push(QueryRecord {
                provider: provider.to_string(),
                model: model.to_string(),
                input_tokens: input,
                output_tokens: output,
            });
        }
    }

    /// Clear all accumulated usage data.
    pub fn reset(&mut self) {
        self.entries.clear();
        self.query_history.clear();
    }

    /// Read access to the accumulated entries.
    pub fn entries(&self) -> &HashMap<(String, String), UsageEntry> {
        &self.entries
    }

    /// Read access to per-query history for cost calculation.
    pub fn query_history(&self) -> &[QueryRecord] {
        &self.query_history
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
    /// HWND of the foreground window captured BEFORE showing overlay (Windows only).
    /// Used by focus restoration on overlay dismiss.
    /// HWND is a pointer-sized integer (isize) on Windows.
    pub previous_hwnd: Mutex<Option<isize>>,
    /// Last dragged overlay position (logical coordinates).
    /// Session-scoped only — resets to None on app launch (Default impl).
    /// When None, position_overlay() uses the default centered position.
    pub last_position: Mutex<Option<(f64, f64)>>,
    /// Session-scoped token usage accumulator, keyed by (provider, model).
    pub usage: Mutex<UsageAccumulator>,
    /// Cached OpenRouter model pricing: model_id -> (input_price_per_m, output_price_per_m).
    pub openrouter_pricing: Mutex<HashMap<String, (f64, f64)>>,
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
            previous_hwnd: Mutex::new(None),
            last_position: Mutex::new(None),
            usage: Mutex::new(UsageAccumulator::default()),
            openrouter_pricing: Mutex::new(HashMap::new()),
        }
    }
}
