---
phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding
plan: 01
subsystem: ui
tags: [macos, accessibility, ax-api, tauri, rust, onboarding, ffi, core-foundation-sys]

requires:
  - phase: 02-settings-configuration
    provides: "check_accessibility_permission Tauri command, OnboardingWizard with step logic, App.tsx startup onboarding check"

provides:
  - "ax_probe_self() Rust function: live AX API probe on own PID, returns true unless kAXErrorNotTrusted"
  - "Dual-check check_accessibility_permission: AXIsProcessTrusted fast path + ax_probe_self fallback"
  - "Reordered OnboardingWizard steps: API Key (0) -> Model (1) -> Accessibility (2) -> Done (3)"
  - "Fixed effectiveStep logic in App.tsx: uses onboardingStep <= 0 for API-key-exists resume path"

affects: [onboarding, accessibility, permissions, overlay-badge]

tech-stack:
  added: []
  patterns:
    - "AX probe fallback: treat kAXErrorNotTrusted (-25211) as the ONLY definitive no-permission code; all other AX error codes mean OS allowed the call and permission is granted"
    - "Dual-check pattern: AXIsProcessTrusted fast path -> ax_probe_self fallback; all frontend call sites updated transparently via IPC"

key-files:
  created: []
  modified:
    - src-tauri/src/commands/permissions.rs
    - src/components/Onboarding/OnboardingWizard.tsx
    - src/App.tsx

key-decisions:
  - "ax_probe_self uses own PID (std::process::id()) not a target app PID -- fast, avoids cross-process AX tree walk"
  - "kAXErrorNotTrusted (-25211) is the only false return; -25205 (attr unsupported), -25204 (no value), -25212 (cannot complete), 0 (success) all return true because OS allowed the call"
  - "In-place replacement of check_accessibility_permission function body -- no call site changes needed anywhere in frontend"
  - "effectiveStep changed from onboardingStep <= 1 to <= 0 because API Key moved from step 1 to step 0"

patterns-established:
  - "AX probe pattern: declare AXUIElementCreateApplication + AXUIElementCopyAttributeValue + CFRelease in local extern C block; probe own PID; return err != -25211"

requirements-completed: [SETT-04]

duration: 3min
completed: 2026-02-26
---

# Phase 7 Plan 01: Fix Accessibility Permission Detection and Reorder Onboarding Summary

**AX probe fallback in Rust permissions.rs (dual-check: AXIsProcessTrusted fast path + own-PID AX call) plus onboarding reorder to API Key -> Model -> Accessibility -> Done with corrected effectiveStep resume logic**

## Performance

- **Duration:** 3 min
- **Started:** 2026-02-26T17:41:17Z
- **Completed:** 2026-02-26T17:44:27Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments

- Added ax_probe_self() private Rust function that probes actual AX API access on the process's own PID, returning false only for kAXErrorNotTrusted (-25211) and true for all other codes
- Updated check_accessibility_permission() with dual-check: AXIsProcessTrusted() fast path, falling back to ax_probe_self() when the flag returns false -- fixes unsigned build false negatives
- Reordered OnboardingWizard.tsx steps from Accessibility/API Key/Model/Done to API Key/Model/Accessibility/Done with matching stepLabels update
- Fixed App.tsx effectiveStep logic from onboardingStep <= 1 to <= 0, correctly advancing users with existing API keys past the new step 0 to step 1 (Model)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AX probe fallback to check_accessibility_permission in Rust** - `ec20f83` (feat)
2. **Task 2: Reorder onboarding steps and fix effectiveStep logic** - `52a2e62` (feat)

## Files Created/Modified

- `src-tauri/src/commands/permissions.rs` - Added ax_probe_self() function + updated check_accessibility_permission() with dual-check fallback
- `src/components/Onboarding/OnboardingWizard.tsx` - Updated stepLabels array and step content rendering order
- `src/App.tsx` - Updated effectiveStep calculation from <= 1 to <= 0

## Decisions Made

- Used own PID (std::process::id() as i32) as the AX probe target -- always valid, sub-millisecond, avoids cross-process AX tree walks
- Treat kAXErrorNotTrusted (-25211) as the only definitive "no permission" code per research pitfall analysis; all other codes including -25212 (kAXErrorCannotComplete) return true since Tauri WKWebView may not expose AXRole
- Declared CFRelease locally in the extern "C" block alongside AX functions to avoid type mismatch with core_foundation_sys::base::CFRelease
- In-place function replacement preserves the Tauri IPC name -- all frontend call sites (store/index.ts, Overlay.tsx, App.tsx) pick up the fix automatically with zero changes

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Dual-check is permanent: works correctly for both signed (AXIsProcessTrusted fast path) and unsigned builds (AX probe fallback)
- Onboarding reorder complete: users configure API key and model before being prompted for accessibility
- No further changes needed for this phase; accessibility badge polling and compact overlay badge (Phase 07 scope) can build on the fixed check_accessibility_permission command

## Self-Check: PASSED

- FOUND: src-tauri/src/commands/permissions.rs (ax_probe_self function + updated check_accessibility_permission)
- FOUND: src/components/Onboarding/OnboardingWizard.tsx (stepLabels reordered + step content render reordered)
- FOUND: src/App.tsx (effectiveStep uses onboardingStep <= 0)
- FOUND: 07-01-SUMMARY.md
- FOUND commit ec20f83 (Task 1 - AX probe fallback)
- FOUND commit 52a2e62 (Task 2 - onboarding reorder)
- cargo check: PASSED (no errors)
- npm run build: PASSED (1863 modules transformed, 0 errors)

---
*Phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding*
*Completed: 2026-02-26*
