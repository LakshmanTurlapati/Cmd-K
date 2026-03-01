---
phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding
plan: 02
subsystem: ui
tags: [macos, accessibility, tauri, react, radix-ui, polling, overlay, onboarding]

requires:
  - phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding
    provides: "Dual-check check_accessibility_permission Rust command (AXIsProcessTrusted + ax_probe_self fallback); reordered onboarding steps"

provides:
  - "StepAccessibility auto-polling: 1.5s interval starts after 'Open System Settings' click, detects grant without 'Check Again'"
  - "Overlay compact accessibility badge: ShieldAlert icon + 'No AX access' text with Radix Tooltip replacing multi-line banner"
  - "Overlay background polling: 5s interval auto-hides accessibility badge when permission granted without restart"

affects: [onboarding, accessibility, overlay-badge]

tech-stack:
  added: []
  patterns:
    - "Auto-polling on user action: setPollingActive(true) in handleOpenSettings triggers useEffect interval that clears on grant"
    - "mountedRef cleanup guard: prevents state updates on unmounted components in async polling intervals"
    - "Compact badge with tooltip pattern: Radix Tooltip.Provider/Root/Trigger/Content replaces multi-line warning block"
    - "Background interval for reactive UI: 5s polling clears itself via useEffect dependency on accessibilityGranted"

key-files:
  created: []
  modified:
    - src/components/Onboarding/StepAccessibility.tsx
    - src/components/Overlay.tsx

key-decisions:
  - "pollingActive bool state gate: polling useEffect returns early when pollingActive=false, starts only after explicit user action (Open System Settings)"
  - "mountedRef for safe async updates: ref persists across renders without triggering re-renders; cleared on component unmount"
  - "5s overlay polling interval (vs 1.5s onboarding): less aggressive for background badge scenario; still fast enough for UX"
  - "Badge click opens System Settings directly: onClick={() => invoke('open_accessibility_settings')} on the Tooltip.Trigger button"

patterns-established:
  - "Polling with cleanup: useEffect returns clearInterval, dependency array on the gate state; safe for async invoke calls"
  - "Tooltip.Provider wrapping badge: delayDuration=300 for hover responsiveness; Portal for z-index correctness in overlay context"

requirements-completed: [SETT-04]

duration: 2min
completed: 2026-02-26
---

# Phase 7 Plan 02: Add Auto-Polling and Compact Accessibility Badge Summary

**1.5s auto-polling in StepAccessibility for instant grant detection plus compact Radix Tooltip badge replacing the multi-line accessibility banner in Overlay with 5s background polling for auto-dismiss**

## Performance

- **Duration:** 2 min
- **Started:** 2026-02-26T17:49:05Z
- **Completed:** 2026-02-26T17:51:04Z
- **Tasks:** 1 (of 2; Task 2 is human-verify checkpoint)
- **Files modified:** 2

## Accomplishments

- Added pollingActive state + mountedRef to StepAccessibility; polling starts after "Open System Settings" click (1.5s interval), detects grant silently, shows green checkmark 1s, then auto-advances to Done step
- Replaced multi-line accessibility banner in Overlay.tsx with compact amber "No AX access" badge using ShieldAlert icon and Radix Tooltip explaining limitation and click-to-open-settings
- Added 5s background polling useEffect in Overlay that clears when accessibilityGranted becomes true, auto-hiding the badge without requiring restart

## Task Commits

Each task was committed atomically:

1. **Task 1: Add auto-polling to StepAccessibility and compact badge to Overlay** - `5187186` (feat)

**Plan metadata:** (pending checkpoint completion)

## Files Created/Modified

- `src/components/Onboarding/StepAccessibility.tsx` - Added pollingActive state, mountedRef, cleanup useEffect, polling useEffect (1.5s), updated handleOpenSettings to activate polling
- `src/components/Overlay.tsx` - Imported Tooltip from @radix-ui/react-tooltip and ShieldAlert from lucide-react; replaced multi-line banner with compact Tooltip badge; added 5s background polling useEffect

## Decisions Made

- pollingActive state gate prevents polling from starting until user explicitly clicks "Open System Settings" -- avoids unnecessary background calls on onboarding mount
- mountedRef (not state) used as cleanup guard to avoid stale closure issues and extra re-renders in async interval callbacks
- 5s interval in Overlay background polling vs 1.5s in onboarding -- less aggressive because badge scenario is ambient, not active user-waiting-for-result scenario
- Tooltip.Provider delayDuration=300ms for snappy tooltip appearance; Tooltip.Portal for correct z-index layering inside the overlay panel

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All Phase 7 automation tasks complete: dual-check Rust command (Plan 01), onboarding reorder (Plan 01), auto-polling in onboarding (Plan 02), compact overlay badge (Plan 02)
- Human verification checkpoint (Task 2) covers full end-to-end flow including onboarding order, auto-detect, compact badge, badge tooltip, and badge auto-disappear
- No further automation required -- all SETT-04 requirements addressed

## Self-Check: PASSED

- FOUND: src/components/Onboarding/StepAccessibility.tsx (pollingActive state + mountedRef + setInterval 1500ms)
- FOUND: src/components/Overlay.tsx (Tooltip.Provider + "No AX access" badge text + setInterval 5000ms)
- No "Accessibility permission not detected" multi-line banner text in Overlay.tsx
- npm run build: PASSED (1863 modules transformed, 0 errors)
- FOUND commit 5187186 (Task 1 - auto-polling + compact badge)

---
*Phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding*
*Completed: 2026-02-26*
