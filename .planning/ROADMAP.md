# Roadmap: CMD+K

## Overview

CMD+K transforms from concept to production through six phases that establish the foundation (overlay + permissions), enable configuration (settings + onboarding), gather terminal context (directory + output reading), generate AI commands (xAI integration + streaming), ensure safety (destructive command detection), and finally automate pasting (AppleScript terminal integration). Each phase builds on the previous, delivering verifiable capabilities that combine into a lightweight macOS overlay app for AI-powered terminal command generation.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation & Overlay** - System-wide overlay with global hotkey (completed 2026-02-21)
- [x] **Phase 2: Settings & Configuration** - API key management and onboarding (completed 2026-02-21)
- [ ] **Phase 3: Terminal Context Reading** - Detect terminal state without shell plugins
- [ ] **Phase 4: AI Command Generation** - Natural language to terminal commands via xAI
- [ ] **Phase 5: Safety Layer** - Destructive command detection and warnings
- [ ] **Phase 6: Terminal Pasting** - Auto-paste commands to active terminal

## Phase Details

### Phase 1: Foundation & Overlay
**Goal**: System-wide overlay appears on top of active application with instant keyboard access
**Depends on**: Nothing (first phase)
**Requirements**: OVRL-01, OVRL-02, OVRL-03, OVRL-04, OVRL-05
**Success Criteria** (what must be TRUE):
  1. User presses Cmd+Shift+K from any application and overlay appears on top of current window
  2. User can type text into overlay input field using keyboard only
  3. User presses Escape and overlay disappears without affecting underlying application
  4. User can configure hotkey to avoid conflicts via menu bar settings
  5. App runs silently in background with menu bar icon (no dock icon)
**Plans**: 3 plans

Plans:
- [x] 01-01-PLAN.md -- Tauri v2 project scaffolding and Rust backend (NSPanel, vibrancy, global hotkey, tray icon)
- [x] 01-02-PLAN.md -- Frontend overlay UI (Overlay, CommandInput, ResultsArea, animations, keyboard/click dismiss)
- [~] 01-03-PLAN.md -- Hotkey configuration dialog (presets, custom recorder, persistence) + human verification [CHECKPOINT: awaiting human verify]

### Phase 2: Settings & Configuration
**Goal**: User can configure xAI API credentials and model preferences securely
**Depends on**: Phase 1 (needs menu bar and UI framework)
**Requirements**: SETT-01, SETT-02, SETT-03, SETT-04
**Success Criteria** (what must be TRUE):
  1. User can enter xAI API key and app validates it against xAI API
  2. User can select which Grok model to use from available options
  3. API key is stored in macOS Keychain (not plaintext config file)
  4. First-run wizard guides user through Accessibility permissions and API key setup
**Plans**: 3 plans

Plans:
- [x] 02-01-PLAN.md -- Rust backend: Keychain storage (keyring crate), xAI API validation/model fetching (reqwest), Accessibility permission check
- [x] 02-02-PLAN.md -- Settings panel UI: Zustand store extension, tabbed settings (Account/Model/Preferences), tray + /settings wiring
- [ ] 02-03-PLAN.md -- Onboarding wizard: 4-step setup flow (Accessibility, API key, Model, Done) with persistence + human verification

### Phase 3: Terminal Context Reading
**Goal**: App detects active app context (terminal state, browser console, app identity) and displays appropriate badge without shell plugins
**Depends on**: Phase 2 (needs Accessibility permissions from onboarding)
**Requirements**: TERM-02, TERM-03, TERM-04
**Success Criteria** (what must be TRUE):
  1. User working in terminal at specific directory, overlay shows correct current working directory
  2. App reads recent terminal output visible on screen for command context
  3. Detection works across Terminal.app, iTerm2, Alacritty, kitty, and WezTerm
  4. When Accessibility permission denied, app gracefully degrades with clear explanation
  5. Every frontmost app gets a badge: shell type for terminals, "Console" for browsers with DevTools, cleaned app name otherwise
  6. Browser console detection works for Chrome, Safari, Firefox, Arc, Edge, Brave
  7. Shells inside code editors (VS Code, Cursor) show shell badge, not editor name
**Plans**: 5 plans

Plans:
- [x] 03-01-PLAN.md -- Rust backend: dependencies, AppState PID capture, terminal detection module (detect.rs + process.rs), get_terminal_context IPC command
- [x] 03-02-PLAN.md -- AX tree text reading for Terminal.app/iTerm2, sensitive data filtering, 500ms timeout wrapper, full detection pipeline wiring
- [ ] 03-03-PLAN.md -- Frontend overlay integration: Zustand terminal context state, shell type label, accessibility banner, spinner + human verification
- [ ] 03-04-PLAN.md -- App name resolution, browser console detection, AppContext struct, get_app_context IPC command
- [ ] 03-05-PLAN.md -- Frontend badge priority system (shell > console > app name) + human verification of all badge scenarios

### Phase 4: AI Command Generation
**Goal**: User describes intent in natural language and receives appropriate terminal command via xAI
**Depends on**: Phase 2 (needs API key), Phase 3 (needs terminal context)
**Requirements**: AICG-01, AICG-02
**Success Criteria** (what must be TRUE):
  1. User types "find all PDFs modified this week" and receives working find command
  2. Command streams token-by-token in real-time as xAI generates response
  3. Generated command includes context from current directory and recent terminal output
  4. User can copy command to clipboard with single keystroke
**Plans**: 3 plans

Plans:
- [x] 04-01-PLAN.md -- Rust backend: stream_ai_response command with SSE streaming, eventsource-stream, two-mode system prompts, Tauri IPC Channel
- [ ] 04-02-PLAN.md -- Frontend streaming UI: Zustand state machine, ResultsArea token renderer, CommandInput updates, two-Escape keyboard, clipboard integration
- [ ] 04-03-PLAN.md -- CSS polish (shake animation) + human verification of complete Phase 4 experience (22 checks)

### Phase 5: Safety Layer
**Goal**: Destructive commands are flagged with warnings before user can execute them
**Depends on**: Phase 4 (needs command generation)
**Requirements**: AICG-03
**Success Criteria** (what must be TRUE):
  1. Commands containing rm -rf, DROP TABLE, git push --force trigger warning UI
  2. User must explicitly confirm destructive command before copying to clipboard
  3. Warning displays what the command will do in plain language
  4. User can configure destructive pattern detection sensitivity
**Plans**: TBD

Plans:
- [ ] TBD during phase planning

### Phase 6: Terminal Pasting
**Goal**: Generated command is automatically pasted into active terminal for immediate execution
**Depends on**: Phase 5 (needs safety layer), Phase 3 (needs terminal detection)
**Requirements**: TERM-01
**Success Criteria** (what must be TRUE):
  1. User approves command and it appears in active terminal's input line
  2. Pasting works for Terminal.app and iTerm2 via AppleScript
  3. When auto-paste unavailable (unsupported terminal), fallback to clipboard with notification
  4. Focus returns to terminal after paste completes
**Plans**: TBD

Plans:
- [ ] TBD during phase planning

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & Overlay | 2/3 (3rd at checkpoint) | Complete    | 2026-02-21 |
| 2. Settings & Configuration | 2/3 | Complete    | 2026-02-21 |
| 3. Terminal Context Reading | 2/5 | In Progress|  |
| 4. AI Command Generation | 1/3 | In Progress | - |
| 5. Safety Layer | 0/TBD | Not started | - |
| 6. Terminal Pasting | 0/TBD | Not started | - |
