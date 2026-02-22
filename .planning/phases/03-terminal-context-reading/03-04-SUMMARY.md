---
phase: 03-terminal-context-reading
plan: "04"
subsystem: terminal-context
tags: [rust, tauri, accessibility-api, objc-ffi, browser-detection, app-context]
dependency_graph:
  requires: ["03-01", "03-02", "03-03"]
  provides: ["AppContext IPC", "browser console detection", "app name resolution"]
  affects: ["03-05"]
tech_stack:
  added: ["browser.rs module"]
  patterns:
    - "AX window title heuristic for DevTools detection"
    - "NSRunningApplication.localizedName via ObjC FFI (ARM64-safe)"
    - "AppContext wrapper struct for unified frontend response"
    - "detect_full() 500ms timeout wrapper pattern"
key_files:
  created:
    - src-tauri/src/terminal/browser.rs
  modified:
    - src-tauri/src/terminal/detect.rs
    - src-tauri/src/terminal/mod.rs
    - src-tauri/src/commands/terminal.rs
    - src-tauri/src/lib.rs
decisions:
  - "AX window title heuristic (not AX tree deep walk) for DevTools: simpler, browser-agnostic, works for Chrome/Safari/Firefox"
  - "detect_console returns (bool, Option<String>) so console presence is known even when text is unreadable"
  - "detect_full() as new public API alongside detect() for backward compatibility"
  - "AppContext wraps both terminal and browser state so frontend gets one unified response"
  - "is_some_and/is_none_or over map_or(false/true) per clippy::unnecessary_map_or"
metrics:
  duration: "4 min"
  completed: "2026-02-22"
  tasks_completed: 2
  files_changed: 5
---

# Phase 3 Plan 4: App Context and Browser Console Detection Summary

AppContext struct with app name resolution, browser DevTools detection via AX window title heuristic, and a unified get_app_context IPC command for any frontmost app.

## What Was Built

### Task 1: App name resolution, browser detection, name cleaning in detect.rs

(Changes were already in the repository from the previous session's commit ba710aa -- detect.rs was extended alongside the 03-03 fix work. Verified all required symbols present and cargo check clean.)

Key additions already in detect.rs:
- `BROWSER_BUNDLE_IDS`: 6 browsers mapped to canonical display names (Chrome, Safari, Firefox, Arc, Edge, Brave). Chromium-based browsers use their own name, not "Chrome".
- `is_known_browser()` / `browser_display_name()`: helpers for bundle ID to browser name lookup
- `APP_NAME_MAP`: static mapping for verbose macOS names to short versions ("Google Chrome" -> "Chrome", "Visual Studio Code" -> "Code", etc.)
- `clean_app_name()`: applies the map then strips common suffixes (" - Insiders", " Beta", " Nightly", etc.)
- `get_app_display_name(pid)`: resolves NSRunningApplication.localizedName via ObjC FFI using the same ARM64-safe MsgSendI32 pattern as get_bundle_id

### Task 2: browser.rs, AppContext struct, IPC command

**src-tauri/src/terminal/browser.rs** (new file):
- `detect_console(app_pid, bundle_id) -> (bool, Option<String>)`
- Creates AXUIElement for the browser app PID with 1.0s messaging timeout
- Enumerates all AXWindows, checks titles for "devtools", "web inspector", "developer tools", "browser console" (case-insensitive)
- If DevTools window found: walks AX children to find AXTextArea, reads its value, returns the last non-empty line
- Returns (true, None) if DevTools detected but text unreadable (common -- AX exposure varies per browser version)
- macOS-only with non-macOS stub returning (false, None)

**src-tauri/src/terminal/mod.rs**:
- Added `pub mod browser`
- Added `AppContext` struct: `app_name: Option<String>`, `terminal: Option<TerminalContext>`, `console_detected: bool`, `console_last_line: Option<String>`
- Added `detect_app_context()`: orchestrates bundle ID + display name + process info + browser console in sequence
- Added `detect_full()`: 500ms timeout wrapper for detect_app_context()
- Kept existing `detect()` and `detect_inner()` unchanged for backward compatibility

**src-tauri/src/commands/terminal.rs**:
- Added `get_app_context` IPC command returning `Option<terminal::AppContext>`
- Reads PID from AppState, calls `terminal::detect_full(pid)`
- Kept `get_terminal_context` unchanged for backward compatibility

**src-tauri/src/lib.rs**:
- Registered `get_app_context` in `generate_handler!`

## Deviations from Plan

### Pre-committed Task 1 changes

**Found during:** Task 1 verification

**Issue:** When reading detect.rs before editing, the file already contained BROWSER_BUNDLE_IDS, APP_NAME_MAP, clean_app_name(), get_app_display_name(), and related helpers. These were committed in the previous session as part of fix(03-03) commit ba710aa.

**Fix:** Verified all required symbols present, cargo check passes. No re-implementation needed.

**Files modified:** src-tauri/src/terminal/detect.rs (already committed)

### Auto-fixed: unnecessary_map_or clippy warnings

**Rule 2 - Auto-fix:** New mod.rs code used `map_or(false, fn)` and `map_or(true, fn)` patterns. Clippy flagged these as unnecessary_map_or (implied by `-W clippy::all`).

**Fix:** Replaced with `is_some_and(fn)` and `is_none_or(fn)` idioms in detect_inner() and detect_app_context().

**Files modified:** src-tauri/src/terminal/mod.rs

## Pre-existing warnings (out of scope)

The following warnings were present before this plan and are not fixed:
- `manual_c_str_literals` in detect.rs and hotkey.rs: Uses byte string pattern (`b"NSWorkspace\0".as_ptr()`) established in get_bundle_id before this plan. Fixing would require converting all ObjC FFI calls across the codebase -- architectural change.
- `browser_display_name` dead_code warning: Function is exported for Plan 03-05 (frontend integration) to use for badge display mapping. Will be used in the next plan.

## Self-Check: PASSED

Files created/modified:
- FOUND: src-tauri/src/terminal/browser.rs
- FOUND: src-tauri/src/terminal/mod.rs (AppContext struct, detect_full)
- FOUND: src-tauri/src/commands/terminal.rs (get_app_context)
- FOUND: src-tauri/src/lib.rs (get_app_context registered)

Commits verified:
- ba710aa: detect.rs with browser detection, app name helpers (previous session)
- 2255ed6: browser.rs, AppContext, get_app_context IPC command

cargo check: PASSED (0 errors, 1 pre-existing warning)
cargo clippy: PASSED (no errors in new code, pre-existing warnings documented above)
