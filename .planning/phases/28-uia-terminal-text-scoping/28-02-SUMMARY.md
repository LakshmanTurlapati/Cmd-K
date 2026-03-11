---
phase: 28-uia-terminal-text-scoping
plan: 02
subsystem: terminal
tags: [uia, accessibility, tree-walk, xterm, vscode, scoping]

# Dependency graph
requires:
  - phase: 14-uia-reading
    provides: "UIA text reading infrastructure (uia_reader.rs)"
provides:
  - "Scoped UIA tree walk targeting ControlType::List elements for terminal-only text capture"
  - "looks_like_terminal_text heuristic for filtering terminal content from UI labels"
  - "3-strategy cascade: TextPattern -> scoped walk -> full tree fallback"
affects: [30-wsl-hardening]

# Tech tracking
tech-stack:
  added: []
  patterns: ["ControlType::List filtering for xterm.js accessibility tree", "IsOffscreen property check for inactive tab exclusion"]

key-files:
  created: []
  modified:
    - src-tauri/src/terminal/uia_reader.rs

key-decisions:
  - "ControlType::List identifies xterm.js terminal panels in VS Code UIA tree"
  - "IsOffscreen property filters inactive terminal tabs from text capture"
  - "looks_like_terminal_text heuristic prevents UI label noise from reaching detection pipeline"

patterns-established:
  - "3-strategy UIA text cascade: TextPattern (native terminals) -> scoped walk (IDE terminals) -> full tree walk (fallback)"

requirements-completed: [UIAS-01]

# Metrics
duration: 12min
completed: 2026-03-11
---

# Phase 28 Plan 02: Scoped UIA Tree Walk Summary

**ControlType::List scoped UIA walk eliminates IDE chrome text from terminal capture in VS Code/Cursor**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-11T17:20:00Z
- **Completed:** 2026-03-11T17:32:00Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Added `try_scoped_terminal_walk` function that finds ControlType::List elements (xterm.js accessibility nodes) and reads only their children
- Added `looks_like_terminal_text` heuristic to distinguish terminal output from UI labels
- Updated `read_terminal_text_inner` to use 3-strategy cascade: TextPattern -> scoped walk -> full tree fallback
- Verified scoped walk captures terminal-only text in VS Code while Windows Terminal remains unaffected via Strategy 1

## Task Commits

Each task was committed atomically:

1. **Task 1: Add scoped terminal walk with List-element filtering** - `861f343` (feat)
2. **Task 2: Verify scoped UIA text reading in VS Code** - human-verify checkpoint (approved)

**Integration fix:** `53ac56c` (fix) - Cross-plan compilation errors resolved from parallel execution

## Files Created/Modified
- `src-tauri/src/terminal/uia_reader.rs` - Added try_scoped_terminal_walk, looks_like_terminal_text, and 3-strategy cascade in read_terminal_text_inner

## Decisions Made
- ControlType::List is the correct UIA element type for xterm.js terminal panels in VS Code
- IsOffscreen property reliably identifies inactive terminal tabs for exclusion
- Best-effort heuristic (looks_like_terminal_text) preferred over strict filtering -- fallback covers misses

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cross-plan compilation errors from parallel execution**
- **Found during:** Between Task 1 and Task 2
- **Issue:** Plans 28-01 and 28-02 both modified uia_reader.rs; parallel execution caused import conflicts
- **Fix:** Integration commit 53ac56c resolved the compilation errors
- **Files modified:** src-tauri/src/terminal/uia_reader.rs
- **Verification:** cargo check --lib passes
- **Committed in:** 53ac56c

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Expected from parallel plan execution. No scope creep.

## Issues Encountered
None beyond the cross-plan integration fix documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 28 complete -- UIA text reading now scoped to terminal panel elements
- Phase 29 (Active Tab Matching) can proceed independently
- Phase 30 (WSL Detection Hardening) benefits from both scoped text and multi-signal scoring

## Self-Check: PASSED

- FOUND: src-tauri/src/terminal/uia_reader.rs
- FOUND: 861f343 (Task 1 commit)
- FOUND: 53ac56c (integration fix commit)

---
*Phase: 28-uia-terminal-text-scoping*
*Completed: 2026-03-11*
