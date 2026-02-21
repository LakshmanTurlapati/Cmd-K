# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** Phase 3 - Terminal Context Reading

## Current Position

Phase: 3 of 6 (Terminal Context Reading)
Plan: 2 of 3 in current phase
Status: In progress -- 03-01 complete, ready for 03-02 (AX text reading)
Last activity: 2026-02-21 - Executed Phase 3 Plan 1 (Rust terminal detection backend: libproc FFI, ObjC bundle ID, get_terminal_context IPC)

Progress: [██████░░░░] 60%

## Performance Metrics

**Velocity:**
- Total plans completed: 5
- Average duration: 8 min
- Total execution time: 0.8 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation-overlay | 3 | 22 min | 7 min |
| 02-settings-configuration | 2 | 24 min | 12 min |
| 03-terminal-context-reading | 1 | 4 min | 4 min |

**Recent Trend:**
- Last 5 plans: 8 min (01-02), 3 min (01-03), 12 min (02-01), 12 min (02-02), 4 min (03-01)
- Trend: Stable

*Updated after each plan completion*

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- Tauri over Electron: Lighter weight, smaller binary, less RAM
- xAI only for v1: Simplify scope, add other providers later
- macOS only for v1: Focus on one platform, nail the overlay UX
- Zero shell setup: Lower adoption friction, use Accessibility API + AppleScript instead
- tauri_panel! macro for NSPanel config: tauri-nspanel v2.1 uses macro-based panel config (removed setter methods)
- PanelLevel::Status (25) for window level: Chosen to float above fullscreen apps and all normal windows
- NSPanel FullScreenAuxiliary+CanJoinAllSpaces: Required collection behavior for overlay on fullscreen apps
- time crate pinned to 0.3.36: rustc 1.85.0 compatibility (0.3.47 requires 1.88.0)
- Animation phase state machine (entering/visible/exiting/hidden): Keeps overlay mounted during exit animation so overlay-out keyframe plays before unmount
- useKeyboard hook centralizes Escape + event listeners: Invoked once in App.tsx, keeps components clean
- submit() always sets showApiWarning in Phase 1: No API configured yet; Phase 4 replaces with actual AI call
- useRef for key tracking in HotkeyRecorder: avoids stale closures and excess re-renders vs useState
- invoke<string | null> return type for register_hotkey: Rust Result<(), String) maps null=success, string=error
- keyring crate used directly (no community plugin): fewer dependencies, full control, same Keychain result
- AXIsProcessTrusted via extern C block: stable macOS public API, avoids tauri-plugin-macos-permissions dependency for a single boolean
- tauri_plugin_http reqwest re-export lacks json() feature: use .body(serde_json) + .bytes() + serde_json::from_slice for HTTP I/O
- 404 fallback for GET /v1/models built from day one: xAI does not document this endpoint explicitly; fallback validates via POST /v1/chat/completions
- Custom Tailwind tab UI (no shadcn/ui): shadcn requires CLI setup; plain border-b-2 pattern is sufficient and zero-dependency
- Change Hotkey tray item routes to openSettings(preferences): keeps all configuration within settings panel tabs
- Auto-save on API key validation success: no save button per user decision; key persisted immediately via Rust IPC
- Raw libproc FFI instead of darwin-libproc crate: darwin-libproc 0.2 pins memchr ~2.3 which conflicts with tauri chain; raw extern C is equivalent and avoids the conflict
- PROC_PIDVNODEPATHINFO for CWD reading: proc_pidinfo flavor 9 returns proc_vnodepathinfo with pvi_cdir.pvip.vip_path
- Capture-before-show PID pattern: frontmost app PID must be captured BEFORE toggle_overlay()/show_and_make_key() or NSWorkspace reports our own app as frontmost

### Pending Todos

[From .planning/todos/pending/ - ideas captured during sessions]

None yet.

### Blockers/Concerns

[Issues that affect future work]

- Phase 1 Plan 1 COMPLETE: NSPanel integration resolved -- overlay floats above fullscreen apps using Status level + FullScreenAuxiliary
- Phase 1 Plan 2 COMPLETE: React overlay UI complete -- 640px frosted glass panel with animation, auto-focus textarea, keyboard handling, click-outside dismiss
- Phase 1 Plan 3 CHECKPOINT: Hotkey config dialog complete; awaiting human verification of full Phase 1 overlay experience (17 verification steps)
- Phase 2 Plan 1 COMPLETE: 6 Rust Tauri IPC commands ready (keychain CRUD, xAI model validation, accessibility settings opener and permission check)
- Phase 2 Plan 2 COMPLETE: Settings panel UI complete -- tabbed Account/Model/Preferences, API key masked entry with debounced validation, model dropdown with persistence, dual entry points (tray + /settings)
- Phase 2 Plan 3 CHECKPOINT: Onboarding wizard built (5 components: Accessibility/ApiKey/ModelSelect/Done/Wizard) + App.tsx startup check + Overlay integration; awaiting human verification of complete Phase 2 experience (25 steps)
- Phase 2: Accessibility permission must be granted before terminal context reading works
- Phase 3 Plan 1 COMPLETE: get_terminal_context IPC command, terminal/detect.rs (bundle ID matching for 5 terminals), terminal/process.rs (libproc raw FFI: CWD, shell name, child PIDs, tmux walk), AppState.previous_app_pid capture before overlay show
- Phase 3: darwin-libproc crate incompatible (memchr conflict); raw FFI approach used instead, works identically
- Phase 5: AppleScript command injection must be solved before any terminal pasting

## Session Continuity

Last session: 2026-02-21 (Phase 3 Plan 1 execution)
Stopped at: Completed 03-01-PLAN.md (all tasks done, ready for 03-02)
Resume file: None
