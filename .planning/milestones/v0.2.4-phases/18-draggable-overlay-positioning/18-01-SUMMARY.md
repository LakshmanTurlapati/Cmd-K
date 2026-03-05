---
phase: 18-draggable-overlay-positioning
plan: 01
subsystem: ui
tags: [tauri, react, drag, overlay, window-positioning, hooks]

# Dependency graph
requires:
  - phase: 17-overlay-z-order
    provides: "NSPanel at Floating level that drag positioning interacts with"
provides:
  - "useDrag React hook for drag-to-move Tauri windows"
  - "In-memory session-scoped overlay position state (AppState.last_position)"
  - "set_overlay_position IPC command for persisting drag position to Rust"
  - "Position-aware show_overlay that reopens at last dragged location"
affects: [overlay-positioning, window-management]

# Tech tracking
tech-stack:
  added: []
  patterns: ["useDrag hook with screen-coordinate delta tracking", "session-scoped in-memory Mutex state for transient preferences"]

key-files:
  created:
    - src/hooks/useDrag.ts
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/window.rs
    - src-tauri/src/lib.rs
    - src/App.tsx

key-decisions:
  - "In-memory Mutex<Option<(f64, f64)>> for position -- no disk persistence, resets on relaunch"
  - "Screen coordinates (screenX/Y) for drag deltas -- window moves during drag making clientX/Y unreliable"
  - "2px dead zone before persisting position -- prevents accidental position changes from clicks"

patterns-established:
  - "useDrag pattern: mousedown on element, mousemove/mouseup on window for smooth cross-boundary dragging"
  - "Interactive element exclusion via target.closest() selector to prevent drag on inputs/buttons"
  - "Physical-to-logical coordinate conversion using outerPosition()/scaleFactor() for Retina displays"

requirements-completed: [OPOS-01, OPOS-02, OPOS-03]

# Metrics
duration: 12min
completed: 2026-03-03
---

# Phase 18 Plan 01: Draggable Overlay Positioning Summary

**Drag-to-reposition overlay using useDrag hook with in-memory session-scoped position persistence via Rust AppState**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-03T01:16:57Z
- **Completed:** 2026-03-03T01:33:40Z
- **Tasks:** 3 (2 auto + 1 human-verify checkpoint)
- **Files modified:** 5

## Accomplishments
- User can click and drag the overlay to any position on screen with smooth real-time movement
- Dismissed overlay reopens at last dragged position within the same app session
- Position resets to default centered location on app relaunch (no disk persistence)
- Interactive elements (input, textarea, buttons) are excluded from drag initiation
- Grab/grabbing cursor affordance provides visual drag feedback

## Task Commits

Each task was committed atomically:

1. **Task 1: Add in-memory position state and position-aware show_overlay** - `9983332` (feat)
2. **Task 2: Add useDrag hook and wire into App.tsx** - `df2797f` (feat)
3. **Task 3: Verify drag-to-reposition and position memory** - checkpoint:human-verify (approved)

## Files Created/Modified
- `src/hooks/useDrag.ts` - React hook enabling click-and-drag window repositioning with screen-coordinate tracking
- `src-tauri/src/state.rs` - Added `last_position: Mutex<Option<(f64, f64)>>` to AppState for session-scoped position memory
- `src-tauri/src/commands/window.rs` - Added `set_overlay_position` IPC command and position_overlay now checks last_position first
- `src-tauri/src/lib.rs` - Registered `set_overlay_position` in invoke handler
- `src/App.tsx` - Wired useDrag hook with panelRef, added cursor-grab/cursor-grabbing classes

## Decisions Made
- Used in-memory `Mutex<Option<(f64, f64)>>` for position state -- no disk persistence means position naturally resets on relaunch (satisfies OPOS-03)
- Used `e.screenX/Y` for drag deltas instead of `clientX/Y` -- the window moves during drag, making client coordinates unreliable
- Added 2px dead zone before persisting position -- prevents accidental position changes from simple clicks
- Attached mousemove/mouseup listeners to `window` (not element) -- ensures drag continues smoothly even if cursor leaves the panel

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 18 is the final phase of milestone v0.2.2 (Overlay UX Fixes)
- All success criteria met: drag works, position memory works within session, resets on relaunch
- No blockers or concerns

## Self-Check: PASSED

- All 5 source files verified present on disk
- Commit `9983332` (Task 1) verified in git log
- Commit `df2797f` (Task 2) verified in git log
- Task 3 checkpoint approved by user

---
*Phase: 18-draggable-overlay-positioning*
*Completed: 2026-03-03*
