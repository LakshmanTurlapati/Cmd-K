# Project Research Summary

**Project:** CMD+K v0.2.1 -- Windows Platform Support
**Domain:** Cross-platform Tauri overlay app porting (macOS -> Windows)
**Researched:** 2026-03-01
**Confidence:** HIGH (stack, features, architecture), MEDIUM (specific Win32 edge cases)

## Executive Summary

CMD+K is a macOS overlay app built on Tauri v2 (Rust + React) that generates terminal commands via AI. The v0.2.1 milestone ports the entire experience to Windows. Research across all four domains reveals a clear finding: roughly 60% of the codebase is already cross-platform and compiles for Windows with zero changes. The remaining 40% -- overlay window management, terminal process inspection, text reading, and paste mechanics -- requires Windows-specific implementations using Win32 APIs. The existing `#[cfg(target_os)]` conditional compilation pattern already in the codebase is the correct abstraction strategy; no trait-based dispatch is needed.

The recommended approach is a module-by-module port using the `windows` crate (Microsoft's official Rust Win32 bindings), the `sysinfo` crate for process enumeration, the `uiautomation` crate for terminal text reading, and `enigo`/`arboard` for input simulation and clipboard. The most architecturally significant change is replacing `tauri-nspanel` (macOS NSPanel) with a standard Tauri window augmented by Win32 extended styles (`WS_EX_NOACTIVATE`, `WS_EX_TOOLWINDOW`) and explicit focus management via `SetForegroundWindow`. This is the single hardest piece of the port because NSPanel's non-activating keyboard-accepting behavior has no direct Windows equivalent.

The top three risks are: (1) overlay focus stealing -- the overlay must not break the capture-before-show pattern where the terminal PID is recorded before the overlay appears; (2) CWD reading via PEB traversal -- Windows has no public API for reading another process's working directory, requiring undocumented but stable `NtQueryInformationProcess` + `ReadProcessMemory`; and (3) SmartScreen blocking unsigned binaries -- a code signing certificate should be purchased at project start because reputation takes weeks to build. Mitigations exist for all three and are detailed below.

## Key Findings

### Recommended Stack

The existing Tauri v2 + React 19 + Zustand + xAI stack is fully cross-platform for AI, HTTP, state management, safety checks, and the entire frontend layer. Windows-specific additions are all Rust-side, platform-gated behind `[target.'cfg(target_os = "windows")'.dependencies]`. See [STACK.md](STACK.md) for full details.

**New Windows-only dependencies:**
- `windows` crate (0.61+): Win32 APIs -- foreground window, process tree, PEB CWD, HWND manipulation, SendInput
- `uiautomation` (0.22+): Windows UI Automation for terminal text reading -- replaces macOS AX reader
- `enigo` (0.6+): Cross-platform input simulation -- replaces CGEventPost for keystroke injection
- `arboard` (3.4+): Cross-platform clipboard -- replaces `pbcopy` shell call
- `sysinfo` (0.38+): Process enumeration, parent PID, exe path -- replaces libproc for non-CWD operations

**Modified existing dependencies:**
- `keyring`: Add `windows-native` feature alongside `apple-native` (zero code changes in keychain.rs)
- `window-vibrancy`: Update to 0.7+ for latest Acrylic/Mica Windows support

**macOS-only dependencies to platform-gate:**
- `tauri-nspanel`, `accessibility-sys`, `core-foundation-sys` must move to `[target.'cfg(target_os = "macos")'.dependencies]`

### Expected Features

**Must have (table stakes for Windows launch):**
- Acrylic frosted glass overlay (Fluent Design) with always-on-top and skip-taskbar
- Ctrl+Shift+K global hotkey (NOT Ctrl+K -- too many conflicts on Windows)
- Terminal CWD detection for PowerShell, CMD, Windows Terminal, Git Bash
- Auto-paste into active terminal via clipboard + SendInput Ctrl+V
- API key storage via Windows Credential Manager (keyring crate, zero code changes)
- System tray icon (Tauri tray API, cross-platform)
- NSIS installer with WebView2 bootstrapper (per-user, no admin)
- Per-window command history and AI context (state logic is cross-platform)
- Safety warnings for destructive commands (add Windows-specific patterns: `Remove-Item -Recurse`, `rd /s`, `bcdedit`, etc.)

**Should have (differentiators):**
- Zero-setup terminal context (NtQueryInformationProcess for CWD, no PowerShell profile edits)
- Works across all terminal types (Windows Terminal, PowerShell, CMD, Git Bash, Hyper)
- Per-terminal-window AI context (unique among Windows AI tools)
- No admin required (per-user NSIS install, no UAC)

**Defer (v2+):**
- WSL CWD detection (VM PID namespace boundary -- cannot read Linux process CWD from Windows side)
- Git Bash (mintty) terminal output reading (mintty does not expose UIA or console API)
- MSI installer (requires Windows-only WiX build tools)
- ConPTY injection for output reading (invasive, antivirus will flag)
- Auto-start on login (tauri-plugin-autostart exists, but defer to polish phase)

See [FEATURES.md](FEATURES.md) for the full feature landscape and dependency graph.

### Architecture Approach

The port follows a `cfg(target_os)` conditional compilation strategy, replacing existing `#[cfg(not(target_os = "macos"))]` stubs with real `#[cfg(target_os = "windows")]` implementations. Each platform-specific module gets a parallel Windows implementation with an identical public API surface. The Tauri IPC command layer and the entire React frontend remain completely platform-unaware. See [ARCHITECTURE.md](ARCHITECTURE.md) for component-by-component analysis.

**Modules requiring Windows implementation (7 files):**

| Module | macOS Dependency | Windows Replacement |
|--------|-----------------|---------------------|
| `commands/window.rs` | tauri-nspanel (NSPanel) | Tauri window + WS_EX_NOACTIVATE + SetForegroundWindow |
| `commands/hotkey.rs` | NSWorkspace FFI | GetForegroundWindow + GetWindowThreadProcessId |
| `commands/paste.rs` | AppleScript + CGEventPost | arboard clipboard + enigo SendInput Ctrl+V |
| `commands/permissions.rs` | AXIsProcessTrusted | No-op (Windows needs no permission gate) |
| `terminal/process.rs` | libproc FFI | sysinfo + NtQueryInformationProcess PEB read |
| `terminal/ax_reader.rs` | Accessibility API (AXUIElement) | uiautomation crate (IUIAutomation COM) |
| `terminal/detect.rs` | NSRunningApplication (bundle_id) | GetModuleFileNameExW (exe_name) |

**Modules already cross-platform (zero changes, 10+ files):**
- `commands/ai.rs`, `commands/xai.rs`, `commands/safety.rs`, `commands/history.rs`, `commands/keychain.rs`, `commands/tray.rs`, `state.rs`, `terminal/filter.rs`, all React/TypeScript frontend code, Zustand store

### Critical Pitfalls

Top 5 pitfalls from [PITFALLS.md](PITFALLS.md), ordered by severity:

1. **No NSPanel equivalent -- overlay steals focus.** The entire capture-before-show pattern breaks if GetForegroundWindow returns the overlay PID instead of the terminal. Use WS_EX_NOACTIVATE + WS_EX_TOOLWINDOW on the HWND post-creation, remove WS_EX_NOACTIVATE on show for keyboard input, restore on hide with SetForegroundWindow back to terminal. Must be solved in Phase 1.

2. **PEB-based CWD reading is complex and undocumented.** Windows has no public API for remote process CWD. Must use NtQueryInformationProcess + ReadProcessMemory to traverse the PEB. This is unsafe Rust code that reads cross-process memory. Fails silently for elevated processes. Supplement with window title parsing as fallback for PowerShell (which may not update process CWD via Set-Location).

3. **SendInput blocked by UIPI for elevated terminals.** If the user runs PowerShell as Administrator, CMD+K (running non-elevated) cannot inject keystrokes. SendInput silently succeeds but keystrokes vanish. Must detect elevation via OpenProcessToken + GetTokenInformation and show a clear warning. Do NOT run CMD+K as admin.

4. **Breaking macOS when adding Windows cfg blocks.** The codebase has 15+ files with cfg(target_os) blocks. Editing the wrong platform block is invisible until the other platform builds. Must establish cross-platform CI (build on both macOS and Windows) before writing any Windows code.

5. **SmartScreen blocks unsigned EXE.** New certificates start with zero reputation. Even EV certificates no longer get instant bypass. Must purchase a signing certificate at project start and begin distributing signed beta builds to build reputation. Recovery is slow (weeks to months).

## Implications for Roadmap

Based on combined research, here is the recommended phase structure. Phases 2 and 3 can be developed in parallel since they are architecturally independent.

### Phase 1: Build Infrastructure and Overlay Foundation
**Rationale:** Everything depends on the overlay working. The Cargo.toml restructure and cross-platform CI must come first to prevent macOS regressions. The overlay window is the foundation -- hotkey, context reading, and paste all depend on it.
**Delivers:** Windows build compiles and runs. Overlay window with Acrylic vibrancy appears on Ctrl+Shift+K. Focus capture-and-restore works. Alt+Tab and taskbar are hidden. Cross-platform CI validates both platforms.
**Addresses:** Overlay UX (table stakes), hotkey registration, Cargo.toml platform gating
**Avoids:** Pitfall 1 (focus stealing), Pitfall 4 (breaking macOS), Pitfall 8 (Ctrl+K conflicts)
**Uses:** `windows` crate (HWND manipulation), `window-vibrancy` (Acrylic/Mica), Tauri window APIs
**Key decisions:** Default hotkey Ctrl+Shift+K (not Ctrl+K). Acrylic for Win10, Mica for Win11. WS_EX_TOOLWINDOW to hide from Alt+Tab.

### Phase 2: Terminal Context (Process Tree + CWD + Detection)
**Rationale:** Terminal context is the core value proposition ("know my context without shell plugins"). CWD detection is the hardest Windows-specific implementation. This is independent of the overlay's paste mechanism.
**Delivers:** Process tree walking identifies shell PIDs under Windows Terminal, PowerShell, CMD, Git Bash. CWD read from PEB. Shell type detected. Window key computed as `exe_name:shell_pid`. Known terminal exe list populated.
**Addresses:** CWD detection (table stakes), shell detection, window identification, per-window history enablement
**Avoids:** Pitfall 3 (Windows Terminal multi-process architecture), Pitfall 6 (PEB CWD complexity)
**Uses:** `sysinfo` (process enumeration), `windows` crate (NtQueryInformationProcess, CreateToolhelp32Snapshot, ReadProcessMemory)
**Key decisions:** Use sysinfo for process tree, windows crate for CWD. Window title parsing as CWD fallback. PlatformPid type alias (i32 macOS, u32 Windows) to avoid truncation. WSL CWD deferred.

### Phase 3: Paste and Input Simulation
**Rationale:** Paste depends on Phase 1 (HWND stored at capture time) but is independent of Phase 2 (context reading). Can be developed in parallel with Phase 2.
**Delivers:** Command written to clipboard via arboard. Terminal activated via SetForegroundWindow. Ctrl+V sent via enigo. Elevated terminal detection with user warning. Line clearing (Escape for PowerShell/CMD, Ctrl+U for bash).
**Addresses:** Auto-paste (table stakes), focus restoration after paste
**Avoids:** Pitfall 2 (UIPI blocking SendInput to elevated terminals)
**Uses:** `arboard` (clipboard), `enigo` (input simulation), `windows` crate (SetForegroundWindow, OpenProcessToken)
**Key decisions:** Clipboard + Ctrl+V as universal paste (all Windows terminals accept it except mintty which needs Shift+Insert). Detect elevation before paste attempt. Accept clipboard clobber for MVP; save/restore clipboard later.

### Phase 4: Terminal Output Reading (UIA)
**Rationale:** Terminal output reading enhances AI context quality but the app is functional without it (CWD alone is sufficient for basic context). This is the riskiest implementation due to UIA crate maturity.
**Delivers:** Windows UI Automation reader equivalent to macOS AX reader. Reads visible terminal text from Windows Terminal, PowerShell (conhost), CMD (conhost). Graceful None for terminals without UIA (mintty, GPU terminals).
**Addresses:** Terminal output reading (differentiator), browser DevTools detection
**Uses:** `uiautomation` crate (IUIAutomation COM wrapper)
**Key decisions:** UIA as primary strategy. No Console Buffer API fallback in MVP (Windows Terminal uses ConPTY, not classic console). Graceful degradation for mintty.

### Phase 5: Platform Polish and Safety
**Rationale:** After core functionality works, adapt peripheral features: onboarding, AI prompts, safety patterns, tray behavior, auto-start.
**Delivers:** Windows onboarding (skip accessibility step, API key only). AI system prompt says "Windows". Destructive command patterns for PowerShell/CMD added. Tray icon left-click shows menu (Windows convention). Platform-aware keyboard shortcut display in UI.
**Addresses:** Onboarding adaptation, safety warnings, tray UX, platform-aware strings
**Uses:** `tauri-plugin-autostart` (optional), conditional compilation for UI strings
**Key decisions:** Skip accessibility permission step entirely on Windows. Add Windows-specific destructive patterns (Remove-Item -Recurse, rd /s, bcdedit, format C:, Reg Delete).

### Phase 6: Build, Distribution, and Integration Testing
**Rationale:** NSIS installer config, code signing, and end-to-end testing across Windows Terminal, PowerShell, CMD, Git Bash. This is the final phase before release.
**Delivers:** Signed NSIS installer (.exe). WebView2 bootstrapper embedded. ICO tray icon. End-to-end testing on Windows 10 and Windows 11. Verified macOS still works.
**Addresses:** Installer (table stakes), code signing, multi-terminal validation
**Avoids:** Pitfall 5 (SmartScreen blocking unsigned EXE)
**Key decisions:** NSIS over MSI (cross-compilable, consumer-friendly). Per-user install (no admin). WebView2 downloadBootstrapper mode. Certificate purchased in Phase 1, signing configured here.

### Phase Ordering Rationale

- **Phase 1 must come first** because every other phase depends on the overlay window existing and focus management working correctly on Windows. The Cargo.toml restructure prevents macOS regressions throughout.
- **Phases 2 and 3 are parallel** because terminal context reading (process tree, CWD) is architecturally independent of paste (clipboard, SendInput). Both depend on Phase 1's HWND capture.
- **Phase 4 follows Phase 2** because UIA text reading requires understanding the terminal process architecture established in Phase 2.
- **Phase 5 follows all functional phases** because polish items (onboarding, safety patterns, tray UX) depend on the core features being implemented.
- **Phase 6 is last** because integration testing requires all features to be present, and code signing requires the final binary.

### Research Flags

**Phases likely needing deeper research during planning:**
- **Phase 1 (Overlay):** WS_EX_NOACTIVATE + keyboard input interaction needs prototyping. The exact sequence (remove on show, restore on hide) must be validated. Tauri bugs #10422 (skipTaskbar) and #11566 (focus) may require workarounds.
- **Phase 2 (Terminal Context):** PEB CWD reading requires unsafe Rust with cross-process memory access. PowerShell's Set-Location may not update process CWD. Windows Terminal multi-tab active-tab detection has no public API -- heuristics needed.
- **Phase 4 (UIA Reader):** The `uiautomation` crate is less battle-tested than macOS AX APIs. TextPattern availability varies by terminal. Needs hands-on prototyping on a Windows machine.

**Phases with standard patterns (skip phase research):**
- **Phase 3 (Paste):** Clipboard + SendInput Ctrl+V is a well-documented, standard Windows pattern. arboard and enigo have clear APIs.
- **Phase 5 (Polish):** Conditional compilation, string changes, safety regex additions -- all straightforward.
- **Phase 6 (Distribution):** Tauri NSIS bundling is well-documented with official guides.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All recommended crates verified via official docs. `windows` crate is Microsoft-maintained. `sysinfo` has 35M+ downloads. |
| Features | HIGH | Feature landscape well-understood. Table stakes clearly identified. Anti-features (WSL CWD, ConPTY injection) correctly scoped out. |
| Architecture | HIGH (core), MEDIUM (edge cases) | cfg(target_os) pattern already established in codebase. Module boundaries clear. WS_EX_NOACTIVATE + keyboard interaction needs runtime validation. |
| Pitfalls | HIGH | Pitfalls verified against Tauri GitHub issues, Windows API docs, and community reports. UIPI, SmartScreen, PEB CWD are well-documented domain hazards. |

**Overall confidence:** HIGH

### Gaps to Address

- **WS_EX_NOACTIVATE + WebView keyboard input:** Exact behavior untested. May need to toggle extended style on show/hide rather than keeping it persistent. Needs a prototype on Windows hardware.
- **PowerShell CWD lag:** PowerShell's `Set-Location` may not update the process CWD in the PEB immediately. Window title parsing fallback needs validation.
- **Windows Terminal active tab detection:** No public API to determine which tab is focused in a multi-tab Windows Terminal window. Heuristic (highest PID, window title parsing) needs testing.
- **uiautomation crate maturity:** Less used than macOS AX APIs in Rust. TextPattern extraction from Windows Terminal specifically needs a proof-of-concept.
- **Tauri transparent window on Windows:** Known v2 bugs with transparency. May need CSS fallback (solid semi-transparent background) if Tauri's transparent flag causes issues.
- **DPI scaling on mixed-monitor setups:** Windows users commonly have laptop at 150% + external at 100%. Overlay positioning must handle DPI changes gracefully (Tauri bug #3610).

## Sources

### Primary (HIGH confidence)
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) -- overlay config, vibrancy
- [Tauri v2 Windows Installer](https://v2.tauri.app/distribute/windows-installer/) -- NSIS, WebView2
- [Tauri v2 Code Signing (Windows)](https://v2.tauri.app/distribute/sign/windows/) -- SmartScreen, certificates
- [Microsoft windows crate](https://microsoft.github.io/windows-docs-rs/) -- GetForegroundWindow, SendInput, NtQueryInformationProcess, CreateToolhelp32Snapshot
- [keyring crate v3.6.3](https://docs.rs/keyring) -- Windows Credential Manager
- [sysinfo crate v0.38](https://docs.rs/sysinfo) -- process enumeration
- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy) -- Acrylic/Mica
- [arboard crate (1Password)](https://github.com/1Password/arboard) -- cross-platform clipboard
- [enigo crate](https://docs.rs/enigo) -- cross-platform input simulation
- [MSDN: SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow)
- [MSDN: Extended Window Styles (WS_EX_NOACTIVATE)](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
- [MSDN: NtQueryInformationProcess](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess)

### Secondary (MEDIUM confidence)
- [uiautomation-rs crate](https://github.com/leexgone/uiautomation-rs) -- Windows UIA wrapper (maintained but less battle-tested)
- [Windows Terminal UIA Provider (PR #1691)](https://github.com/microsoft/terminal/pull/1691) -- confirms UIA text reading from WT
- [NVDA UIA console support (PR #9614)](https://github.com/nvaccess/nvda/pull/9614) -- validates UIA text reading approach
- [Tauri #10422: skipTaskbar not working on Windows](https://github.com/tauri-apps/tauri/issues/10422)
- [Tauri #7348: NSIS uninstall.exe not signed](https://github.com/tauri-apps/tauri/issues/7348)

### Tertiary (needs validation during implementation)
- WS_EX_NOACTIVATE + WebView keyboard input interaction (no Tauri-specific reference found)
- PowerShell Set-Location PEB CWD update timing (anecdotal community reports)
- Windows Terminal active tab identification heuristics (no public API documented)

---
*Research completed: 2026-03-01*
*Ready for roadmap: yes*
