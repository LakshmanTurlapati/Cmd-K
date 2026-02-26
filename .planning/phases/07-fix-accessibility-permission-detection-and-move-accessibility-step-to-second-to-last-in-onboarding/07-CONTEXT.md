# Phase 7: Fix Accessibility Permission Detection and Move Accessibility Step to Second-to-Last in Onboarding - Context

**Gathered:** 2026-02-26
**Status:** Ready for planning

<domain>
## Phase Boundary

Fix the accessibility permission detection that always returns false in production builds (unsigned local builds via `cargo tauri build`), and reorder the onboarding wizard so the accessibility step is second-to-last (before Done). Currently: Accessibility -> API Key -> Model -> Done. New order: API Key -> Model -> Accessibility -> Done.

</domain>

<decisions>
## Implementation Decisions

### Detection Fix
- AXIsProcessTrusted/AXIsProcessTrustedWithOptions returns false in production builds even when the app is listed and toggled ON in System Settings > Accessibility
- Works correctly in dev mode (cargo tauri dev), fails in unsigned production builds
- Re-adding the app to the Accessibility list does not help
- **Fix approach:** Research and fix the root cause (likely unsigned build identity mismatch with macOS trust database) AND add a fallback detection method regardless
- **Fallback:** Attempt an actual AX API call as a silent probe -- if the AX call succeeds, permission is granted even if AXIsProcessTrusted reports false
- Fallback probe should be completely silent -- no UI indication that a fallback is being used

### Skip & Degraded UX
- Users CAN skip the accessibility step during onboarding
- When skipped, a compact warning badge appears in the overlay (not a full banner -- something concise like a small warning icon with brief text)
- Badge behavior: tooltip on hover explaining what's limited and how to fix, click opens System Settings directly
- Badge auto-disappears: periodically check permission status in the background and hide the badge as soon as accessibility is granted (no app restart needed)

### Re-check Flow (During Onboarding)
- After user clicks "Open System Settings", auto-poll every 1-2 seconds to detect when permission is granted
- Polling runs indefinitely while the accessibility step is visible (user may take time in System Settings)
- Polling uses dual approach: try fixed AXIsProcessTrusted first, if false try fallback probe
- On detection of grant: show green checkmark / "Granted!" for ~1 second, then auto-advance to next step

### Onboarding Reorder
- New step order: API Key (step 0) -> Model (step 1) -> Accessibility (step 2) -> Done (step 3)
- Step labels update accordingly

### Claude's Discretion
- Exact polling interval (1-2 second range)
- Specific fallback AX API call to use as probe
- Warning badge visual design (icon choice, text, positioning in overlay)
- Technical approach to fixing the unsigned build detection issue

</decisions>

<specifics>
## Specific Ideas

- Warning badge should be "super concise" -- not a banner, more like a small badge with a warning indicator
- Badge interaction: hover for tooltip, click to open System Settings (both behaviors)
- The dual detection (primary + fallback) should be used everywhere: onboarding polling, overlay badge auto-hide check, and initial permission check

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 07-fix-accessibility-permission-detection-and-move-accessibility-step-to-second-to-last-in-onboarding*
*Context gathered: 2026-02-26*
