# CMD+K

## What This Is

A lightweight macOS overlay app that generates terminal commands using AI. Press Cmd+K from anywhere, a prompt overlay appears on top of your active window with frosted glass vibrancy, describe what you need in natural language, it generates the right terminal command using xAI (Grok) with real-time streaming, flags destructive commands with warnings, and pastes the result directly into your active terminal. It knows your context -- current directory, recent commands, shell type, browser console -- without requiring any shell configuration.

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

## Current Milestone: v0.1.1 Command History & Follow-ups

**Goal:** Per-terminal-window command history with arrow key navigation and AI follow-up context

**Target features:**
- Per-terminal-window command history (up to 7 entries, session-scoped)
- Arrow up/down navigation in overlay input to recall previous commands
- Follow-up context -- AI sees full conversation history for the active terminal window
- Terminal window identification to keep histories separate

### Active

- [ ] WKEY-01: Stable per-terminal-window key (bundle_id:shell_pid) computed before overlay shows
- [ ] WKEY-02: Window key captured synchronously in hotkey handler before overlay steals focus
- [ ] WKEY-03: Non-terminal apps fall back to global key
- [ ] HIST-01: Arrow-Up recalls previous query for active terminal window
- [ ] HIST-02: Arrow-Down navigates forward, restoring current draft at end
- [ ] HIST-03: Current draft preserved during history navigation
- [ ] HIST-04: Up to 7 queries per terminal window, session-scoped
- [ ] CTXT-01: AI turnHistory persists per terminal window across overlay cycles
- [ ] CTXT-02: turnHistory restored from per-window map on overlay open
- [ ] CTXT-03: Terminal context only in first user message to prevent token bloat

### Out of Scope

- VS Code extension -- dropped in favor of standalone overlay app
- OpenAI provider -- deferred to future milestone
- Anthropic provider -- deferred to future milestone
- Windows/Linux support -- macOS first, cross-platform later
- Command favorites/bookmarks -- future feature (history is v0.1.1 scope)
- Multi-step command workflows -- future feature
- Command explanation mode -- future feature
- Offline mode -- requires internet for AI generation
- App Store distribution -- incompatible with Accessibility API requirement
- Auto-execution without review -- safety risk, always paste, never execute directly

## Context

Shipped v0.1.0 with 4,042 LOC Rust + 2,868 LOC TypeScript.
Tech stack: Tauri v2 (Rust + React + TypeScript), NSPanel for overlay, xAI/Grok for AI, macOS Accessibility API + raw libproc FFI for terminal context.
8 phases, 21 plans executed over 8 days (2026-02-21 to 2026-02-28).
All 16 v0.1.0 requirements satisfied. 12 non-critical tech debt items remain.

## Constraints

- **Tech stack**: Tauri (Rust + web frontend)
- **Platform**: macOS only for v1
- **Zero setup**: No shell plugins, no .zshrc modifications. One-time macOS accessibility permission is acceptable.
- **Single provider**: xAI (Grok) only for v1. Provider architecture should allow easy addition of OpenAI/Anthropic later.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Drop VS Code extension | Standalone overlay is the real product | Good -- cleaner UX |
| Tauri over Electron | Lighter weight, smaller binary, less RAM | Good -- ~20MB binary |
| xAI only for v1 | Simplify scope, add other providers later | Good -- shipped faster |
| macOS only for v1 | Focus on one platform, nail the overlay UX | Good -- NSPanel critical |
| Zero shell setup | Lower adoption friction, use Accessibility API + AppleScript instead | Good -- no .zshrc needed |
| NSPanel with Status window level | Float above fullscreen apps and all normal windows | Good -- works everywhere |
| Raw libproc FFI over darwin-libproc crate | Avoids memchr version conflict with Tauri dependency chain | Good -- identical functionality |
| AX probe fallback for permission detection | AXIsProcessTrusted returns false on unsigned builds | Good -- fixes production DMG |
| Two-mode AI system prompt | Terminal mode (command-only) vs assistant mode (conversational) | Good -- context-appropriate |
| AppleScript dispatch for terminal pasting | iTerm2 write text + Terminal.app keystroke, neither auto-executes | Good -- safe pasting |
| once_cell Lazy<RegexSet> for destructive patterns | Compiled once, zero allocation on subsequent checks | Good -- fast safety checks |
| Capture-before-show PID pattern | Must capture frontmost PID before overlay steals focus | Good -- reliable context |

---
*Last updated: 2026-02-28 after v0.1.1 milestone start*
