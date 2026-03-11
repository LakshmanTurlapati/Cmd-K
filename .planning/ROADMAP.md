# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- v0.2.4 Overlay UX, Safety & CI/CD -- Phases 17-20 (shipped 2026-03-04) | [Archive](milestones/v0.2.4-ROADMAP.md)
- v0.2.6 Multi-Provider, WSL & Auto-Update -- Phases 21-24 (shipped 2026-03-09) | [Archive](milestones/v0.2.6-ROADMAP.md)
- v0.2.7 Cost Estimation -- Phases 25-26 (shipped 2026-03-10) | [Archive](milestones/v0.2.7-ROADMAP.md)
- v0.2.8 Windows Terminal Detection Fix & Provider Icons -- Phases 27-29 (in progress)

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

**Milestone Goal:** Fix unreliable WSL detection and shell type differentiation across all Windows terminal hosts. Add provider branding to onboarding and settings UI.

- [x] **Phase 27: ConPTY Discovery & Process Snapshot** - Replace highest-PID heuristic with ConPTY-aware shell discovery and consolidated process snapshot (completed 2026-03-11)
- [x] **Phase 28: UIA Terminal Text Scoping** - Scope UIA text reading to terminal panel elements only, eliminating false positives from IDE chrome (completed 2026-03-11)
- [x] **Phase 29: Provider Icon Branding** - Add provider SVG icons to onboarding wizard and settings provider selection (completed 2026-03-11)

## Phase Details

### Phase 27: ConPTY Discovery & Process Snapshot
**Goal**: User's active shell correctly identified in multi-tab IDE terminals without false positives from internal IDE processes
**Depends on**: Nothing (foundation phase)
**Requirements**: PROC-01, PROC-02, PROC-03
**Success Criteria** (what must be TRUE):
  1. In VS Code or Cursor with multiple terminal tabs, CMD+K detects the correct interactive shell (PowerShell, cmd.exe, or bash) -- not an internal IDE git or extension process
  2. Opening CMD+K in a terminal running cmd.exe correctly identifies "cmd" as the shell type, while internal IDE cmd.exe processes (running /C or /D /C flags) are ignored
  3. A single process snapshot is taken per hotkey press and reused for all detection queries (shell discovery, WSL check, diagnostics) -- no redundant CreateToolhelp32Snapshot calls
**Plans:** 3/3 plans complete
Plans:
- [x] 27-01-PLAN.md -- ProcessSnapshot struct, PEB command line reader, cmd.exe filtering logic with unit tests
- [x] 27-02-PLAN.md -- ConPTY-first shell selection, snapshot threading through detection pipeline
- [x] 27-03-PLAN.md -- UIA shell type hint for multi-tab disambiguation (gap closure)

### Phase 28: UIA Terminal Text Scoping
**Goal**: Terminal text reading captures only terminal panel content, not editor or sidebar text from the IDE window
**Depends on**: Nothing (independent of Phase 27)
**Requirements**: UIAS-01, UIAS-02
**Success Criteria** (what must be TRUE):
  1. When CMD+K reads terminal text in VS Code or Cursor, the captured text contains only terminal output -- no code editor content, sidebar text, or menu labels leak into the reading
  2. A Linux-style path appearing only in the VS Code editor (not the terminal) does not trigger WSL detection -- multiple corroborating signals are required before declaring a session as WSL
**Plans:** 2/2 plans complete
Plans:
- [x] 28-01-PLAN.md -- Multi-signal WSL text detection with scoring threshold (TDD)
- [x] 28-02-PLAN.md -- Scoped UIA tree walk targeting terminal List elements

### Phase 29: Provider Icon Branding
**Goal**: Provider selection in onboarding and settings shows recognizable SVG icons (same as showcase site) instead of plain text initials
**Depends on**: Nothing (UI-only phase)
**Requirements**: ICON-01, ICON-02
**Success Criteria** (what must be TRUE):
  1. The onboarding provider selection step shows inline SVG icons (OpenAI, Anthropic, Gemini, xAI, OpenRouter) matching the showcase site's provider cards
  2. The settings provider dropdown/selector shows the same SVG icons next to provider names
**Plans:** 1/1 plans complete
Plans:
- [ ] 29-01-PLAN.md -- ProviderIcon component and integration into onboarding + settings UI

## Progress

**Execution Order:**
Phases execute in numeric order: 27 -> 28 -> 29

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7.1 | v0.1.0 | 21/21 | Complete | 2026-02-28 |
| 8-10 | v0.1.1 | 6/6 | Complete | 2026-03-01 |
| 11-16, 01 | v0.2.1 | 7/7 | Complete | 2026-03-03 |
| 17-20 | v0.2.4 | 5/5 | Complete | 2026-03-04 |
| 21-24 | v0.2.6 | 10/10 | Complete | 2026-03-09 |
| 25-26 | v0.2.7 | 3/3 | Complete | 2026-03-10 |
| 27. ConPTY Discovery | 3/3 | Complete    | 2026-03-11 | 2026-03-11 |
| 28. UIA Scoping | 2/2 | Complete    | 2026-03-11 | 2026-03-11 |
| 29. Provider Icons | 1/1 | Complete   | 2026-03-11 | - |
