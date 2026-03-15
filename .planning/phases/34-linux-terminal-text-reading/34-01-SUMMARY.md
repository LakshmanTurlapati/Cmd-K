---
phase: 34-linux-terminal-text-reading
plan: 01
subsystem: terminal
tags: [linux, at-spi2, dbus, zbus, kitty, wezterm, accessibility]

# Dependency graph
requires:
  - phase: 30-linux-process-detection
    provides: "/proc-based process detection and exe name resolution"
  - phase: 33-smart-terminal-context
    provides: "ANSI stripping, budget truncation, and sensitive data filtering pipeline"
provides:
  - "Linux terminal text reading via AT-SPI2, kitty, and WezTerm strategies"
  - "read_terminal_text_linux(pid, exe_name) pub fn in linux_reader.rs"
  - "visible_output populated on Linux for known terminals"
affects: []

# Tech tracking
tech-stack:
  added: [zbus]
  patterns: [zbus-blocking-dbus, subprocess-with-timeout, strategy-dispatch-by-exe-name]

key-files:
  created: [src-tauri/src/terminal/linux_reader.rs]
  modified: [src-tauri/src/terminal/mod.rs, src-tauri/Cargo.toml]

key-decisions:
  - "zbus with default features (async-io runtime) since blocking-only feature set fails to compile in zbus 5"
  - "OwnedObjectPath derefs directly to &str via ObjectPath intermediate (no .as_ref() needed)"
  - "filter::filter_sensitive applied to captured terminal text in mod.rs wiring, not in linux_reader itself"

patterns-established:
  - "Strategy dispatch by exe_name match in linux_reader parallels ax_reader (macOS) and uia_reader (Windows)"
  - "AT-SPI2 tree walk: registry root -> app by PID -> recursive child walk for role=Terminal(60) -> GetText"

requirements-completed: [LTXT-01, LTXT-02, LTXT-03, LTXT-04]

# Metrics
duration: 13min
completed: 2026-03-15
---

# Phase 34 Plan 01: Linux Terminal Text Reading Summary

**AT-SPI2 D-Bus, kitty remote control, and WezTerm CLI strategies for reading visible terminal text on Linux, wired into the detection pipeline**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-15T07:10:49Z
- **Completed:** 2026-03-15T07:24:08Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Created linux_reader.rs with three terminal text reading strategies (AT-SPI2 via zbus, kitty subprocess, WezTerm subprocess)
- All strategies use 500ms timeout via thread+channel pattern
- Wired into both detect_inner_linux and detect_app_context_linux to populate visible_output
- Unsupported terminals (alacritty, st, foot, xterm, urxvt) return None gracefully

## Task Commits

Each task was committed atomically:

1. **Task 1: Create linux_reader.rs with AT-SPI2, kitty, and WezTerm strategies** - `ea6a1cf` (feat)
2. **Task 2: Wire linux_reader into mod.rs detection pipeline** - `b0bec10` (feat)

## Files Created/Modified
- `src-tauri/src/terminal/linux_reader.rs` - Linux terminal text reading with three strategies and non-Linux stub
- `src-tauri/src/terminal/mod.rs` - Wired linux_reader into detect_inner_linux and detect_app_context_linux
- `src-tauri/Cargo.toml` - Added zbus dependency for Linux AT-SPI2 D-Bus

## Decisions Made
- Used zbus with default features instead of `default-features = false, features = ["blocking"]` because zbus 5 requires an async runtime (async-io) even for blocking API usage
- Applied filter::filter_sensitive in mod.rs at the call site (not inside linux_reader) to match the macOS ax_reader pattern
- Used i32::try_from(OwnedValue) for AT-SPI2 PID extraction instead of manual Value variant matching

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed zbus feature configuration**
- **Found during:** Task 1
- **Issue:** Plan specified `zbus = { version = "5", default-features = false, features = ["blocking"] }` but zbus 5 has no `blocking` feature (it's called `blocking-api`), and disabling default features removes the required async-io runtime causing 82 compilation errors
- **Fix:** Used `zbus = { version = "5" }` with default features (async-io + blocking-api)
- **Files modified:** src-tauri/Cargo.toml
- **Verification:** cargo check passes cleanly
- **Committed in:** ea6a1cf (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed zbus API usage for Connection builder and type conversions**
- **Found during:** Task 1
- **Issue:** Plan's zbus API examples didn't match zbus 5 actual API -- Connection::builder() doesn't exist (use Builder::address()), OwnedObjectPath doesn't deref to &str via .as_ref() (use direct deref), OwnedValue variant matching needed try_from instead of manual deref
- **Fix:** Used zbus::blocking::connection::Builder::address() for AT-SPI2 bus connection, direct deref for OwnedObjectPath, i32::try_from(OwnedValue) for PID extraction
- **Files modified:** src-tauri/src/terminal/linux_reader.rs
- **Verification:** cargo check passes, all 79 tests pass
- **Committed in:** ea6a1cf (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 blocking)
**Impact on plan:** Both auto-fixes necessary to make zbus compile and work correctly. No scope creep.

## Issues Encountered
None beyond the zbus API deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Linux terminal text reading complete, visible_output populated for supported terminals
- Phase 33 smart context pipeline already handles ANSI stripping and budget truncation downstream
- AT-SPI2 reliability across terminal emulators untested at runtime (requires Linux desktop with accessibility enabled)

---
*Phase: 34-linux-terminal-text-reading*
*Completed: 2026-03-15*
