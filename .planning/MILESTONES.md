# Milestones

## v0.2.6 Multi-Provider, WSL & Auto-Update (Shipped: 2026-03-09)

**Phases completed:** 5 phases (21-24 + 23.1), 10 executed plans
**Timeline:** 2026-03-09 (1 day)
**Git range:** 25 feat commits (73 files changed, 8,920 insertions)
**Codebase:** 7,616 LOC Rust + 3,765 LOC TypeScript

**Key accomplishments:**
1. Multi-provider AI support — OpenAI, Anthropic, Gemini, xAI, OpenRouter with per-provider streaming adapters, API key validation, and platform keychain storage
2. Provider-aware frontend — Onboarding provider selection, settings switching with key status checkmarks, tier-grouped model lists (Fast/Balanced/Most Capable), per-provider model memory
3. WSL terminal context — Detects WSL sessions in Windows Terminal, VS Code, Cursor, and standalone wsl.exe via process ancestry walk; reads Linux CWD, shell type, and generates Linux commands
4. VS Code WSL detection — Multi-signal approach using window title [WSL:] patterns, UIA tree walking, CWD path style fallback, and IDE-aware shell priority deprioritizing cmd.exe
5. Auto-updater — Background update checks (launch + 24h), tray menu state machine with text transitions, one-click download, install-on-quit via tauri-plugin-updater with Ed25519 signing
6. CI/CD updater pipeline — Release workflow generates .sig files, assembles latest.json manifest with platform keys, uploads signed updater artifacts to GitHub Releases

**Delivered:** Multi-provider AI with 5 provider options, full WSL terminal awareness on Windows, and silent auto-update with signed artifacts.

**Archives:**
- milestones/v0.2.6-ROADMAP.md
- milestones/v0.2.6-REQUIREMENTS.md

---

## v0.2.4 Overlay UX, Safety & CI/CD (Shipped: 2026-03-04)

**Phases completed:** 4 phases (17-20), 5 plans
**Timeline:** 2026-03-03 to 2026-03-04 (2 days)
**Git range:** 30 commits (41 files changed, 3,528 insertions)
**Codebase:** 6,121 LOC Rust

**Key accomplishments:**
1. NSPanel window level lowered from Status (25) to Floating (3) -- system dialogs, Notification Center, and Spotlight render above the CMD+K overlay while it still floats above normal app windows
2. Drag-to-reposition overlay via useDrag React hook with session-scoped in-memory position persistence (Mutex<Option<(f64, f64)>>), 2px dead zone, and screen-coordinate delta tracking
3. Destructive command pattern library expanded from ~80 to 150 regex patterns across 10 sections: macOS, Linux, Windows, containers/orchestration, package managers, and config file overwrites
4. Windows Terminal shell type detection fix -- ConPTY fallback correctly identifies PowerShell, cmd.exe, and WSL sessions
5. Automated CI/CD pipeline via GitHub Actions: 3-job architecture (build-macos, build-windows, release) triggered on v* tag push, with signed+notarized macOS DMG and Windows NSIS installer, auto-published GitHub Release with SHA256 checksums
6. Parameterized build-dmg.sh with env var fallbacks for CI/local compatibility, plus comprehensive secrets setup guide

**Delivered:** Overlay UX polish (z-order fix, draggable positioning), comprehensive cross-platform destructive command safety, and fully automated CI/CD release pipeline.

**Archives:**
- milestones/v0.2.4-ROADMAP.md
- milestones/v0.2.4-REQUIREMENTS.md

---

## v0.2.1 Windows Support (Shipped: 2026-03-03)

**Phases completed:** 7 phases (11-16, 01-merge), 6 GSD plans + 5 windows-branch phases
**Timeline:** 2026-03-01 to 2026-03-03 (3 days)
**Git range:** 30 commits (48 files changed, 4,734 insertions)
**Codebase:** 5,879 LOC Rust + 3,262 LOC TypeScript

**Key accomplishments:**
1. Windows overlay with Acrylic frosted glass vibrancy, WS_EX_TOOLWINDOW (hidden from Alt+Tab/taskbar), HWND capture and AttachThreadInput focus restoration
2. Windows terminal context: process tree walking via CreateToolhelp32Snapshot, CWD via NtQueryInformationProcess PEB traversal, shell type detection for PowerShell/CMD/Git Bash
3. Clipboard paste via arboard + Ctrl+V SendInput keystroke injection, with elevated terminal detection and user warnings
4. Windows UI Automation terminal text reader with TextPattern + tree walker fallback, graceful None for unsupported terminals
5. Platform polish: Windows-specific AI prompts, 10 Windows destructive patterns (Remove-Item, rd /s, bcdedit, etc.), onboarding skip, Ctrl key labels, tray conventions
6. NSIS installer with per-user install, embedded WebView2 bootstrapper, ICO tray icon
7. Windows branch merged into main with cross-platform build scripts (build:mac, build:windows)

**Delivered:** Full Windows port of CMD+K — hotkey overlay, terminal context, AI command generation, and paste, all via native Win32 APIs with zero shell configuration.

### Known Gaps (deferred to v0.2.2)

- **WBLD-06**: E2E testing on Windows Terminal, PowerShell, CMD, Git Bash (requires Windows hardware)
- **WOVL-06 spec mismatch**: Default hotkey is Ctrl+K in code but Ctrl+Shift+K in spec — needs alignment
- **Latent race**: confirm_terminal_command and hide_overlay are concurrent IPC calls — previous_hwnd may clear before paste reads it
- **UIA conditional**: Terminal output reading skipped if shell type and CWD both fail (elevated terminals)
- All 17 UAT test cases pending Windows hardware verification

**Archives:**
- milestones/v0.2.1-ROADMAP.md
- milestones/v0.2.1-REQUIREMENTS.md
- milestones/v0.2.1-MILESTONE-AUDIT.md

---

## v0.1.0 MVP (Shipped: 2026-02-28)

**Phases completed:** 8 phases, 21 plans
**Timeline:** 2026-02-21 to 2026-02-28 (8 days)
**Codebase:** 4,042 LOC Rust + 2,868 LOC TypeScript (247 files changed, 53,086 insertions)

**Key accomplishments:**
1. NSPanel-based floating overlay with frosted glass vibrancy, system-wide Cmd+K hotkey, and menu bar tray -- floats above fullscreen apps
2. Settings panel with tabbed UI (Account/Model/Preferences), macOS Keychain API key storage, and first-run onboarding wizard
3. Terminal context detection for 5 terminals (Terminal.app, iTerm2, Alacritty, kitty, WezTerm) using Accessibility API and raw libproc FFI -- zero shell plugins
4. AI command generation via xAI (Grok) with SSE streaming, two-mode system prompts (terminal vs assistant), and context-aware responses
5. Safety layer with 30+ destructive command patterns, AI-powered explanations via Radix tooltip, and configurable detection toggle
6. Auto-paste to active terminal via AppleScript dispatch (iTerm2 + Terminal.app) with destructive command guard

**Delivered:** A complete macOS overlay app that generates and pastes terminal commands using AI, with zero shell configuration required.

**Archives:**
- milestones/v1.0-ROADMAP.md
- milestones/v1.0-REQUIREMENTS.md
- milestones/v1.0-MILESTONE-AUDIT.md

---


## v0.1.1 Command History & Follow-ups (Shipped: 2026-03-01)

**Phases completed:** 3 phases, 6 plans
**Timeline:** 2026-02-28 to 2026-03-01 (2 days)
**Git range:** 32 commits (47 files changed, 4,637 insertions)

**Key accomplishments:**
1. Per-terminal-window identity via bundle_id:shell_pid keys computed in hotkey handler, with bounded HashMap history (50/window, 50 windows) and 3 IPC commands
2. AX-based focused terminal tab CWD extraction for Cursor/VS Code multi-tab shell disambiguation
3. Shell-like Arrow-Up/Down history recall in overlay input with draft preservation and dimmed text styling
4. Per-window AI conversation history (turnHistory) reconstructed from windowHistory on overlay open, enabling follow-up queries
5. Conditional terminal context -- CWD/shell/output included only in first message of a session to prevent token bloat
6. Turn limit slider (5-50) and clear conversation history button in Preferences with Rust IPC backend

**Delivered:** Per-terminal-window command history with arrow key navigation and AI follow-up context that persists across overlay open/close cycles.

**Archives:**
- milestones/v0.1.1-ROADMAP.md
- milestones/v0.1.1-REQUIREMENTS.md
- milestones/v0.1.1-MILESTONE-AUDIT.md

---

