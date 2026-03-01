# Milestones

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

