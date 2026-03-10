# CMD+K

## What This Is

A lightweight overlay app that generates terminal commands using AI on macOS and Windows. Press Cmd+K (or Ctrl+K on Windows) from anywhere, a prompt overlay appears on top of your active window with frosted glass vibrancy, describe what you need in natural language, and it generates the right command using your choice of AI provider (OpenAI, Anthropic, Gemini, xAI, or OpenRouter) with real-time streaming. It flags destructive commands with warnings (150+ patterns across macOS/Linux/Windows), and pastes the result directly into your active terminal — including WSL sessions on Windows. It knows your context — current directory, recent commands, shell type — without requiring any shell configuration. Silent auto-updates keep the app current. Automated CI/CD publishes signed builds for both platforms on every release.

## Core Value

The overlay must appear on top of the currently active application and feel instant -- if the overlay doesn't show up where the user is working, nothing else matters.

## Requirements

### Validated

- OVRL-01: System-wide Cmd+K hotkey triggers the overlay from any application -- v0.1.0
- OVRL-02: Overlay appears as a floating panel on top of the currently active window -- v0.1.0
- OVRL-03: User can dismiss overlay with Escape key without executing -- v0.1.0
- OVRL-04: User can configure the trigger hotkey to avoid conflicts -- v0.1.0
- OVRL-05: App runs as background daemon with menu bar icon -- v0.1.0
- SETT-01: User can store and validate their xAI API key -- v0.1.0
- SETT-02: User can select which Grok model to use -- v0.1.0
- SETT-03: API keys stored securely in macOS Keychain -- v0.1.0
- SETT-04: First-run onboarding guides user through API key, model selection, and Accessibility permissions -- v0.1.0
- TERM-01: Generated command is pasted into the active terminal (Terminal.app, iTerm2) -- v0.1.0
- TERM-02: App detects the current working directory without shell plugins -- v0.1.0
- TERM-03: App reads recent terminal output for context without shell plugins -- v0.1.0
- TERM-04: Works with Terminal.app, iTerm2, Alacritty, kitty, WezTerm -- v0.1.0
- AICG-01: User can type natural language and receive a terminal command via xAI (Grok) -- v0.1.0
- AICG-02: Command generation streams in real-time -- v0.1.0
- AICG-03: Destructive commands flagged with warning before paste -- v0.1.0
- WKEY-01: Stable per-terminal-window key (bundle_id:shell_pid) computed before overlay shows -- v0.1.1
- WKEY-02: Window key captured synchronously in hotkey handler before overlay steals focus -- v0.1.1
- WKEY-03: Non-terminal apps fall back to global key -- v0.1.1
- HIST-01: Arrow-Up recalls previous query for active terminal window -- v0.1.1
- HIST-02: Arrow-Down navigates forward, restoring current draft at end -- v0.1.1
- HIST-03: Current draft preserved during history navigation -- v0.1.1
- HIST-04: Up to 50 queries per terminal window, session-scoped -- v0.1.1
- CTXT-01: AI turnHistory persists per terminal window across overlay cycles -- v0.1.1
- CTXT-02: turnHistory restored from per-window map on overlay open -- v0.1.1
- CTXT-03: Terminal context only in first user message to prevent token bloat -- v0.1.1
- ZORD-01: System permission and accessibility dialogs can appear above the CMD+K overlay -- v0.2.4
- ZORD-02: System UI elements (Notification Center, Spotlight) can appear above the CMD+K overlay -- v0.2.4
- OPOS-01: User can drag the overlay to reposition it on screen -- v0.2.4
- OPOS-02: Overlay reopens at the last dragged position within the same app session -- v0.2.4
- OPOS-03: Overlay position resets to default on app relaunch -- v0.2.4
- SAFE-01: macOS-specific destructive commands detected (csrutil, dscl, nvram, etc.) -- v0.2.4
- SAFE-02: Linux-specific destructive commands detected (systemctl, iptables, userdel, etc.) -- v0.2.4
- SAFE-03: Container/orchestration destructive commands detected (docker, kubectl, helm) -- v0.2.4
- SAFE-04: Package manager uninstall commands detected (apt, brew, pip, npm, etc.) -- v0.2.4
- SAFE-05: Config file overwrites via redirect detected -- v0.2.4
- SAFE-06: All patterns organized with clear section headers -- v0.2.4
- CICD-01: release.yml workflow triggered by v* tag push builds macOS and Windows artifacts -- v0.2.4
- CICD-02: macOS build produces signed, notarized, stapled universal DMG -- v0.2.4
- CICD-03: Windows build produces unsigned NSIS installer with conditional signing -- v0.2.4
- CICD-04: GitHub Release auto-published with both platform artifacts and SHA256 checksums -- v0.2.4
- CICD-05: Apple signing credentials migrated to GitHub Secrets -- v0.2.4
- CICD-06: build-dmg.sh parameterized via environment variables -- v0.2.4
- PROV-01 through PROV-07: Multi-provider AI with OpenAI, Anthropic, Gemini, xAI, OpenRouter -- v0.2.6
- PFUI-01 through PFUI-05: Provider selection onboarding, settings switching, tier-grouped models -- v0.2.6
- ORTR-01, ORTR-02: OpenRouter single-key access with filtered model list -- v0.2.6
- WSLT-01 through WSLT-10: WSL terminal context detection, CWD, shell type, output, AI commands, badge -- v0.2.6
- UPDT-01 through UPDT-08: Auto-updater with background checks, tray menu, Ed25519 signing, CI pipeline -- v0.2.6
- TRAK-01 through TRAK-04: Token extraction from OpenAI-compat, Anthropic, Gemini streaming adapters with session-scoped accumulation -- v0.2.7
- PRIC-01 through PRIC-03: Curated pricing for 47 models, OpenRouter dynamic pricing, pricing-unavailable indicator -- v0.2.7
- DISP-01 through DISP-04: Live cost display in Settings Model tab with token breakdown, sparkline, and reset -- v0.2.7

### Active

(No active milestone — run `/gsd:new-milestone` to start next)

### Out of Scope

- VS Code extension -- dropped in favor of standalone overlay app
- Linux support -- Windows is v0.2.1 scope, Linux deferred to future
- Command favorites/bookmarks -- future feature (history is v0.1.1 scope)
- Multi-step command workflows -- future feature
- Command explanation mode -- future feature
- Offline mode -- requires internet for AI generation
- App Store distribution -- incompatible with Accessibility API requirement
- Auto-execution without review -- safety risk, always paste, never execute directly
- Windows OV/EV code signing -- purchase when distribution warrants it
- Linux builds -- separate phase

## Context

Shipped v0.2.7 with session cost estimation across all 5 AI providers.
Tech stack: Tauri v2 (Rust + React + TypeScript), NSPanel for overlay, 5 AI providers (OpenAI/Anthropic/Gemini/xAI/OpenRouter), macOS Accessibility API + raw libproc FFI, Win32 APIs + UIA for Windows, WSL process ancestry + wsl.exe subprocess for Linux context.
29 phases across 7 milestones (v0.1.0, v0.1.1, v0.2.1, v0.2.4, v0.2.6, v0.2.7), 56 plans executed over 17 days.
All 86 requirements satisfied across milestones. 7,967 LOC Rust + 3,878 LOC TypeScript.
CI/CD pipeline produces signed macOS DMG, Windows installer, and auto-update artifacts (latest.json + .sig files) on every v* tag push.
Ed25519 update signing configured with GitHub secrets.

## Constraints

- **Tech stack**: Tauri (Rust + web frontend)
- **Platform**: macOS + Windows
- **Zero setup**: No shell plugins, no .zshrc modifications. One-time macOS accessibility permission is acceptable.
- **Multi-provider**: 5 AI providers supported via provider abstraction layer

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Drop VS Code extension | Standalone overlay is the real product | Good -- cleaner UX |
| Tauri over Electron | Lighter weight, smaller binary, less RAM | Good -- ~20MB binary |
| xAI only for v1 | Simplify scope, add other providers later | Good -- shipped faster |
| macOS only for v1 | Focus on one platform, nail the overlay UX | Good -- NSPanel critical |
| Zero shell setup | Lower adoption friction, use Accessibility API + AppleScript instead | Good -- no .zshrc needed |
| NSPanel with Floating window level | Float above normal apps, yield to system UI | Good -- works everywhere |
| Raw libproc FFI over darwin-libproc crate | Avoids memchr version conflict with Tauri dependency chain | Good -- identical functionality |
| AX probe fallback for permission detection | AXIsProcessTrusted returns false on unsigned builds | Good -- fixes production DMG |
| Two-mode AI system prompt | Terminal mode (command-only) vs assistant mode (conversational) | Good -- context-appropriate |
| AppleScript dispatch for terminal pasting | iTerm2 write text + Terminal.app keystroke, neither auto-executes | Good -- safe pasting |
| once_cell Lazy<RegexSet> for destructive patterns | Compiled once, zero allocation on subsequent checks | Good -- fast safety checks |
| Capture-before-show PID pattern | Must capture frontmost PID before overlay steals focus | Good -- reliable context |
| bundle_id:shell_pid as window key | Simpler than CGWindowID, no screen recording permission risk | Good -- stable per-window identity |
| Rust AppState HashMap for history | show() resets React state, Rust survives overlay cycles | Good -- persistent without disk I/O |
| Session-scoped history only | Privacy, simplicity; no disk persistence in v0.1.1 | Good -- clean scope boundary |
| AX-based multi-tab CWD matching | Extracts focused tab CWD from AXTitle for IDE shell disambiguation | Good -- resolves Cursor/VS Code multi-tab |
| turnHistory from windowHistory | No separate storage needed; reconstruct on overlay open | Good -- single source of truth |
| Frontend-side turn limit capping | Pre-cap in Zustand; Rust stores all, frontend slices | Good -- user-configurable without IPC |
| PanelLevel::Floating for overlay | Standard macOS level for utility panels, below system UI | Good -- system dialogs visible |
| In-memory Mutex for drag position | No disk persistence, resets on relaunch per user preference | Good -- session-scoped |
| Screen coordinates for drag deltas | Window moves during drag making clientX/Y unreliable | Good -- smooth dragging |
| 3-job CI architecture | Parallel macOS/Windows builds, sequential release publish | Good -- fast pipeline |
| grep+sed for version extraction | No jq dependency required in CI | Good -- minimal dependencies |
| Conditional Windows signing | Gates on secret presence, graceful skip when unconfigured | Good -- works without certificate |
| Provider abstraction with 3 streaming adapters | OpenAI-compat, Anthropic, Gemini cover all 5 providers | Good -- minimal code for 5 providers |
| Tier-grouped model lists | Fast/Balanced/Most Capable grouping across all providers | Good -- consistent UX |
| OpenRouter as meta-provider | Single API key for all providers, filtered to chat models | Good -- low-friction onboarding |
| WSL detection via process ancestry | CreateToolhelp32Snapshot walk finds wsl.exe in tree | Good -- works for all 4 host types |
| Multi-signal VS Code WSL detection | Window title + UIA + CWD path + shell priority | Good -- handles IDE complexity |
| UpdateState as separate managed state | tauri_plugin_updater::Update not Default, can't embed in AppState | Good -- clean separation |
| Install-on-quit pattern | download() in background, install() in quit handler | Good -- no forced restart |
| Ed25519 signing with empty password | Simplifies CI secrets, acceptable for open-source project | Good -- works with GitHub Actions |
| Decoupled UsageAccumulator keys | (String, String) not (Provider, String) keeps state.rs independent | Good -- clean separation |
| Two-tier pricing lookup | Curated models first, OpenRouter dynamic as fallback | Good -- fast for known models |
| Per-query cost at read time | QueryRecord stores raw tokens; cost calculated in get_usage_stats | Good -- pricing changes apply retroactively |
| Div-based sparkline | Angular bars with flex layout, no canvas/SVG | Good -- simple, matches design system |

---
*Last updated: 2026-03-10 after v0.2.7 milestone*
