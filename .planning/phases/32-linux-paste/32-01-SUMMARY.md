---
phase: 32-linux-paste
plan: 01
subsystem: paste
tags: [xdotool, xclip, wl-copy, wayland, x11, clipboard, linux]

# Dependency graph
requires:
  - phase: 31-linux-overlay
    provides: X11 active window PID capture via x11rb
provides:
  - Linux paste_to_terminal via xdotool on X11
  - Linux confirm_terminal_command via xdotool on X11
  - Wayland clipboard fallback with inline hint UI
  - LinuxToolAvailability cached at startup
  - DisplayServer detection (X11/Wayland/Unknown)
  - Frontend pasteHint state for clipboard_hint/confirm_hint display
affects: []

# Tech tracking
tech-stack:
  added: [xdotool, xclip, wl-copy]
  patterns: [subprocess CLI tools for X11 automation, return-value hint communication]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/paste.rs
    - src-tauri/src/state.rs
    - src/store/index.ts
    - src/hooks/useKeyboard.ts
    - src/components/Overlay.tsx

key-decisions:
  - "Removed arboard fallback on Linux (arboard is Windows-only dep); xclip/wl-copy are sufficient"
  - "Return-value hint communication (Result<String, String>) over Tauri events for synchronous hint display"
  - "Only dismiss overlay on auto-confirm; keep open when confirm_hint is returned"

patterns-established:
  - "Linux paste uses subprocess CLI tools (xdotool, xclip, wl-copy) matching industry standard"
  - "paste_to_terminal returns 'auto' or 'clipboard_hint' to signal frontend hint display"
  - "confirm_terminal_command returns 'auto' or 'confirm_hint' for same pattern"

requirements-completed: [LPST-01, LPST-02, LPST-03]

# Metrics
duration: 6min
completed: 2026-03-15
---

# Phase 32 Plan 01: Linux Paste Summary

**Linux paste via xdotool clipboard+Ctrl+Shift+V on X11 with Wayland clipboard fallback and inline hint UI**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-15T02:30:00Z
- **Completed:** 2026-03-15T02:36:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Replaced all three Linux stubs in paste.rs with real implementations using xdotool/xclip
- Added LinuxToolAvailability struct with startup-cached tool detection (xdotool, xclip, wl-copy)
- Added DisplayServer detection supporting X11, Wayland, and GDK_BACKEND=x11 XWayland override
- Frontend shows inline amber hint on Wayland/fallback: "Copied to clipboard -- press Ctrl+Shift+V to paste"
- Changed paste_to_terminal and confirm_terminal_command return types to Result<String, String> across all platforms

## Task Commits

Each task was committed atomically:

1. **Task 1: Linux paste backend** - `46101d3` (feat)
2. **Task 2: Frontend paste hint display** - `e3f3401` (feat)

## Files Created/Modified
- `src-tauri/src/state.rs` - Added LinuxToolAvailability struct with detect() constructor
- `src-tauri/src/commands/paste.rs` - DisplayServer enum, Linux clipboard write, paste_to_terminal_linux, confirm_command_linux
- `src/store/index.ts` - Added pasteHint state, setPasteHint action, clipboard_hint handling
- `src/hooks/useKeyboard.ts` - confirm_hint handling, conditional overlay dismiss
- `src/components/Overlay.tsx` - Inline paste hint display with amber styling

## Decisions Made
- Removed arboard fallback on Linux since arboard is a Windows-only dependency in Cargo.toml; if neither xclip nor wl-copy is available, a warning is logged but no hard error
- Used return-value communication (Result<String, String>) instead of Tauri events for paste hint signaling -- simpler and synchronous with the paste action
- On confirm_hint, overlay stays open so user can see the "Press Enter" hint; overlay only auto-dismisses on "auto" result

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Removed arboard fallback from Linux clipboard**
- **Found during:** Task 1 (Linux paste backend)
- **Issue:** Plan specified arboard as final clipboard fallback, but arboard is in Windows-only deps section of Cargo.toml
- **Fix:** Replaced arboard fallback with a log warning when no clipboard tools available
- **Files modified:** src-tauri/src/commands/paste.rs
- **Verification:** cargo check passes without errors
- **Committed in:** 46101d3

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary fix -- arboard cannot compile on Linux with current Cargo.toml. No functional impact since xclip/wl-copy are always available on supported Linux desktops.

## Issues Encountered
None beyond the arboard dependency issue documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Linux paste workflow is complete end-to-end
- Wayland users need xdotool/xclip installed for best experience; without them, clipboard+hint fallback works
- Ready for any subsequent phases

---
*Phase: 32-linux-paste*
*Completed: 2026-03-15*
