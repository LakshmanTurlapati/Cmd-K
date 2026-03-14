---
phase: 27-conpty-discovery-process-snapshot
plan: 03
subsystem: terminal-detection
tags: [uia, windows, shell-disambiguation, process-tree, conpty]

requires:
  - phase: 27-02
    provides: ConPTY-aware process snapshot and shell discovery pipeline
provides:
  - detect_shell_type_from_uia_text() for extracting shell type from UIA text
  - shell_type_hint parameter threaded through entire shell selection pipeline
  - UIA-guided multi-tab shell disambiguation
affects: [28-uia-verification, terminal-detection]

tech-stack:
  added: []
  patterns: [UIA text read before process tree walk, hint-based candidate filtering with fallback]

key-files:
  created: []
  modified:
    - src-tauri/src/terminal/detect_windows.rs
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/terminal/mod.rs
    - src-tauri/src/commands/hotkey.rs

key-decisions:
  - "Read UIA text before process tree walk, extract shell hint, pass through pipeline"
  - "Call detect_app_context_windows directly from detect_full_with_hwnd to avoid adding hint to generic dispatcher"
  - "Hint filtering falls back to pick_most_recent when no candidates match hint"

patterns-established:
  - "UIA-first pattern: read UIA text once, reuse for both shell hint extraction and WSL/output detection"
  - "Hint-based filtering: prefer matching candidates, fall back gracefully when no match"

requirements-completed: [PROC-01]

duration: 5min
completed: 2026-03-11
---

# Phase 27 Plan 03: Multi-Tab Shell Disambiguation Summary

**UIA text-guided shell selection that prefers focused tab's shell type over most-recent heuristic**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T14:44:30Z
- **Completed:** 2026-03-11T14:49:27Z
- **Tasks:** 1
- **Files modified:** 4

## Accomplishments
- Added detect_shell_type_from_uia_text() that maps UIA text patterns to shell types (cmd, powershell, pwsh, bash)
- Threaded shell_type_hint through full pipeline: detect_full_with_hwnd -> detect_app_context_windows -> get_foreground_info -> find_shell_pid -> find_shell_by_ancestry
- Restructured detect_full_with_hwnd to read UIA text BEFORE process tree walk (single read, reused for hint + WSL + visible_output)
- 9 new unit tests for detect_shell_type_from_uia_text, all 34 tests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add detect_shell_type_from_uia_text and thread shell_type_hint through pipeline** - `8a5d21e` (feat)

## Files Created/Modified
- `src-tauri/src/terminal/detect_windows.rs` - Added detect_shell_type_from_uia_text() and 9 unit tests
- `src-tauri/src/terminal/process.rs` - Added shell_type_hint param to get_foreground_info, find_shell_pid, find_shell_by_ancestry; pick_with_hint helper
- `src-tauri/src/terminal/mod.rs` - Restructured detect_full_with_hwnd to read UIA first, call detect_app_context_windows directly with hint
- `src-tauri/src/commands/hotkey.rs` - Updated find_shell_pid call site with new 4th parameter

## Decisions Made
- Read UIA text before process tree walk so shell hint is available during candidate selection
- Call detect_app_context_windows directly from detect_full_with_hwnd (bypasses generic dispatcher, avoids adding Windows-only param to cross-platform function)
- Hint filtering uses exe_to_shell_type for matching, falls back to unfiltered pick_most_recent when no candidates match

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Multi-tab shell disambiguation complete via UIA shell type hint
- Ready for UAT re-testing: PowerShell vs cmd tabs in VS Code and Windows Terminal
- WSL detection still works via existing UIA text fallback (unchanged)

---
*Phase: 27-conpty-discovery-process-snapshot*
*Completed: 2026-03-11*
