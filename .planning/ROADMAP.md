# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (in progress)

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

### v0.1.1 Command History & Follow-ups (In Progress)

**Milestone Goal:** Per-terminal-window command history with arrow key navigation and AI follow-up context that persists across overlay open/close cycles

- [ ] **Phase 8: Window Identification & History Storage** - Stable per-terminal-window key and Rust-side per-window history map
- [ ] **Phase 9: Arrow Key History Navigation** - Arrow up/down recall of previous queries with draft preservation
- [ ] **Phase 10: AI Follow-up Context Per Window** - Per-window AI conversation history that survives overlay cycles

## Phase Details

### Phase 8: Window Identification & History Storage
**Goal**: Every overlay invocation knows which terminal window triggered it, and per-window history survives across overlay open/close cycles
**Depends on**: Phase 7.1 (v0.1.0 complete)
**Requirements**: WKEY-01, WKEY-02, WKEY-03, HIST-04
**Success Criteria** (what must be TRUE):
  1. When user presses Cmd+K from iTerm2 window A then from iTerm2 window B, the app reports different window keys for each
  2. When user presses Cmd+K from a non-terminal app (e.g., Finder), the app falls back to a global key without errors
  3. The window key is available to the frontend before the user can type anything in the overlay
  4. Queries submitted from a terminal window are retrievable from the per-window history map after the overlay is dismissed and reopened
  5. History is capped at 7 entries per window -- the 8th query evicts the oldest
**Plans**: 2 plans

Plans:
- [ ] 08-01-PLAN.md -- Rust backend: AppState extension, window key computation, VS Code detection, history IPC commands
- [ ] 08-02-PLAN.md -- Frontend: Zustand store integration with window key and history IPC

### Phase 9: Arrow Key History Navigation
**Goal**: Users can navigate their per-window query history using arrow keys, just like shell history
**Depends on**: Phase 8
**Requirements**: HIST-01, HIST-02, HIST-03
**Success Criteria** (what must be TRUE):
  1. User can press Arrow-Up in an empty overlay input to recall the most recent query for the active terminal window
  2. User can press Arrow-Down after navigating up to move forward through history, and pressing Arrow-Down past the newest entry restores the current draft
  3. If the user types partial text and then presses Arrow-Up, the draft is preserved and restored when they Arrow-Down back past the end of history
  4. Arrow-Up and Arrow-Down do not interfere with cursor movement in multi-line input (Shift+Enter prompts)
**Plans**: TBD

Plans:
- [ ] 09-01: TBD

### Phase 10: AI Follow-up Context Per Window
**Goal**: AI can do follow-up responses because it sees the full conversation history for the active terminal window
**Depends on**: Phase 8
**Requirements**: CTXT-01, CTXT-02, CTXT-03
**Success Criteria** (what must be TRUE):
  1. User submits a query, dismisses overlay, reopens overlay on the same terminal window, and submits a follow-up -- the AI response demonstrates awareness of the prior exchange
  2. User submits a query on terminal window A, switches to terminal window B and submits a different query, then returns to window A -- the AI context is window A's history, not window B's
  3. Terminal context (CWD, shell, recent output) appears only in the first message of a window's session -- follow-up messages do not repeat terminal context
**Plans**: TBD

Plans:
- [ ] 10-01: TBD

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
| 8. Window Identification & History Storage | v0.1.1 | 0/2 | Planning complete | - |
| 9. Arrow Key History Navigation | v0.1.1 | 0/? | Not started | - |
| 10. AI Follow-up Context Per Window | v0.1.1 | 0/? | Not started | - |
