---
gsd_state_version: 1.0
milestone: v0.2.8
milestone_name: Windows Terminal Detection Fix
status: active
stopped_at: null
last_updated: "2026-03-11"
last_activity: 2026-03-11 -- Roadmap created for v0.2.8
progress:
  total_phases: 4
  completed_phases: 0
  total_plans: 0
  completed_plans: 0
  percent: 0
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-11)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** Phase 27 - ConPTY Discovery & Process Snapshot

## Current Position

Phase: 27 of 30 (ConPTY Discovery & Process Snapshot)
Plan: 0 of TBD in current phase
Status: Ready to plan
Last activity: 2026-03-11 -- Roadmap created for v0.2.8 Windows Terminal Detection Fix

Progress: [░░░░░░░░░░] 0% (v0.2.8)

## Performance Metrics

**All Milestones:**
- v0.1.0: 8 phases, 21 plans, 8 days
- v0.1.1: 3 phases, 6 plans, 2 days
- v0.2.1: 7 phases, 11 plans, 3 days
- v0.2.4: 4 phases, 5 plans, 2 days
- v0.2.6: 5 phases, 10 plans, 1 day
- v0.2.7: 2 phases, 3 plans, 1 day
- Cumulative: 29 phases, 56 plans, 17 days

## Accumulated Context

### Decisions

All decisions archived in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- [v0.2.8]: ConPTY parentage replaces highest-PID heuristic for shell discovery
- [v0.2.8]: No new crates -- uses existing windows-sys 0.59 + uiautomation 0.24
- [v0.2.8]: Never add wsl.exe to KNOWN_SHELL_EXES (caused prior revert)
- [v0.2.8]: WSL 2 Linux processes invisible to Win32 APIs -- output signals only

### Pending Todos

None.

### Blockers/Concerns

- UIA tree structure for VS Code terminal panel needs empirical verification (Phase 28)
- Cursor IDE assumed same as VS Code fork -- needs testing (Phase 27)
- Windows 10 vs 11 ConPTY differences may exist (conhost.exe vs OpenConsole.exe)

## Session Continuity

Last session: 2026-03-11
Stopped at: Roadmap created for v0.2.8 milestone
Next action: Plan Phase 27
