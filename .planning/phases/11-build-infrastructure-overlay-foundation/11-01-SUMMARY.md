---
phase: 11-build-infrastructure-overlay-foundation
plan: 01
subsystem: infra
tags: [cargo, cfg-gate, cross-platform, windows-sys, window-vibrancy, platform-gating]

# Dependency graph
requires:
  - phase: 10-hardening-polish
    provides: "Stable macOS-only codebase with AX text reading, paste, and overlay"
provides:
  - "Platform-gated Cargo.toml with macOS and Windows dependency sections"
  - "All .rs files with cfg-gated macOS-only imports"
  - "Cross-platform show/hide overlay stubs using standard Tauri window API"
  - "AppState extended with previous_hwnd for Windows HWND tracking"
  - "window-vibrancy 0.7 for cross-platform HasWindowHandle support"
affects: [11-02, 11-03, 12, 13, 14, 15, 16]

# Tech tracking
tech-stack:
  added: [windows-sys 0.59, window-vibrancy 0.7]
  patterns: [cfg-gate macOS-only imports, platform-branching in setup, extract platform-specific code into gated helper functions]

key-files:
  created: []
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src-tauri/src/state.rs
    - src-tauri/src/commands/window.rs
    - src-tauri/src/commands/hotkey.rs
    - src-tauri/src/commands/paste.rs
    - src-tauri/src/commands/permissions.rs
    - src-tauri/src/commands/tray.rs

key-decisions:
  - "Extract macOS paste/confirm logic into dedicated gated functions rather than inline cfg blocks for readability"
  - "Use standard Tauri window.show()/hide() as cross-platform overlay stubs instead of no-op"
  - "Make open_url cross-platform with cmd /c start for Windows and xdg-open fallback for Linux"
  - "Platform-gate tray icon_as_template (true on macOS, false on Windows) and show_menu_on_left_click (false on macOS, true on Windows)"

patterns-established:
  - "cfg-gate pattern: #[cfg(target_os = \"macos\")] for macOS-only code, #[cfg(not(target_os = \"macos\"))] for cross-platform stubs"
  - "Platform-specific helper extraction: paste_to_terminal_macos() and confirm_terminal_command_macos() as gated helpers"
  - "Builder pattern for conditional plugin registration: let mut builder = ...; #[cfg(macos)] { builder = builder.plugin(...); }"

requirements-completed: [WBLD-01, WBLD-02]

# Metrics
duration: 6min
completed: 2026-03-02
---

# Phase 11 Plan 01: Platform-gate Dependencies and Imports Summary

**Platform-gated Cargo.toml with macOS/Windows dependency sections, cfg-gated all macOS-only imports across 8 Rust files, and extended AppState with Windows HWND field**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-02T15:54:55Z
- **Completed:** 2026-03-02T16:01:25Z
- **Tasks:** 2
- **Files modified:** 9 (including Cargo.lock)

## Accomplishments
- Platform-gated all macOS-only crate dependencies (tauri-nspanel, accessibility-sys, core-foundation-sys, keyring) into cfg(target_os = "macos") section
- Added Windows-only dependencies (windows-sys 0.59, keyring with windows-native) in cfg(target_os = "windows") section
- Upgraded window-vibrancy from 0.5 to 0.7 for cross-platform HasWindowHandle compatibility
- cfg-gated all macOS-only imports across lib.rs, window.rs, hotkey.rs, paste.rs, permissions.rs, tray.rs
- Added cross-platform show/hide overlay stubs using standard Tauri window API
- Extended AppState with previous_hwnd field for Windows HWND tracking
- Made open_url cross-platform, platform-gated tray behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Platform-gate Cargo.toml dependencies and add Windows deps** - `aa53731` (feat)
2. **Task 2: cfg-gate all macOS-only imports and extend AppState for Windows** - `e7d0375` (feat)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Platform-gated dependencies: macOS section (tauri-nspanel, accessibility-sys, core-foundation-sys, keyring), Windows section (windows-sys, keyring), cross-platform section (window-vibrancy 0.7)
- `src-tauri/Cargo.lock` - Updated lockfile for window-vibrancy 0.7
- `src-tauri/src/lib.rs` - cfg-gated tauri_nspanel/NSVisualEffectMaterial imports, NSPanel macro, plugin init, and panel setup block; added Windows placeholder
- `src-tauri/src/state.rs` - Added previous_hwnd: Mutex<Option<isize>> field for Windows HWND tracking
- `src-tauri/src/commands/window.rs` - cfg-gated ManagerExt import; added cross-platform show/hide using standard Tauri window API
- `src-tauri/src/commands/hotkey.rs` - cfg-gated ax_reader import and AX pre-capture calls; added non-macOS stubs
- `src-tauri/src/commands/paste.rs` - Extracted macOS paste/confirm into gated helper functions; added clipboard and non-macOS stubs
- `src-tauri/src/commands/permissions.rs` - cfg-gated open_accessibility_settings; made open_url cross-platform
- `src-tauri/src/commands/tray.rs` - Platform-gated show_menu_on_left_click and icon_as_template

## Decisions Made
- Extracted macOS-specific paste and confirm logic into dedicated gated helper functions (paste_to_terminal_macos, confirm_terminal_command_macos) rather than wrapping every individual line in inline cfg blocks -- improves readability and makes future Windows implementations parallel
- Used standard Tauri window.show()/set_focus()/hide() as cross-platform overlay stubs rather than no-ops -- gives a functional (if basic) overlay on non-macOS immediately
- Made open_url fully cross-platform with cmd /c start on Windows, xdg-open on Linux
- Platform-gated tray conventions: macOS gets right-click menu + template icon; Windows gets left-click menu + non-template icon

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unused variable warning in confirm_terminal_command_macos**
- **Found during:** Task 2 (cfg-gate macOS imports)
- **Issue:** Extracted confirm_terminal_command_macos function had `pid` parameter unused (only used in eprintln in the original, which was moved to the caller)
- **Fix:** Prefixed with underscore: `_pid`
- **Files modified:** src-tauri/src/commands/paste.rs
- **Verification:** cargo check shows no unused variable warning
- **Committed in:** e7d0375 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Trivial fix to suppress compiler warning. No scope creep.

## Issues Encountered
None - cargo check compiled cleanly on macOS after all changes. The only remaining warning is the expected dead_code warning for `previous_hwnd` which will be used in Plan 02 for Windows overlay setup.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All macOS-only imports are behind cfg gates -- project is ready for Windows compilation in Plan 02
- windows-sys 0.59 is available as a Windows dependency for Win32 API calls
- AppState has previous_hwnd field ready for Windows HWND capture
- Cross-platform show/hide stubs provide a functional baseline for Windows overlay
- Plan 02 (Windows overlay window setup) can proceed immediately

## Self-Check: PASSED

All 9 modified files verified on disk. Both task commits (aa53731, e7d0375) verified in git log.

---
*Phase: 11-build-infrastructure-overlay-foundation*
*Completed: 2026-03-02*
