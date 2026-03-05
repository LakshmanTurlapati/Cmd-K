---
phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management
plan: 01
subsystem: infra
tags: [ci-cd, github-actions, macos, code-signing, notarization, apple]

# Dependency graph
requires: []
provides:
  - CI-compatible build-dmg.sh with env var parameterization
  - Step-by-step credential migration guide for GitHub Secrets
affects: [20-02 release workflow]

# Tech tracking
tech-stack:
  added: []
  patterns: [env-var-with-fallback, ci-local-branching]

key-files:
  created:
    - docs/ci-secrets-setup.md
  modified:
    - scripts/build-dmg.sh

key-decisions:
  - "grep+sed for version extraction from tauri.conf.json instead of jq (no extra dependency)"
  - "APPLE_ID env var presence determines CI vs local notarization path"
  - "CI env var check skips DMG Finder window styling (AppleScript requires display)"

patterns-established:
  - "Env var with fallback: ${VAR:-default} pattern for CI/local compatibility"
  - "CI detection: ${CI:-} != true guard for GUI-dependent operations"

requirements-completed: [CICD-05, CICD-06]

# Metrics
duration: 2min
completed: 2026-03-04
---

# Phase 20 Plan 01: Parameterize Build Script and Secrets Guide Summary

**CI-compatible build-dmg.sh with env var parameterization for VERSION/SIGNING_IDENTITY/KEYCHAIN_PROFILE, dual-path notarization (keychain vs explicit creds), and comprehensive secrets setup guide**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-04T19:31:17Z
- **Completed:** 2026-03-04T19:33:34Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Eliminated hardcoded VERSION=0.2.2 -- now auto-extracted from tauri.conf.json with env var override
- Added CI/local branching for notarization (APPLE_ID credentials vs keychain profile)
- Wrapped DMG Finder window styling in CI guard to avoid headless failures
- Created step-by-step guide covering p12 export, app-specific password, and GitHub Secrets setup

## Task Commits

Each task was committed atomically:

1. **Task 1: Parameterize build-dmg.sh for CI compatibility** - `414951e` (feat)
2. **Task 2: Create CI secrets setup guide** - `2c518d8` (docs)

## Files Created/Modified
- `scripts/build-dmg.sh` - Parameterized VERSION, SIGNING_IDENTITY, KEYCHAIN_PROFILE with env var fallbacks; added CI/local notarization branching; CI guard for DMG styling
- `docs/ci-secrets-setup.md` - Complete guide for exporting p12 certificate, generating app-specific password, configuring all 5 GitHub Secrets, and verifying the setup

## Decisions Made
- Used grep+sed to extract version from tauri.conf.json rather than requiring jq as a dependency
- APPLE_ID env var presence is the signal for CI notarization path (vs keychain profile for local)
- DMG styling skip in CI is acceptable -- the DMG still functions correctly without the pretty drag-to-Applications layout

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required

The user must export their Apple Developer ID certificate and configure GitHub Secrets before the CI pipeline can sign and notarize builds. See `docs/ci-secrets-setup.md` for exact steps.

## Next Phase Readiness
- build-dmg.sh is ready to be called from a GitHub Actions workflow
- All 5 required secrets are documented with sourcing instructions
- Plan 20-02 (release workflow) can now reference this parameterized script

---
*Phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management*
*Completed: 2026-03-04*
