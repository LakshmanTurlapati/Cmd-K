---
phase: 11-build-infrastructure-overlay-foundation
plan: 04
subsystem: infra
tags: [windows, hwnd, focus-management, dead-code-fix, getforegroundwindow, gap-closure]

# Dependency graph
requires:
  - phase: 11-build-infrastructure-overlay-foundation
    plan: 03
    provides: "Windows HWND capture function (get_foreground_hwnd), restore_focus function, AppState.previous_hwnd field"
provides:
  - "Reachable Windows HWND capture -- get_foreground_hwnd() executes independently of macOS PID capture on Windows"
  - "Functional focus restoration chain -- previous_hwnd is populated so restore_focus() in hide_overlay receives valid HWND data"
affects: [12, 13, 14, 15, 16]

# Tech tracking
tech-stack:
  added: []
  patterns: [Platform-independent HWND capture at correct scope level outside macOS PID-gated block]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/hotkey.rs

key-decisions:
  - "Move HWND capture block to before get_frontmost_pid() call rather than after -- ensures capture happens even if PID logic changes in the future"

patterns-established:
  - "Windows-specific capture code must be at the if !is_currently_visible scope level, not nested inside macOS PID branches"

requirements-completed: [WOVL-01, WOVL-02, WOVL-03, WOVL-04, WOVL-05, WOVL-06, WOVL-07, WBLD-01, WBLD-02]

# Metrics
duration: 1min
completed: 2026-03-02
---

# Phase 11 Plan 04: Windows HWND Capture Dead Code Fix Summary

**Fixed unreachable Windows HWND capture by moving get_foreground_hwnd() block outside the always-None PID-gated branch in hotkey.rs**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-02T16:40:40Z
- **Completed:** 2026-03-02T16:41:41Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Moved the `#[cfg(target_os = "windows")]` HWND capture block from inside `if let Some(pid) = pid` (always None on Windows) to the outer `if !is_currently_visible` scope level
- HWND capture now executes independently of macOS PID capture, resolving the root cause for both WOVL-04 (HWND capture) and WOVL-05 (focus restoration) verification gaps
- macOS code paths completely unchanged -- all code inside `if let Some(pid) = pid` remains untouched
- cargo check passes cleanly with only pre-existing dead_code warnings for Windows-only stubs

## Task Commits

Each task was committed atomically:

1. **Task 1: Move Windows HWND capture outside the PID-gated block** - `bf9795c` (fix)

## Files Created/Modified
- `src-tauri/src/commands/hotkey.rs` - Moved `#[cfg(target_os = "windows")]` HWND capture block from inside `if let Some(pid) = pid` to direct child of `if !is_currently_visible`, with explanatory comments

## Decisions Made
- Placed the HWND capture block BEFORE the `let pid = get_frontmost_pid()` call rather than after it -- ensures the capture happens as early as possible in the show path, and makes the independence from PID capture visually clear in the code

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - the structural change was straightforward and cargo check confirmed no compilation issues.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 11 gap closure complete -- all 9 verification truths now pass (7 previously verified + 2 fixed by this plan)
- The HWND capture -> previous_hwnd -> restore_focus chain is now fully connected on Windows
- Ready for Phase 12 (Terminal Context on Windows) and Phase 13 (Clipboard Paste on Windows)
- Human verification still needed on actual Windows hardware for: Acrylic vibrancy visual quality, Alt+Tab exclusion behavior, and focus restoration UX

## Self-Check: PASSED

All modified files verified on disk. Task commit (bf9795c) verified in git log.

---
*Phase: 11-build-infrastructure-overlay-foundation*
*Completed: 2026-03-02*
