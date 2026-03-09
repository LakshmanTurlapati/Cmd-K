# Phase 24: Auto-Updater - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Ship a silent auto-updater for CMD+K on macOS and Windows. The app checks for updates on launch and every 24 hours, auto-downloads in background, and installs on quit. Uses Ed25519 signed updates via GitHub Releases. CI/CD generates manifests and signatures. No forced restarts.

</domain>

<decisions>
## Implementation Decisions

### Update Notification UX
- Tray menu item only — no badge dots, no toast notifications
- Permanent "Check for Updates..." menu item in tray menu (above Settings)
- When update found: menu item changes to "⬆ Update Available (v0.2.7)"
- Menu item stays visible until user quits or updates — no dismiss-to-hide behavior
- State transitions shown via menu text: "Check for Updates..." → "Update Available" → "Downloading..." → "Update Ready (restart to apply)"

### Update Check Timing
- Check on launch (non-blocking, background)
- Silent background re-check every 24 hours while app is running
- No check on metered connections or airplane mode — silent failure, retry next cycle

### Update Install Flow
- Auto-download in background as soon as update detected (no click required to start download)
- Download failure: silent retry on next 24h check or manual click — no error dialog
- After download: menu shows "Update Ready (restart to apply)"
- Update installs when user quits the app normally
- No forced restart — update applies on next launch
- Option to disable auto-update checks in settings/preferences

### Signing & Security
- Ed25519 keypair with password protection
- Private key stored as GitHub Actions secret (TAURI_SIGNING_PRIVATE_KEY)
- Key password stored as separate secret (TAURI_SIGNING_PRIVATE_KEY_PASSWORD)
- Public key embedded in tauri.conf.json
- Updates verified against embedded public key before installation

### Update Distribution
- GitHub Releases as update endpoint (zero infrastructure cost)
- Tauri's built-in GitHub endpoint support
- Single latest.json manifest with platform-specific entries (macOS + Windows)

### CI/CD Integration
- Tauri built-in manifest generation (no custom scripts)
- release.yml extended to upload .sig files and latest.json alongside existing DMG/NSIS artifacts
- Ship as v0.2.6 (part of current milestone scope)
- Users on v0.2.4 manually update to v0.2.6, then get auto-updates from v0.2.7+

### Claude's Discretion
- Exact settings UI placement for the auto-update disable toggle
- Error handling edge cases (partial downloads, corrupt manifests)
- Update check backoff strategy on repeated failures

</decisions>

<specifics>
## Specific Ideas

- Menu text state machine: "Check for Updates..." → "Update Available (vX.Y.Z)" → "Downloading vX.Y.Z..." → "Update Ready (restart to apply)"
- First release with updater (v0.2.6) establishes the update pipeline — all future releases auto-deliver to users
- Ed25519 keypair must be generated BEFORE the first updater-enabled release ships (blocker from STATE.md)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src-tauri/src/commands/tray.rs`: Tray menu with Settings, Change Hotkey, About, Quit — extend with update items
- `.github/workflows/release.yml`: Full CI/CD pipeline with macOS signing/notarization and Windows NSIS — extend with updater artifacts

### Established Patterns
- Tauri plugin pattern: global-shortcut, positioner, store, http already integrated — updater follows same pattern
- Commands module pattern: each feature has its own `src-tauri/src/commands/{feature}.rs` file

### Integration Points
- `src-tauri/Cargo.toml`: Add `tauri-plugin-updater = "2"` dependency
- `src-tauri/tauri.conf.json`: Add updater configuration block with GitHub endpoint and public key
- `src-tauri/src/lib.rs`: Register updater plugin and commands
- `src-tauri/src/commands/tray.rs`: Add update menu items and event handlers

</code_context>

<deferred>
## Deferred Ideas

- Update channel selector (stable/beta) — captured as UPDT-F01 in REQUIREMENTS.md
- Release notes display in-app before updating

</deferred>

---

*Phase: 24-auto-updater*
*Context gathered: 2026-03-09*
