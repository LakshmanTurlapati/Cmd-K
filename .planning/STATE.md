# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** v0.1.1 Phase 10 -- AI Follow-up Context Per Window

## Current Position

Phase: 10 of 10 (AI Follow-up Context Per Window)
Plan: 1 of 2 complete
Status: Phase 10 Plan 01 complete, ready for Plan 02
Last activity: 2026-03-01 -- Completed 10-01 (per-window turnHistory reconstruction + conditional AI context)

Progress: [============================-] 87% (26/30 est. plans across all milestones)

## Performance Metrics

**v0.1.0 Summary:**
- Total phases: 8
- Total plans: 21
- Timeline: 8 days (2026-02-21 to 2026-02-28)
- Codebase: 4,042 LOC Rust + 2,868 LOC TypeScript

**v0.1.1:**
- Total phases: 3 (Phases 8-10)
- Total plans: TBD (pending phase planning)

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 08-01 | Window key + history backend | 5min | 2 | 7 |
| 08-02 | Frontend IPC integration | 2min | 2 | 1 |
| 08-03 | Multi-tab IDE shell PID resolution | 4min | 2 | 4 |
| 09-01 | Arrow-key history navigation | 3min | 2 | 3 |
| 10-01 | Per-window turnHistory + conditional AI context | 3min | 2 | 3 |

## Accumulated Context

### Decisions

All v0.1.0 decisions archived in PROJECT.md Key Decisions table.

v0.1.1 decisions:
- Use bundle_id:shell_pid as window key (simpler than CGWindowID, no screen recording permission risk)
- Store per-window history in Rust AppState HashMap (not Zustand -- show() resets React state)
- Session-scoped only, no disk persistence (privacy, simplicity)
- Window key format: bundle_id:shell_pid for terminals/IDEs, bundle_id:app_pid for other apps
- History IPC uses individual parameters (not struct) so frontend does not need to generate timestamps
- History capped at 7 entries/window, 50 windows total with oldest-window eviction
- Window key + history fetch placed before get_app_context in show() (key is fast, already computed by hotkey handler)
- add_history_entry uses fire-and-forget pattern to avoid blocking UI flow
- Error queries persisted with isError: true for arrow-key recall in Phase 9
- Multi-tab IDE shell disambiguation uses AX-derived focused tab CWD matched against candidate shell CWDs
- CWD extraction from AXTitle first (more reliable), AXValue last line as fallback
- 0.3s AX messaging timeout for CWD extraction in hotkey handler critical path
- text-white/60 for dimmed recalled history text, consistent with existing opacity patterns
- History navigation index as local component state (not Zustand) -- resets on overlay close/open
- Arrow-Down always navigates history (asymmetric with Arrow-Up first-line check)
- Local windowHistory sync after fire-and-forget invoke avoids IPC re-fetch overhead
- Reconstruct turnHistory from windowHistory on overlay open (no separate storage needed)
- Filter out is_error entries and empty responses when reconstructing turnHistory
- Follow-up AI messages omit terminal context entirely; system prompt has shell type
- Frontend pre-caps history via turnLimit; Rust-side cap removed
- MAX_HISTORY_PER_WINDOW increased from 7 to 50 to match turn limit slider range

### Pending Todos

None yet.

### Blockers/Concerns

- 12 non-critical tech debt items from v0.1.0 (see milestones/v1.0-MILESTONE-AUDIT.md)
- (RESOLVED by 10-01) turnHistory no longer resets on overlay open -- reconstructed from windowHistory
- GPU terminals (Alacritty, kitty, WezTerm) may not expose shell PID -- needs fallback testing in Phase 8

## Session Continuity

Last session: 2026-03-01 (Phase 10 Plan 01 execution)
Stopped at: Completed 10-01-PLAN.md -- Per-window turnHistory reconstruction + conditional AI context
Resume file: None
