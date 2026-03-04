# Phase 20: GitHub Actions CI/CD for Windows and macOS Builds - Context

**Gathered:** 2026-03-04
**Status:** Ready for planning

<domain>
## Phase Boundary

Create a GitHub Actions CI/CD pipeline that builds signed+notarized macOS DMG and unsigned Windows NSIS installers when a release tag is pushed. Migrate Apple notarization credentials from local keychain to GitHub Secrets. Auto-publish GitHub Releases with artifacts and checksums.

</domain>

<decisions>
## Implementation Decisions

### Release trigger & tagging
- Tag format: `v*` prefix (e.g., `v0.2.4`, `v1.0.0`)
- Trigger: Tag push only — no PR or branch builds
- GitHub Release: Auto-publish on tag push (not draft)
- Version management: Manual bump in package.json + tauri.conf.json before tagging — CI builds whatever version is committed

### macOS signing in CI
- Reuse existing `scripts/build-dmg.sh` — parameterize hardcoded values via environment variables for CI
- Developer ID certificate: Export as .p12 from Keychain Access, base64-encode, store as GitHub Secret
- Include step-by-step guide for exporting p12 and storing credentials
- Notarization credentials as separate GitHub Secrets: `APPLE_ID`, `APPLE_TEAM_ID`, `APPLE_APP_PASSWORD`
- Workflow sets up a temporary keychain on the macOS runner, imports p12, then runs build-dmg.sh
- Universal binary (x86_64 + aarch64) — matches current local build output

### Windows code signing
- Unsigned NSIS installers for now — users get SmartScreen warning but can click through
- Include a conditional signing block in the workflow that's skipped when signing secrets aren't set
- Easy to enable later: just add `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` secrets
- Build steps inlined in workflow YAML (not reusing build-windows.sh — too simple to warrant a script call and avoids shell compat issues)

### Artifact distribution
- Naming: `CMD+K-{version}-universal.dmg` and `CMD+K-{version}-windows-x64.exe`
- SHA256 checksums generated alongside each artifact
- Release body: Minimal template with download links and platform info — manually edit release notes after publish
- Single `release.yml` workflow file with matrix strategy for [macos-latest, windows-latest]

### Claude's Discretion
- Exact workflow YAML structure and job dependencies
- How to extract version from tag for artifact naming
- Temporary keychain setup/teardown details on macOS runner
- Whether to use `tauri-apps/tauri-action` or raw build commands
- Rust/Node caching strategy for faster builds
- Conditional signing block implementation details

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `scripts/build-dmg.sh`: Full 10-step macOS pipeline (build → sign → notarize → staple → DMG) — needs parameterization for CI
- `scripts/build-windows.sh`: Minimal NSIS build script — CI will inline these steps instead
- `docs/windows-ci-build.md`: Draft Windows CI workflow with detailed planning (227 lines)
- `src-tauri/entitlements.plist`: Apple entitlements for automation (AppleEvents)
- `src-tauri/Info.plist`: macOS app metadata

### Established Patterns
- Signing identity: `Developer ID Application: VENKAT LUKSSHMAN TURLAPATI (36L722DZ7X)`
- Bundle identifier: `com.lakshmanturlapati.cmd-k`
- Keychain profile name: `CMD-K-NOTARIZE` (local — needs migration to CI secrets)
- Tauri build command: `pnpm tauri build --target universal-apple-darwin` (macOS) / `pnpm tauri build` (Windows)
- NSIS config: `currentUser` install mode, embedded WebView2 bootstrapper

### Integration Points
- `package.json` scripts: `build:mac` and `build:windows` exist
- `src-tauri/tauri.conf.json`: Bundle config with macOS signing identity and Windows NSIS settings
- `src-tauri/Cargo.toml`: Platform-conditional dependencies (macOS: tauri-nspanel, accessibility-sys; Windows: windows-sys, uiautomation)
- Version currently at 0.2.4 in package.json/tauri.conf.json (mismatch with build-dmg.sh hardcoded 0.2.2 — needs fix)

</code_context>

<specifics>
## Specific Ideas

- User specifically wants careful credential migration: keychain profile → GitHub Secrets
- Include documentation for the p12 export process (Keychain Access → Export → base64)
- The existing `docs/windows-ci-build.md` has a ready-to-use workflow template for Windows — leverage it
- Version mismatch in build-dmg.sh (0.2.2) vs package.json (0.2.4) should be fixed as part of parameterization

</specifics>

<deferred>
## Deferred Ideas

- PR build verification (compile-only, no artifacts) — future improvement
- Windows OV/EV code signing certificate — purchase when distribution warrants it
- Auto-updater integration (tauri-plugin-updater) — separate phase
- Linux builds — separate phase

</deferred>

---

*Phase: 20-add-github-actions-ci-cd-for-windows-and-macos-builds-on-release-tag-with-notarization-credential-management*
*Context gathered: 2026-03-04*
