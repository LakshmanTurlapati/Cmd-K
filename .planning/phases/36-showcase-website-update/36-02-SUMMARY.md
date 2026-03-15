---
phase: 36-showcase-website-update
plan: 02
subsystem: ui
tags: [privacy-policy, html, linux, version-history]

requires:
  - phase: 36-showcase-website-update
    provides: Context for showcase website updates
provides:
  - Updated privacy policy with Linux platform details
  - Collapsible version history preserving March 9 policy verbatim
  - Footer version badge with data-version attribute for JS population
affects: [showcase-website]

tech-stack:
  added: []
  patterns: [collapsible-policy-history, data-version-attribute]

key-files:
  created: []
  modified: [showcase/privacy.html]

key-decisions:
  - "Preserved full March 9 policy verbatim in collapsible details element"
  - "Used data-version attribute on footer badge for JS-driven version display"

patterns-established:
  - "Policy version history: each prior version in a details.policy-version block with full text"

requirements-completed: [WEB-04-PRIVACY]

duration: 2min
completed: 2026-03-15
---

# Phase 36 Plan 02: Privacy Policy Update Summary

**Privacy policy updated with Linux-specific data handling (/proc, xdotool, AT-SPI2, libsecret), 12% terminal text budget mention, and March 9 policy preserved in collapsible version history**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-15T19:14:57Z
- **Completed:** 2026-03-15T19:16:38Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Added Linux-specific permissions section covering /proc, xdotool, AT-SPI2, kitty, and WezTerm APIs
- Updated credential store references to include Linux system keyring via libsecret across all relevant sections
- Added terminal text budget bullet (12% of context window) to Terminal Mode data list
- Preserved entire March 9 2026 policy verbatim in collapsible version history entry
- Updated footer version badge from hardcoded v0.2.8 to data-version attribute with v0.3.9 fallback

## Task Commits

Each task was committed atomically:

1. **Task 1: Update privacy policy content for v0.3.9 with Linux details** - `2b944dc` (feat)

**Plan metadata:** pending (docs: complete plan)

## Files Created/Modified
- `showcase/privacy.html` - Privacy policy with Linux platform details, version history, and data-version footer badge

## Decisions Made
- Preserved full March 9 policy text verbatim (not summarized) in collapsible section, matching plan requirement
- Used data-version attribute on footer span for JS population, with v0.3.9 as fallback text

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Privacy policy is complete for v0.3.9 with all three platforms documented
- Footer version badge ready for JS population via data-version attribute from main.js
- Policy history pattern established for future version updates

---
*Phase: 36-showcase-website-update*
*Completed: 2026-03-15*

## Self-Check: PASSED
