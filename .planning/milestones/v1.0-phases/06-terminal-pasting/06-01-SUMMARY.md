---
phase: 06-terminal-pasting
plan: 01
subsystem: terminal
tags: [rust, tauri, applescript, zustand, typescript, iterm2, terminal]

# Dependency graph
requires:
  - phase: 05-safety-layer
    provides: destructiveDetectionEnabled pattern in Zustand and PreferencesTab used as template for autoPasteEnabled
  - phase: 03-terminal-context-reading
    provides: terminal/detect.rs get_bundle_id function and AppState.previous_app_pid used in paste_to_terminal

provides:
  - paste_to_terminal Tauri IPC command dispatching AppleScript to iTerm2 and Terminal.app
  - autoPasteEnabled Zustand state with settings persistence and UI toggle

affects: [06-terminal-pasting-plan-02]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "AppleScript injection via osascript: write text newline NO for iTerm2, Ctrl+U + keystroke for Terminal.app"
    - "String escaping for AppleScript: replace \\ and \" before embedding in double-quoted string literals"

key-files:
  created:
    - src-tauri/src/commands/paste.rs
  modified:
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src/store/index.ts
    - src/components/Settings/PreferencesTab.tsx
    - src/App.tsx

key-decisions:
  - "paste_to_terminal reads previous_app_pid from AppState (not re-detected at paste time) per plan decision"
  - "iTerm2 uses write text newline NO to place text without executing (not do script)"
  - "Terminal.app uses Ctrl+U to clear current line then keystroke to type text (not do script)"
  - "autoPasteEnabled defaults to true; loaded in both onboarding and post-onboarding startup branches"
  - "Auto-paste toggle uses bg-blue-500/60 (not red) to distinguish from destructive detection toggle"

patterns-established:
  - "Preference toggle pattern: Zustand boolean + async handler that persists to settings.json via Store.load/set/save"

requirements-completed: [TERM-01]

# Metrics
duration: 1min
completed: 2026-02-23
---

# Phase 6 Plan 1: Terminal Pasting Infrastructure Summary

**paste_to_terminal Tauri command with AppleScript dispatch (iTerm2 + Terminal.app) and autoPasteEnabled Zustand preference with Settings UI toggle and startup persistence**

## Performance

- **Duration:** ~8 min
- **Started:** 2026-02-23T09:15:10Z
- **Completed:** 2026-02-23T09:23:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created `paste_to_terminal` Rust Tauri command that reads `previous_app_pid` from AppState, resolves bundle ID, and dispatches the correct AppleScript (iTerm2: `write text newline NO`; Terminal.app: `Ctrl+U + keystroke`) without executing the command
- Added `autoPasteEnabled` boolean field to Zustand `OverlayState` interface with initial value `true` and `setAutoPasteEnabled` action
- Added Terminal section to PreferencesTab with auto-paste toggle (blue ON / white OFF) and amber warning label when disabled
- App.tsx loads `autoPasteEnabled` from `settings.json` on startup in both onboarding-incomplete and onboarding-complete branches

## Task Commits

Each task was committed atomically:

1. **Task 1: Create paste_to_terminal Rust command with AppleScript dispatch** - `db9e97c` (feat)
2. **Task 2: Add autoPasteEnabled Zustand state, PreferencesTab toggle, and App.tsx startup load** - `d9cdae4` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified

- `src-tauri/src/commands/paste.rs` - New file: paste_to_terminal Tauri command with build_paste_script helper
- `src-tauri/src/commands/mod.rs` - Added `pub mod paste;`
- `src-tauri/src/lib.rs` - Imported paste_to_terminal and registered in generate_handler![]
- `src/store/index.ts` - Added autoPasteEnabled field, initial state true, and setAutoPasteEnabled action
- `src/components/Settings/PreferencesTab.tsx` - Added Terminal section with auto-paste toggle and conditional warning label
- `src/App.tsx` - Added autoPasteEnabled load from settings.json in both startup branches

## Decisions Made

- `paste_to_terminal` reads `previous_app_pid` from AppState rather than re-detecting at paste time (plan decision, avoids race condition)
- AppleScript for iTerm2 uses `write text ... newline NO` (places text without executing); Terminal.app uses `Ctrl+U` then `keystroke` (clears line and types without executing) - neither uses `do script` which would execute immediately
- `autoPasteEnabled` defaults to `true` when key is absent from settings.json (safe default: auto-paste on)
- Toggle color is `bg-blue-500/60` (blue) to distinguish it visually from the red destructive detection toggle

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- `paste_to_terminal` is callable from JavaScript as `invoke("paste_to_terminal", { command })` and returns `Result<(), String>`
- `autoPasteEnabled` is accessible via `useOverlayStore((s) => s.autoPasteEnabled)` in any component
- Plan 02 can wire the paste trigger into `submitQuery` completion path: check `autoPasteEnabled`, call `paste_to_terminal` with `streamingText`, handle error gracefully

---
*Phase: 06-terminal-pasting*
*Completed: 2026-02-23*

## Self-Check: PASSED

- FOUND: src-tauri/src/commands/paste.rs
- FOUND: src/store/index.ts
- FOUND: src/components/Settings/PreferencesTab.tsx
- FOUND: src/App.tsx
- FOUND: .planning/phases/06-terminal-pasting/06-01-SUMMARY.md
- FOUND commit: db9e97c (Task 1)
- FOUND commit: d9cdae4 (Task 2)
