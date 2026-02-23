# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-21)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** Phase 5 - Safety Layer

## Current Position

Phase: 5 of 6 (Safety Layer)
Plan: 1 of 2 in current phase (05-01 complete)
Status: Phase 5 Plan 1 complete -- Rust safety backend and Zustand state infrastructure ready; Plan 2 will wire badge UI and overlay integration
Last activity: 2026-02-23 - Executed Phase 5 Plan 1 (safety.rs RegexSet detection, xAI explanation API, Zustand destructive state, PreferencesTab toggle)

Progress: [██████████] 97%

## Performance Metrics

**Velocity:**
- Total plans completed: 9
- Average duration: 7 min
- Total execution time: 1.1 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-foundation-overlay | 3 | 22 min | 7 min |
| 02-settings-configuration | 2 | 24 min | 12 min |
| 03-terminal-context-reading | 4 | 27 min | 7 min |
| 04-ai-command-generation | 3 | 35 min | 12 min |
| 05-safety-layer | 1 | 3 min | 3 min |

**Recent Trend:**
- Last 5 plans: 8 min (04-01), 2 min (04-02), 25 min (04-03), 3 min (05-01)
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
- Inline raw CF FFI in ax_reader.rs instead of importing core-foundation-sys: avoids direct dep on transitive crate; CF types are *const c_void aliases with stable ABI
- detect() wraps detect_inner() transparently for 500ms timeout: commands/terminal.rs keeps calling terminal::detect(pid) unchanged
- CFRetain before CFRelease of parent CFArray in find_text_area_in_children: CFArrayGetValueAtIndex does not retain; item must be retained before array is released
- Fire-and-forget async IIFE in show(): keeps overlay appearance instant while context detection runs in background
- lsof fallback for CWD: proc_pidinfo fails for processes spawned under root-owned login wrappers (Terminal.app pattern on macOS)
- pgrep ancestry search for shells: enables detection in deep process trees beyond 3-level recursive walk (VS Code, Cursor, Electron apps)
- terminalContext reset to null on each show(): prevents stale context leaking between consecutive overlay opens
- Accessibility banner re-checked on every open (not cached): ensures banner disappears immediately after permission grant
- AX window title heuristic for DevTools detection: simpler and browser-agnostic vs deep AX tree walk; works for Chrome/Safari/Firefox
- detect_full() as new public API alongside detect(): AppContext for all apps, TerminalContext for backward compatibility
- AppContext wraps terminal and browser state: frontend gets one unified response for badges and context display
- context_json String parameter for AppContext IPC boundary: avoids Deserialize coupling to Rust struct (AppContext only derives Serialize)
- AppContextView lightweight deserialization: only fields needed for prompt building declared in ai.rs; fallback to assistant mode on parse failure
- Keychain account name corrected to xai_api_key: plan had wrong value 'api-key'; must match keychain.rs ACCOUNT constant exactly
- Two-mode AI system prompt: terminal mode (command-only with {shell_type} substitution) vs assistant mode (2-3 sentence conversational)
- [Phase 04-02]: submitQuery reads fresh state via useOverlayStore.getState() inside async IIFE to avoid stale closures
- [Phase 04-02]: displayMode state machine ('input'|'streaming'|'result') replaces submitted/showApiWarning booleans for slot switching
- [Phase 04-02]: show() resets turnHistory: [] to clear session on each overlay open
- [Phase 04-03]: Escape always closes overlay (single press) -- simplified from two-Escape cycling state machine per user preference
- [Phase 04-03]: CommandInput always rendered above ResultsArea (stacked) so query stays visible during streaming
- [Phase 04-03]: handleMouseUp click-position heuristic -- selectionStart !== length triggers clear input vs edit mode
- [Phase 04-03]: hide() resets all streaming state (isStreaming, displayMode, streamingText, streamError) for clean reopen
- [Phase 05-01]: once_cell Lazy<RegexSet> for DESTRUCTIVE_PATTERNS: compiled once at first call, zero allocation on subsequent checks
- [Phase 05-01]: model: String parameter passed from frontend to get_destructive_explanation (Rust cannot read Zustand)
- [Phase 05-01]: temperature: 0.0 for get_destructive_explanation: deterministic safety explanations preferred
- [Phase 05-01]: destructiveDetectionEnabled defaults to true when key absent from settings.json; loaded in both onboarding branches

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
- Phase 3 Plan 2 COMPLETE: ax_reader.rs (AX tree walker for Terminal.app + iTerm2), filter.rs (7-pattern credential filter), detect() timeout wrapper (500ms), full pipeline wired
- Phase 3 Plan 3 AUTOMATION COMPLETE (awaiting human verify): Zustand store wired to Rust detection (terminalContext, isDetectingContext, accessibilityGranted), Overlay.tsx shell label + spinner + accessibility banner, lsof/pgrep fallbacks for robust shell detection
- Phase 3 Plan 4 COMPLETE: AppContext struct, browser.rs DevTools detection, get_app_context IPC command, app name resolution via NSRunningApplication.localizedName
- Phase 3: darwin-libproc crate incompatible (memchr conflict); raw FFI approach used instead, works identically
- Phase 4 Plan 1 COMPLETE: stream_ai_response Rust command with eventsource-stream SSE parsing, Channel<String> token streaming, two-mode system prompts, 10s timeout, Keychain API key read
- Phase 4 Plan 2 COMPLETE: Frontend streaming UI -- Zustand displayMode state machine, ResultsArea monospace renderer with block cursor, two-Escape keyboard flow, session history (7 turns), auto-copy + click-to-copy
- Phase 4 Plan 3 COMPLETE: End-to-end verification passed (all 22 checks); UX polish applied -- single-Escape close, input visible above results, click-position-aware input interaction
- Phase 4 COMPLETE: All requirements AICG-01 and AICG-02 satisfied
- Phase 5 Plan 1 COMPLETE: safety.rs with check_destructive (RegexSet) and get_destructive_explanation (xAI non-streaming); Zustand 4 destructive state fields; PreferencesTab Safety toggle with persistence; App.tsx startup preference loading
- Phase 5: AppleScript command injection must be solved before any terminal pasting

## Session Continuity

Last session: 2026-02-23 (Phase 5 Plan 1 execution)
Stopped at: Completed 05-01-PLAN.md (safety backend + Zustand state + settings toggle complete)
Resume file: None
