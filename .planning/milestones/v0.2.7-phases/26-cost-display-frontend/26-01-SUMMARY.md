---
phase: 26-cost-display-frontend
plan: 01
subsystem: ui
tags: [tauri, react, ipc, sparkline, cost-tracking, tailwind]

requires:
  - phase: 25-token-tracking-and-pricing-backend
    provides: "UsageAccumulator, get_usage_stats/reset_usage IPC, curated+OpenRouter pricing"
provides:
  - "Per-query cost history (QueryRecord + query_costs) in backend"
  - "Live cost display with token breakdown in ModelTab"
  - "Greyscale angular sparkline bar chart for per-query costs"
  - "Reset button to clear session stats"
affects: [cost-display, usage-tracking]

tech-stack:
  added: []
  patterns: ["Per-query metadata stored at record time, costs calculated at read time"]

key-files:
  created: []
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/usage.rs
    - src/components/Settings/ModelTab.tsx

key-decisions:
  - "QueryRecord stores raw tokens per query; cost calculated at read time using pricing data in usage.rs"
  - "Sparkline uses div-based bars with flex layout, angular (no border-radius)"

patterns-established:
  - "Per-query history pattern: store metadata at record(), compute derived values at read time"

requirements-completed: [DISP-01, DISP-02, DISP-03, DISP-04]

duration: 5min
completed: 2026-03-10
---

# Phase 26 Plan 01: Cost Display Frontend Summary

**Live session cost display with token breakdown, per-query sparkline bar chart, and reset capability in ModelTab**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-10T10:12:49Z
- **Completed:** 2026-03-10T10:18:34Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added per-query cost tracking to backend with QueryRecord struct and query_history in UsageAccumulator
- Replaced static placeholder in ModelTab with live cost display showing formatted cost and token counts
- Built greyscale angular sparkline bar chart showing per-query cost history
- Added inline Reset button that clears session stats and refreshes display
- Handled no-pricing state with dash, tooltip, and partial-pricing asterisk note

## Task Commits

Each task was committed atomically:

1. **Task 1: Add per-query cost history to backend** - `986146c` (feat)
2. **Task 2: Replace ModelTab placeholder with live cost display** - `7705821` (feat)

## Files Created/Modified
- `src-tauri/src/state.rs` - Added QueryRecord struct, query_history Vec, push in record(), clear in reset(), getter method
- `src-tauri/src/commands/usage.rs` - Added query_costs to UsageStatsResponse, compute per-query costs from history using pricing lookup
- `src/components/Settings/ModelTab.tsx` - Live cost display with formatted cost, token breakdown, sparkline, reset button, no-pricing handling

## Decisions Made
- QueryRecord stores raw provider/model/tokens per query; cost calculation happens at read time in usage.rs where pricing data is available -- keeps cost logic centralized
- Sparkline uses div-based bars with flex layout (not SVG/canvas) for simplicity and consistency with existing Tailwind patterns
- Used IIFE pattern in JSX for complex rendering logic to keep component structure clean

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Cost display is fully functional and ready for visual verification
- Backend per-query tracking provides foundation for any future per-model breakdown views

---
*Phase: 26-cost-display-frontend*
*Completed: 2026-03-10*
