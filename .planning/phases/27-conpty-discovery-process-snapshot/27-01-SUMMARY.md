---
phase: 27-conpty-discovery-process-snapshot
plan: 01
subsystem: terminal-detection
tags: [windows, conpty, process-snapshot, peb, cmd-filtering]

requires:
  - phase: none
    provides: existing PEB CWD reading pattern in process.rs
provides:
  - ProcessSnapshot struct with single-snapshot capture()
  - read_command_line_from_peb() for PEB command line reading at offset 0x70
  - extract_cmd_args() and has_batch_flag_in_cmdline() for cmd.exe flag parsing
  - is_interactive_cmd() with two-signal approach
  - check_batch_flags() helper for flag pattern matching
affects: [27-02, conpty-shell-discovery, cmd-exe-filtering]

tech-stack:
  added: []
  patterns: [two-signal-cmd-detection, single-process-snapshot, peb-command-line-reading]

key-files:
  created: []
  modified:
    - src-tauri/src/terminal/process.rs

key-decisions:
  - "Check both conhost.exe and OpenConsole.exe as ConPTY hosts for Win10/Win11 compatibility"
  - "Conservative fallback: treat cmd.exe as interactive when PEB read fails"
  - "Cross-platform string parsing functions enable unit testing on all platforms"

patterns-established:
  - "ProcessSnapshot::capture() pattern: single CreateToolhelp32Snapshot building all maps at once"
  - "Two-signal cmd.exe detection: fast parent check + definitive PEB command line check"

requirements-completed: [PROC-03, PROC-02]

duration: 11min
completed: 2026-03-11
---

# Phase 27 Plan 01: ProcessSnapshot & cmd.exe Filtering Summary

**ProcessSnapshot struct with single-snapshot capture, PEB command line reader, and cmd.exe batch flag detection using two-signal approach**

## Performance

- **Duration:** 11 min
- **Started:** 2026-03-11T08:20:24Z
- **Completed:** 2026-03-11T08:31:53Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- ProcessSnapshot struct with capture() that builds parent_map, exe_map, shell_candidates, and conpty_host_pids from a single CreateToolhelp32Snapshot call
- PEB command line reader at offset 0x70 using same pattern as existing CWD reader at offset 0x38
- Cross-platform extract_cmd_args() and has_batch_flag_in_cmdline() with 14 unit tests
- is_interactive_cmd() with two-signal approach: fast ConPTY parent check + definitive PEB batch flag check
- ConPTY host detection covers both OpenConsole.exe (Win11) and conhost.exe (Win10)

## Task Commits

Each task was committed atomically:

1. **Task 1: ProcessSnapshot struct and cmd.exe filtering infrastructure** - `823acd6` (feat)

## Files Created/Modified
- `src-tauri/src/terminal/process.rs` - Added ProcessSnapshot struct, capture(), read_command_line_from_peb(), extract_cmd_args(), has_batch_flag_in_cmdline(), check_batch_flags(), is_interactive_cmd(), and 14 unit tests

## Decisions Made
- Check both conhost.exe and OpenConsole.exe (case-insensitive) as ConPTY hosts, covering Windows 10 and 11
- Conservative fallback in is_interactive_cmd(): when PEB read fails (e.g., access denied on elevated processes), treat cmd.exe as interactive rather than filtering it out
- has_batch_flag_in_cmdline() checks both extracted args and raw input to handle edge cases where input is just flags without exe prefix
- IDE child detection uses case-insensitive contains check for node/code/cursor parent names

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed has_batch_flag_in_cmdline for flag-only input**
- **Found during:** Task 1 (TDD RED phase)
- **Issue:** has_batch_flag_in_cmdline("/C dir") failed because extract_cmd_args treated "/C" as the exe name, returning "dir" as args
- **Fix:** Added check_batch_flags() helper that checks both extracted args and raw input string
- **Files modified:** src-tauri/src/terminal/process.rs
- **Verification:** All 14 tests pass including edge cases
- **Committed in:** 823acd6

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Auto-fix necessary for correctness of flag parsing. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- ProcessSnapshot struct ready for Plan 02 to wire into existing detection pipeline
- is_interactive_cmd() ready to replace blanket cmd.exe filtering in find_shell_by_ancestry()
- Plan 02 will refactor find_shell_by_ancestry(), detect_wsl_in_ancestry(), and scan_wsl_processes_diagnostic() to use shared ProcessSnapshot

---
*Phase: 27-conpty-discovery-process-snapshot*
*Completed: 2026-03-11*
