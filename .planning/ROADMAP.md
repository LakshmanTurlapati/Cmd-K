# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- v0.2.2 Overlay UX Fixes (macOS) -- Phases 17-18 (in progress)

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

### v0.2.2 Overlay UX Fixes (macOS) -- In Progress

- [ ] **Phase 17: Overlay Z-Order** - System dialogs and OS overlays appear above the CMD+K panel
- [ ] **Phase 18: Draggable Overlay Positioning** - User can drag the overlay and it remembers position within the session

## Phase Details

### Phase 17: Overlay Z-Order
**Goal**: System UI elements (permission dialogs, Notification Center, Spotlight) can appear above the CMD+K overlay while the overlay still floats above normal application windows
**Depends on**: Nothing (first phase of v0.2.2)
**Requirements**: ZORD-01, ZORD-02
**Success Criteria** (what must be TRUE):
  1. When an Accessibility permission dialog appears while the overlay is open, the dialog is visible and interactable above the overlay
  2. Notification Center, Spotlight, and other system overlays render above the CMD+K overlay when invoked
  3. The overlay still floats above all normal application windows (the existing behavior that matters most is preserved)
  4. Fullscreen app behavior is not regressed -- the overlay still appears on top of fullscreen apps
**Plans**: TBD

### Phase 18: Draggable Overlay Positioning
**Goal**: User can reposition the overlay by dragging it, and the overlay reopens at the last dragged position until the app is relaunched
**Depends on**: Phase 17 (z-order changes may affect window properties that dragging interacts with)
**Requirements**: OPOS-01, OPOS-02, OPOS-03
**Success Criteria** (what must be TRUE):
  1. User can click and drag the overlay to move it to a different position on screen
  2. After dismissing and re-invoking the overlay (Cmd+K), it appears at the last dragged position rather than the default center
  3. After quitting and relaunching the app, the overlay appears at the default position (not the previously dragged position)
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 17 then 18.

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
| 17. Overlay Z-Order | v0.2.2 | 0/TBD | Not started | - |
| 18. Draggable Overlay Positioning | v0.2.2 | 0/TBD | Not started | - |
