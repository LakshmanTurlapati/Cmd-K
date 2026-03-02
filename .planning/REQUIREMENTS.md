# Requirements: CMD+K v0.2.1 Windows Support

**Defined:** 2026-03-01
**Core Value:** The overlay must appear on top of the active application and feel instant -- same experience on Windows as macOS

## v0.2.1 Requirements

Requirements for Windows platform port. Each maps to roadmap phases.

### Windows Overlay (WOVL)

- [x] **WOVL-01**: Overlay window appears with Acrylic (Win10) or Mica (Win11) frosted glass vibrancy
- [x] **WOVL-02**: Overlay floats above all windows with always-on-top and skip-taskbar behavior
- [x] **WOVL-03**: Overlay does not appear in Alt+Tab or taskbar (WS_EX_TOOLWINDOW)
- [ ] **WOVL-04**: Previous window HWND captured before overlay shows for focus restoration
- [ ] **WOVL-05**: Focus returns to previous terminal window on overlay dismiss (SetForegroundWindow)
- [ ] **WOVL-06**: Ctrl+Shift+K default hotkey triggers overlay system-wide (configurable)
- [x] **WOVL-07**: Escape dismisses overlay without executing

### Windows Terminal Context (WCTX)

- [ ] **WCTX-01**: Shell PID detected via process tree walking for Windows Terminal, PowerShell, CMD, Git Bash
- [ ] **WCTX-02**: Current working directory read from shell process via NtQueryInformationProcess PEB traversal
- [ ] **WCTX-03**: Shell type detected (powershell.exe, pwsh.exe, cmd.exe, bash.exe)
- [ ] **WCTX-04**: Window key computed as exe_name:shell_pid for per-window history
- [ ] **WCTX-05**: Known terminal exe list identifies Windows Terminal, PowerShell, CMD, Git Bash, Hyper, Alacritty, WezTerm
- [ ] **WCTX-06**: CWD falls back gracefully to None for elevated/inaccessible processes

### Windows Paste (WPST)

- [ ] **WPST-01**: Command written to clipboard via cross-platform API (replaces pbcopy)
- [ ] **WPST-02**: Terminal activated via SetForegroundWindow before paste
- [ ] **WPST-03**: Ctrl+V keystroke sent via SendInput to paste command into terminal
- [ ] **WPST-04**: Elevated terminal detected with user warning instead of silent failure
- [ ] **WPST-05**: Enter keystroke sent via SendInput for command confirmation

### Windows Output Reading (WOUT)

- [ ] **WOUT-01**: Terminal text read via Windows UI Automation for Windows Terminal
- [ ] **WOUT-02**: Terminal text read via UIA for PowerShell and CMD (conhost)
- [ ] **WOUT-03**: Graceful None returned for terminals without UIA support (mintty, GPU terminals)

### Windows Polish (WPLH)

- [ ] **WPLH-01**: Onboarding skips accessibility permission step on Windows (no equivalent needed)
- [ ] **WPLH-02**: AI system prompt identifies platform as Windows for context-appropriate command generation
- [ ] **WPLH-03**: Destructive command patterns include Windows-specific commands (Remove-Item -Recurse, rd /s, bcdedit, format, Reg Delete)
- [ ] **WPLH-04**: System tray shows context menu on right-click (Windows convention)
- [ ] **WPLH-05**: No macOS-specific permission API is called on Windows (permissions.rs returns true)
- [ ] **WPLH-06**: Keyboard shortcuts displayed as Ctrl (not Cmd) in UI on Windows

### Windows Build & Distribution (WBLD)

- [x] **WBLD-01**: Cargo.toml platform-gates macOS-only deps and adds Windows-only deps
- [x] **WBLD-02**: Project compiles on both macOS and Windows without regressions
- [ ] **WBLD-03**: NSIS installer produces signed .exe setup with per-user install (no admin)
- [ ] **WBLD-04**: WebView2 runtime bootstrapper embedded in installer
- [ ] **WBLD-05**: ICO format tray icon included for Windows
- [ ] **WBLD-06**: End-to-end testing verified on Windows Terminal, PowerShell, CMD, Git Bash

## Deferred Requirements

### WSL Support

- **DWSL-01**: WSL shell CWD resolves to Windows path (VM PID namespace boundary)
- **DWSL-02**: WSL terminal context provides Linux distro information

### Advanced Features

- **DADV-01**: Auto-start on login via tauri-plugin-autostart
- **DADV-02**: MSI installer for enterprise Group Policy deployment
- **DADV-03**: Clipboard save/restore around paste to prevent clobber
- **DADV-04**: Git Bash (mintty) output reading
- **DADV-05**: UIAccess manifest for cross-integrity SendInput to elevated terminals

## Out of Scope

| Feature | Reason |
|---------|--------|
| WSL CWD detection | WSL2 runs in VM with separate PID namespace -- cannot read Linux process CWD from Windows |
| ConPTY output injection | Invasive, antivirus will flag, fragile |
| Running CMD+K as Administrator | Security anti-pattern, triggers UAC, breaks auto-start |
| PowerShell profile auto-modification | Violates zero-setup constraint |
| Custom DWM composition | Massive complexity for marginal visual improvement |
| Linux support | Deferred to future milestone |
| MSIX containerized package | Restricts filesystem access needed for process inspection |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| WOVL-01 | Phase 11 | Complete |
| WOVL-02 | Phase 11 | Complete |
| WOVL-03 | Phase 11 | Complete |
| WOVL-04 | Phase 11 | Pending |
| WOVL-05 | Phase 11 | Pending |
| WOVL-06 | Phase 11 | Pending |
| WOVL-07 | Phase 11 | Complete |
| WCTX-01 | Phase 12 | Pending |
| WCTX-02 | Phase 12 | Pending |
| WCTX-03 | Phase 12 | Pending |
| WCTX-04 | Phase 12 | Pending |
| WCTX-05 | Phase 12 | Pending |
| WCTX-06 | Phase 12 | Pending |
| WPST-01 | Phase 13 | Pending |
| WPST-02 | Phase 13 | Pending |
| WPST-03 | Phase 13 | Pending |
| WPST-04 | Phase 13 | Pending |
| WPST-05 | Phase 13 | Pending |
| WOUT-01 | Phase 14 | Pending |
| WOUT-02 | Phase 14 | Pending |
| WOUT-03 | Phase 14 | Pending |
| WPLH-01 | Phase 15 | Pending |
| WPLH-02 | Phase 15 | Pending |
| WPLH-03 | Phase 15 | Pending |
| WPLH-04 | Phase 15 | Pending |
| WPLH-05 | Phase 15 | Pending |
| WPLH-06 | Phase 15 | Pending |
| WBLD-01 | Phase 11 | Complete |
| WBLD-02 | Phase 11 | Complete |
| WBLD-03 | Phase 16 | Pending |
| WBLD-04 | Phase 16 | Pending |
| WBLD-05 | Phase 16 | Pending |
| WBLD-06 | Phase 16 | Pending |

**Coverage:**
- v0.2.1 requirements: 33 total
- Mapped to phases: 33
- Unmapped: 0

---
*Requirements defined: 2026-03-01*
*Last updated: 2026-03-01 -- all requirements mapped to phases 11-16*
