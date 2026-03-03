---
phase: 11-build-infrastructure-overlay-foundation
plan: 03
subsystem: infra
tags: [windows, hwnd, focus-management, getforegroundwindow, setforegroundwindow, attachthreadinput, win32]

# Dependency graph
requires:
  - phase: 11-build-infrastructure-overlay-foundation
    plan: 01
    provides: "Platform-gated Cargo.toml, cfg-gated macOS imports, AppState with previous_hwnd field"
  - phase: 11-build-infrastructure-overlay-foundation
    plan: 02
    provides: "Windows overlay with Acrylic vibrancy, WS_EX_TOOLWINDOW, raw-window-handle for HWND access"
provides:
  - "Windows HWND capture via GetForegroundWindow before overlay show"
  - "Focus restoration via AttachThreadInput + SetForegroundWindow with IsWindow validation"
  - "Intelligent click-outside vs Escape/hotkey dismiss detection in hide_overlay"
  - "AllowSetForegroundWindow fallback for edge cases"
  - "Cross-platform 200ms hotkey debounce (verified not in macOS cfg block)"
affects: [12, 13, 14, 15, 16]

# Tech tracking
tech-stack:
  added: []
  patterns: [Win32 HWND capture before overlay show, AttachThreadInput + SetForegroundWindow focus restoration, foreground window comparison for dismiss-type detection]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/hotkey.rs
    - src-tauri/src/commands/window.rs

key-decisions:
  - "Always check if overlay is still foreground before restoring focus -- distinguishes Escape/hotkey dismiss from click-outside dismiss without changing IPC signature"
  - "AttachThreadInput + SetForegroundWindow as primary focus restoration with AllowSetForegroundWindow retry as fallback"
  - "IsWindow validation before focus restoration to gracefully handle stale HWNDs (window closed during overlay)"

patterns-established:
  - "Focus restoration pattern: capture HWND before show, validate with IsWindow, restore with AttachThreadInput/SetForegroundWindow on dismiss"
  - "Dismiss-type detection: compare GetForegroundWindow with our HWND before hide to determine if user clicked outside"

requirements-completed: [WOVL-04, WOVL-05, WOVL-06]

# Metrics
duration: 2min
completed: 2026-03-02
---

# Phase 11 Plan 03: Windows Focus Management Summary

**Windows HWND capture via GetForegroundWindow before overlay show, focus restoration via AttachThreadInput + SetForegroundWindow on dismiss, with intelligent click-outside detection**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-02T16:09:38Z
- **Completed:** 2026-03-02T16:11:50Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Added `get_foreground_hwnd()` function capturing the active window HWND before overlay steals focus, using Win32 GetForegroundWindow
- Added `restore_focus()` function implementing the AttachThreadInput + SetForegroundWindow workaround with IsWindow stale-HWND validation and AllowSetForegroundWindow retry fallback
- Wired HWND capture into hotkey handler: captures on show, clears on hide
- Implemented intelligent focus restoration in hide_overlay: compares current foreground window with our overlay HWND to distinguish Escape/hotkey dismiss (restore focus) from click-outside dismiss (skip restoration)
- Verified 200ms hotkey debounce is cross-platform (not inside any cfg(target_os = "macos") block)

## Task Commits

Each task was committed atomically:

1. **Task 1: Windows HWND capture and focus restoration in hotkey.rs** - `a994d98` (feat)
2. **Task 2: Wire focus restoration into hide_overlay Windows path** - `3f5d213` (feat)

## Files Created/Modified
- `src-tauri/src/commands/hotkey.rs` - Added get_foreground_hwnd() with GetForegroundWindow, restore_focus() with AttachThreadInput + SetForegroundWindow + IsWindow + AllowSetForegroundWindow, HWND capture in show path, HWND clear in hide path
- `src-tauri/src/commands/window.rs` - Added Windows focus restoration in hide_overlay with foreground window comparison for click-outside detection, using raw-window-handle for overlay HWND access

## Decisions Made
- Used foreground window comparison (GetForegroundWindow vs our HWND) to detect dismiss type rather than adding a parameter to hide_overlay IPC -- avoids breaking frontend contract while correctly handling both Escape/hotkey and click-outside dismiss scenarios
- Placed HWND capture inside the `if let Some(pid)` block alongside PID capture, ensuring both macOS PID and Windows HWND are captured in the same "about to show" code path
- Used AllowSetForegroundWindow as a retry fallback after AttachThreadInput + SetForegroundWindow fails, covering edge cases where thread input attachment alone is insufficient

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None - cargo check compiled cleanly on macOS after all changes. The dead_code warnings for get_foreground_hwnd, restore_focus, and previous_hwnd are expected because these are cfg(target_os = "windows") stubs that are unused on the macOS compilation target.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 11 (Build Infrastructure and Overlay Foundation) is fully complete with all 3 plans executed
- Windows overlay has: vibrancy (Acrylic), window styles (WS_EX_TOOLWINDOW), always-on-top, Ctrl+Shift+K hotkey, HWND capture, and focus restoration
- macOS code paths are completely unchanged -- all Windows additions are behind cfg gates
- Ready for Phase 12 (Terminal Context on Windows) and Phase 13 (Clipboard Paste on Windows) which can proceed in parallel

## Self-Check: PASSED

All modified files verified on disk. Both task commits (a994d98, 3f5d213) verified in git log.

---
*Phase: 11-build-infrastructure-overlay-foundation*
*Completed: 2026-03-02*
