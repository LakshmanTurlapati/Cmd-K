---
status: testing
phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management
source: 20-01-SUMMARY.md, 20-02-SUMMARY.md
started: 2026-03-04T20:00:00Z
updated: 2026-03-04T20:00:00Z
---

## Current Test

number: 1
name: Build Script Version Auto-Detection
expected: |
  In `scripts/build-dmg.sh`, there is no hardcoded VERSION value. The script extracts the version from `src-tauri/tauri.conf.json` automatically (using grep+sed), with an env var override option (`VERSION` env var).
awaiting: user response

## Tests

### 1. Build Script Version Auto-Detection
expected: In `scripts/build-dmg.sh`, there is no hardcoded VERSION value. The script extracts the version from `src-tauri/tauri.conf.json` automatically (using grep+sed), with an env var override option (`VERSION` env var).
result: [pending]

### 2. Build Script CI/Local Notarization Branching
expected: In `scripts/build-dmg.sh`, when `APPLE_ID` env var is set, the script uses explicit credential-based notarization (`notarytool submit` with `--apple-id`, `--team-id`, `--password`). When `APPLE_ID` is not set, it uses the local keychain profile. This allows the same script to work in both CI and local environments.
result: [pending]

### 3. DMG Styling CI Guard
expected: In `scripts/build-dmg.sh`, the DMG Finder window styling (AppleScript that creates the drag-to-Applications layout) is wrapped in a CI guard -- it only runs when `CI` env var is not `true`. This prevents headless CI failures.
result: [pending]

### 4. CI Secrets Setup Guide
expected: `docs/ci-secrets-setup.md` exists and documents all 5 required GitHub Secrets: `APPLE_CERTIFICATE_BASE64`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD`. Includes step-by-step instructions for p12 export, app-specific password generation, and how to add secrets in GitHub.
result: [pending]

### 5. Release Workflow File and Tag Trigger
expected: `.github/workflows/release.yml` exists and is triggered by `v*` tag pushes (e.g., pushing tag `v0.2.2` starts the pipeline). The workflow has 3 jobs: `build-macos`, `build-windows`, and `release`.
result: [pending]

### 6. macOS Build Job - Signing and Notarization
expected: The `build-macos` job in `release.yml` imports a p12 certificate into a temporary keychain, stores notarization credentials, and calls the parameterized `build-dmg.sh` script to produce a signed+notarized universal DMG.
result: [pending]

### 7. Windows Build Job with Conditional Signing
expected: The `build-windows` job in `release.yml` builds an NSIS installer via `pnpm tauri build`. Windows code signing is conditional -- gated on `WINDOWS_CERTIFICATE` secret presence so the workflow succeeds even without signing configured.
result: [pending]

### 8. Release Publishing with Checksums
expected: The `release` job downloads both platform artifacts, generates SHA256 checksums for each, and auto-publishes a GitHub Release using `softprops/action-gh-release@v2` with the DMG, NSIS installer, and checksum files attached.
result: [pending]

## Summary

total: 8
passed: 0
issues: 0
pending: 8
skipped: 0

## Gaps

[none yet]
