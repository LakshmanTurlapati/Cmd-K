# Phase 1: Merge Windows Branch - Research

**Researched:** 2026-03-03
**Domain:** Git merge operations, cross-platform Rust/Tauri development, platform-independent builds
**Confidence:** HIGH

## Summary

This phase merges the `windows` branch (30 commits, 58 files, +7.6k/-2k lines) into `main` to enable cross-platform support for both macOS and Windows. The Windows port is complete with comprehensive platform-specific code isolation using Rust `#[cfg(target_os)]` gates, platform-specific dependencies in Cargo.toml, and Windows-specific modules for terminal detection and UI automation.

The merge is straightforward with only 1 conflict in `showcase/index.html` where the download links diverged. The windows branch is at v0.2.1 while main is at v0.1.1, requiring version alignment across `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`. The codebase demonstrates excellent platform isolation patterns with no apparent platform leakage.

**Primary recommendation:** Execute a standard merge (not rebase/squash) to preserve the 30-commit Windows development history, resolve the showcase conflict by combining both platform download links, align all three version files to v0.2.1, verify the DMG background asset survives, and audit `#[cfg]` gates before finalizing.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

#### Merge strategy
- Standard merge of `windows` into `main` (not rebase, not squash)
- Preserves full 30-commit history from the Windows port
- Only 1 conflict: `showcase/index.html` — resolve by keeping both macOS DMG download links (from main) and Windows download links (from windows), producing a cross-platform download page
- Delete `windows` branch (local + remote) after successful merge
- Quick audit of all `#[cfg(target_os)]` gates before merging to confirm no platform leakage

#### Version alignment
- Merged version: v0.2.1 (from windows branch)
- All 3 version files must match: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Post-merge grep for any surviving "0.1.1" strings to catch stragglers

#### Build scripts
- Keep main's `build-dmg.sh` (full notarized DMG pipeline) intact — do NOT use the stripped-down version from windows
- Ensure `scripts/dmg-background.png` survives the merge (windows branch deleted it)
- Add a separate Windows build script (`build-windows.sh` or similar) for NSIS installer
- Add `build:mac` and `build:windows` npm scripts to `package.json` for discoverability
- CI/GitHub Actions is out of scope for this phase — local build scripts only

### Claude's Discretion

- Exact merge commit message wording
- Order of post-merge verification steps
- Whether to add `build-pkg.sh` script alias

### Deferred Ideas (OUT OF SCOPE)

- GitHub Actions CI for cross-platform builds — separate phase
- Linux support — future milestone
</user_constraints>

## Standard Stack

### Core Git Operations

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| git merge | 2.x+ | Three-way merge with conflict resolution | Built-in, preserves commit history |
| git diff | 2.x+ | Examine changes between branches | Essential for merge preview |
| git branch -d/-D | 2.x+ | Delete local/remote branches | Standard cleanup after merge |

### Tauri Cross-Platform Build

| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| Tauri | 2.x | Cross-platform desktop framework | Project's core framework |
| Rust | 1.70+ | Backend language with platform-specific compilation | Required by Tauri |
| pnpm | Latest | Package manager | Project's chosen tool |

### Platform-Specific Dependencies

**macOS (main branch):**
- `tauri-nspanel` - Floating panel overlay
- `accessibility-sys` - AX API for terminal reading
- `window-vibrancy` - Frosted glass effect

**Windows (windows branch):**
- `windows-sys` - Win32 API bindings
- `uiautomation` - UIA for terminal reading
- `arboard` - Clipboard operations
- `raw-window-handle` - Window handle access

**Shared:**
- `keyring` with platform-specific features (`apple-native`, `windows-native`)

### Configuration Files Requiring Version Sync

| File | Current (main) | Target (windows) | Purpose |
|------|---------------|------------------|---------|
| package.json | 0.2.1 | 0.2.1 | NPM metadata, frontend deps |
| src-tauri/Cargo.toml | 0.1.1 | 0.2.1 | Rust package metadata |
| src-tauri/tauri.conf.json | 0.1.1 | 0.2.1 | Tauri bundle configuration |

**Note:** Main branch `package.json` already at v0.2.1, but Cargo.toml and tauri.conf.json are at v0.1.1.

## Architecture Patterns

### Platform Isolation Pattern (Rust)

**What:** Use `#[cfg(target_os = "...")]` attributes to compile different code paths for each platform
**When to use:** For all OS-specific APIs (NSPanel, Win32, AX/UIA APIs)

**Example from windows branch:**
```rust
// lib.rs platform-specific plugin initialization
#[cfg(target_os = "macos")]
use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt};

#[cfg(target_os = "macos")]
tauri_nspanel::tauri_panel! {
    OverlayPanel {
        config: {
            can_become_key_window: true,
            is_floating_panel: true
        }
    }
}

pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // ... shared plugins
}
```

### Platform-Specific Module Organization

**Pattern:** Separate modules for platform-specific implementations
```
src-tauri/src/terminal/
├── mod.rs              # Shared interface
├── ax_reader.rs        # macOS-only (#[cfg(target_os = "macos")])
├── uia_reader.rs       # Windows-only (#[cfg(target_os = "windows")])
├── detect.rs           # Shared with platform branches
└── detect_windows.rs   # Windows constants/helpers
```

**From windows branch mod.rs:**
```rust
#[cfg(target_os = "windows")]
pub mod uia_reader;

#[allow(dead_code)]
pub mod detect_windows;  // Constants used on Windows, compiled everywhere
```

### Platform-Specific Dependencies (Cargo.toml)

**Pattern:** Use `[target.'cfg(...)'.dependencies]` for conditional compilation

**Example from windows branch:**
```toml
[target.'cfg(target_os = "macos")'.dependencies]
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
accessibility-sys = "0.2"
core-foundation-sys = "0.8"
keyring = { version = "3", features = ["apple-native"] }

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    # ... more features
] }
uiautomation = "0.24"
arboard = "3"
keyring = { version = "3", features = ["windows-native"] }
```

### Frontend Platform Detection

**Pattern:** Runtime platform detection for UI differences (border radius, hotkey display)

**Example from windows branch `src/utils/platform.ts`:**
```typescript
export function isWindows(): boolean {
  return navigator.userAgent.includes("Windows");
}

export function displayModifier(key: string): string {
  if (key === "Super") {
    return isWindows() ? "Ctrl" : "Cmd";
  }
  if (key === "Alt") {
    return isWindows() ? "Alt" : "Option";
  }
  return key;
}
```

### NSIS Installer Configuration (Windows)

**What:** Tauri bundle configuration for Windows installer
**When to use:** Windows builds require NSIS config in tauri.conf.json

**Example from windows branch:**
```json
{
  "bundle": {
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      },
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      }
    }
  }
}
```

### Anti-Patterns to Avoid

- **Runtime OS checks in Rust instead of compile-time cfg:** Use `#[cfg(target_os)]` not `if cfg!(target_os)` for entire functions/modules
- **Platform-specific code without cfg guards:** All macOS/Windows-specific code MUST be behind `#[cfg]` gates
- **Shared dependencies without platform-specific features:** Use `keyring` with `apple-native` on macOS, `windows-native` on Windows
- **Single-platform build scripts:** Each platform needs its own script due to different bundlers (DMG vs NSIS)

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Version sync across 3 files | Custom script to update all versions | Manual sync + grep verification | Only 3 files, infrequent changes, low complexity |
| Git merge conflict resolution | Custom merge driver | git mergetool or manual edit | Standard git workflow, one-time operation |
| Platform detection at runtime | Custom env checks | `#[cfg(target_os)]` + navigator.userAgent | Language-native, compile-time optimization |
| Windows installer packaging | Custom NSIS template | Tauri's built-in NSIS bundler | Already configured, handles WebView2 bootstrapping |
| Cross-platform builds on single machine | Custom Docker/VM setup | Native builds on each platform OR GitHub Actions | Tauri requires native toolchains, can't meaningfully cross-compile |

**Key insight:** Tauri's architecture requires native compilation on target platforms. The framework handles bundler differences (DMG on macOS, NSIS on Windows) automatically via `tauri build` with platform-specific configuration in tauri.conf.json.

## Common Pitfalls

### Pitfall 1: Version Desync Across Configuration Files

**What goes wrong:** Tauri apps have 3 version sources: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`. If these drift, builds may succeed but installers show wrong versions or updates fail.

**Why it happens:** No built-in sync mechanism in Tauri. Each file is manually maintained. The windows branch bumped all three to v0.2.1, but main is at v0.1.1 in Cargo.toml/tauri.conf.json while package.json is already v0.2.1.

**How to avoid:**
- Choose single source of truth (recommend: Cargo.toml per Tauri docs)
- Manually sync all three before merge finalization
- Add grep check: `grep -r "0\.1\.1" package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json` should return nothing after merge

**Warning signs:**
- DMG/NSIS installer shows v0.1.1 after supposedly building v0.2.1
- GitHub release upload picks wrong version tag
- Grep for version shows inconsistent results

**Sources:**
- [Tauri GitHub Discussion #6347](https://github.com/orgs/tauri-apps/discussions/6347) - Users requesting version sync automation
- [Tauri GitHub Issue #8265](https://github.com/tauri-apps/tauri/issues/8265) - Feature request for sync between Cargo.toml, package.json, tauri.conf.json

### Pitfall 2: Platform-Specific Asset Loss During Merge

**What goes wrong:** The windows branch development didn't need `scripts/dmg-background.png` (macOS-only asset), so it was deleted. A naive merge accepting all windows changes would delete this file, breaking the macOS DMG build pipeline.

**Why it happens:** Git sees deletion on one branch, no change on other. Merge may auto-resolve to deletion depending on strategy. The `build-dmg.sh` script expects `dmg-background.png` to exist for the styled DMG window background.

**How to avoid:**
- Inspect `git diff main...windows --name-status` for deletions before merging
- Verify `scripts/dmg-background.png` survives the merge
- Current state: File exists on main, NOT on windows branch
- Must ensure merge keeps the main branch version

**Warning signs:**
- `git diff --name-status` shows `D scripts/dmg-background.png`
- Running `build-dmg.sh` after merge fails with "background image not found"
- DMG builds but lacks styled Finder window

### Pitfall 3: cfg Gate Scope Errors

**What goes wrong:** Forgetting to gate platform-specific imports or misplacing `#[cfg]` attributes can cause compilation failures on the opposite platform.

**Why it happens:** Rust compiles all code unless explicitly told not to. An ungated `use accessibility_sys::*` compiles fine on macOS but fails on Windows where the crate isn't available.

**How to avoid:**
- All platform-specific imports need `#[cfg(target_os)]` gates
- Use `#[allow(dead_code)]` for modules with platform-specific constants compiled everywhere but only used on one platform
- Audit pattern: `grep -r "accessibility_sys\|windows_sys\|uiautomation" src-tauri/src/` should show `#[cfg]` guard above each usage

**Example from windows branch:**
```rust
// CORRECT - gated import
#[cfg(target_os = "macos")]
use accessibility_sys::*;

// CORRECT - module compiled everywhere, but only used on Windows
#[allow(dead_code)]
pub mod detect_windows;

// CORRECT - Windows-only module
#[cfg(target_os = "windows")]
pub mod uia_reader;
```

**Warning signs:**
- Build fails on opposite platform: "crate not found: accessibility_sys"
- Clippy warnings about unused imports on one platform
- Dead code warnings for platform-specific utilities

**Sources:**
- [Rust Reference: Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Sling Academy: cfg Attributes in Rust Modules](https://www.slingacademy.com/article/handling-platform-specific-code-with-cfg-attributes-in-rust-modules/)

### Pitfall 4: Merge Conflict in showcase/index.html

**What goes wrong:** Both branches modified `showcase/index.html`. Main added DMG download link (v0.2.1-beta), windows generalized text and added Windows installer link. Git can't auto-resolve because same lines changed.

**Why it happens:** HTML download section was the natural place to add platform-specific installers on both branches. Standard merge conflict.

**How to avoid:** N/A - this is expected. Resolution strategy is to combine both:
- Keep macOS DMG link from main (with updated v0.2.1 URL)
- Keep Windows installer link from windows
- Keep generalized text ("from anywhere on your desktop" instead of "on your Mac")
- Result: Cross-platform download page with both installer types

**Conflict preview:**
```diff
- macOS only text: "from anywhere on your Mac"
+ generalized text: "from anywhere on your desktop"

- Single DMG download button
+ Two download buttons: DMG for macOS + NSIS for Windows
```

**Resolution approach:** Accept both changes, producing a combined download section with platform-specific buttons side-by-side.

**Warning signs:**
- `git status` shows "both modified: showcase/index.html"
- File contains `<<<<<<<`, `=======`, `>>>>>>>` conflict markers

### Pitfall 5: NSIS Configuration Loss

**What goes wrong:** The windows branch added `bundle.windows.nsis` configuration to `tauri.conf.json`. If main's version of tauri.conf.json is accepted during merge, Windows builds will fail or use incorrect installer settings.

**Why it happens:** tauri.conf.json on main doesn't have Windows bundle config. Windows branch added:
```json
"windows": {
  "nsis": {
    "installMode": "currentUser"
  },
  "webviewInstallMode": {
    "type": "embedBootstrapper"
  }
}
```

**How to avoid:**
- Accept windows branch version of `tauri.conf.json` bundle section
- Only update version field if windows branch is still at older version
- Verify NSIS config survives: `grep -A5 '"windows"' src-tauri/tauri.conf.json` should show nsis block

**Warning signs:**
- Windows build produces unsigned/system-wide installer instead of currentUser
- WebView2 not embedded, requiring separate download on clean Windows machines
- `tauri build` on Windows fails with missing bundle config

**Sources:**
- [Tauri Windows Installer Docs](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri NsisConfig Rust Docs](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.NsisConfig.html)

### Pitfall 6: Incomplete Build Script Coverage

**What goes wrong:** After merge, developers expect `build:mac` and `build:windows` npm scripts but only find `tauri build` in package.json. No clear cross-platform build workflow.

**Why it happens:** Main branch has standalone `scripts/build-dmg.sh` and `scripts/build-pkg.sh` but no npm script wrappers. Windows has no equivalent build script at all (relies on `tauri build` directly).

**How to avoid:**
- Add Windows build script: `scripts/build-windows.sh` (calls `pnpm tauri build` + NSIS-specific steps if needed)
- Add npm scripts to package.json:
  ```json
  "scripts": {
    "build:mac": "./scripts/build-dmg.sh",
    "build:windows": "./scripts/build-windows.sh"
  }
  ```
- Document platform-specific build requirements in each script

**Warning signs:**
- No clear command for building Windows installer
- Developers run `tauri build` directly, bypassing any necessary pre/post steps
- Inconsistent build artifacts between macOS and Windows

## Code Examples

Verified patterns from the project's windows branch:

### Platform-Branched Setup (lib.rs)

```rust
// Source: windows branch src-tauri/src/lib.rs
pub fn run() {
    let mut builder = tauri::Builder::default();

    // NSPanel plugin for floating overlay above fullscreen apps (macOS only)
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        // ... other shared plugins
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            let window = app.get_webview_window("main")
                .expect("Window 'main' should exist per tauri.conf.json");

            #[cfg(target_os = "macos")]
            {
                apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))
                    .expect("Failed to apply NSVisualEffectView vibrancy");

                let panel = window.to_panel::<OverlayPanel>()
                    .expect("Failed to convert window to NSPanel");
                panel.set_level(PanelLevel::Status.value());
                // ... more NSPanel setup
            }

            #[cfg(target_os = "windows")]
            {
                // Windows-specific window setup
                use raw_window_handle::HasWindowHandle;
                let handle = window.window_handle().unwrap();
                // ... Win32 API calls for WS_EX_TOOLWINDOW, etc.
            }

            Ok(())
        })
        // ... invoke handlers
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Platform-Specific Command Implementations

```rust
// Source: windows branch src-tauri/src/commands/window.rs
#[tauri::command]
pub fn show_overlay(app: AppHandle) -> Result<(), String> {
    position_overlay(&app)?;

    // macOS: use NSPanel show_and_make_key for non-activating overlay
    #[cfg(target_os = "macos")]
    {
        let panel = app.get_webview_panel("main")
            .map_err(|e| format!("Panel 'main' not found: {:?}", e))?;
        panel.show_and_make_key();
    }

    // Non-macOS: use standard Tauri window show + focus
    #[cfg(not(target_os = "macos"))]
    {
        let window = app.get_webview_window("main")
            .ok_or_else(|| "Window 'main' not found".to_string())?;
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }

    // Shared: update state and emit event
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut visible) = state.overlay_visible.lock() {
            *visible = true;
        }
    }
    let _ = app.emit("overlay-shown", ());
    Ok(())
}
```

### Resolving showcase/index.html Conflict

**Main branch (v0.2.1 macOS-only):**
```html
<div class="hero-buttons reveal reveal-delay-3">
  <a href="https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v0.2.1-beta/CMD+K-0.2.1-universal.dmg"
     target="_blank" rel="noopener" class="btn btn-primary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- download icon --></svg>
    Download for macOS
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K"
     target="_blank" rel="noopener" class="btn btn-secondary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- GitHub icon --></svg>
    View on GitHub
  </a>
</div>
```

**Windows branch (cross-platform):**
```html
<div class="hero-buttons reveal reveal-delay-3">
  <a href="https://github.com/LakshmanTurlapati/Cmd-K/releases"
     target="_blank" rel="noopener" class="btn btn-primary">
    <svg class="btn-icon" viewBox="0 0 814 1000"><!-- macOS icon --></svg>
    Download for macOS
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K/releases"
     target="_blank" rel="noopener" class="btn btn-primary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- Windows icon --></svg>
    Download for Windows
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K"
     target="_blank" rel="noopener" class="btn btn-secondary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- GitHub icon --></svg>
    View on GitHub
  </a>
</div>
```

**Merged resolution (combine both platforms):**
```html
<div class="hero-buttons reveal reveal-delay-3">
  <a href="https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v0.2.1-beta/CMD+K-0.2.1-universal.dmg"
     target="_blank" rel="noopener" class="btn btn-primary">
    <svg class="btn-icon" viewBox="0 0 814 1000"><!-- macOS icon --></svg>
    Download for macOS
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K/releases"
     target="_blank" rel="noopener" class="btn btn-primary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- Windows icon --></svg>
    Download for Windows
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K"
     target="_blank" rel="noopener" class="btn btn-secondary">
    <svg class="btn-icon" viewBox="0 0 24 24"><!-- GitHub icon --></svg>
    View on GitHub
  </a>
</div>
```

### Version Alignment Check

```bash
# Verify all three files show v0.2.1
grep '"version"' package.json                    # Should show: "version": "0.2.1"
grep '^version' src-tauri/Cargo.toml             # Should show: version = "0.2.1"
grep '"version"' src-tauri/tauri.conf.json       # Should show: "version": "0.2.1"

# Catch any stragglers
grep -r "0\.1\.1" package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
# Should return: (nothing)
```

### Platform-Specific Dependency Verification

```bash
# Check that platform-specific deps are properly gated
grep -A10 "^\[target\.'cfg" src-tauri/Cargo.toml

# Expected output should show:
# [target.'cfg(target_os = "macos")'.dependencies]
# tauri-nspanel = ...
# accessibility-sys = ...
#
# [target.'cfg(target_os = "windows")'.dependencies]
# windows-sys = ...
# uiautomation = ...
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Tauri v1 | Tauri v2 | 2024 | New plugin system, improved cross-platform APIs |
| Manual version sync | Still manual (feature request open) | N/A | No built-in sync yet, requires manual/scripted updates |
| Cross-compilation attempts | Native builds on each platform | Ongoing | Tauri requires native toolchains, can't meaningfully cross-compile |
| Single DMG bundler | Platform-specific bundlers (DMG, NSIS, AppImage) | Tauri v1+ | Automated installer creation per platform |

**Deprecated/outdated:**
- **Tauri v1 plugin API:** Replaced with Tauri v2 plugin system (Builder pattern)
- **Ad-hoc platform checks in Rust:** Use `#[cfg]` compile-time gates, not runtime `cfg!()`
- **Storing API keys in localStorage:** Use system keychain (`keyring` crate with platform-specific features)

## Open Questions

1. **Should we create build-windows.sh or rely on npm scripts calling tauri build directly?**
   - What we know: macOS has dedicated build-dmg.sh with notarization, signing, DMG styling
   - What's unclear: Windows NSIS build may not need equivalent complexity (Tauri handles most of it)
   - Recommendation: Start with simple `build-windows.sh` wrapper around `pnpm tauri build`, add complexity only if needed (signing, store submission, etc.)

2. **Do we need to update build-pkg.sh or is it deprecated in favor of build-dmg.sh?**
   - What we know: build-pkg.sh creates unsigned .pkg, build-dmg.sh creates signed+notarized DMG
   - What's unclear: Is .pkg distribution still needed?
   - Recommendation: Leave build-pkg.sh as-is (may be useful for enterprise distribution), primary flow is DMG

3. **Should the merged version be v0.2.1 or v0.3.0?**
   - What we know: User decision specifies v0.2.1 from windows branch
   - What's unclear: Whether adding cross-platform support warrants minor version bump
   - Recommendation: Follow user decision - use v0.2.1 as specified in CONTEXT.md

## Verification Checklist

Post-merge validation steps:

### Version Consistency
- [ ] `package.json` shows `"version": "0.2.1"`
- [ ] `src-tauri/Cargo.toml` shows `version = "0.2.1"`
- [ ] `src-tauri/tauri.conf.json` shows `"version": "0.2.1"`
- [ ] `grep -r "0\.1\.1" package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json` returns nothing

### Asset Preservation
- [ ] `scripts/dmg-background.png` exists (1175 bytes)
- [ ] `scripts/build-dmg.sh` exists and references dmg-background.png
- [ ] `scripts/build-pkg.sh` exists (unchanged from main)

### Platform Isolation
- [ ] `git grep -n "accessibility_sys" src-tauri/src/` shows `#[cfg(target_os = "macos")]` guard
- [ ] `git grep -n "windows_sys" src-tauri/src/` shows `#[cfg(target_os = "windows")]` guard
- [ ] `git grep -n "uiautomation" src-tauri/src/` shows `#[cfg(target_os = "windows")]` guard
- [ ] `src-tauri/Cargo.toml` has `[target.'cfg(target_os = "macos")'.dependencies]` section
- [ ] `src-tauri/Cargo.toml` has `[target.'cfg(target_os = "windows")'.dependencies]` section

### Windows-Specific Features
- [ ] `src-tauri/src/terminal/detect_windows.rs` exists
- [ ] `src-tauri/src/terminal/uia_reader.rs` exists with `#[cfg(target_os = "windows")]`
- [ ] `src-tauri/tauri.conf.json` has `bundle.windows.nsis` configuration
- [ ] `src/utils/platform.ts` exists with `isWindows()` and `displayModifier()` functions

### Build Scripts
- [ ] `package.json` has `build:mac` script pointing to `./scripts/build-dmg.sh`
- [ ] `package.json` has `build:windows` script (either pointing to script or inline `tauri build`)
- [ ] Windows build script exists OR npm script documented for Windows builds

### Showcase Page
- [ ] `showcase/index.html` has both macOS and Windows download buttons
- [ ] Text generalized to "from anywhere on your desktop" (not "on your Mac")
- [ ] macOS button links to DMG (v0.2.1-beta release)
- [ ] Windows button links to releases page or specific NSIS installer

### Branch Cleanup
- [ ] Local `windows` branch deleted: `git branch -d windows`
- [ ] Remote `windows` branch deleted: `git push origin --delete windows`

## Sources

### Primary (HIGH confidence)

- [Git Documentation - merge-strategies](https://git-scm.com/docs/merge-strategies)
- [Rust Reference: Conditional Compilation](https://doc.rust-lang.org/reference/conditional-compilation.html)
- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri Windows Installer Configuration](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri Configuration Files](https://v2.tauri.app/develop/configuration-files/)
- Project source code inspection: windows branch at commit dddc49e, main branch at current HEAD

### Secondary (MEDIUM confidence)

- [Atlassian: Git Merge Conflicts Tutorial](https://www.atlassian.com/git/tutorials/using-branches/merge-conflicts)
- [GitHub Docs: Resolving Merge Conflicts](https://docs.github.com/articles/resolving-a-merge-conflict-using-the-command-line)
- [Sling Academy: cfg Attributes in Rust Modules](https://www.slingacademy.com/article/handling-platform-specific-code-with-cfg-attributes-in-rust-modules/)
- [Tauri GitHub Discussion #6347](https://github.com/orgs/tauri-apps/discussions/6347) - Version sync discussion
- [Tauri GitHub Issue #8265](https://github.com/tauri-apps/tauri/issues/8265) - Version sync feature request
- [Tauri GitHub Discussion #9650](https://github.com/orgs/tauri-apps/discussions/9650) - Cross-platform builds from single machine
- [Tauri NsisConfig Rust Docs](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.NsisConfig.html)

### Tertiary (LOW confidence)

- Web search results for "git merge conflict resolution 2026" - General best practices
- Web search results for "Rust cfg target_os patterns 2026" - Community patterns

## Metadata

**Confidence breakdown:**
- Git merge strategy: HIGH - Standard git operation with clear conflict (showcase/index.html)
- Platform isolation patterns: HIGH - Verified by inspecting windows branch code directly
- Version alignment: HIGH - File contents examined on both branches
- Build scripts: HIGH - Verified existence and contents on main branch, absence on windows branch
- Windows NSIS config: HIGH - Verified in windows branch tauri.conf.json

**Research date:** 2026-03-03
**Valid until:** 2026-04-03 (30 days - stable domain, git/Tauri patterns change slowly)
