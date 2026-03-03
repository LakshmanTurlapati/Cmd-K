# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16 (in progress)

## Phases

<details>
<summary>v0.1.0 MVP (Phases 1-7.1) -- SHIPPED 2026-02-28</summary>

- [x] Phase 1: Foundation & Overlay (3/3 plans) -- completed 2026-02-21
- [x] Phase 2: Settings & Configuration (3/3 plans) -- completed 2026-02-21
- [x] Phase 3: Terminal Context Reading (5/5 plans) -- completed 2026-02-23
- [x] Phase 4: AI Command Generation (3/3 plans) -- completed 2026-02-23
- [x] Phase 5: Safety Layer (2/2 plans) -- completed 2026-02-23
- [x] Phase 6: Terminal Pasting (2/2 plans) -- completed 2026-02-23
- [x] Phase 7: Fix Accessibility Permission Detection (2/2 plans) -- completed 2026-02-26
- [x] Phase 7.1: Debug AI Streaming on Production DMG (1/1 plan) -- completed 2026-02-28

</details>

<details>
<summary>v0.1.1 Command History & Follow-ups (Phases 8-10) -- SHIPPED 2026-03-01</summary>

- [x] Phase 8: Window Identification & History Storage (3/3 plans) -- completed 2026-03-01
- [x] Phase 9: Arrow Key History Navigation (1/1 plan) -- completed 2026-03-01
- [x] Phase 10: AI Follow-up Context Per Window (2/2 plans) -- completed 2026-03-01

</details>

### v0.2.1 Windows Support (Phases 11-16)

- [x] **Phase 11: Build Infrastructure and Overlay Foundation** - Cross-platform Cargo.toml, Windows overlay with Acrylic/Mica vibrancy, hotkey, focus management (completed 2026-03-02)
- [x] **Phase 12: Terminal Context -- Process Tree, CWD, Detection** - Shell PID detection, CWD reading via PEB, shell type detection, window key computation (code complete 2026-03-02)
- [x] **Phase 13: Paste and Input Simulation** - Clipboard write, terminal activation, Ctrl+V keystroke injection, elevation detection (code complete 2026-03-02)
- [x] **Phase 14: Terminal Output Reading via UIA** - Windows UI Automation text reading for Windows Terminal, PowerShell, CMD (code complete 2026-03-02)
- [x] **Phase 15: Platform Polish and Safety** - Onboarding adaptation, AI prompt platform awareness, Windows destructive patterns, tray and UI conventions (code complete 2026-03-02)
- [x] **Phase 16: Build, Distribution, and Integration Testing** - NSIS installer, WebView2 bootstrapper, ICO icon (code complete 2026-03-02; E2E testing pending)

## Phase Details

### Phase 11: Build Infrastructure and Overlay Foundation
**Goal**: Windows build compiles without breaking macOS, and the overlay window appears with native vibrancy on Ctrl+Shift+K with correct focus management
**Depends on**: Phase 10 (v0.1.1 complete)
**Requirements**: WOVL-01, WOVL-02, WOVL-03, WOVL-04, WOVL-05, WOVL-06, WOVL-07, WBLD-01, WBLD-02
**Success Criteria** (what must be TRUE):
  1. Project compiles on Windows with `cargo build` and on macOS without regressions (no cfg breakage)
  2. Pressing Ctrl+Shift+K on Windows shows the overlay with Acrylic or Mica frosted glass vibrancy
  3. Overlay does not appear in Alt+Tab window switcher or taskbar
  4. Previous terminal window HWND is captured before overlay appears, and focus returns to that terminal on dismiss (Escape or command accept)
  5. Overlay floats above all windows with always-on-top behavior
**Plans**: 4 plans

Plans:
- [x] 11-01-PLAN.md -- Build infrastructure: platform-gate Cargo.toml deps, cfg-gate macOS imports, extend AppState
- [x] 11-02-PLAN.md -- Windows overlay window: Acrylic/Mica vibrancy, WS_EX_TOOLWINDOW, always-on-top, show/hide commands
- [x] 11-03-PLAN.md -- Windows focus management: HWND capture, AttachThreadInput focus restore, Ctrl+Shift+K hotkey
- [ ] 11-04-PLAN.md -- Gap closure: fix HWND capture dead code (move outside PID-gated block)

### Phase 12: Terminal Context -- Process Tree, CWD, Detection
**Goal**: CMD+K identifies the active terminal's shell process, reads its working directory, and detects the shell type -- all without shell plugins
**Depends on**: Phase 11
**Requirements**: WCTX-01, WCTX-02, WCTX-03, WCTX-04, WCTX-05, WCTX-06
**Success Criteria** (what must be TRUE):
  1. Shell PID is correctly detected via process tree walking for Windows Terminal, PowerShell, CMD, and Git Bash
  2. Current working directory is read from the shell process and displayed in the overlay context
  3. Shell type (powershell.exe, pwsh.exe, cmd.exe, bash.exe) is detected and sent to AI for platform-appropriate commands
  4. Window key is computed as exe_name:shell_pid and per-window history works on Windows
  5. CWD gracefully returns None for elevated or inaccessible processes instead of crashing
**Plans**: TBD

Plans:
- [ ] 12-01: TBD
- [ ] 12-02: TBD

Note: Phase 12 and Phase 13 are architecturally independent and can be developed in parallel.

### Phase 13: Paste and Input Simulation
**Goal**: Generated commands are pasted into the active Windows terminal automatically using clipboard and keystroke injection
**Depends on**: Phase 11
**Requirements**: WPST-01, WPST-02, WPST-03, WPST-04, WPST-05
**Success Criteria** (what must be TRUE):
  1. Generated command is written to clipboard and pasted into the active terminal via Ctrl+V
  2. Terminal window is activated (brought to foreground) before paste occurs
  3. Enter keystroke is sent after paste for command confirmation
  4. When terminal is running as Administrator, user sees a clear warning instead of silent paste failure
**Plans**: TBD

Plans:
- [ ] 13-01: TBD
- [ ] 13-02: TBD

Note: Phase 13 depends only on Phase 11 (HWND capture). Can be developed in parallel with Phase 12.

### Phase 14: Terminal Output Reading via UIA
**Goal**: CMD+K reads visible terminal text on Windows to provide AI with recent command output context
**Depends on**: Phase 12
**Requirements**: WOUT-01, WOUT-02, WOUT-03
**Success Criteria** (what must be TRUE):
  1. Terminal text is read from Windows Terminal and displayed in AI context
  2. Terminal text is read from PowerShell and CMD (conhost) windows
  3. Terminals without UIA support (mintty, GPU-rendered terminals) gracefully return None without errors
**Plans**: TBD

Plans:
- [ ] 14-01: TBD

### Phase 15: Platform Polish and Safety
**Goal**: Windows-specific UX adaptations make CMD+K feel native -- correct onboarding, platform-aware AI, Windows safety patterns, and native UI conventions
**Depends on**: Phase 13, Phase 14
**Requirements**: WPLH-01, WPLH-02, WPLH-03, WPLH-04, WPLH-05, WPLH-06
**Success Criteria** (what must be TRUE):
  1. Onboarding flow on Windows skips the macOS accessibility permission step entirely
  2. AI generates Windows-appropriate commands (PowerShell, CMD syntax) because system prompt identifies the platform
  3. Destructive Windows commands (Remove-Item -Recurse, rd /s, bcdedit, format, Reg Delete) trigger safety warnings
  4. System tray icon shows context menu on right-click (Windows convention, not macOS left-click)
  5. Keyboard shortcuts display Ctrl (not Cmd) in UI when running on Windows
**Plans**: TBD

Plans:
- [ ] 15-01: TBD
- [ ] 15-02: TBD

### Phase 16: Build, Distribution, and Integration Testing
**Goal**: CMD+K ships as a signed Windows installer that works across all target terminals
**Depends on**: Phase 15
**Requirements**: WBLD-03, WBLD-04, WBLD-05, WBLD-06
**Success Criteria** (what must be TRUE):
  1. NSIS installer produces a signed .exe setup that installs per-user without requiring admin
  2. WebView2 runtime bootstrapper is embedded so the app works on fresh Windows installs
  3. ICO format tray icon is included and displays correctly in system tray
  4. End-to-end workflow (hotkey, context, generate, paste) verified on Windows Terminal, PowerShell, CMD, and Git Bash
**Plans**: TBD

Plans:
- [ ] 16-01: TBD
- [ ] 16-02: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation & Overlay | v0.1.0 | 3/3 | Complete | 2026-02-21 |
| 2. Settings & Configuration | v0.1.0 | 3/3 | Complete | 2026-02-21 |
| 3. Terminal Context Reading | v0.1.0 | 5/5 | Complete | 2026-02-23 |
| 4. AI Command Generation | v0.1.0 | 3/3 | Complete | 2026-02-23 |
| 5. Safety Layer | v0.1.0 | 2/2 | Complete | 2026-02-23 |
| 6. Terminal Pasting | v0.1.0 | 2/2 | Complete | 2026-02-23 |
| 7. Accessibility Detection Fix | v0.1.0 | 2/2 | Complete | 2026-02-26 |
| 7.1. Production DMG Fix | v0.1.0 | 1/1 | Complete | 2026-02-28 |
| 8. Window Identification & History Storage | v0.1.1 | 3/3 | Complete | 2026-03-01 |
| 9. Arrow Key History Navigation | v0.1.1 | 1/1 | Complete | 2026-03-01 |
| 10. AI Follow-up Context Per Window | v0.1.1 | 2/2 | Complete | 2026-03-01 |
| 11. Build Infrastructure and Overlay Foundation | v0.2.1 | 4/4 | Complete | 2026-03-02 |
| 12. Terminal Context -- Process Tree, CWD, Detection | v0.2.1 | code complete | Complete | 2026-03-02 |
| 13. Paste and Input Simulation | v0.2.1 | code complete | Complete | 2026-03-02 |
| 14. Terminal Output Reading via UIA | v0.2.1 | code complete | Complete | 2026-03-02 |
| 15. Platform Polish and Safety | v0.2.1 | code complete | Complete | 2026-03-02 |
| 16. Build, Distribution, and Integration Testing | v0.2.1 | code complete | Human Needed (E2E) | 2026-03-02 |
| 01. Merge Windows branch | v0.2.1 | Complete    | 2026-03-03 | 2026-03-03 |

### Phase 1: Merge Windows branch — resolve conflicts and ensure platform-independent builds

**Goal:** Merge the `windows` branch (30 commits, 58 files) into `main`, resolve conflicts, align versions to v0.2.1, add platform-specific build scripts, and verify macOS compilation
**Requirements**: TBD
**Depends on:** Phase 0
**Plans:** 2 plans

Plans:
- [x] 01-01-PLAN.md — Merge windows branch, resolve showcase conflict, align versions to v0.2.1
- [x] 01-02-PLAN.md — Add build scripts, verify macOS build, delete windows branch
