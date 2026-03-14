---
phase: 31-linux-overlay-hotkey
plan: 01
subsystem: hotkey
tags: [x11rb, linux, x11, ewmh, overlay, hotkey, pid-capture]

# Dependency graph
requires:
  - phase: 30-linux-process-detection
    provides: detect_linux.rs exe detection and process tree walking
provides:
  - Linux X11 PID capture via EWMH _NET_ACTIVE_WINDOW + _NET_WM_PID
  - Linux window key computation (exe_name:shell_pid)
  - Linux always-on-top overlay setup
  - Linux cfg block in hotkey handler for capture-before-show
affects: [31-linux-overlay-hotkey]

# Tech tracking
tech-stack:
  added: [x11rb 0.13]
  patterns: [cfg-gated Linux X11 PID capture, EWMH property queries]

key-files:
  created: []
  modified: [src-tauri/Cargo.toml, src-tauri/src/commands/hotkey.rs, src-tauri/src/lib.rs]

key-decisions:
  - "x11rb for direct EWMH property queries (already transitive dep, no subprocess needed)"
  - "Fresh X11 connection per hotkey press (1ms overhead acceptable, no state management)"

patterns-established:
  - "Linux cfg block in hotkey handler mirrors macOS/Windows capture-before-show pattern"
  - "get_active_window_pid() with DISPLAY env var guard for graceful Wayland fallback"

requirements-completed: [LOVRL-01, LOVRL-02, LOVRL-03, LOVRL-04]

# Metrics
duration: 3min
completed: 2026-03-14
---

# Phase 31 Plan 01: Linux Overlay Hotkey Summary

**X11 PID capture via x11rb EWMH properties with window key computation and always-on-top overlay setup**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-14T23:01:28Z
- **Completed:** 2026-03-14T23:04:03Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added x11rb as Linux-only dependency for direct X11 property queries
- Implemented get_active_window_pid() reading _NET_ACTIVE_WINDOW + _NET_WM_PID
- Implemented compute_window_key_linux() resolving exe name and shell PID via detect_linux
- Added Linux cfg block in hotkey handler for capture-before-show pattern
- Added Linux always-on-top setup block in lib.rs

## Task Commits

Each task was committed atomically:

1. **Task 1: Add x11rb dependency and implement Linux PID capture + window key functions** - `fc1194e` (feat)
2. **Task 2: Add Linux setup block in lib.rs for always-on-top window** - `0efa863` (feat)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Added x11rb 0.13 as Linux-only dependency
- `src-tauri/src/commands/hotkey.rs` - Added get_active_window_pid(), compute_window_key_linux(), Linux handler block
- `src-tauri/src/lib.rs` - Added Linux always-on-top setup block in .setup() callback

## Decisions Made
- Used x11rb for direct EWMH property queries instead of xdotool subprocess (already transitive dep, pure Rust, no runtime dependency)
- Fresh X11 connection per hotkey press rather than cached connection (1ms overhead negligible with 200ms debounce)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Linux hotkey and PID capture infrastructure complete
- Ready for Plan 02 (CSS frosted glass for Linux) to complete the overlay visuals
- Focus restoration on hide may need follow-up testing on various WMs

---
*Phase: 31-linux-overlay-hotkey*
*Completed: 2026-03-14*
