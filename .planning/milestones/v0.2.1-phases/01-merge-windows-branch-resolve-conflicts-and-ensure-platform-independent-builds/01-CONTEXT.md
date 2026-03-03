# Phase 1: Merge Windows Branch — Context

**Gathered:** 2026-03-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Merge the `windows` branch (30 commits, 58 files, +7.6k/-2k lines) into `main`. Resolve the 1 merge conflict (showcase/index.html), audit `#[cfg]` gates for platform isolation, align version numbers, and ensure both macOS and Windows build scripts are independent and functional. Delete the `windows` branch after merge.

</domain>

<decisions>
## Implementation Decisions

### Merge strategy
- Standard merge of `windows` into `main` (not rebase, not squash)
- Preserves full 30-commit history from the Windows port
- Only 1 conflict: `showcase/index.html` — resolve by keeping both macOS DMG download links (from main) and Windows download links (from windows), producing a cross-platform download page
- Delete `windows` branch (local + remote) after successful merge
- Quick audit of all `#[cfg(target_os)]` gates before merging to confirm no platform leakage

### Version alignment
- Merged version: v0.2.1 (from windows branch)
- All 3 version files must match: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
- Post-merge grep for any surviving "0.1.1" strings to catch stragglers

### Build scripts
- Keep main's `build-dmg.sh` (full notarized DMG pipeline) intact — do NOT use the stripped-down version from windows
- Ensure `scripts/dmg-background.png` survives the merge (windows branch deleted it)
- Add a separate Windows build script (`build-windows.sh` or similar) for NSIS installer
- Add `build:mac` and `build:windows` npm scripts to `package.json` for discoverability
- CI/GitHub Actions is out of scope for this phase — local build scripts only

### Claude's Discretion
- Exact merge commit message wording
- Order of post-merge verification steps
- Whether to add `build-pkg.sh` script alias

</decisions>

<specifics>
## Specific Ideas

- The showcase/index.html should have platform-specific download buttons (macOS DMG + Windows installer) side by side
- `dmg-background.png` must not be lost during merge — it's part of the notarized DMG pipeline

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `windows` branch already has comprehensive `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "windows")]` gating across all Rust files
- Platform-specific dependencies properly separated in Cargo.toml using `[target.'cfg(target_os)'.dependencies]`
- New Windows-only files: `detect_windows.rs`, `uia_reader.rs`, `platform.ts`
- `src/utils/platform.ts` provides frontend platform detection utilities

### Established Patterns
- All macOS-specific code (NSPanel, Accessibility API, AppleScript, libproc FFI) is already `#[cfg(target_os = "macos")]` gated on windows branch
- Windows equivalents: Win32 API (WS_EX_TOOLWINDOW), UIA reader, arboard clipboard, SendInput paste
- Frontend uses `platform.ts` utility for conditional rendering (border radius, hotkey display)

### Integration Points
- `src-tauri/src/lib.rs` — main setup function has platform-branched initialization
- `src-tauri/src/commands/hotkey.rs` — platform-branched HWND/PID capture
- `src-tauri/src/commands/paste.rs` — platform-branched paste dispatch
- `src-tauri/src/commands/window.rs` — platform-branched overlay show/hide
- `src-tauri/tauri.conf.json` — Windows NSIS installer config added alongside macOS signing

</code_context>

<deferred>
## Deferred Ideas

- GitHub Actions CI for cross-platform builds — separate phase
- Linux support — future milestone

</deferred>

---

*Phase: 01-merge-windows-branch-resolve-conflicts-and-ensure-platform-independent-builds*
*Context gathered: 2026-03-03*
