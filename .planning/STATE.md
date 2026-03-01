# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** v0.1.1 Phase 8 -- Window Identification & History Storage

## Current Position

Phase: 8 of 10 (Window Identification & History Storage) -- COMPLETE
Plan: 3 of 3 complete
Status: Phase 8 complete (including gap closure), ready for Phase 9
Last activity: 2026-03-01 -- Completed 08-03 (multi-tab IDE shell PID resolution gap closure)

Progress: [=========================---] 80% (24/30 est. plans across all milestones)

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

### Pending Todos

None yet.

### Blockers/Concerns

- 12 non-critical tech debt items from v0.1.0 (see milestones/v1.0-MILESTONE-AUDIT.md)
- Existing turnHistory in Zustand store resets on each overlay open -- Phase 10 addresses this
- GPU terminals (Alacritty, kitty, WezTerm) may not expose shell PID -- needs fallback testing in Phase 8

## Session Continuity

Last session: 2026-03-01 (Phase 8 Plan 03 execution)
Stopped at: Completed 08-03-PLAN.md -- Phase 8 gap closure complete
Resume file: None
