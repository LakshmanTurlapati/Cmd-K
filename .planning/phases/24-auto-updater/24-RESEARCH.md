# Phase 24: Auto-Updater - Research

**Researched:** 2026-03-09
**Domain:** Tauri v2 updater plugin, Ed25519 signing, CI/CD update pipeline
**Confidence:** HIGH

## Summary

The Tauri v2 updater ecosystem is mature and well-documented. The `tauri-plugin-updater` (v2.10.0) provides a complete solution: Ed25519 signature verification, GitHub Releases as a static endpoint, and separate `download()` / `install()` Rust APIs that enable the "download in background, install on quit" pattern the user wants.

The key architectural decision is that ALL update logic lives in Rust (not JS). The updater check runs on launch and every 24h via a tokio timer. Tray menu text updates reflect state transitions. On macOS, `install()` does not force-quit the app, so "install on next launch" works naturally by calling `install()` when the user clicks Quit. On Windows, the NSIS installer forces an exit during `install()`, but with `installMode: "passive"` this is nearly invisible and still satisfies "update applies on next launch."

**Primary recommendation:** Use `tauri-plugin-updater = "2"` with the Rust-side `UpdaterExt` API. Keep the entire update lifecycle in `src-tauri/src/commands/updater.rs`. The CI pipeline extends naturally -- `pnpm tauri build` with `TAURI_SIGNING_PRIVATE_KEY` env var auto-generates `.sig` files and (with `createUpdaterArtifacts: true`) the `.app.tar.gz` for macOS. A small CI script assembles `latest.json` from the generated signatures.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions
- Tray menu item only -- no badge dots, no toast notifications
- Permanent "Check for Updates..." menu item in tray menu (above Settings)
- When update found: menu item changes to "Update Available (vX.Y.Z)"
- Menu item stays visible until user quits or updates -- no dismiss-to-hide behavior
- State transitions shown via menu text: "Check for Updates..." -> "Update Available" -> "Downloading..." -> "Update Ready (restart to apply)"
- Check on launch (non-blocking, background)
- Silent background re-check every 24 hours while app is running
- No check on metered connections or airplane mode -- silent failure, retry next cycle
- Auto-download in background as soon as update detected (no click required to start download)
- Download failure: silent retry on next 24h check or manual click -- no error dialog
- After download: menu shows "Update Ready (restart to apply)"
- Update installs when user quits the app normally
- No forced restart -- update applies on next launch
- Option to disable auto-update checks in settings/preferences
- Ed25519 keypair with password protection
- Private key stored as GitHub Actions secret (TAURI_SIGNING_PRIVATE_KEY)
- Key password stored as separate secret (TAURI_SIGNING_PRIVATE_KEY_PASSWORD)
- Public key embedded in tauri.conf.json
- Updates verified against embedded public key before installation
- GitHub Releases as update endpoint (zero infrastructure cost)
- Tauri's built-in GitHub endpoint support
- Single latest.json manifest with platform-specific entries (macOS + Windows)
- Tauri built-in manifest generation (no custom scripts)
- release.yml extended to upload .sig files and latest.json alongside existing DMG/NSIS artifacts
- Ship as v0.2.6 (part of current milestone scope)
- Users on v0.2.4 manually update to v0.2.6, then get auto-updates from v0.2.7+

### Claude's Discretion
- Exact settings UI placement for the auto-update disable toggle
- Error handling edge cases (partial downloads, corrupt manifests)
- Update check backoff strategy on repeated failures

### Deferred Ideas (OUT OF SCOPE)
- Update channel selector (stable/beta) -- captured as UPDT-F01 in REQUIREMENTS.md
- Release notes display in-app before updating

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| UPDT-01 | App checks for updates on launch without blocking the UI | Rust-side `app.updater()?.check().await?` in a `tokio::spawn` task; non-blocking by design |
| UPDT-02 | User sees an "Update Available" indicator in the tray menu when an update exists | Tray menu item text updated via `MenuItem::set_text()` on the existing tray menu |
| UPDT-03 | User can download and install the update with one click from the tray | `update.download()` auto-starts on detection; menu click triggers `update.install()` on macOS or prompts quit on Windows |
| UPDT-04 | Update is applied on next app launch (no forced restart) | macOS: `install()` replaces .app bundle without force-quit; Windows: `install()` triggers passive NSIS then app exits |
| UPDT-05 | Updates are cryptographically signed and verified before installation | Ed25519 signing via `TAURI_SIGNING_PRIVATE_KEY` env var; verification via `pubkey` in tauri.conf.json |
| UPDT-06 | CI/CD pipeline generates signed update artifacts and latest.json manifest | `createUpdaterArtifacts: true` generates .tar.gz/.sig (macOS) and .exe/.sig (Windows); CI assembles latest.json |
| UPDT-07 | Background update checks run silently every 24 hours after launch | `tokio::time::interval(Duration::from_secs(86400))` in a spawned task |
| UPDT-08 | Dismissing the update notification suppresses it until next app launch | Per CONTEXT.md: menu item stays visible (no dismiss). Meets requirement by persisting indicator until quit |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-updater | 2 (latest: 2.10.0) | Update checking, downloading, signature verification, installation | Official Tauri plugin; Ed25519 built-in; GitHub endpoint support |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-plugin-store | 2 (already installed) | Persist auto-update preference toggle | Already used for settings.json |
| tokio | 1 (already installed) | Background timer for 24h re-check | Already used; add `time` feature (already present) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tauri-plugin-updater | Custom HTTP + manual verification | Far more work; already solved by the plugin |
| Static latest.json | Dynamic update server | Zero-infrastructure vs. requires server hosting; GitHub Releases is free |
| tauri-action | Custom CI script | Project already has custom build-dmg.sh; extending existing CI is simpler than adopting tauri-action |

**Installation:**
```bash
cd src-tauri && cargo add tauri-plugin-updater --target 'cfg(any(target_os = "macos", windows, target_os = "linux"))'
```

No JS/frontend package needed -- all update logic stays in Rust.

## Architecture Patterns

### Recommended Module Structure
```
src-tauri/src/commands/
  updater.rs          # New: update check, download, install, state management
  tray.rs             # Modified: add update menu items, handle state transitions
  mod.rs              # Modified: export updater commands
```

### Pattern 1: Rust-Side Update State Machine
**What:** An enum-based state machine tracking the update lifecycle, stored in `AppState`.
**When to use:** Always -- this drives the tray menu text.
**Example:**
```rust
// Source: Derived from tauri-plugin-updater docs
use std::sync::Mutex;
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Debug, Clone)]
pub enum UpdateStatus {
    Idle,                           // "Check for Updates..."
    Available(String),              // "Update Available (vX.Y.Z)"
    Downloading(String),            // "Downloading vX.Y.Z..."
    Ready(String, Vec<u8>),         // "Update Ready (restart to apply)"
    Disabled,                       // Auto-check disabled by user
}

// Store in AppState alongside existing fields
pub struct UpdateState {
    pub status: Mutex<UpdateStatus>,
    pub pending_update: Mutex<Option<Update>>,
    pub pending_bytes: Mutex<Option<Vec<u8>>>,
}
```

### Pattern 2: Background Check with tokio::spawn
**What:** Non-blocking update check on launch + 24h interval.
**When to use:** In `setup()` callback after plugin registration.
**Example:**
```rust
// Source: Tauri v2 updater docs + tokio patterns
use tokio::time::{interval, Duration};
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

fn spawn_update_checker(app_handle: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Initial check on launch
        check_for_update(&app_handle).await;

        // Re-check every 24 hours
        let mut timer = interval(Duration::from_secs(86400));
        timer.tick().await; // skip first immediate tick
        loop {
            timer.tick().await;
            check_for_update(&app_handle).await;
        }
    });
}

async fn check_for_update(app: &tauri::AppHandle) {
    // Read auto-update preference from store
    // If disabled, return early

    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                // Update tray menu text
                // Auto-start download
            }
            Ok(None) => { /* no update available */ }
            Err(_) => { /* silent failure, retry next cycle */ }
        }
        Err(_) => { /* silent failure */ }
    }
}
```

### Pattern 3: Install-on-Quit via Tray Quit Handler
**What:** Intercept the "Quit" tray menu event to install pending update before exiting.
**When to use:** When update bytes are downloaded and ready.
**Example:**
```rust
// In tray.rs quit handler:
"quit" => {
    // Check if update bytes are pending
    let state = app.state::<UpdateState>();
    let bytes = state.pending_bytes.lock().unwrap().take();
    let update = state.pending_update.lock().unwrap().take();

    if let (Some(update), Some(bytes)) = (update, bytes) {
        // On macOS: install() replaces .app, then we exit
        // On Windows: install() triggers NSIS passive installer which auto-exits
        let _ = update.install(&bytes);
    }
    app.exit(0);
}
```

### Pattern 4: Dynamic Tray Menu Text Updates
**What:** Update the tray menu item text to reflect current update state.
**When to use:** On each state transition.
**Example:**
```rust
// Source: Tauri v2 menu API
use tauri::menu::MenuItem;
use tauri::Manager;

fn update_tray_menu_text(app: &tauri::AppHandle, text: &str) {
    // Get the menu item by ID and update its text
    if let Some(tray) = app.tray_by_id("main") {
        if let Some(menu) = tray.menu() {
            // MenuItem::set_text() updates the displayed text
        }
    }
}
```

### Anti-Patterns to Avoid
- **JS-side update logic:** Keep all update logic in Rust. The JS layer has no need to know about updates; the tray menu is the sole UI.
- **Blocking the setup() callback:** Never await update checks in setup(). Always spawn as a background task.
- **Storing update bytes in AppState without Arc<Mutex<>>:** The bytes can be large; use proper interior mutability.
- **Calling install() from a non-quit context on Windows:** Windows NSIS installer force-exits the app. Only call install() when the user is already quitting.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Ed25519 signature verification | Custom crypto verification | `tauri-plugin-updater` built-in verification | Crypto is hard; plugin handles it correctly |
| Update artifact packaging | Custom .tar.gz/.sig generation | `createUpdaterArtifacts: true` in tauri.conf.json | Tauri CLI auto-generates correct format |
| Version comparison | Manual semver parsing | Plugin's built-in version comparator | Handles pre-release, build metadata correctly |
| latest.json format | Freeform JSON | Standard Tauri updater manifest format | Plugin expects specific structure |
| HTTP download with progress | Custom reqwest download | `update.download()` with progress callback | Plugin handles retries, TLS, proxy |

**Key insight:** The updater plugin handles the entire lifecycle (check, download, verify, install). The only custom code needed is: (1) state management for tray UI, (2) timing logic for 24h checks, (3) CI script to assemble latest.json.

## Common Pitfalls

### Pitfall 1: Forgetting TAURI_SIGNING_PRIVATE_KEY in CI
**What goes wrong:** Build succeeds but no .sig files are generated; updater artifacts are missing.
**Why it happens:** The signing key must be in environment variables, not .env files. If the env var is missing, Tauri silently skips signature generation.
**How to avoid:** Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as GitHub Actions secrets. Verify .sig files exist in CI artifacts.
**Warning signs:** No `.sig` files in build output directory.

### Pitfall 2: Wrong Platform Keys in latest.json
**What goes wrong:** App finds no update because platform key doesn't match.
**Why it happens:** The platform key must match `{target}-{arch}` exactly. For universal macOS builds, the key can be `darwin-aarch64` and `darwin-x86_64` (both pointing to the same universal .tar.gz), NOT `universal-apple-darwin`.
**How to avoid:** Use `darwin-aarch64` and `darwin-x86_64` as platform keys for macOS universal builds, both pointing to the same `.app.tar.gz` URL. Use `windows-x86_64` for Windows.
**Warning signs:** Update check returns `None` even though latest.json exists with a newer version.

### Pitfall 3: macOS Custom Build Script + Updater Artifacts
**What goes wrong:** The custom build-dmg.sh script creates a DMG but the updater needs a .app.tar.gz.
**Why it happens:** The project uses a custom DMG build flow, but `createUpdaterArtifacts: true` makes `pnpm tauri build` also generate .app.tar.gz and .sig alongside the .app bundle.
**How to avoid:** The .app.tar.gz is generated by `pnpm tauri build --target universal-apple-darwin` (called inside build-dmg.sh). Both the DMG (for manual download) and the .app.tar.gz (for auto-update) are produced by the same build. Upload both to GitHub Releases.
**Warning signs:** Only DMG in release assets, no .tar.gz.

### Pitfall 4: Windows NSIS Force-Exit During install()
**What goes wrong:** Calling `install()` at an unexpected time kills the app abruptly.
**Why it happens:** On Windows, the NSIS installer must replace the running executable, so it force-exits the process.
**How to avoid:** Only call `install()` in the quit handler. Use `on_before_exit` on the `UpdaterBuilder` for cleanup if needed. Set `installMode: "passive"` for a non-interactive installer.
**Warning signs:** App closes unexpectedly mid-use.

### Pitfall 5: Ed25519 Key Loss
**What goes wrong:** Cannot publish updates to existing users; they are permanently stuck on the last version.
**Why it happens:** The public key is embedded in the shipped binary. If the private key is lost, no new update can be signed that the embedded public key will verify.
**How to avoid:** Store the private key securely in GitHub Actions secrets AND in a backup location (password manager, secure vault). Document the key backup location.
**Warning signs:** Irreversible -- this must be prevented, not detected.

### Pitfall 6: GitHub Releases latest.json Caching
**What goes wrong:** Users don't see new updates for hours after release.
**Why it happens:** GitHub CDN caches release assets. The `latest/download/latest.json` URL may serve stale content.
**How to avoid:** Use the versioned release URL pattern if caching is severe. In practice, GitHub's cache TTL for release assets is short (< 5 minutes) and acceptable for this use case.
**Warning signs:** Update check returns old version despite new release being published.

## Code Examples

### tauri.conf.json Updater Configuration
```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/nicepkg/cmd-k/releases/latest/download/latest.json"
      ],
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIH...",
      "windows": {
        "installMode": "passive"
      }
    }
  }
}
```
Source: [Tauri v2 Updater Docs](https://v2.tauri.app/plugin/updater/)

**Note:** Replace the GitHub URL with the actual repository URL. The pubkey value is the CONTENTS of the `.pub` file generated by `pnpm tauri signer generate`.

### Plugin Registration in lib.rs
```rust
// In the builder chain, inside setup():
#[cfg(desktop)]
app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
```
Source: [Tauri v2 Updater Docs](https://v2.tauri.app/plugin/updater/)

### Capabilities Permission
```json
{
  "permissions": [
    "updater:default"
  ]
}
```
Source: [Tauri v2 Updater Docs](https://v2.tauri.app/plugin/updater/)

`updater:default` grants: `allow-check`, `allow-download`, `allow-install`, `allow-download-and-install`.

### Ed25519 Key Generation
```bash
pnpm tauri signer generate -w ~/.tauri/cmd-k-updater.key
```
Source: [Tauri v2 Updater Docs](https://v2.tauri.app/plugin/updater/)

This produces:
- `~/.tauri/cmd-k-updater.key` (private key -- NEVER share)
- `~/.tauri/cmd-k-updater.key.pub` (public key -- embed in tauri.conf.json)

### CI Environment Variables for Signing
```yaml
env:
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
```
Source: [Tauri v2 Updater Docs](https://v2.tauri.app/plugin/updater/)

### latest.json Assembly Script (CI)
```bash
#!/bin/bash
VERSION=$(jq -r '.version' src-tauri/tauri.conf.json)
# macOS: .app.tar.gz.sig is in the macos bundle directory
SIG_MACOS=$(cat target/universal-apple-darwin/release/bundle/macos/*.app.tar.gz.sig)
# Windows: .exe.sig is in the nsis bundle directory
SIG_WINDOWS=$(cat src-tauri/target/release/bundle/nsis/*.exe.sig)

REPO_URL="https://github.com/OWNER/REPO/releases/download/v${VERSION}"

cat > latest.json <<EOF
{
  "version": "${VERSION}",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "darwin-aarch64": {
      "signature": "${SIG_MACOS}",
      "url": "${REPO_URL}/CMD+K.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "${SIG_MACOS}",
      "url": "${REPO_URL}/CMD+K.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "${SIG_WINDOWS}",
      "url": "${REPO_URL}/CMD+K-${VERSION}-windows-x64.exe"
    }
  }
}
EOF
```

### Separate download() and install() API (Rust)
```rust
// Source: https://docs.rs/tauri-plugin-updater/latest/tauri_plugin_updater/struct.Update.html
use tauri_plugin_updater::{Update, UpdaterExt};

// download() returns Vec<u8>
let bytes = update.download(
    |chunk_length, content_length| {
        // Progress callback: chunk_length bytes received, content_length is total (Option)
    },
    || {
        // Download finished callback
    },
).await?;

// install() takes the bytes -- can be called later (e.g., on quit)
update.install(&bytes)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| tauri-plugin-updater v1 (zip-based) | v2 (direct installer reuse) | Tauri v2 stable | `createUpdaterArtifacts: true` uses .exe directly instead of .nsis.zip on Windows |
| JS-only update API | Rust UpdaterExt + JS bindings | Tauri v2 | Full Rust control enables install-on-quit pattern |
| tauri-action required | Manual CI with `createUpdaterArtifacts` | Tauri v2 | Can generate artifacts without tauri-action |

**Deprecated/outdated:**
- `createUpdaterArtifacts: "v1Compatible"`: Only needed for v1->v2 migration. Use `true` for new apps.
- JS-only `check()` from `@tauri-apps/plugin-updater`: Still works but unnecessary when all logic is Rust-side.

## Open Questions

1. **Exact GitHub repository URL for endpoints**
   - What we know: The endpoint format is `https://github.com/OWNER/REPO/releases/latest/download/latest.json`
   - What's unclear: The actual GitHub org/repo name for CMD+K (need to confirm the repo URL)
   - Recommendation: Planner should use a placeholder; implementer fills in actual URL

2. **macOS install-on-quit behavior verification**
   - What we know: `update.install(&bytes)` on macOS replaces the .app bundle. The docs say it does NOT force-quit.
   - What's unclear: Whether calling `install()` synchronously in the quit handler works reliably, or if the process exits before install completes
   - Recommendation: Test this flow manually. If synchronous install is unreliable, consider calling install() THEN `app.exit(0)` with a small delay.

3. **Windows createUpdaterArtifacts artifact naming**
   - What we know: With `createUpdaterArtifacts: true`, Windows generates the .exe installer directly (no .zip wrapper) plus a .sig file
   - What's unclear: The exact filename pattern for the .exe.sig (whether it matches the renamed artifact name from CI)
   - Recommendation: CI must rename/copy consistently. The .sig is generated pre-rename, so CI should read the sig before renaming the exe, or sign the renamed file.

## Validation Architecture

> Note: `workflow.nyquist_validation` is not set in config.json, treating as enabled.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual testing (no automated test framework detected) |
| Config file | none |
| Quick run command | N/A -- updater requires real GitHub Release |
| Full suite command | N/A |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UPDT-01 | Non-blocking update check on launch | manual | Verify app launches without delay when no network | N/A |
| UPDT-02 | Tray menu shows update indicator | manual | Publish test release, verify menu text changes | N/A |
| UPDT-03 | One-click download and install from tray | manual | Click menu item, verify download starts | N/A |
| UPDT-04 | Update applied on next launch | manual | Quit app, relaunch, verify new version | N/A |
| UPDT-05 | Ed25519 signature verification | unit | Verify tauri.conf.json has pubkey; verify .sig files in CI output | Wave 0 |
| UPDT-06 | CI generates signed artifacts + latest.json | integration | Run release workflow on test tag, verify artifacts | N/A |
| UPDT-07 | 24h background re-check | manual | Set interval to 10s for dev testing, verify re-check fires | N/A |
| UPDT-08 | Dismiss suppresses until next launch | manual | Per CONTEXT: no dismiss behavior, menu stays visible | N/A |

### Sampling Rate
- **Per task commit:** Compile check (`cargo check` in src-tauri)
- **Per wave merge:** Manual smoke test with a local/test GitHub release
- **Phase gate:** Full manual test on both macOS and Windows with a real release

### Wave 0 Gaps
- [ ] Ed25519 keypair must be generated and secrets added to GitHub Actions BEFORE first updater build
- [ ] A test/pre-release tag should be used to validate the CI pipeline before shipping v0.2.6

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Updater Plugin Docs](https://v2.tauri.app/plugin/updater/) - Full configuration, API, artifact generation
- [tauri-plugin-updater docs.rs](https://docs.rs/tauri-plugin-updater/latest/tauri_plugin_updater/) - Rust API: Update struct, Builder, UpdaterExt trait
- [Update struct API](https://docs.rs/tauri-plugin-updater/latest/tauri_plugin_updater/struct.Update.html) - download(), install(), download_and_install() signatures
- [Builder struct API](https://docs.rs/tauri-plugin-updater/latest/tauri_plugin_updater/struct.Builder.html) - Plugin configuration methods
- [UpdaterBuilder API](https://docs.rs/tauri-plugin-updater/latest/tauri_plugin_updater/struct.UpdaterBuilder.html) - Runtime configuration including on_before_exit

### Secondary (MEDIUM confidence)
- [Tauri GitHub Pipelines Guide](https://v2.tauri.app/distribute/pipelines/github/) - CI/CD patterns for Tauri releases
- [That Gurjot - Tauri Auto Updater Guide](https://thatgurjot.com/til/tauri-auto-updater/) - Verified step-by-step manual setup
- [Ratul's Blog - Tauri v2 Updater](https://ratulmaharaj.com/posts/tauri-automatic-updates/) - Community verified patterns

### Tertiary (LOW confidence)
- macOS install-on-quit synchronous behavior: Not explicitly documented; needs manual verification
- GitHub CDN caching behavior for latest.json: Anecdotal; generally < 5 min TTL

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Official Tauri plugin, well-documented, v2.10.0 stable
- Architecture: HIGH - Patterns derived from official docs and Rust API docs
- CI/CD integration: MEDIUM - Custom build-dmg.sh interaction with createUpdaterArtifacts needs manual verification
- Pitfalls: HIGH - Well-documented community issues; multiple sources confirm
- Install-on-quit on macOS: MEDIUM - API supports it but exact quit-handler behavior needs testing

**Research date:** 2026-03-09
**Valid until:** 2026-04-09 (stable ecosystem, 30-day validity)
