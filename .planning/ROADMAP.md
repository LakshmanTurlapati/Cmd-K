# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- v0.2.4 Overlay UX, Safety & CI/CD -- Phases 17-20 (shipped 2026-03-04) | [Archive](milestones/v0.2.4-ROADMAP.md)
- v0.2.6 Multi-Provider, WSL & Auto-Update -- Phases 21-24 (in progress)

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

### v0.2.6 Multi-Provider, WSL & Auto-Update (In Progress)

- [x] **Phase 21: Provider Abstraction Layer** - Rust backend with provider enum, per-provider streaming, parameterized keychain, and v0.2.4 migration (completed 2026-03-09)
- [x] **Phase 22: Multi-Provider Frontend** - Provider selection in onboarding and settings, model picker, OpenRouter integration (completed 2026-03-09)
- [x] **Phase 23: WSL Terminal Context** - Detect WSL sessions, read CWD/shell/output, generate Linux commands, apply Linux safety patterns (completed 2026-03-09)
- [x] **Phase 23.1: VS Code WSL Terminal Tab Detection** - Window title and focused-element UIA detection for VS Code/Cursor Remote-WSL terminals, IDE-aware shell priority (completed 2026-03-09)
- [x] **Phase 24: Auto-Updater** - Check on launch, tray notification, signed updates, CI/CD manifest generation (completed 2026-03-09)

## Phase Details

### Phase 21: Provider Abstraction Layer
**Goal**: Users can generate commands from any of the 5 supported AI providers with correct streaming, key storage, and error handling
**Depends on**: Phase 20 (v0.2.4 shipped baseline)
**Requirements**: PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06, PROV-07
**Success Criteria** (what must be TRUE):
  1. User can select a provider (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) and the selection persists across app restarts
  2. User can store and retrieve a separate API key per provider using the platform keychain
  3. Existing v0.2.4 users upgrading see their xAI API key preserved automatically with xAI as default provider
  4. User can validate any provider's API key and see a provider-specific success or error message
  5. User can generate a command and see it stream in real-time from any of the 5 providers
**Plans:** 3 plans (2 complete + 1 gap closure)

Plans:
- [ ] 21-01-PLAN.md — Provider enum, 3 streaming adapters, parameterized keychain, v0.2.4 migration
- [ ] 21-02-PLAN.md — Per-provider key validation, model fetching, frontend IPC updates

### Phase 22: Multi-Provider Frontend
**Goal**: Users can discover, select, and switch providers through polished onboarding and settings UI
**Depends on**: Phase 21 (backend IPC signatures must be stable)
**Requirements**: PFUI-01, PFUI-02, PFUI-03, PFUI-04, PFUI-05, ORTR-01, ORTR-02
**Success Criteria** (what must be TRUE):
  1. New user can pick their preferred provider during first-run onboarding and proceed to API key entry
  2. User can switch providers in the settings Account tab and the overlay immediately uses the new provider
  3. User can pick a model from a dropdown that shows only models for their selected provider, grouped by capability tier (Fast, Balanced, Most Capable)
  4. User can switch providers without losing their conversation history
  5. User with an OpenRouter API key can access models from multiple providers through a single key, with the model list filtered to chat-capable models
**Plans:** 3 plans (2 complete + 1 gap closure)

Plans:
- [ ] 22-01-PLAN.md — Provider selection in onboarding + store foundation (StepProviderSelect, 5-step wizard, provider-aware text, App.tsx startup loading)
- [ ] 22-02-PLAN.md — Settings provider dropdown, tier-grouped model lists, per-provider model memory, arrow key navigation

### Phase 23: WSL Terminal Context
**Goal**: Users in WSL sessions get the same context-aware command generation experience as native terminal users
**Depends on**: Phase 21 (independent of Phase 22; can run after Phase 21 or in parallel with Phase 22)
**Requirements**: WSLT-01, WSLT-02, WSLT-03, WSLT-04, WSLT-05, WSLT-06, WSLT-07, WSLT-08, WSLT-09, WSLT-10
**Success Criteria** (what must be TRUE):
  1. User in a WSL session (Windows Terminal, VS Code Remote-WSL, Cursor Remote-WSL, or standalone wsl.exe) triggers CMD+K and the app detects it as a WSL session
  2. User sees their Linux CWD and shell type (bash, zsh, fish) in the context badge, with the WSL distro name displayed (e.g., "bash (WSL: Ubuntu)")
  3. User can read visible terminal output from WSL sessions for AI context
  4. User asking for a command in a WSL session gets a Linux command (not a Windows command)
  5. Destructive Linux command patterns (rm -rf, systemctl, etc.) are applied when the user is in a WSL session
**Plans:** 3 plans (2 complete + 1 gap closure)

Plans:
- [ ] 23-01-PLAN.md — WSL detection in process tree, Linux CWD/shell reading, secret filtering
- [ ] 23-02-PLAN.md — WSL system prompt, safety awareness, frontend badge
- [ ] 23-03-PLAN.md — Gap closure: UIA text-based WSL detection (fixes WSL 2 detection failure)

### Phase 23.1: VS Code WSL terminal tab detection via UIA (INSERTED)

**Goal:** VS Code and Cursor Remote-WSL terminals are correctly detected as WSL sessions using window title and focused-element UIA strategies
**Requirements**: WSLT-02, WSLT-03, WSLT-05, WSLT-06
**Depends on:** Phase 23
**Success Criteria** (what must be TRUE):
  1. VS Code Remote-WSL terminal triggers WSL detection via window title "[WSL: distro]" pattern
  2. Cursor Remote-WSL terminal triggers WSL detection via same window title mechanism
  3. VS Code WSL terminal shows Linux CWD and bash/zsh shell type (not Windows path or cmd)
  4. UIA text reading for VS Code uses focused-element sub-tree to reduce noise from UI chrome
  5. Non-WSL VS Code terminals are not affected (no false positives)
**Plans:** 2/2 plans complete

Plans:
- [x] 23.1-01-PLAN.md — Window title WSL detection, focused-element UIA strategy, end-to-end verification
- [x] 23.1-02-PLAN.md — IDE-aware shell priority, wsl.exe removal from KNOWN_TERMINAL_EXES

### Phase 24: Auto-Updater
**Goal**: Users are notified of new versions and can update with one click without forced restarts
**Depends on**: Phase 21 (independent of Phases 22-23; can run after Phase 21 or in parallel)
**Requirements**: UPDT-01, UPDT-02, UPDT-03, UPDT-04, UPDT-05, UPDT-06, UPDT-07, UPDT-08
**Success Criteria** (what must be TRUE):
  1. User launches the app and it checks for updates silently without blocking the UI
  2. User sees an "Update Available" indicator in the tray menu when a new version exists, and can dismiss it until next launch
  3. User can download and install the update with one click from the tray, with the update applied on next app launch (no forced restart)
  4. Updates are cryptographically signed (Ed25519) and verified before installation
  5. CI/CD pipeline produces signed update artifacts and a latest.json manifest alongside existing release artifacts
**Plans:** 2/2 plans complete

Plans:
- [x] 24-01-PLAN.md — Updater plugin config, update state machine, background checker, tray menu integration, install-on-quit
- [x] 24-02-PLAN.md — CI/CD pipeline: signing env vars, updater artifact generation, latest.json assembly

## Progress

**Execution Order:**
Phases 21 and 22 are sequential (22 depends on 21). Phases 23 and 24 are independent of each other and can follow Phase 21.
Recommended order: 21 -> 22 -> 23 -> 23.1 -> 24

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
| 16. Build, Distribution, and Integration Testing | v0.2.1 | code complete | Complete | 2026-03-02 |
| 01. Merge Windows branch | v0.2.1 | 2/2 | Complete | 2026-03-03 |
| 17. Overlay Z-Order | v0.2.4 | 1/1 | Complete | 2026-03-03 |
| 18. Draggable Overlay Positioning | v0.2.4 | 1/1 | Complete | 2026-03-03 |
| 19. Exhaustive Destructive Patterns | v0.2.4 | 1/1 | Complete | 2026-03-04 |
| 20. CI/CD Pipeline | v0.2.4 | 2/2 | Complete | 2026-03-04 |
| 21. Provider Abstraction Layer | v0.2.6 | Complete | Complete | 2026-03-09 |
| 22. Multi-Provider Frontend | v0.2.6 | Complete | Complete | 2026-03-09 |
| 23. WSL Terminal Context | v0.2.6 | Complete | Complete | 2026-03-09 |
| 23.1. VS Code WSL Terminal Tab Detection | 2/2 | Complete   | 2026-03-09 | 2026-03-09 |
| 24. Auto-Updater | 2/2 | Complete   | 2026-03-09 | 2026-03-09 |
