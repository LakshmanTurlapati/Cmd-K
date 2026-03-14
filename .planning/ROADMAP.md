# Roadmap: CMD+K

## Milestones

- ✅ v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- ✅ v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- ✅ v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- ✅ v0.2.4 Overlay UX, Safety & CI/CD -- Phases 17-20 (shipped 2026-03-04) | [Archive](milestones/v0.2.4-ROADMAP.md)
- ✅ v0.2.6 Multi-Provider, WSL & Auto-Update -- Phases 21-24 (shipped 2026-03-09) | [Archive](milestones/v0.2.6-ROADMAP.md)
- ✅ v0.2.7 Cost Estimation -- Phases 25-26 (shipped 2026-03-10) | [Archive](milestones/v0.2.7-ROADMAP.md)
- ✅ v0.2.8 Windows Terminal Detection Fix & Provider Icons -- Phases 27-29 (shipped 2026-03-14) | [Archive](milestones/v0.2.8-ROADMAP.md)
- **v0.3.9 Linux Support & Smart Terminal Context** -- Phases 30-35 (in progress)

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

<details>
<summary>v0.2.1 Windows Support (Phases 11-16, 01-merge) -- SHIPPED 2026-03-03</summary>

- [x] Phase 11: Build Infrastructure and Overlay Foundation (4/4 plans) -- completed 2026-03-02
- [x] Phase 12: Terminal Context -- Process Tree, CWD, Detection (code complete) -- completed 2026-03-02
- [x] Phase 13: Paste and Input Simulation (code complete) -- completed 2026-03-02
- [x] Phase 14: Terminal Output Reading via UIA (code complete) -- completed 2026-03-02
- [x] Phase 15: Platform Polish and Safety (code complete) -- completed 2026-03-02
- [x] Phase 16: Build, Distribution, and Integration Testing (code complete) -- completed 2026-03-02
- [x] Phase 01: Merge Windows Branch (2/2 plans) -- completed 2026-03-03

</details>

<details>
<summary>v0.2.4 Overlay UX, Safety & CI/CD (Phases 17-20) -- SHIPPED 2026-03-04</summary>

- [x] Phase 17: Overlay Z-Order (1/1 plan) -- completed 2026-03-03
- [x] Phase 18: Draggable Overlay Positioning (1/1 plan) -- completed 2026-03-03
- [x] Phase 19: Exhaustive Destructive Command Patterns (1/1 plan) -- completed 2026-03-04
- [x] Phase 20: GitHub Actions CI/CD Pipeline (2/2 plans) -- completed 2026-03-04

</details>

<details>
<summary>v0.2.6 Multi-Provider, WSL & Auto-Update (Phases 21-24) -- SHIPPED 2026-03-09</summary>

- [x] Phase 21: Provider Abstraction Layer (2/2 plans) -- completed 2026-03-09
- [x] Phase 22: Multi-Provider Frontend (2/2 plans) -- completed 2026-03-09
- [x] Phase 23: WSL Terminal Context (2/2 plans) -- completed 2026-03-09
- [x] Phase 23.1: VS Code WSL Terminal Tab Detection (2/2 plans) -- completed 2026-03-09
- [x] Phase 24: Auto-Updater (2/2 plans) -- completed 2026-03-09

</details>

<details>
<summary>v0.2.7 Cost Estimation (Phases 25-26) -- SHIPPED 2026-03-10</summary>

- [x] Phase 25: Token Tracking & Pricing Backend (2/2 plans) -- completed 2026-03-10
- [x] Phase 26: Cost Display Frontend (1/1 plan) -- completed 2026-03-10

</details>

<details>
<summary>v0.2.8 Windows Terminal Detection Fix & Provider Icons (Phases 27-29) -- SHIPPED 2026-03-14</summary>

- [x] Phase 27: ConPTY Discovery & Process Snapshot (3/3 plans) -- completed 2026-03-11
- [x] Phase 28: UIA Terminal Text Scoping (2/2 plans) -- completed 2026-03-11
- [x] Phase 29: Provider Icon Branding (1/1 plan) -- completed 2026-03-11

</details>

### v0.3.9 Linux Support & Smart Terminal Context (In Progress)

- [x] **Phase 30: Linux Process Detection** - CWD, shell type, and process tree via /proc filesystem (completed 2026-03-14)
- [ ] **Phase 31: Linux Overlay & Hotkey** - X11 system-wide hotkey, floating overlay, PID capture, CSS glass fallback
- [ ] **Phase 32: Linux Paste** - xdotool paste on X11, clipboard fallback on Wayland, destructive pattern integration
- [ ] **Phase 33: Smart Terminal Context** - Cross-platform ANSI stripping, token-budget truncation, command-output pairing
- [ ] **Phase 34: Linux Terminal Text Reading** - AT-SPI2 for VTE terminals, kitty/WezTerm remote control APIs
- [ ] **Phase 35: AppImage Distribution & CI/CD** - AppImage bundling, third CI job, auto-updater support, GitHub Release artifacts

## Phase Details

### Phase 30: Linux Process Detection
**Goal**: User's terminal CWD and shell type are detected on Linux without any shell configuration
**Depends on**: Nothing (foundation phase)
**Requirements**: LPROC-01, LPROC-02, LPROC-03
**Success Criteria** (what must be TRUE):
  1. User opens a terminal on Linux and CMD+K detects the correct current working directory
  2. User's shell type (bash, zsh, fish) is correctly identified regardless of terminal emulator
  3. Process tree walking finds the shell process from any terminal emulator PID (GNOME Terminal, kitty, Alacritty, etc.)
  4. The project compiles on Linux with real /proc code paths replacing stubs
**Plans:** 2/2 plans complete
Plans:
- [ ] 30-01-PLAN.md -- /proc leaf functions and ancestry search in process.rs
- [ ] 30-02-PLAN.md -- Linux detection orchestration (detect_linux.rs + mod.rs wiring)

### Phase 31: Linux Overlay & Hotkey
**Goal**: User can press Ctrl+K on Linux X11 and see a floating overlay appear above their active terminal
**Depends on**: Phase 30
**Requirements**: LOVRL-01, LOVRL-02, LOVRL-03, LOVRL-04, LOVRL-05
**Success Criteria** (what must be TRUE):
  1. User presses Ctrl+K on X11 and the overlay appears as a floating panel above the active window
  2. Active terminal's PID is captured before the overlay steals focus (capture-before-show pattern works on X11)
  3. Overlay has CSS-only frosted glass styling that looks reasonable without window-vibrancy
  4. Wayland users running with GDK_BACKEND=x11 get full overlay functionality via XWayland
  5. Overlay can be dismissed with Escape, repositioned by dragging (existing functionality carries over)
**Plans**: TBD

### Phase 32: Linux Paste
**Goal**: User completes the full Ctrl+K workflow on Linux -- query to AI response pasted into terminal
**Depends on**: Phase 31
**Requirements**: LPST-01, LPST-02, LPST-03
**Success Criteria** (what must be TRUE):
  1. User accepts an AI-generated command on X11 and it is pasted into the active terminal via xdotool
  2. User on Wayland sees the command copied to clipboard with a "press Ctrl+Shift+V" hint
  3. Destructive commands trigger the warning overlay before paste on Linux (existing patterns already cover Linux commands)
**Plans**: TBD

### Phase 33: Smart Terminal Context
**Goal**: AI receives intelligently truncated terminal context that maximizes useful information within token budget
**Depends on**: Nothing (cross-platform, can parallel with Phases 30-32)
**Requirements**: SCTX-01, SCTX-02, SCTX-03, SCTX-04
**Success Criteria** (what must be TRUE):
  1. Terminal output sent to AI has ANSI escape sequences stripped (no color codes wasting tokens)
  2. Terminal context uses approximately 10-15% of the model's context window, adapting to whichever model is selected
  3. When terminal output exceeds the budget, oldest complete command+output segments are removed while recent output is preserved
  4. Smart truncation works identically on macOS, Windows, and Linux
**Plans**: TBD

### Phase 34: Linux Terminal Text Reading
**Goal**: User's recent terminal output is captured on Linux for AI context, across multiple terminal emulators
**Depends on**: Phase 30
**Requirements**: LTXT-01, LTXT-02, LTXT-03, LTXT-04
**Success Criteria** (what must be TRUE):
  1. User in GNOME Terminal (or other VTE-based terminal) has visible terminal text read via AT-SPI2
  2. User in kitty has terminal text read via kitty remote control API
  3. User in WezTerm has terminal text read via WezTerm CLI
  4. User in Alacritty or other unsupported terminals gets graceful degradation (CWD and shell type still work, just no visible output)
**Plans**: TBD

### Phase 35: AppImage Distribution & CI/CD
**Goal**: Linux users can download and auto-update CMD+K as an AppImage from GitHub Releases
**Depends on**: Phases 30-32 (needs working Linux build)
**Requirements**: APKG-01, APKG-02, APKG-03, APKG-04
**Success Criteria** (what must be TRUE):
  1. AppImage built on Ubuntu 22.04 runs on Ubuntu 22.04+ and other mainstream distros without glibc errors
  2. GitHub Release includes Linux AppImage alongside macOS DMG and Windows NSIS installer with SHA256 checksum
  3. Auto-updater checks for and installs AppImage updates using the existing Ed25519 signing infrastructure
**Plans**: TBD

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7.1 | v0.1.0 | 21/21 | Complete | 2026-02-28 |
| 8-10 | v0.1.1 | 6/6 | Complete | 2026-03-01 |
| 11-16, 01 | v0.2.1 | 7/7 | Complete | 2026-03-03 |
| 17-20 | v0.2.4 | 5/5 | Complete | 2026-03-04 |
| 21-24 | v0.2.6 | 10/10 | Complete | 2026-03-09 |
| 25-26 | v0.2.7 | 3/3 | Complete | 2026-03-10 |
| 27-29 | v0.2.8 | 6/6 | Complete | 2026-03-14 |
| 30. Linux Process Detection | 2/2 | Complete   | 2026-03-14 | - |
| 31. Linux Overlay & Hotkey | v0.3.9 | 0/? | Not started | - |
| 32. Linux Paste | v0.3.9 | 0/? | Not started | - |
| 33. Smart Terminal Context | v0.3.9 | 0/? | Not started | - |
| 34. Linux Terminal Text Reading | v0.3.9 | 0/? | Not started | - |
| 35. AppImage Distribution & CI/CD | v0.3.9 | 0/? | Not started | - |
