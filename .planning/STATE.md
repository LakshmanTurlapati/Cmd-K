# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-28)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** v0.1.1 Phase 9 -- Arrow Key History Navigation

## Current Position

Phase: 9 of 10 (Arrow Key History Navigation) -- COMPLETE
Plan: 1 of 1 complete
Status: Phase 9 complete, ready for Phase 10
Last activity: 2026-03-01 -- Completed 09-01 (arrow-key history navigation)

Progress: [==========================--] 83% (25/30 est. plans across all milestones)

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

### Pending Todos

None yet.

### Blockers/Concerns

- 12 non-critical tech debt items from v0.1.0 (see milestones/v1.0-MILESTONE-AUDIT.md)
- Existing turnHistory in Zustand store resets on each overlay open -- Phase 10 addresses this
- GPU terminals (Alacritty, kitty, WezTerm) may not expose shell PID -- needs fallback testing in Phase 8

## Session Continuity

Last session: 2026-03-01 (Phase 9 Plan 01 execution)
Stopped at: Completed 09-01-PLAN.md -- Phase 9 arrow-key history navigation complete
Resume file: None
