# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- v0.2.4 Overlay UX, Safety & CI/CD -- Phases 17-20 (shipped 2026-03-04) | [Archive](milestones/v0.2.4-ROADMAP.md)
- v0.2.6 Multi-Provider, WSL & Auto-Update -- Phases 21-24 (shipped 2026-03-09) | [Archive](milestones/v0.2.6-ROADMAP.md)
- v0.2.7 Cost Estimation -- Phases 25-26 (shipped 2026-03-10) | [Archive](milestones/v0.2.7-ROADMAP.md)
- v0.2.8 Windows Terminal Detection Fix -- Phases 27-30 (in progress)

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

### v0.2.8 Windows Terminal Detection Fix (In Progress)

**Milestone Goal:** Fix unreliable WSL detection and shell type differentiation across all Windows terminal hosts.

- [ ] **Phase 27: ConPTY Discovery & Process Snapshot** - Replace highest-PID heuristic with ConPTY-aware shell discovery and consolidated process snapshot
- [ ] **Phase 28: UIA Terminal Text Scoping** - Scope UIA text reading to terminal panel elements only, eliminating false positives from IDE chrome
- [ ] **Phase 29: Active Tab Matching** - Identify the active terminal tab's shell via CWD-based disambiguation and window title parsing
- [ ] **Phase 30: WSL Detection Hardening** - Reliable WSL detection across all scenarios using multi-signal waterfall with sibling and environment detection

## Phase Details

### Phase 27: ConPTY Discovery & Process Snapshot
**Goal**: User's active shell correctly identified in multi-tab IDE terminals without false positives from internal IDE processes
**Depends on**: Nothing (foundation phase)
**Requirements**: PROC-01, PROC-02, PROC-03
**Success Criteria** (what must be TRUE):
  1. In VS Code or Cursor with multiple terminal tabs, CMD+K detects the correct interactive shell (PowerShell, cmd.exe, or bash) -- not an internal IDE git or extension process
  2. Opening CMD+K in a terminal running cmd.exe correctly identifies "cmd" as the shell type, while internal IDE cmd.exe processes (running /C or /D /C flags) are ignored
  3. A single process snapshot is taken per hotkey press and reused for all detection queries (shell discovery, WSL check, diagnostics) -- no redundant CreateToolhelp32Snapshot calls
**Plans**: TBD

### Phase 28: UIA Terminal Text Scoping
**Goal**: Terminal text reading captures only terminal panel content, not editor or sidebar text from the IDE window
**Depends on**: Nothing (independent of Phase 27)
**Requirements**: UIAS-01, UIAS-02
**Success Criteria** (what must be TRUE):
  1. When CMD+K reads terminal text in VS Code or Cursor, the captured text contains only terminal output -- no code editor content, sidebar text, or menu labels leak into the reading
  2. A Linux-style path appearing only in the VS Code editor (not the terminal) does not trigger WSL detection -- multiple corroborating signals are required before declaring a session as WSL
**Plans**: TBD

### Phase 29: Active Tab Matching
**Goal**: The focused terminal tab's shell is correctly identified even when multiple tabs with different shell types are open
**Depends on**: Phase 27 (needs ConPTY shell list to match against)
**Requirements**: TABM-01, TABM-02
**Success Criteria** (what must be TRUE):
  1. With three terminal tabs open (PowerShell, cmd.exe, bash) in VS Code, CMD+K identifies the shell matching the currently focused tab -- not whichever process was most recently spawned
  2. In Windows Terminal with multiple panes or tabs, CMD+K identifies the shell process belonging to the focused pane via UIA tree correlation
**Plans**: TBD

### Phase 30: WSL Detection Hardening
**Goal**: WSL sessions reliably detected across all terminal hosts without false positives or missed detections
**Depends on**: Phases 27, 28, 29 (benefits from clean shell candidates, scoped text, and correct tab identification)
**Requirements**: WSLD-01, WSLD-02, WSLD-03
**Success Criteria** (what must be TRUE):
  1. A WSL terminal tab in Windows Terminal, VS Code, or Cursor is correctly detected as WSL and generates Linux commands -- even without Remote-WSL mode enabled
  2. WSL detection uses wsl.exe sibling relationship (shared ConPTY parent with the detected shell) as a primary signal, working for both WSL 1 and WSL 2 without relying on Linux process visibility
  3. When multiple WSL distros are installed, CMD+K identifies the correct distro for the active tab via process args or window title, and reads the correct Linux CWD
  4. Non-WSL terminals (native PowerShell, cmd.exe) are never falsely identified as WSL, even when WSL-related processes exist in the system process tree
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 27 -> 28 -> 29 -> 30

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7.1 | v0.1.0 | 21/21 | Complete | 2026-02-28 |
| 8-10 | v0.1.1 | 6/6 | Complete | 2026-03-01 |
| 11-16, 01 | v0.2.1 | 7/7 | Complete | 2026-03-03 |
| 17-20 | v0.2.4 | 5/5 | Complete | 2026-03-04 |
| 21-24 | v0.2.6 | 10/10 | Complete | 2026-03-09 |
| 25-26 | v0.2.7 | 3/3 | Complete | 2026-03-10 |
| 27. ConPTY Discovery | v0.2.8 | 0/0 | Not started | - |
| 28. UIA Scoping | v0.2.8 | 0/0 | Not started | - |
| 29. Tab Matching | v0.2.8 | 0/0 | Not started | - |
| 30. WSL Hardening | v0.2.8 | 0/0 | Not started | - |
