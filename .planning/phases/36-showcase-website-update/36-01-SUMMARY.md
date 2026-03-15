---
phase: 36-showcase-website-update
plan: 01
subsystem: ui
tags: [html, css, javascript, showcase, downloads, os-detection]

# Dependency graph
requires:
  - phase: 35-ci-release-pipeline
    provides: Linux AppImage release artifacts and naming conventions
provides:
  - VERSION constant in main.js as single source of truth for all version strings
  - OS auto-detection highlighting visitor platform download button
  - Three-platform download buttons (macOS, Windows, Linux) with data-download attributes
  - Linux arch chooser popup (x86_64, aarch64)
  - Platform badges in hero section
  - GNOME Terminal in terminal carousel
affects: [36-02-privacy-policy]

# Tech tracking
tech-stack:
  added: []
  patterns: [data-attribute driven download URLs, navigator.userAgent OS detection, click-outside popup dismiss]

key-files:
  created: []
  modified:
    - showcase/js/main.js
    - showcase/index.html
    - showcase/css/home.css

key-decisions:
  - "data-download/data-version attributes for JS-driven URL and version population"
  - "navigator.userAgent with Android exclusion for OS detection"
  - "Lightweight CSS popup for Linux arch chooser (not modal)"

patterns-established:
  - "Version injection: single VERSION variable populates all download URLs and version badges via data attributes"
  - "Platform detection: detectOS() returns linux/macos/windows/null, adds platform-detected class"

requirements-completed: [WEB-01-VERSION, WEB-02-DOWNLOADS, WEB-03-CONTENT]

# Metrics
duration: 3min
completed: 2026-03-15
---

# Phase 36 Plan 01: Showcase Website Update Summary

**v0.3.9 version infrastructure with OS-aware three-platform download buttons, Linux arch chooser popup, and feature card/carousel updates reflecting Linux support**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-15T19:15:11Z
- **Completed:** 2026-03-15T19:18:22Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Centralized version string (v0.3.9) in main.js with data-attribute driven population across all pages
- Three-platform download buttons with OS auto-detection highlighting the visitor's platform
- Linux arch popup with x86_64 and aarch64 AppImage download options
- Platform badges (macOS, Windows, Linux) in hero section
- Feature cards updated to mention Linux alongside macOS and Windows
- GNOME Terminal added to terminal carousel (both original and aria-hidden duplicate sets)
- All 5 hardcoded v0.2.8 strings replaced with data-attribute-driven content

## Task Commits

Each task was committed atomically:

1. **Task 1: Version infrastructure and OS detection in main.js** - `7937c9d` (feat)
2. **Task 2: Update index.html with Linux downloads, badges, cards, and carousel** - `54cccbd` (feat)

## Files Created/Modified
- `showcase/js/main.js` - VERSION constant, URLS map, detectOS(), data-attribute population, OS highlight, arch popup toggle
- `showcase/index.html` - Three-platform download buttons, Linux arch popup, platform badges, updated feature cards, GNOME Terminal in carousel, data-version on settings and footer
- `showcase/css/home.css` - Platform badges, linux-download-wrapper, platform-detected highlight, arch popup styles, responsive platform badges

## Decisions Made
- Used data-download and data-version attributes for JS-driven URL/version population (enables single source of truth)
- navigator.userAgent with Android exclusion for OS detection (broadest compatibility)
- Lightweight CSS popup for Linux arch chooser rather than full modal (matches site's minimal design)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Version infrastructure in main.js works across both index.html and privacy.html via data attributes
- Privacy policy update (plan 02) can use data-version for its footer badge automatically
- All download URLs will auto-update when VERSION constant is changed for future releases

---
*Phase: 36-showcase-website-update*
*Completed: 2026-03-15*
