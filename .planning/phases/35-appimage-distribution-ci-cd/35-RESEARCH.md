# Phase 35: AppImage Distribution & CI/CD - Research

**Researched:** 2026-03-15
**Domain:** Linux AppImage packaging, GitHub Actions CI/CD, Tauri v2 updater
**Confidence:** HIGH

## Summary

This phase adds a Linux AppImage build job to the existing release workflow, integrates Linux into the `latest.json` updater manifest, and includes AppImage artifacts in GitHub Releases. The core technology is well-understood: Tauri v2 natively supports AppImage bundling via `linuxdeploy`, the updater plugin already handles Linux platform keys (`linux-x86_64`, `linux-aarch64`), and the existing `release.yml` is cleanly structured for adding a third platform job.

The main complexity lies in (1) cross-architecture builds -- linuxdeploy cannot cross-compile AppImages, so aarch64 must be built on a native ARM runner (`ubuntu-22.04-arm`, free for public repos since August 2025), and (2) the `NO_STRIP=true` environment variable required to work around linuxdeploy's incompatibility with modern `.relr.dyn` ELF sections on Ubuntu 22.04. The updater integration is straightforward: with `createUpdaterArtifacts: true` already set, Tauri generates `.AppImage.sig` files that slot directly into `latest.json` using `linux-x86_64` and `linux-aarch64` platform keys.

**Primary recommendation:** Use a build matrix with `ubuntu-22.04` (x86_64) and `ubuntu-22.04-arm` (aarch64) runners, set `NO_STRIP=true` to avoid linuxdeploy strip failures, and extend the existing `latest.json` assembly to include Linux platform entries.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Naming convention: `CMD+K-{version}-linux-x86_64.AppImage` and `CMD+K-{version}-linux-aarch64.AppImage`
- AppImage only -- no .deb/.rpm
- Updater artifacts (`.tar.gz` + `.sig` or `.AppImage` + `.sig`) follow same pattern as macOS/Windows, referenced by `latest.json`
- Replace-in-place on quit -- same mechanism as macOS (Tauri updater overwrites AppImage file)
- If AppImage location is not writable: warn and skip update (tray message, don't attempt write)
- Auto-update enabled by default on Linux
- x86_64 AND aarch64 targets -- cross-compile aarch64 on x86_64 runner
- Ubuntu 22.04 runner for glibc compatibility floor
- System packages via `apt-get install` in workflow (no caching)
- Linux build job blocks release -- `release` job `needs: [build-macos, build-windows, build-linux]`
- Restructure release body into OS-grouped sections (macOS, Windows, Linux)
- One-liner chmod hint for Linux
- Auto-update note updated to mention Linux

### Claude's Discretion
- Exact apt package list for Tauri + zbus build dependencies
- Cross-compilation toolchain setup for aarch64 (gcc-aarch64-linux-gnu, pkg-config configuration)
- AppImage desktop file and icon integration details
- Tauri bundler configuration specifics for AppImage target
- How to handle FUSE requirement for AppImage execution

### Deferred Ideas (OUT OF SCOPE)
- Showcase site updates -- separate phase after milestone completion
- .deb/.rpm packages -- out of scope per REQUIREMENTS.md
- Snap/Flatpak packaging -- sandboxing conflicts with /proc access and xdotool
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| APKG-01 | AppImage built via Tauri bundler with ubuntu-22.04 CI base for glibc compatibility | System deps list, NO_STRIP workaround, Tauri bundle config, glibc floor guidance |
| APKG-02 | Third CI job in release.yml builds Linux AppImage alongside macOS DMG and Windows NSIS | Build matrix pattern, artifact upload structure, runner labels for x86_64 and aarch64 |
| APKG-03 | Auto-updater supports Linux AppImage (Ed25519 signed, latest.json manifest) | Platform key names, updater artifact paths, latest.json assembly extension |
| APKG-04 | GitHub Release includes Linux AppImage artifact with SHA256 checksum | Release body restructure, artifact glob patterns, checksum generation |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri v2 bundler | 2.x | AppImage generation via linuxdeploy | Already configured, `bundle.targets: "all"` includes AppImage |
| tauri-plugin-updater | 2.x | Auto-update with Ed25519 signing | Already in use for macOS/Windows |
| linuxdeploy | latest | AppImage assembly tool (downloaded by Tauri) | Tauri's built-in AppImage toolchain |

### Supporting
| Tool | Purpose | When to Use |
|------|---------|-------------|
| `NO_STRIP=true` env var | Bypass linuxdeploy strip failures | Always in CI (Ubuntu 22.04 `.relr.dyn` issue) |
| `APPIMAGE_EXTRACT_AND_RUN=1` env var | Run AppImage without FUSE | During CI build steps if linuxdeploy itself needs to run |
| `patchelf` | ELF binary manipulation | Required by Tauri AppImage bundling |
| `ubuntu-22.04-arm` runner | Native aarch64 builds | aarch64 AppImage (linuxdeploy cannot cross-compile) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Native ARM runner | QEMU emulation (pguyot/arm-runner-action) | ~1 hour build time vs ~10 min; native runner is free for public repos |
| linuxdeploy cross-compile | N/A | Not supported -- linuxdeploy requires same-arch build |

**System packages (apt-get) for Ubuntu 22.04:**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libxdo-dev \
  libssl-dev \
  libdbus-1-dev \
  build-essential \
  curl \
  wget \
  file
```

Note: `libdbus-1-dev` is needed for the `zbus` crate (D-Bus AT-SPI2 integration from Phase 34). `libxdo-dev` is needed for xdotool paste integration from Phase 32.

## Architecture Patterns

### CI Workflow Structure

The existing `release.yml` has two parallel build jobs feeding a `release` job. Add a third `build-linux` job with a matrix for architecture:

```yaml
build-linux:
  strategy:
    matrix:
      include:
        - arch: x86_64
          runner: ubuntu-22.04
          rust-target: x86_64-unknown-linux-gnu
        - arch: aarch64
          runner: ubuntu-22.04-arm
          rust-target: aarch64-unknown-linux-gnu
  runs-on: ${{ matrix.runner }}
```

### Artifact Upload Pattern (matching existing macOS/Windows)
```
artifacts/
  linux-appimage-x86_64/
    CMD+K-{version}-linux-x86_64.AppImage
    CMD+K-{version}-linux-x86_64.AppImage.sha256
    CMD+K-{version}-linux-x86_64.AppImage.sig       # updater signature
  linux-appimage-aarch64/
    CMD+K-{version}-linux-aarch64.AppImage
    CMD+K-{version}-linux-aarch64.AppImage.sha256
    CMD+K-{version}-linux-aarch64.AppImage.sig       # updater signature
```

### latest.json Extension
```json
{
  "version": "0.3.9",
  "platforms": {
    "darwin-aarch64": { "signature": "...", "url": "..." },
    "darwin-x86_64": { "signature": "...", "url": "..." },
    "windows-x86_64": { "signature": "...", "url": "..." },
    "linux-x86_64": {
      "signature": "<contents of .AppImage.sig>",
      "url": "https://github.com/.../CMD+K-0.3.9-linux-x86_64.AppImage"
    },
    "linux-aarch64": {
      "signature": "<contents of .AppImage.sig>",
      "url": "https://github.com/.../CMD+K-0.3.9-linux-aarch64.AppImage"
    }
  }
}
```

### Release Body Restructure
```markdown
## CMD+K vX.Y.Z

### Downloads

**macOS**
| Variant | File |
|---------|------|
| Universal (Intel + Apple Silicon) | CMD+K-X.Y.Z-universal.dmg |

**Windows**
| Variant | File |
|---------|------|
| x64 | CMD+K-X.Y.Z-windows-x64.exe |

**Linux**
| Variant | File |
|---------|------|
| x86_64 | CMD+K-X.Y.Z-linux-x86_64.AppImage |
| aarch64 | CMD+K-X.Y.Z-linux-aarch64.AppImage |

> Linux: `chmod +x CMD+K-*.AppImage && ./CMD+K-*.AppImage`

### Auto-Update
Users on v0.3.9+ will receive this update automatically (macOS, Windows, Linux).

### Checksums (SHA256)
See the `.sha256` files attached to this release.
```

### Anti-Patterns to Avoid
- **Cross-compiling AppImage on x86_64 for aarch64:** linuxdeploy does not support this. Use native ARM runners instead.
- **Building on Ubuntu 24.04:** Would raise the glibc floor and break compatibility with Ubuntu 22.04 systems.
- **Caching apt packages:** User decision says no caching for system deps. `apt-get install` is fast enough.
- **Using `tauri-apps/tauri-action`:** The existing workflow uses direct `pnpm tauri build` calls, not the action. Stay consistent.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AppImage packaging | Custom AppDir assembly | Tauri bundler (`pnpm tauri build`) | Handles linuxdeploy, desktop file, icon, AppRun automatically |
| Update signature | Custom signing | `TAURI_SIGNING_PRIVATE_KEY` env var | Tauri generates `.sig` files automatically during build |
| Update delivery | Custom update server | Static `latest.json` on GitHub Releases | Already working for macOS/Windows |
| Desktop integration | Custom .desktop file | Tauri auto-generates from `tauri.conf.json` | productName, identifier, icon all sourced from config |

**Key insight:** Tauri's AppImage bundler handles all the complex linuxdeploy orchestration, desktop entry generation, and icon embedding. The CI job just needs to run `pnpm tauri build` with the right environment variables and system packages installed.

## Common Pitfalls

### Pitfall 1: linuxdeploy Strip Failure
**What goes wrong:** Build fails with `eu-strip: unknown type [0x13] section '.relr.dyn'`
**Why it happens:** linuxdeploy's bundled `eu-strip` cannot handle modern ELF relocation sections present in Ubuntu 22.04 system libraries
**How to avoid:** Set `NO_STRIP=true` environment variable in the build step
**Warning signs:** Build succeeds locally but fails in CI, or error message mentioning `.relr.dyn`

### Pitfall 2: FUSE Not Available in CI
**What goes wrong:** linuxdeploy or AppImage tools fail because FUSE is not installed on the CI runner
**Why it happens:** GitHub Actions runners don't have FUSE by default; AppImage tools use FUSE to mount themselves
**How to avoid:** Set `APPIMAGE_EXTRACT_AND_RUN=1` environment variable so tools extract-and-run instead of FUSE-mount
**Warning signs:** Error messages about `fusermount` or `libfuse`

### Pitfall 3: Wrong Updater Artifact Format
**What goes wrong:** Updater finds the artifact but fails to install
**Why it happens:** Mismatch between `createUpdaterArtifacts: true` (v2 format: `.AppImage` + `.AppImage.sig`) and v1 format (`.AppImage.tar.gz` + `.tar.gz.sig`). The `latest.json` URL must point to the correct artifact.
**How to avoid:** With `createUpdaterArtifacts: true` (already set), the updater URL in `latest.json` should point to the `.AppImage` file directly, and the signature is from `.AppImage.sig`
**Warning signs:** Updater reports signature mismatch or download failure

### Pitfall 4: Artifact Name Contains Special Characters
**What goes wrong:** Upload/download artifact actions fail or mangle filenames
**Why it happens:** The `+` in `CMD+K` may cause issues in some shell contexts or URL encoding
**How to avoid:** Test artifact names in CI; the existing macOS/Windows builds already handle `CMD+K` naming so this should work, but verify Linux paths too
**Warning signs:** File not found errors during release assembly

### Pitfall 5: aarch64 Runner Availability
**What goes wrong:** Workflow queues indefinitely or fails with "no runner available"
**Why it happens:** `ubuntu-22.04-arm` runners are only available for public repositories
**How to avoid:** Verify the repository is public; if private, would need self-hosted ARM runner or QEMU emulation
**Warning signs:** Job stuck in "queued" state

### Pitfall 6: Missing Linux-Specific System Dependencies
**What goes wrong:** Cargo build fails compiling zbus, x11rb, or other Linux-specific crates
**Why it happens:** Missing `-dev` packages for D-Bus, X11, etc.
**How to avoid:** Include `libdbus-1-dev` (for zbus), `libxdo-dev` (for xdotool paste), and all Tauri prerequisites
**Warning signs:** Compilation errors mentioning missing `.h` files or pkg-config failures

## Code Examples

### Build Step for Linux AppImage
```yaml
# Source: Tauri v2 docs + existing project patterns
- name: Build Tauri app
  env:
    TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
    TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
    NO_STRIP: "true"
    APPIMAGE_EXTRACT_AND_RUN: "1"
  run: pnpm tauri build
```

### Rename AppImage Artifact
```bash
# Source: existing Windows rename pattern in release.yml
APPIMAGE_DIR="src-tauri/target/release/bundle/appimage"
ORIG_APPIMAGE=$(ls "$APPIMAGE_DIR"/*.AppImage | grep -v '.sig' | head -1)
ORIG_SIG="${ORIG_APPIMAGE}.sig"
mv "$ORIG_APPIMAGE" "$APPIMAGE_DIR/CMD+K-${VERSION}-linux-x86_64.AppImage"
if [ -f "$ORIG_SIG" ]; then
  mv "$ORIG_SIG" "$APPIMAGE_DIR/CMD+K-${VERSION}-linux-x86_64.AppImage.sig"
fi
```

### Checksum Generation (Linux)
```bash
# Source: existing pattern from macOS/Windows jobs
cd src-tauri/target/release/bundle/appimage
sha256sum "CMD+K-${VERSION}-linux-x86_64.AppImage" > "CMD+K-${VERSION}-linux-x86_64.AppImage.sha256"
```

### latest.json Assembly Extension
```bash
# Read Linux signatures
SIG_LINUX_X64=""
if [ -f "artifacts/linux-appimage-x86_64/CMD+K-${VERSION}-linux-x86_64.AppImage.sig" ]; then
  SIG_LINUX_X64=$(cat "artifacts/linux-appimage-x86_64/CMD+K-${VERSION}-linux-x86_64.AppImage.sig")
fi

SIG_LINUX_ARM64=""
if [ -f "artifacts/linux-appimage-aarch64/CMD+K-${VERSION}-linux-aarch64.AppImage.sig" ]; then
  SIG_LINUX_ARM64=$(cat "artifacts/linux-appimage-aarch64/CMD+K-${VERSION}-linux-aarch64.AppImage.sig")
fi

# Add to platforms object:
# "linux-x86_64": { "signature": "${SIG_LINUX_X64}", "url": "${REPO_URL}/CMD+K-${VERSION}-linux-x86_64.AppImage" }
# "linux-aarch64": { "signature": "${SIG_LINUX_ARM64}", "url": "${REPO_URL}/CMD+K-${VERSION}-linux-aarch64.AppImage" }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| QEMU emulation for ARM builds | Native `ubuntu-22.04-arm` runners | Aug 2025 GA | ~10 min builds instead of ~60 min |
| linuxdeploy with strip | `NO_STRIP=true` workaround | 2024-2025 | Required on modern Ubuntu due to `.relr.dyn` sections |
| `createUpdaterArtifacts: "v1Compatible"` | `createUpdaterArtifacts: true` | Tauri v2 | `.AppImage` + `.sig` directly (no `.tar.gz` wrapper) |
| FUSE required for AppImage | `APPIMAGE_EXTRACT_AND_RUN=1` fallback | AppImage standard | Allows running without FUSE in CI and on systems without it |

**Note:** Tauri team is developing a replacement for linuxdeploy (PR #12491) to address fundamental compatibility issues, but it is not yet available. Use the `NO_STRIP=true` workaround for now.

## Open Questions

1. **Exact updater artifact format with `createUpdaterArtifacts: true` on Linux**
   - What we know: Tauri generates `.AppImage` + `.AppImage.sig` (v2 format) when set to `true`
   - What's unclear: Whether the updater URL in `latest.json` should point to the raw `.AppImage` or a `.tar.gz` archive
   - Recommendation: Build locally on Linux first to inspect exact output files in `target/release/bundle/appimage/`, then match `latest.json` URLs accordingly. HIGH confidence this is just `.AppImage` + `.sig` based on docs.

2. **Write-permission check for AppImage updates**
   - What we know: User wants warn-and-skip if AppImage location is not writable
   - What's unclear: Whether Tauri's updater plugin already handles this gracefully or if custom code is needed in `updater.rs`
   - Recommendation: Test the updater behavior when AppImage is in a read-only location. If it throws an error, catch it in `install_pending_update` and surface via tray. Likely LOW effort since error handling already exists.

3. **Desktop file and icon in AppImage**
   - What we know: Tauri auto-generates `.desktop` file from `tauri.conf.json` (productName, identifier)
   - What's unclear: Whether existing icons in `src-tauri/icons/` are sufficient or if a specific size/format is needed for AppImage
   - Recommendation: The existing `128x128.png` and `32x32.png` icons should work. Tauri picks the appropriate one. Verify after first build.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 AppImage docs](https://v2.tauri.app/distribute/appimage/) - Configuration, FUSE, glibc floor guidance
- [Tauri v2 Updater plugin](https://v2.tauri.app/plugin/updater/) - Platform keys, artifact format, latest.json structure
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/) - System package list for Ubuntu
- [Tauri v2 GitHub pipelines](https://v2.tauri.app/distribute/pipelines/github/) - CI workflow patterns
- [GitHub ARM runners GA](https://github.blog/changelog/2025-08-07-arm64-hosted-runners-for-public-repositories-are-now-generally-available/) - `ubuntu-22.04-arm` availability
- Existing `release.yml` in project - Current CI patterns to extend

### Secondary (MEDIUM confidence)
- [Tauri issue #14796](https://github.com/tauri-apps/tauri/issues/14796) - linuxdeploy strip failure and NO_STRIP workaround
- [linuxdeploy issue #175](https://github.com/linuxdeploy/linuxdeploy/issues/175) - aarch64 support status

### Tertiary (LOW confidence)
- Exact updater artifact filenames for Linux (needs local build verification)
- Write-permission error handling behavior in Tauri updater on Linux

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Tauri v2 AppImage bundling is well-documented and the project already has the updater configured
- Architecture: HIGH - Extending existing CI pattern is straightforward; native ARM runners eliminate cross-compile complexity
- Pitfalls: HIGH - The linuxdeploy strip issue and FUSE requirement are well-documented with known workarounds

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable -- Tauri v2 bundler and GitHub ARM runners are GA)
