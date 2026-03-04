---
phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management
plan: 02
subsystem: infra
tags: [ci-cd, github-actions, macos, windows, code-signing, notarization, release-pipeline, nsis]

# Dependency graph
requires:
  - phase: 20-01
    provides: CI-compatible build-dmg.sh with env var parameterization
provides:
  - Complete GitHub Actions release pipeline triggered by v* tags
  - Signed and notarized macOS universal DMG builds
  - Windows NSIS installer builds with conditional signing
  - Auto-published GitHub Releases with SHA256 checksums
affects: []

# Tech tracking
tech-stack:
  added: [softprops/action-gh-release@v2, actions/upload-artifact@v4, actions/download-artifact@v4]
  patterns: [tag-triggered-workflow, multi-platform-build-matrix, conditional-signing, artifact-upload-download]

key-files:
  created:
    - .github/workflows/release.yml
  modified: []

key-decisions:
  - "3-job architecture: build-macos, build-windows, release -- parallel builds then sequential publish"
  - "Temporary keychain with notarytool store-credentials for CI notarization (not inline credentials)"
  - "Conditional Windows signing gated on WINDOWS_CERTIFICATE secret presence -- graceful skip when not configured"
  - "softprops/action-gh-release@v2 for auto-publish with template release body"

patterns-established:
  - "Tag-triggered release: v* push triggers full build+release pipeline"
  - "Artifact pass-through: build jobs upload, release job downloads and publishes"
  - "Conditional signing: if env.SECRET != '' guards for optional platform signing"

requirements-completed: [CICD-01, CICD-02, CICD-03, CICD-04]

# Metrics
duration: 1min
completed: 2026-03-04
---

# Phase 20 Plan 02: GitHub Actions Release Workflow Summary

**Tag-triggered CI/CD pipeline with parallel macOS (signed+notarized DMG) and Windows (NSIS) builds, auto-published as GitHub Release with SHA256 checksums**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-04T19:36:18Z
- **Completed:** 2026-03-04T19:37:44Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments
- Created complete release.yml with 3 jobs (build-macos, build-windows, release) triggered on v* tag push
- macOS job imports p12 certificate into temporary keychain, stores notarization credentials, and runs parameterized build-dmg.sh
- Windows job builds NSIS installer via pnpm tauri build with conditional signing block for future certificate support
- Release job auto-publishes GitHub Release with both platform artifacts and SHA256 checksums via softprops/action-gh-release@v2
- Both build jobs cache Rust and Node dependencies for faster subsequent runs

## Task Commits

Each task was committed atomically:

1. **Task 1: Create release.yml workflow with macOS signing job, Windows build job, and release publishing job** - `bf9d2a6` (feat)

## Files Created/Modified
- `.github/workflows/release.yml` - Complete CI/CD release pipeline: tag trigger, macOS build with code signing and notarization, Windows NSIS build with conditional signing, auto-published GitHub Release with artifacts and checksums

## Decisions Made
- Used 3-job architecture (build-macos, build-windows, release) for parallel platform builds followed by sequential release publishing
- Temporary keychain with `notarytool store-credentials` approach matches Plan 01's parameterized build-dmg.sh expectations
- Windows signing is conditional on `WINDOWS_CERTIFICATE` secret presence -- workflow runs successfully without it
- Release body uses a minimal template with download table and checksum reference -- user can edit after publish

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required

The user must configure GitHub Secrets before the workflow can sign and notarize builds. See `docs/ci-secrets-setup.md` for exact steps:
- `APPLE_CERTIFICATE_BASE64` - base64-encoded p12 certificate
- `APPLE_CERTIFICATE_PASSWORD` - p12 export password
- `APPLE_ID` - Apple ID email
- `APPLE_TEAM_ID` - Apple Developer Team ID
- `APPLE_APP_PASSWORD` - App-specific password
- (Optional) `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` for Windows signing

## Next Phase Readiness
- Complete CI/CD pipeline ready -- push a v* tag to trigger builds
- All GitHub Secrets must be configured first (see docs/ci-secrets-setup.md)
- Phase 20 is now complete -- no further plans

## Self-Check: PASSED

- FOUND: .github/workflows/release.yml
- FOUND: 20-02-SUMMARY.md
- FOUND: commit bf9d2a6

---
*Phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management*
*Completed: 2026-03-04*
