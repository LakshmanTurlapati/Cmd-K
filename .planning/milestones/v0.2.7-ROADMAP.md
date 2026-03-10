# Roadmap: CMD+K

## Milestones

- v0.1.0 MVP -- Phases 1-7.1 (shipped 2026-02-28) | [Archive](milestones/v1.0-ROADMAP.md)
- v0.1.1 Command History & Follow-ups -- Phases 8-10 (shipped 2026-03-01) | [Archive](milestones/v0.1.1-ROADMAP.md)
- v0.2.1 Windows Support -- Phases 11-16, 01-merge (shipped 2026-03-03) | [Archive](milestones/v0.2.1-ROADMAP.md)
- v0.2.4 Overlay UX, Safety & CI/CD -- Phases 17-20 (shipped 2026-03-04) | [Archive](milestones/v0.2.4-ROADMAP.md)
- v0.2.6 Multi-Provider, WSL & Auto-Update -- Phases 21-24 (shipped 2026-03-09) | [Archive](milestones/v0.2.6-ROADMAP.md)
- v0.2.7 Cost Estimation -- Phases 25-26 (in progress)

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

### v0.2.7 Cost Estimation (In Progress)

- [x] **Phase 25: Token Tracking & Pricing Backend** - Extract token usage from all 3 streaming adapters, accumulate in session state, bundled + dynamic pricing data (completed 2026-03-10)
- [x] **Phase 26: Cost Display Frontend** - Replace placeholder with live cost display, token breakdown, reset button (completed 2026-03-10)

## Phase Details

### Phase 25: Token Tracking & Pricing Backend
**Goal**: Every AI query records input/output token counts and the app can calculate estimated cost using per-model pricing
**Depends on**: Phase 24 (v0.2.6 shipped baseline)
**Requirements**: TRAK-01, TRAK-02, TRAK-03, TRAK-04, PRIC-01, PRIC-02, PRIC-03
**Success Criteria** (what must be TRUE):
  1. After a streaming query completes via any provider, the app has recorded the input and output token counts for that request
  2. Token counts accumulate across multiple queries in the same session, grouped by provider+model
  3. An IPC command returns accumulated usage stats with estimated cost calculated from per-model pricing
  4. OpenRouter model pricing is fetched from their API; curated model pricing is bundled in the binary
  5. Models without known pricing return token counts with a "pricing unavailable" indicator

Plans:
- [ ] 25-01-PLAN.md — Token extraction from all 3 streaming adapters (OpenAI-compat `stream_options.include_usage`, Anthropic `message_start`/`message_delta` usage events, Gemini `usageMetadata`), UsageState accumulator, pricing data module
- [ ] 25-02-PLAN.md — IPC command `get_usage_stats`, OpenRouter dynamic pricing fetch, cost calculation logic, reset command

### Phase 26: Cost Display Frontend
**Goal**: Users can see their session's AI usage cost in the Settings Model tab with token breakdown and reset capability
**Depends on**: Phase 25 (backend IPC must be stable)
**Requirements**: DISP-01, DISP-02, DISP-03, DISP-04
**Success Criteria** (what must be TRUE):
  1. Settings Model tab shows estimated session cost in place of the old placeholder text
  2. User can see input tokens, output tokens, and total estimated cost
  3. Cost display updates automatically after each AI query completes
  4. User can click a reset button to clear session usage stats

Plans:
- [ ] 26-01-PLAN.md — Replace ModelTab.tsx placeholder with live cost display, IPC integration, auto-refresh after query, reset button

## Progress

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1-7.1 | v0.1.0 | 21/21 | Complete | 2026-02-28 |
| 8-10 | v0.1.1 | 6/6 | Complete | 2026-03-01 |
| 11-16, 01 | v0.2.1 | 7/7 | Complete | 2026-03-03 |
| 17-20 | v0.2.4 | 5/5 | Complete | 2026-03-04 |
| 21-24 | v0.2.6 | 10/10 | Complete | 2026-03-09 |
| 25 | 2/2 | Complete    | 2026-03-10 | -- |
| 26 | 1/1 | Complete    | 2026-03-10 | -- |
