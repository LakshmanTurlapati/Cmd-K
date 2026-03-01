---
phase: 08-window-identification-history-storage
plan: 03
subsystem: terminal
tags: [accessibility, ax-api, cwd, process-tree, electron, cursor, vscode]

# Dependency graph
requires:
  - phase: 08-01
    provides: "Window key computation and find_shell_by_ancestry with highest-PID heuristic"
  - phase: 08-02
    provides: "Frontend IPC integration for window key and history"
provides:
  - "AX-based focused terminal tab CWD extraction for multi-tab IDE shell disambiguation"
  - "CWD-aware shell PID resolution in find_shell_by_ancestry"
  - "Pre-captured focused CWD in hotkey handler before overlay steals focus"
affects: [phase-9, phase-10]

# Tech tracking
tech-stack:
  added: []
  patterns: ["AX title parsing for CWD extraction", "CWD-based process disambiguation"]

key-files:
  created: []
  modified:
    - src-tauri/src/terminal/ax_reader.rs
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/commands/hotkey.rs
    - src-tauri/src/state.rs

key-decisions:
  - "Extract CWD from AXTitle first (more reliable than AXValue prompt parsing)"
  - "Handle tilde expansion and common tab title formats (zsh - /path, 1: zsh - /path)"
  - "Use 0.3s AX messaging timeout for focused CWD extraction (fast path in hotkey handler)"
  - "CWD matching inserted as Step 2.5 between mixed-type filter and highest-PID fallback"
  - "When multiple shells match same CWD, pick highest PID among matches as best effort"
  - "Fast path (find_shell_recursive) unchanged -- Terminal.app/iTerm2 never reach CWD matching"

patterns-established:
  - "Pre-capture pattern: AX data must be captured BEFORE overlay steals focus"
  - "Focused CWD flows through hotkey handler -> compute_window_key -> find_shell_pid -> find_shell_by_ancestry"

requirements-completed: [WKEY-01]

# Metrics
duration: 4min
completed: 2026-03-01
---

# Phase 08 Plan 03: Multi-Tab IDE Shell PID Resolution Summary

**AX-based focused terminal tab CWD extraction for Cursor/VS Code multi-tab shell disambiguation using AXTitle parsing and CWD-to-shell-PID matching**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-01T07:54:20Z
- **Completed:** 2026-03-01T07:58:15Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Added get_focused_terminal_cwd() to ax_reader.rs that extracts CWD from AXFocusedUIElement title/value text
- Hotkey handler now pre-captures focused terminal tab CWD for IDEs before overlay steals focus
- find_shell_by_ancestry now uses CWD matching (Step 2.5) to disambiguate between multiple candidate shells
- Terminal.app and iTerm2 behavior preserved unchanged (fast path never reaches CWD matching)
- Graceful fallback to highest-PID heuristic when AX data unavailable or no CWD match found

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AX-based focused terminal CWD extraction and pre-capture in hotkey handler** - `d11537c` (feat)
2. **Task 2: Add AX-aware shell selection in find_shell_by_ancestry** - `88cd478` (feat)

## Files Created/Modified
- `src-tauri/src/terminal/ax_reader.rs` - Added get_focused_terminal_cwd(), extract_dir_path_from_text(), try_as_dir_path() with tilde expansion and common tab title format parsing
- `src-tauri/src/terminal/process.rs` - Added focused_cwd parameter to find_shell_pid/find_shell_by_ancestry, CWD matching Step 2.5 before highest-PID fallback
- `src-tauri/src/commands/hotkey.rs` - Pre-capture focused CWD for IDEs, pass through to compute_window_key with new focused_cwd parameter
- `src-tauri/src/state.rs` - Added pre_captured_focused_cwd field to AppState

## Decisions Made
- Extract CWD from AXTitle before AXValue (tab titles in Cursor/VS Code are more reliable than parsing shell prompts)
- Use 0.3s AX messaging timeout for CWD extraction (shorter than the 1.0s general read timeout, since this is in the hotkey handler critical path)
- Only pre-capture focused CWD for IDEs (is_ide_with_terminal check) -- Terminal.app/iTerm2 use the fast path and don't need it
- CWD matching as Step 2.5 preserves existing Step 1 (sub-shell filter), Step 2 (mixed-type filter), and Step 3 (highest-PID fallback)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 8 gap closure complete: multi-tab IDE shell PID resolution now uses AX-derived CWD for disambiguation
- Ready for UAT re-test: Cursor with 3 terminal tabs in different directories should produce different window keys
- Phase 9 (query history UI) can proceed with confidence that window keys are stable per-tab

## Self-Check: PASSED

All files verified present. All commit hashes verified in git log.

---
*Phase: 08-window-identification-history-storage*
*Completed: 2026-03-01*
