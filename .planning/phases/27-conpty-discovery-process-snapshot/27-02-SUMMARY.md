---
phase: 27-conpty-discovery-process-snapshot
plan: 02
subsystem: terminal-detection
tags: [windows, conpty, process-snapshot, shell-discovery, cmd-filtering, getprocesstimes]

requires:
  - phase: 27-01
    provides: ProcessSnapshot struct, is_interactive_cmd(), read_command_line_from_peb()
provides:
  - ConPTY-first shell selection in find_shell_by_ancestry
  - Shared ProcessSnapshot threading through entire detection pipeline
  - GetProcessTimes-based recency sorting replacing highest-PID heuristic
  - cmd.exe filtering via is_interactive_cmd instead of blanket exclusion
affects: [terminal-detection, conpty-discovery, windows-process-tree]

tech-stack:
  added: []
  patterns: [conpty-first-selection, getprocesstimes-recency, shared-snapshot-threading]

key-files:
  created: []
  modified:
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/terminal/mod.rs

key-decisions:
  - "Tasks 1 and 2 committed together since mod.rs caller updates required for process.rs signature changes to compile"
  - "get_child_pids_windows retains its own CreateToolhelp32Snapshot call (used by find_shell_recursive fast path, shared across platforms)"
  - "ConPTY shell selection uses 3-tier priority: ConPTY descendants > direct descendants > all ConPTY shells"

patterns-established:
  - "get_foreground_info has platform-divergent signatures: Windows accepts Option<&ProcessSnapshot>, non-Windows has no snapshot parameter"
  - "pick_most_recent() helper: GetProcessTimes-based with PID fallback, reusable for any candidate set"

requirements-completed: [PROC-01, PROC-02, PROC-03]

duration: 5min
completed: 2026-03-11
---

# Phase 27 Plan 02: ConPTY Shell Discovery Pipeline Summary

**ConPTY-first shell selection with shared ProcessSnapshot threading, GetProcessTimes recency sorting, and is_interactive_cmd-based cmd.exe filtering**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T13:55:38Z
- **Completed:** 2026-03-11T14:01:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- find_shell_by_ancestry refactored with 3-tier ConPTY-first priority: (1) ConPTY-hosted descendant shells, (2) direct descendant shells, (3) all ConPTY shells as fallback
- detect_wsl_in_ancestry and scan_wsl_processes_diagnostic now use shared ProcessSnapshot (eliminated 2 redundant CreateToolhelp32Snapshot calls)
- cmd.exe candidates filtered through is_interactive_cmd() using PEB command line analysis instead of blanket exclusion
- GetProcessTimes-based recency sorting replaces highest-PID heuristic for shell selection
- detect_app_context_windows creates single ProcessSnapshot and threads it through get_foreground_info

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Refactor detection functions + thread ProcessSnapshot through mod.rs** - `8e046ae` (feat)

## Files Created/Modified
- `src-tauri/src/terminal/process.rs` - Refactored find_shell_by_ancestry (ConPTY-first, snapshot param, GetProcessTimes), detect_wsl_in_ancestry (snapshot param), scan_wsl_processes_diagnostic (snapshot param), get_foreground_info (platform-divergent with optional snapshot on Windows), find_shell_pid (Windows version accepts snapshot)
- `src-tauri/src/terminal/mod.rs` - detect_app_context_windows creates ProcessSnapshot::capture() and passes to get_foreground_info; detect_inner_windows passes None

## Decisions Made
- Tasks 1 and 2 were committed together because the mod.rs caller updates were required for the process.rs signature changes to compile (interdependent changes)
- get_child_pids_windows retains its own CreateToolhelp32Snapshot call since it serves the find_shell_recursive fast path which is shared across platforms and cannot accept a Windows-only snapshot parameter
- ConPTY shell selection uses a 3-tier priority system rather than a simple ConPTY-or-descendant binary choice, handling the Windows Terminal architecture where shells are NOT descendants of the terminal app
- get_foreground_info uses cfg-gated platform-divergent signatures rather than adding an unused parameter to the macOS version

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Combined Task 1 and Task 2 into single commit**
- **Found during:** Task 1 (signature refactoring)
- **Issue:** Changing find_shell_by_ancestry and get_foreground_info signatures required simultaneous caller updates in mod.rs (Task 2 content) for compilation
- **Fix:** Applied both sets of changes together and committed as one atomic unit
- **Files modified:** src-tauri/src/terminal/process.rs, src-tauri/src/terminal/mod.rs
- **Verification:** cargo check and cargo test both pass
- **Committed in:** 8e046ae

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Combined commit necessary for compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full ConPTY-aware detection pipeline complete with shared ProcessSnapshot
- All PROC-01, PROC-02, PROC-03 requirements satisfied
- Phase 27 complete - ready for Phase 28 (UIA tree structure verification)

---
*Phase: 27-conpty-discovery-process-snapshot*
*Completed: 2026-03-11*
