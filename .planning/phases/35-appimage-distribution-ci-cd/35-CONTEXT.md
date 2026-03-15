# Phase 35: AppImage Distribution & CI/CD - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Linux users can download and auto-update CMD+K as an AppImage from GitHub Releases. Third CI job builds AppImage for x86_64 and aarch64. Auto-updater supports Linux AppImage with existing Ed25519 signing. No .deb/.rpm packages — AppImage is the sole Linux distribution format.

</domain>

<decisions>
## Implementation Decisions

### AppImage naming & release layout
- Naming convention: `CMD+K-{version}-linux-x86_64.AppImage` and `CMD+K-{version}-linux-aarch64.AppImage`
- Matches existing pattern: CMD+K-0.2.8-universal.dmg, CMD+K-0.2.8-windows-x64.exe
- AppImage only — no .deb/.rpm (explicitly out of scope per REQUIREMENTS.md)
- Updater artifacts (`.tar.gz` + `.sig`) follow same pattern as macOS/Windows, referenced by `latest.json`

### Auto-update behavior on Linux
- Replace-in-place on quit — same mechanism as macOS (Tauri updater overwrites AppImage file)
- If AppImage location is not writable: warn and skip update (tray message, don't attempt write)
- Auto-update enabled by default on Linux — same as macOS/Windows, user can disable in settings
- No special Linux-specific update logic beyond the write-permission check

### CI runner & system deps
- x86_64 AND aarch64 targets — native ARM runner (`ubuntu-22.04-arm`) per research finding that linuxdeploy cannot cross-compile
- Ubuntu 22.04 runner for glibc compatibility floor
- System packages via `apt-get install` in workflow (no caching) — include webkit2gtk, libayatana-appindicator, libdbus-1-dev, and other Tauri prerequisites
- Linux build job blocks release — `release` job `needs: [build-macos, build-windows, build-linux]`

### Release notes & download table
- Restructure release body into OS-grouped sections (macOS, Windows, Linux) instead of flat table
- One-liner chmod hint for Linux: `chmod +x CMD+K-*.AppImage && ./CMD+K-*.AppImage`
- Auto-update note updated to mention Linux: "Users on v0.3.9+ will receive this update automatically (macOS, Windows, Linux)"

### Claude's Discretion
- Exact apt package list for Tauri + zbus build dependencies
- Cross-compilation toolchain setup for aarch64 (gcc-aarch64-linux-gnu, pkg-config configuration)
- AppImage desktop file and icon integration details
- Tauri bundler configuration specifics for AppImage target
- How to handle FUSE requirement for AppImage execution

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/commands/updater.rs`: Full auto-update pipeline (check → download → install on quit). Comment already mentions macOS/Windows — Linux path needs addition
- `.github/workflows/release.yml`: Two build jobs (macOS, Windows) + release assembly job. Well-structured for adding third Linux job
- `src-tauri/tauri.conf.json`: Updater plugin configured with Ed25519 pubkey and latest.json endpoint. `bundle.targets: "all"` already set
- `latest.json` assembly in release job: Already generates platform entries for darwin-aarch64, darwin-x86_64, windows-x86_64. Needs linux-x86_64 and linux-aarch64 entries

### Established Patterns
- Each platform has its own build job uploading artifacts to actions/upload-artifact
- Release job downloads all artifacts and assembles latest.json
- Updater signatures (.sig files) uploaded alongside installers
- SHA256 checksums generated per artifact
- `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` secrets already configured

### Integration Points
- `release.yml` → add `build-linux` job, update `release.needs`, update latest.json assembly, update release body template
- `tauri.conf.json` → may need Linux-specific bundle configuration
- `updater.rs` → verify install_pending_update works for AppImage (may need Linux-specific path)
- `Cargo.toml` → ensure zbus and Linux-specific deps compile in CI environment

</code_context>

<specifics>
## Specific Ideas

- Release body should group downloads by OS with sections (macOS / Windows / Linux) rather than a flat table
- Include brief chmod hint inline, not full installation instructions
- Auto-update note should explicitly mention all three platforms

</specifics>

<deferred>
## Deferred Ideas

- **Showcase site updates** — update version numbers everywhere, platform-specific download buttons, updated privacy policy page with option to view previous policy notes. This is a separate phase after milestone completion.
- **.deb/.rpm packages** — out of scope per REQUIREMENTS.md, AppImage covers all distros
- **Snap/Flatpak packaging** — sandboxing conflicts with /proc access and xdotool

</deferred>

---

*Phase: 35-appimage-distribution-ci-cd*
*Context gathered: 2026-03-15*
