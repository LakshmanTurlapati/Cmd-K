# CMD+K

## What This Is

A lightweight macOS overlay app that generates terminal commands using AI. Press Cmd+K from anywhere, a prompt overlay appears on top of your active window, you describe what you need in natural language, it generates the right terminal command using xAI (Grok), and pastes it directly into your active terminal. It knows your context -- current directory, recent commands -- without requiring any shell configuration.

## Core Value

The overlay must appear on top of the currently active application and feel instant -- if the overlay doesn't show up where the user is working, nothing else matters.

## Requirements

### Validated

- Existing AI provider integration pattern (prompt engineering, streaming, context assembly) -- existing
- Concept of Cmd+K global hotkey to trigger command generation -- existing

### Active

- [ ] System-wide overlay appears on top of the active window when Cmd+K is pressed
- [ ] Overlay accepts natural language input and generates terminal commands via xAI (Grok)
- [ ] Generated command is pasted directly into the active terminal (Terminal.app, iTerm2, etc.)
- [ ] App detects current working directory of the active terminal without shell plugins
- [ ] App captures recent terminal commands/output for context without shell plugins
- [ ] Streaming response displayed in the overlay as the command is generated
- [ ] User can dismiss the overlay (Escape key) without executing
- [ ] xAI (Grok) provider with API key management
- [ ] Settings UI for API key configuration and model selection
- [ ] App runs as a background daemon with menu bar presence
- [ ] Lightweight resource footprint (Tauri, not Electron)

### Out of Scope

- VS Code extension -- dropped in favor of standalone overlay app
- OpenAI provider -- deferred to later phase
- Anthropic provider -- deferred to later phase
- Windows/Linux support -- macOS first, cross-platform later
- Command history/favorites -- future feature
- Multi-step command workflows -- future feature
- Command explanation mode -- future feature
- Offline mode -- requires internet for AI generation

## Context

**Existing codebase:** There is an Electron-based CLI app and a VS Code extension in the repo. Both are being replaced by a single Tauri-based overlay app. The existing code provides reference for AI prompt engineering, provider integration patterns, and the general UX concept, but the application shell is a complete rebuild.

**Key technical challenge -- overlay positioning:** The current Electron app fails to position the overlay on top of the active window. It appears on the desktop instead. This is the #1 problem to solve. Reference apps that do this well: Superwhisper, Raycast, Alfred, Spotlight.

**Key technical challenge -- terminal context without shell plugins:** Reading terminal state (current directory, recent output) on macOS without requiring users to modify their shell config. Approaches to investigate:
- macOS Accessibility API to read terminal screen content (requires one-time permission grant)
- AppleScript/JXA to query Terminal.app and iTerm2 for current directory
- Process inspection (lsof) to find the terminal shell process and its working directory
- Combination of the above for robust context gathering

**Target terminal apps:** Terminal.app, iTerm2, Hyper, Alacritty, kitty, WezTerm

**AI provider:** xAI (Grok) only for v1. The existing xAI integration uses axios with custom SSE parsing against `https://api.x.ai/v1/chat/completions`. This pattern carries forward.

## Constraints

- **Tech stack**: Tauri (Rust + web frontend) -- chosen for lightweight footprint over Electron
- **Platform**: macOS only for v1
- **Zero setup**: No shell plugins, no .zshrc modifications. One-time macOS accessibility permission is acceptable.
- **Single provider**: xAI (Grok) only for v1. Provider architecture should allow easy addition of OpenAI/Anthropic later.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Drop VS Code extension | Standalone overlay is the real product | -- Pending |
| Tauri over Electron | Lighter weight, smaller binary, less RAM | -- Pending |
| xAI only for v1 | Simplify scope, add other providers later | -- Pending |
| macOS only for v1 | Focus on one platform, nail the overlay UX | -- Pending |
| Zero shell setup | Lower adoption friction, use Accessibility API + AppleScript instead | -- Pending |

---
*Last updated: 2026-02-21 after initialization*
