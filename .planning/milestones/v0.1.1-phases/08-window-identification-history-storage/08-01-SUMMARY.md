---
phase: 08-window-identification-history-storage
plan: 01
subsystem: state
tags: [rust, tauri, ipc, window-key, history, mutex, hashmap]

# Dependency graph
requires:
  - phase: 05-terminal-context
    provides: "Terminal detection (bundle IDs, process tree walking, find_shell_pid)"
provides:
  - "AppState.current_window_key for per-invocation window identity"
  - "AppState.history HashMap for per-window query history"
  - "HistoryEntry and TerminalContextSnapshot structs"
  - "get_window_key, get_window_history, add_history_entry IPC commands"
  - "VS Code/Cursor IDE detection via IDE_BUNDLE_IDS"
  - "compute_window_key in hotkey handler (sync, before toggle_overlay)"
affects: [08-02, 09-arrow-key-history, 10-ai-followup-context]

# Tech tracking
tech-stack:
  added: []
  patterns: [bounded-deque-history, window-key-identity, sync-pre-overlay-capture]

key-files:
  created:
    - src-tauri/src/commands/history.rs
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/terminal/detect.rs
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/commands/hotkey.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Window key format: bundle_id:shell_pid for terminals/IDEs, bundle_id:app_pid for other apps"
  - "History entry uses individual IPC parameters (not struct) so frontend does not need to generate timestamps"
  - "Oldest-window eviction when MAX_TRACKED_WINDOWS (50) exceeded, oldest-entry eviction when MAX_HISTORY_PER_WINDOW (7) exceeded"

patterns-established:
  - "Sync pre-overlay capture: compute_window_key runs before toggle_overlay in hotkey handler"
  - "Bounded deque history: VecDeque with pop_front eviction for per-window caps"
  - "IDE terminal detection: separate IDE_BUNDLE_IDS list alongside TERMINAL_BUNDLE_IDS"

requirements-completed: [WKEY-01, WKEY-02, WKEY-03, HIST-04]

# Metrics
duration: 5min
completed: 2026-03-01
---

# Phase 8 Plan 01: Window Identification & History Storage Summary

**Per-window identity via bundle_id:shell_pid keys computed in hotkey handler, with bounded HashMap history (7/window, 50 windows) exposed through 3 IPC commands**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-01T06:51:18Z
- **Completed:** 2026-03-01T06:56:18Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments
- Extended AppState with current_window_key (Mutex<Option<String>>) and history (Mutex<HashMap<String, VecDeque<HistoryEntry>>>)
- Window key computed synchronously in hotkey handler before toggle_overlay, capturing shell PID while terminal is still frontmost
- VS Code and Cursor IDE detection via IDE_BUNDLE_IDS, giving IDE terminal tabs per-tab history buckets
- Three IPC commands (get_window_key, get_window_history, add_history_entry) with bounded eviction

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend AppState, add VS Code detection, compute window key** - `cd2ceec` (feat)
2. **Task 2: Create history IPC commands and register in lib.rs** - `4e43bb4` (feat)

## Files Created/Modified
- `src-tauri/src/state.rs` - Added HistoryEntry, TerminalContextSnapshot structs, MAX_HISTORY_PER_WINDOW/MAX_TRACKED_WINDOWS constants, current_window_key and history fields to AppState
- `src-tauri/src/terminal/detect.rs` - Added IDE_BUNDLE_IDS constant and is_ide_with_terminal function
- `src-tauri/src/terminal/process.rs` - Changed find_shell_pid visibility to pub(crate)
- `src-tauri/src/commands/hotkey.rs` - Added compute_window_key function and window key storage before toggle_overlay
- `src-tauri/src/commands/history.rs` - New file with get_window_key, get_window_history, add_history_entry IPC commands
- `src-tauri/src/commands/mod.rs` - Added pub mod history declaration
- `src-tauri/src/lib.rs` - Imported and registered 3 new history IPC commands

## Decisions Made
- Window key format: bundle_id:shell_pid for terminals and IDEs (per-tab identity), bundle_id:app_pid for other apps (per-process identity)
- History entry IPC uses individual parameters (query, response, terminal_context, is_error) rather than a HistoryEntry struct, so the frontend does not need to construct timestamps
- Eviction strategy: oldest-entry per window (pop_front on VecDeque), oldest-window overall (min timestamp of most-recent entry)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow lifetime in get_window_key**
- **Found during:** Task 2 (Create history IPC commands)
- **Issue:** Rust borrow checker rejected `state.current_window_key.lock().ok()?.clone()` as a tail expression due to temporary MutexGuard lifetime
- **Fix:** Saved result to local variable before returning: `let key = state.current_window_key.lock().ok()?.clone(); key`
- **Files modified:** src-tauri/src/commands/history.rs
- **Verification:** cargo check passes with zero errors
- **Committed in:** 4e43bb4 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Standard Rust lifetime fix, no scope change.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Rust backend for window identity and history is complete
- Ready for Plan 02: TypeScript IPC bindings and frontend integration
- get_window_key available for frontend to read current window identity on overlay show
- get_window_history/add_history_entry ready for arrow-key navigation (Phase 9)

---
*Phase: 08-window-identification-history-storage*
*Completed: 2026-03-01*
