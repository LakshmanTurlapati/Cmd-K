---
phase: 10-ai-follow-up-context-per-window
plan: 01
subsystem: ai
tags: [zustand, rust, conversation-history, prompt-construction, per-window-context]

# Dependency graph
requires:
  - phase: 08-window-identification-history-storage
    provides: Per-window HistoryEntry storage in Rust AppState, get_window_history IPC
provides:
  - turnHistory reconstruction from windowHistory on overlay open
  - Configurable turnLimit state (default 7, range 5-50)
  - Conditional terminal context (first message only) in AI prompt builder
  - Removed hardcoded 14-message caps from both frontend and backend
  - MAX_HISTORY_PER_WINDOW increased to 50 to match turn limit slider range
affects: [10-02 preferences UI for turnLimit slider, future AI context phases]

# Tech tracking
tech-stack:
  added: []
  patterns: [turnHistory reconstruction from HistoryEntry on overlay open, is_follow_up conditional prompt construction]

key-files:
  created: []
  modified:
    - src/store/index.ts
    - src-tauri/src/state.rs
    - src-tauri/src/commands/ai.rs

key-decisions:
  - "Reconstruct turnHistory from windowHistory entries on overlay open rather than storing separate turn data"
  - "Filter out is_error entries and empty responses when reconstructing turnHistory"
  - "Follow-up messages omit all terminal context (CWD, shell, output); system prompt provides shell type"
  - "Frontend pre-caps history via turnLimit; Rust no longer applies its own cap"
  - "MAX_HISTORY_PER_WINDOW increased from 7 to 50 to support full turn limit slider range"

patterns-established:
  - "turnHistory reconstruction: flatMap HistoryEntry[] to TurnMessage[] filtering errors, then trim to turnLimit * 2"
  - "is_follow_up detection: history.is_empty() determines whether to include terminal context in user message"

requirements-completed: [CTXT-01, CTXT-02, CTXT-03]

# Metrics
duration: 3min
completed: 2026-03-01
---

# Phase 10 Plan 01: AI Follow-up Context Per Window Summary

**Per-window turnHistory reconstruction from windowHistory on overlay open, conditional terminal context in AI prompts for follow-ups, configurable turnLimit replacing hardcoded 14-message cap**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-01T20:39:45Z
- **Completed:** 2026-03-01T20:42:18Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- turnHistory persists across overlay open/close cycles per terminal window -- AI sees prior exchanges on reopen
- Follow-up messages to AI omit terminal context (CWD, shell, output), reducing token usage on subsequent queries
- Configurable turnLimit (default 7) replaces all hardcoded 14-message caps in both TypeScript and Rust
- MAX_HISTORY_PER_WINDOW increased from 7 to 50 so Rust storage supports the full turn limit slider range

## Task Commits

Each task was committed atomically:

1. **Task 1: Zustand store -- turnHistory reconstruction, turnLimit state, remove reset** - `2f1ccf5` (feat)
2. **Task 2: Rust AI prompt -- conditional terminal context for follow-ups** - `8f19678` (feat)

## Files Created/Modified
- `src/store/index.ts` - Added turnLimit state, setTurnLimit/setTurnHistory actions, removed turnHistory reset from show(), added reconstruction from windowHistory, replaced hardcoded 14-message cap
- `src-tauri/src/state.rs` - Increased MAX_HISTORY_PER_WINDOW from 7 to 50
- `src-tauri/src/commands/ai.rs` - Added is_follow_up parameter to build_user_message, early return for follow-ups, removed Rust-side 14-message cap

## Decisions Made
- Reconstruct turnHistory from windowHistory on overlay open (uses existing data, no new storage needed)
- Filter out is_error entries and empty responses to avoid sending broken context to AI
- Follow-ups get minimal messages: "Task: {query}" for terminal mode, raw query for assistant mode
- Removed Rust-side history capping entirely -- frontend sends pre-capped array via turnLimit
- Increased MAX_HISTORY_PER_WINDOW to 50 to match the configurable slider max (negligible memory impact)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Core conversation persistence is wired -- AI sees prior exchanges on overlay reopen
- Plan 02 (preferences UI) can now add the turn limit slider and clear history button
- turnLimit defaults to 7 until the preferences slider is added in Plan 02

## Self-Check: PASSED

All files verified present. All commits verified in git log.

---
*Phase: 10-ai-follow-up-context-per-window*
*Completed: 2026-03-01*
