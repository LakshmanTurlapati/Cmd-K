# Windows CI Build Plan

CMD+K v0.2.1 — Windows NSIS Installer via GitHub Actions

## Background

The `windows` branch is code-complete with ~1,695 lines of Windows-specific Rust across 14 files. It depends heavily on Win32-native APIs that **cannot be cross-compiled from macOS**:

| Crate | Purpose |
|-------|---------|
| `windows-sys` | Win32 API bindings (GetForegroundWindow, AttachThreadInput, SendInput, etc.) |
| `uiautomation` | COM-based UI Automation for reading terminal text |
| `arboard` | Windows clipboard access |
| `raw-window-handle` | HWND extraction |

All of these require the MSVC linker and Windows SDK. Tauri's docs call cross-compilation via `cargo-xwin` a "last resort" and it is unreliable with this level of Win32 dependency. The solution is to build natively on a Windows runner via GitHub Actions CI.

---

## Step 1: Merge `windows` Branch to `main`

The `windows` branch has already been merged locally. Confirm that `main` contains all Windows-specific code before setting up CI:

```bash
# Verify the merge is complete
git log --oneline main | grep -i windows

# Confirm platform-conditional dependencies in Cargo.toml
grep -A 20 'target_os = "windows"' src-tauri/Cargo.toml

# Confirm NSIS config in tauri.conf.json
grep -A 5 '"nsis"' src-tauri/tauri.conf.json
```

Expected state on `main`:
- `src-tauri/Cargo.toml` has `[target.'cfg(target_os = "windows")'.dependencies]` with `windows-sys`, `uiautomation`, `arboard`, `raw-window-handle`, `keyring`
- `src-tauri/tauri.conf.json` has `bundle.windows.nsis.installMode: "currentUser"` and `bundle.windows.webviewInstallMode.type: "embedBootstrapper"`
- All Windows Rust source files are present under `src-tauri/src/`

---

## Step 2: GitHub Actions Workflow

Create `.github/workflows/build-windows.yml`:

```yaml
name: Build Windows Installer

on:
  push:
    branches: [main]
    paths:
      - 'src-tauri/**'
      - 'src/**'
      - 'package.json'
      - 'pnpm-lock.yaml'
      - '.github/workflows/build-windows.yml'
  workflow_dispatch:

jobs:
  build-windows:
    runs-on: windows-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: src-tauri

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: latest

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - name: Install frontend dependencies
        run: pnpm install

      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # Uncomment when code signing is configured:
          # TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          # TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}

      - name: Upload NSIS installer
        uses: actions/upload-artifact@v4
        with:
          name: cmd-k-windows-installer
          path: src-tauri/target/release/bundle/nsis/*.exe
          if-no-files-found: error
```

### Key Details

- **Runner**: `windows-latest` — free for public repos, includes MSVC toolchain and Windows SDK
- **Rust caching**: `swatinem/rust-cache` speeds up subsequent builds significantly
- **Trigger**: Pushes to `main` that touch relevant source files, plus manual `workflow_dispatch`
- **Output**: NSIS `.exe` installer uploaded as a build artifact
- **Build time**: Expect ~10-15 minutes for the first run, ~5-8 minutes with cache hits

---

## Step 3: Windows Code Signing (Future)

Windows signing is simpler than macOS — no notarization step is required.

### Tier 1: Unsigned (Current)

- `pnpm tauri build` produces an unsigned NSIS installer
- Users see a SmartScreen warning: "Windows protected your PC"
- Users click "More info" then "Run anyway" to install
- This mirrors the initial macOS approach before Developer ID signing was added

### Tier 2: Signed with OV Certificate (Planned)

| Item | Detail |
|------|--------|
| Certificate type | OV (Organization Validation) code signing certificate |
| Cost | ~$200-400/year from DigiCert, Sectigo, or SSL.com |
| SmartScreen | Warnings reduce over time as reputation builds |
| Signing tool | `signtool.exe` (single command, much simpler than macOS `codesign`) |

To enable signing in CI, add these repository secrets:

| Secret | Description |
|--------|-------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded `.pfx` certificate file |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the `.pfx` file |

Then add a signing step before the artifact upload:

```yaml
      - name: Import code signing certificate
        env:
          WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          $pfxBytes = [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE)
          $pfxPath = "$env:RUNNER_TEMP\certificate.pfx"
          [IO.File]::WriteAllBytes($pfxPath, $pfxBytes)
          Import-PfxCertificate -FilePath $pfxPath -CertStoreLocation Cert:\CurrentUser\My -Password (ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force)
        shell: pwsh

      - name: Sign NSIS installer
        run: |
          $exe = Get-ChildItem -Path "src-tauri\target\release\bundle\nsis\*.exe" | Select-Object -First 1
          & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a $exe.FullName
        shell: pwsh
```

### Tier 3: EV Certificate (Optional)

- ~$300-600/year, provides **immediate** SmartScreen trust (no reputation buildup)
- Requires a hardware token (USB key) — complicates CI, may need self-hosted runner or cloud HSM
- Consider only if SmartScreen friction is hurting adoption

### macOS vs Windows Signing Comparison

| Step | macOS | Windows |
|------|-------|---------|
| Certificate | Apple Developer ID ($99/yr) | OV/EV cert ($200-600/yr) |
| Signing tool | `codesign` (inside-out, multi-step) | `signtool` (single command) |
| Notarization | Required (submit + wait + staple) | **Not required** |
| Build machine | macOS only | Windows only (or GitHub Actions) |

---

## Step 4: Future macOS CI Signing

The current macOS build uses `scripts/build-dmg.sh` run locally with:
- Developer ID Application certificate in the local keychain
- Notarytool credentials stored as `CMD-K-NOTARIZE`
- Signing identity: `Developer ID Application: VENKAT LUKSSHMAN TURLAPATI (36L722DZ7X)`

To move macOS builds to CI, a future workflow would need:
- `macos-latest` runner
- Certificate and provisioning profile imported via secrets
- Notarytool credentials passed as environment variables
- The full 10-step build-dmg pipeline adapted for CI

This is out of scope for now — the local build-dmg.sh pipeline works and is already producing signed, notarized, stapled DMGs.

---

## Step 5: Verification Checklist

After adding the workflow file to the repo:

- [ ] Push `.github/workflows/build-windows.yml` to `main`
- [ ] Confirm the workflow appears under **Actions** tab on GitHub
- [ ] Trigger a manual run via `workflow_dispatch`
- [ ] Verify the build completes successfully on `windows-latest`
- [ ] Download the `cmd-k-windows-installer` artifact
- [ ] Test the NSIS installer on a Windows machine:
  - [ ] Installer launches and completes
  - [ ] CMD+K appears in Start Menu
  - [ ] App launches and shows the overlay on `Ctrl+K`
  - [ ] AI command generation works end-to-end
  - [ ] Uninstaller works cleanly
- [ ] Verify SmartScreen behavior (expected: warning for unsigned builds)
- [ ] Update README download links to point to the latest artifact or release

---

## Summary

| What | Status |
|------|--------|
| Windows branch code | Merged to `main` |
| NSIS installer config | Complete in `tauri.conf.json` |
| Build script | `scripts/build-windows.sh` (for local use) |
| CI workflow | Ready to add (see Step 2) |
| Code signing | Deferred (Tier 1 unsigned first) |
| macOS CI | Out of scope (local pipeline works) |
