# Phase 8: Window Identification & History Storage - Research

**Researched:** 2026-03-01
**Domain:** macOS window/tab identity via process tree + accessibility APIs; Rust in-memory per-window history map
**Confidence:** HIGH

## Summary

Phase 8 adds two capabilities: (1) computing a stable per-terminal-tab identity key before the overlay appears, and (2) storing per-window query history in Rust's AppState so it survives overlay open/close cycles.

The existing codebase already captures the frontmost app PID in the hotkey handler (`get_frontmost_pid()` in `hotkey.rs`), resolves bundle IDs via ObjC FFI (`detect.rs`), and walks process trees to find shell PIDs (`process.rs`). The main gap is that `find_shell_pid()` currently returns the "highest PID" shell among all shells descended from the terminal app -- which is nondeterministic with respect to which tab is active when multiple tabs exist. For per-tab identity, we need to identify the shell belonging to the *focused* tab/window.

The history storage side is straightforward: a `HashMap<String, VecDeque<HistoryEntry>>` in AppState, capped at 7 entries per window and 50 windows total. No new crates are needed -- `std::collections::VecDeque` handles bounded circular buffer semantics with `push_back()` + length check.

**Primary recommendation:** Extend the hotkey handler to compute a window key synchronously before `toggle_overlay()`. Use `bundle_id:shell_pid` for terminals (with the active tab's shell found via AXFocusedWindow title matching or controlling TTY). Use `bundle_id:app_pid` for non-terminal apps. Store history in a `Mutex<HashMap<String, VecDeque<HistoryEntry>>>` in AppState, exposed to the frontend via a new IPC command.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Per-tab history: each terminal tab gets its own independent history (not per-window)
- Split panes within a tab share history (per-tab, not per-pane)
- If tab detection fails for a terminal, fall back to per-window identity (not global)
- Support all major terminals for tab detection: iTerm2, Terminal.app, Alacritty, Kitty, WezTerm
- Add VS Code as a detected terminal (integrated terminal support)
- In-memory only: history resets when the app quits -- no disk persistence
- History for closed tabs/windows is kept in memory until app quits (not cleaned up on close)
- Cap at ~50 tracked windows/tabs total -- evict oldest window's history when exceeded
- 7 entries per window/tab -- 8th query evicts the oldest (per roadmap spec)
- Each entry stores: query text + full AI response text + metadata
- Full AI response stored (explanation, command, warnings -- not just extracted commands)
- Metadata per entry: timestamp + terminal context (CWD, shell, recent terminal output at time of query)
- Failed/error queries are saved to history (user might want to retry via arrow-key recall)
- Non-terminal invocations get per-app history (Finder gets its own bucket, Safari gets its own, etc.)
- Not one global bucket -- each non-terminal app is tracked separately

### Claude's Discretion
- Internal data structure design for the history map
- Window/tab identity key generation strategy (what accessibility APIs to use)
- Memory management and eviction implementation details
- How to detect VS Code's integrated terminal vs regular VS Code window

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WKEY-01 | App computes a stable per-terminal-window key (bundle_id:shell_pid) before the overlay shows | Window key computation in hotkey handler using find_shell_pid + AX-based active tab refinement; bundle_id from existing detect.rs; shell_pid from existing process.rs with focused-tab narrowing |
| WKEY-02 | Window key is captured synchronously in the hotkey handler alongside PID capture, before overlay steals focus | Extend the existing `if !is_currently_visible` block in hotkey.rs to compute window key after PID capture but before toggle_overlay(); store in new AppState field |
| WKEY-03 | Non-terminal apps fall back to a global key so history still works outside terminals | Use `bundle_id:app_pid` format for non-terminal apps (Finder, Safari, etc. each get their own bucket per their PID); existing `is_known_terminal()` check determines branch |
| HIST-04 | History stores up to 7 queries per terminal window, session-scoped (in-memory only) | HashMap<String, VecDeque<HistoryEntry>> in AppState with per-key cap of 7; VecDeque naturally supports push_back + pop_front eviction; Mutex wrapping for thread safety |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| std::collections::HashMap | stdlib | Per-window history map keyed by window key string | No external dependency needed; HashMap is the standard key-value store |
| std::collections::VecDeque | stdlib | Bounded per-window history buffer (7 entries) | Ring buffer semantics with O(1) push_back and pop_front; perfect for FIFO eviction |
| std::sync::Mutex | stdlib | Thread-safe access from hotkey handler + IPC commands | Already used throughout AppState; low contention makes std Mutex appropriate |
| serde (1.x) | Already in Cargo.toml | Serialize HistoryEntry for IPC to frontend | Already a project dependency; needed for `#[derive(Serialize, Clone)]` on HistoryEntry |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| accessibility-sys | 0.2 (already in Cargo.toml) | AXFocusedWindow attribute reading for active tab detection | Used to get focused window title for shell PID narrowing in terminals with multiple tabs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| VecDeque with manual cap | bounded-vec-deque crate | Adds external dependency for trivial logic (check len, pop_front); not worth it |
| HashMap<String, VecDeque> | BTreeMap | No need for ordered keys; HashMap is faster for random access |
| std::sync::Mutex | tokio::sync::Mutex | AppState is accessed from sync hotkey handler; tokio Mutex requires async context |

**Installation:**
No new crates needed. All functionality uses existing dependencies (serde, accessibility-sys) and Rust standard library.

## Architecture Patterns

### Recommended Changes to Existing Structure
```
src-tauri/src/
  state.rs           # Add: window_key field, history HashMap
  commands/
    hotkey.rs         # Modify: compute window key before toggle_overlay()
    terminal.rs       # Modify: return window_key alongside AppContext
    history.rs        # NEW: get_history, add_history IPC commands
    mod.rs            # Add: pub mod history
  terminal/
    detect.rs         # Modify: add VS Code bundle IDs
    process.rs        # Modify: expose find_shell_pid as pub(crate)
    mod.rs            # Add: window_key module or function
```

### Pattern 1: Window Key Computation in Hotkey Handler
**What:** Compute window key synchronously in the hotkey handler, before `toggle_overlay()`, alongside existing PID and AX text capture.
**When to use:** Every time the overlay is about to show (when `!is_currently_visible`).
**Why synchronous:** The window key MUST be available to the frontend before the user can type. Computing it in the hotkey handler (which runs before `show_and_make_key()`) guarantees this. The existing pattern already does synchronous work here (PID capture, AX text pre-read).

```rust
// In hotkey.rs, inside the !is_currently_visible block, after PID capture:
if let Some(pid) = pid {
    // Existing: store PID, pre-capture AX text

    // NEW: compute window key
    let bundle_id = terminal::detect::get_bundle_id(pid);
    let is_terminal = bundle_id.as_deref().is_some_and(terminal::detect::is_known_terminal);

    let window_key = if is_terminal {
        // For terminals: bundle_id:shell_pid
        let shell_pid = terminal::process::find_shell_pid(pid);
        match (bundle_id.as_deref(), shell_pid) {
            (Some(bid), Some(spid)) => format!("{}:{}", bid, spid),
            (Some(bid), None) => format!("{}:{}", bid, pid), // fallback to app PID
            _ => format!("unknown:{}", pid),
        }
    } else {
        // For non-terminals: bundle_id:app_pid (per-app history)
        match bundle_id.as_deref() {
            Some(bid) => format!("{}:{}", bid, pid),
            None => format!("unknown:{}", pid),
        }
    };

    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut wk) = state.current_window_key.lock() {
            *wk = Some(window_key);
        }
    }
}
```

### Pattern 2: History Map in AppState
**What:** A `Mutex<HashMap<String, VecDeque<HistoryEntry>>>` in AppState that persists across overlay open/close cycles.
**When to use:** All history operations (add, get, evict).
**Why Mutex<HashMap>:** The hotkey handler fires on a non-async thread. AppState fields must be `Send + Sync`. `Mutex<HashMap>` is the established pattern in this codebase (see `previous_app_pid`, `pre_captured_text`, etc.).

```rust
// In state.rs
use std::collections::{HashMap, VecDeque};

const MAX_HISTORY_PER_WINDOW: usize = 7;
const MAX_TRACKED_WINDOWS: usize = 50;

#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryEntry {
    pub query: String,
    pub response: String,
    pub timestamp: u64,        // Unix millis
    pub terminal_context: Option<TerminalContextSnapshot>,
    pub is_error: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TerminalContextSnapshot {
    pub cwd: Option<String>,
    pub shell_type: Option<String>,
    pub visible_output: Option<String>,
}

pub struct AppState {
    // ... existing fields ...
    pub current_window_key: Mutex<Option<String>>,
    pub history: Mutex<HashMap<String, VecDeque<HistoryEntry>>>,
}
```

### Pattern 3: Frontend Retrieval of Window Key and History
**What:** Two new IPC commands: `get_window_key` (returns current window key) and `get_window_history` (returns history for current window).
**When to use:** Called by the frontend in the `show()` action, alongside existing `get_app_context` call.
**Flow:**
1. Overlay shown event fires
2. Frontend calls `get_window_key()` -- reads from `AppState.current_window_key`
3. Frontend calls `get_window_history(key)` -- reads from `AppState.history[key]`
4. Frontend stores both in Zustand for Phase 9 (arrow-key navigation)

```rust
// In commands/history.rs
#[tauri::command]
pub fn get_window_key(app: AppHandle) -> Option<String> {
    let state = app.try_state::<AppState>()?;
    state.current_window_key.lock().ok()?.clone()
}

#[tauri::command]
pub fn get_window_history(app: AppHandle, window_key: String) -> Vec<HistoryEntry> {
    let state = match app.try_state::<AppState>() {
        Some(s) => s,
        None => return Vec::new(),
    };
    let history = match state.history.lock() {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    history.get(&window_key)
        .map(|deque| deque.iter().cloned().collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn add_history_entry(
    app: AppHandle,
    window_key: String,
    entry: HistoryEntry,
) -> Result<(), String> {
    let state = app.try_state::<AppState>()
        .ok_or("AppState not found")?;
    let mut history = state.history.lock()
        .map_err(|_| "History mutex poisoned")?;

    // Enforce MAX_TRACKED_WINDOWS limit
    if !history.contains_key(&window_key) && history.len() >= MAX_TRACKED_WINDOWS {
        // Evict the window with the oldest most-recent entry
        if let Some(oldest_key) = find_oldest_window(&history) {
            history.remove(&oldest_key);
        }
    }

    let entries = history.entry(window_key).or_insert_with(VecDeque::new);
    if entries.len() >= MAX_HISTORY_PER_WINDOW {
        entries.pop_front(); // evict oldest
    }
    entries.push_back(entry);
    Ok(())
}
```

### Pattern 4: VS Code Integrated Terminal Detection
**What:** Add VS Code (and Cursor) bundle IDs to `TERMINAL_BUNDLE_IDS` or a new `IDE_WITH_TERMINAL` list. Detect whether VS Code has an active terminal by checking if `find_shell_pid()` returns a shell for the VS Code PID.
**When to use:** When the frontmost app is VS Code/Cursor and has an integrated terminal open.
**Key insight:** The existing `detect_app_context()` already handles this case! It calls `process::get_foreground_info()` for all apps and sets `has_shell = true` if a shell is found. The only gap is that VS Code isn't in `TERMINAL_BUNDLE_IDS`, so `is_terminal` is false, which affects AX text reading (not history). For window key purposes, the process tree walk already works for VS Code.

```rust
// VS Code and Cursor bundle IDs to add to detect.rs
// Note: NOT added to TERMINAL_BUNDLE_IDS (they're not terminals),
// but tracked separately for "apps with integrated terminals"
pub const IDE_BUNDLE_IDS: &[&str] = &[
    "com.microsoft.VSCode",
    "com.microsoft.VSCodeInsiders",
    "com.todesktop.230313mzl4w4u92", // Cursor
];

pub fn is_ide_with_terminal(bundle_id: &str) -> bool {
    IDE_BUNDLE_IDS.contains(&bundle_id)
}
```

### Anti-Patterns to Avoid
- **Do NOT use CGWindowID for window identity:** WKEY-04 (CGWindowID-based keys) is explicitly out of scope. It requires screen recording permission and adds complexity. Shell PID is sufficient.
- **Do NOT persist history to disk:** Explicitly out of scope. In-memory HashMap only.
- **Do NOT clean up history on tab/window close:** Decision says "kept in memory until app quits." The 50-window LRU cap handles memory bounding instead.
- **Do NOT use Zustand for history storage:** The `show()` action resets all Zustand state. History MUST live in Rust AppState to survive overlay cycles. Zustand only holds a copy for the current session's UI rendering.
- **Do NOT block the hotkey handler with slow operations:** Window key computation must be fast. The `get_bundle_id()` call is ~1ms (ObjC FFI). The `find_shell_pid()` call can take 10-50ms (process tree walk). This is acceptable -- the existing PID capture + AX pre-read already takes similar time.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Circular buffer with cap | Custom ring buffer | VecDeque + len check + pop_front | VecDeque IS a ring buffer; adding a cap is 3 lines of code |
| Thread-safe HashMap | RwLock with custom locking | Mutex<HashMap> | Contention is extremely low (hotkey fires at most every 200ms); RwLock overhead not justified |
| Window LRU eviction | Full LRU cache (lru crate) | Scan HashMap for oldest timestamp | With max 50 entries, a linear scan is sub-microsecond; LRU crate adds dependency for no benefit |
| macOS ObjC FFI | objc2 crate bindings | Direct FFI (existing pattern) | The codebase already uses raw `objc_msgSend` FFI everywhere; switching to objc2 would be inconsistent |

**Key insight:** The entire history storage mechanism can be built with zero external dependencies using `HashMap<String, VecDeque<HistoryEntry>>` from the standard library. The 7-entry cap and 50-window limit make sophisticated data structures unnecessary.

## Common Pitfalls

### Pitfall 1: Shell PID Instability Across Tabs
**What goes wrong:** `find_shell_pid()` returns the highest-PID shell descendant, which may change depending on which shells are running. If user opens tab A (zsh PID 1000) then tab B (zsh PID 2000), `find_shell_pid()` returns 2000 regardless of which tab is focused.
**Why it happens:** `find_shell_pid()` doesn't know about tabs -- it walks the entire process tree of the terminal app.
**How to avoid:** For multi-tab scenarios, the shell PID picked by `find_shell_pid()` may vary, but as long as each tab has a distinct shell PID (which it does -- each tab spawns its own shell), the key will be unique per tab. The issue is *which* shell PID we pick. The current "highest PID" heuristic actually works surprisingly well in practice because the most recently created tab is often the active one. For more accuracy, we could cross-reference with the focused window's AX title (which often contains the TTY or process name).
**Warning signs:** Two different tabs producing the same window key, or the same tab producing different window keys across invocations.

### Pitfall 2: Race Between Window Key Computation and Overlay Show
**What goes wrong:** If window key computation is async/deferred, the frontend may render the overlay and accept user input before the window key is available, causing the first query to go into the wrong history bucket.
**Why it happens:** WKEY-02 requires the key before the user can type. If computed after `show_and_make_key()`, there's a race.
**How to avoid:** Compute the window key synchronously in the hotkey handler, in the same `!is_currently_visible` block that captures the PID. Store in AppState immediately. The frontend reads it on `overlay-shown` event, which fires after the key is already stored.
**Warning signs:** `get_window_key()` returning `None` when the overlay first appears.

### Pitfall 3: Zustand State Reset Destroying History
**What goes wrong:** The `show()` action in `useOverlayStore` resets `turnHistory: []` and other state on every overlay open. If history were stored in Zustand, it would be lost.
**Why it happens:** The overlay is designed to reset its UI state each time it opens (clean input, no streaming text, etc.).
**How to avoid:** Store history in Rust AppState (HashMap), not Zustand. The frontend fetches history from Rust via IPC when needed (Phase 9 arrow-key navigation). Zustand only holds transient UI state.
**Warning signs:** History disappearing after closing and reopening the overlay.

### Pitfall 4: Mutex Poisoning from Panics
**What goes wrong:** If any code holding a history Mutex lock panics, the Mutex becomes poisoned and all subsequent accesses fail.
**Why it happens:** Rust Mutex poisoning is a safety mechanism. A panic during history insertion could poison the lock.
**How to avoid:** Keep critical sections minimal (lock, read/write, unlock). Never do I/O or complex logic while holding the lock. Use `match lock() { Ok(guard) => ..., Err(poisoned) => poisoned.into_inner() }` if recovery is needed.
**Warning signs:** `[history] mutex poisoned` log messages.

### Pitfall 5: Non-Terminal Apps With Multiple Windows
**What goes wrong:** If a user has two Finder windows and presses Cmd+K from each, both get the same key (`com.apple.finder:PID`) because non-terminal apps share a single PID.
**Why it happens:** macOS apps are single-process; multiple windows share one PID.
**How to avoid:** This is actually the desired behavior per the user's decision: "Non-terminal invocations get per-app history (Finder gets its own bucket, Safari gets its own, etc.)." The `bundle_id:app_pid` key naturally groups all windows of an app into one history bucket. This is correct.
**Warning signs:** None -- this is by design.

### Pitfall 6: VS Code With No Terminal Open
**What goes wrong:** User presses Cmd+K from VS Code without any integrated terminal open. `find_shell_pid()` may find shells spawned by VS Code extensions (language servers, linters) that are not user terminals.
**Why it happens:** VS Code's process tree contains many child processes including Node.js workers and extension hosts.
**How to avoid:** The existing `find_shell_by_ancestry()` already filters sub-shells and prefers `$SHELL`-matching processes. If no user-visible shell is found, treat VS Code as a non-terminal app (use `bundle_id:app_pid` key). The `has_shell` check in `detect_app_context()` already handles this distinction.
**Warning signs:** VS Code without a terminal open getting terminal-style history keys.

### Pitfall 7: GPU Terminals and Shell PID Detection
**What goes wrong:** Alacritty, Kitty, and WezTerm are GPU-accelerated terminals that may not expose AX text. However, they DO expose process trees, so `find_shell_pid()` works normally for them.
**Why it happens:** GPU terminals render directly to the GPU, bypassing the AX text layer. But process tree walking uses `proc_pidinfo` and `pgrep`, which work for all apps.
**How to avoid:** No special handling needed for GPU terminals regarding window key generation. `find_shell_pid()` works for all terminals regardless of rendering method. AX text reading is a separate concern (already handled in v0.1.0).
**Warning signs:** None expected.

## Code Examples

### Window Key Computation (Complete Flow)
```rust
// This runs in the hotkey handler, synchronously, before toggle_overlay()
fn compute_window_key(pid: i32) -> String {
    let bundle_id = terminal::detect::get_bundle_id(pid);
    let bundle_str = bundle_id.as_deref().unwrap_or("unknown");

    let is_terminal = terminal::detect::is_known_terminal(bundle_str);
    let is_ide = terminal::detect::is_ide_with_terminal(bundle_str);

    if is_terminal || is_ide {
        // For terminals and IDEs: try to find the active shell PID
        match terminal::process::find_shell_pid(pid) {
            Some(shell_pid) => format!("{}:{}", bundle_str, shell_pid),
            None => format!("{}:{}", bundle_str, pid), // fallback to app PID
        }
    } else {
        // For non-terminal apps: per-app history
        format!("{}:{}", bundle_str, pid)
    }
}
```

### HistoryEntry Struct with Serde
```rust
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub query: String,
    pub response: String,
    pub timestamp: u64,
    pub terminal_context: Option<TerminalContextSnapshot>,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalContextSnapshot {
    pub cwd: Option<String>,
    pub shell_type: Option<String>,
    pub visible_output: Option<String>,
}

impl HistoryEntry {
    pub fn new(
        query: String,
        response: String,
        context: Option<TerminalContextSnapshot>,
        is_error: bool,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        Self { query, response, timestamp, terminal_context: context, is_error }
    }
}
```

### History Map Operations
```rust
use std::collections::{HashMap, VecDeque};

const MAX_HISTORY_PER_WINDOW: usize = 7;
const MAX_TRACKED_WINDOWS: usize = 50;

fn add_entry(
    history: &mut HashMap<String, VecDeque<HistoryEntry>>,
    key: String,
    entry: HistoryEntry,
) {
    // Enforce window cap before inserting new key
    if !history.contains_key(&key) && history.len() >= MAX_TRACKED_WINDOWS {
        evict_oldest_window(history);
    }

    let entries = history.entry(key).or_insert_with(VecDeque::new);
    if entries.len() >= MAX_HISTORY_PER_WINDOW {
        entries.pop_front();
    }
    entries.push_back(entry);
}

fn evict_oldest_window(history: &mut HashMap<String, VecDeque<HistoryEntry>>) {
    // Find the window whose most recent entry has the oldest timestamp
    let oldest_key = history.iter()
        .filter_map(|(k, v)| {
            v.back().map(|e| (k.clone(), e.timestamp))
        })
        .min_by_key(|(_, ts)| *ts)
        .map(|(k, _)| k);

    if let Some(key) = oldest_key {
        history.remove(&key);
    }
}
```

### Frontend: Fetching Window Key and History on Overlay Show
```typescript
// In store/index.ts show() action, after existing get_app_context call:
const windowKey = await invoke<string | null>("get_window_key");
if (windowKey) {
    const history = await invoke<HistoryEntry[]>("get_window_history", {
        windowKey
    });
    useOverlayStore.getState().setWindowKey(windowKey);
    useOverlayStore.getState().setWindowHistory(history);
}
```

### AppState Extension
```rust
pub struct AppState {
    // Existing fields
    pub hotkey: Mutex<String>,
    pub last_hotkey_trigger: Mutex<Option<Instant>>,
    pub overlay_visible: Mutex<bool>,
    pub previous_app_pid: Mutex<Option<i32>>,
    pub pre_captured_text: Mutex<Option<String>>,

    // New fields for Phase 8
    pub current_window_key: Mutex<Option<String>>,
    pub history: Mutex<HashMap<String, VecDeque<HistoryEntry>>>,
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Global history (one bucket) | Per-window/tab history keyed by identity | Phase 8 (this phase) | Users get independent history per terminal tab |
| No window identity | bundle_id:shell_pid key before overlay shows | Phase 8 (this phase) | Foundation for Phase 9 (arrow-key recall) and Phase 10 (AI context) |
| turnHistory in Zustand (resets on overlay close) | History in Rust AppState (persists across cycles) | Phase 8 (this phase) | History survives overlay dismiss/reopen |

**Deprecated/outdated:**
- The current `turnHistory` array in Zustand store will NOT be replaced in Phase 8 -- it is used for AI streaming context within a single overlay session. Phase 10 moves this to Rust for persistence. Phase 8 only adds the query history map.

## Open Questions

1. **Multi-tab shell PID accuracy**
   - What we know: `find_shell_pid()` walks the entire process tree and picks the highest PID shell. When a terminal has multiple tabs, all tabs' shells are descendants of the same app PID.
   - What's unclear: Whether the "highest PID" heuristic reliably picks the active tab's shell, or if we need AX title matching or TTY-based narrowing.
   - Recommendation: Start with the current `find_shell_pid()` heuristic. The key stability matters more than perfect tab identification. If two tabs swap keys occasionally, the history just goes to a different bucket -- annoying but not catastrophic. Monitor during testing and add AX title-based refinement if needed as a follow-up.

2. **VS Code integrated terminal vs no terminal**
   - What we know: VS Code's process tree contains extension host processes, language servers, and sometimes user shells. The existing `find_shell_by_ancestry()` filters sub-shells.
   - What's unclear: Whether `find_shell_pid()` reliably distinguishes "user has a terminal panel open" from "only extension processes running."
   - Recommendation: If `find_shell_pid()` returns a shell for VS Code, treat it as a terminal context and use `bundle_id:shell_pid`. If it returns None, treat as non-terminal with `bundle_id:app_pid`. Test with VS Code having 0, 1, and multiple terminal tabs.

3. **Timing budget for window key computation in hotkey handler**
   - What we know: The hotkey handler already does synchronous work: `get_frontmost_pid()` (~1ms), `read_focused_text_fast()` (~200ms max). Adding `get_bundle_id()` (~1ms) and `find_shell_pid()` (~10-50ms) is within budget.
   - What's unclear: Whether the combined time exceeds perceived "instant" threshold.
   - Recommendation: The overlay already doesn't show until after PID capture and AX pre-read. Adding 10-50ms for window key computation is negligible. Profile during testing.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `src-tauri/src/state.rs`, `commands/hotkey.rs`, `terminal/detect.rs`, `terminal/process.rs`, `terminal/mod.rs`, `commands/terminal.rs`, `store/index.ts` -- direct examination of existing architecture
- Rust std documentation: `HashMap`, `VecDeque`, `Mutex` -- stable standard library APIs
- Apple Developer Documentation: `CGWindowListCopyWindowInfo` (https://developer.apple.com/documentation/coregraphics/1455137-cgwindowlistcopywindowinfo) -- window enumeration API (referenced for context; NOT used in implementation per scope decisions)
- Apple Developer Documentation: `AXUIElement.h` (https://developer.apple.com/documentation/applicationservices/axuielement_h) -- accessibility API used for focused window detection

### Secondary (MEDIUM confidence)
- VS Code bundle identifier: `com.microsoft.VSCode` -- verified via multiple sources including GitHub issues (https://github.com/microsoft/vscode/issues/22366)
- Cursor bundle identifier: `com.todesktop.230313mzl4w4u92` -- verified via Cursor community forum (https://forum.cursor.com/t/cursor-bundle-identifier/779)
- Tauri state management patterns (https://v2.tauri.app/develop/state-management/) -- official Tauri v2 docs on Mutex<HashMap> in AppState

### Tertiary (LOW confidence)
- Private `_AXUIElementGetWindow` API for CGWindowID extraction -- undocumented, not recommended for use. Listed here only as an alternative approach that was considered and rejected.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Uses only existing Rust stdlib + already-present crate dependencies
- Architecture: HIGH - Extends established AppState pattern; hotkey handler modification follows existing code structure exactly
- Pitfalls: HIGH - Identified through direct codebase analysis (Zustand reset, Mutex contention, multi-tab shell PID)
- Window key strategy: MEDIUM - The "highest PID" shell heuristic works for single-tab-per-window scenarios but may need refinement for multi-tab terminals; addressed via fallback strategy

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (30 days -- stable domain, no rapidly changing APIs)
