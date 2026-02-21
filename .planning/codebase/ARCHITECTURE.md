# Architecture

**Analysis Date:** 2026-02-21

## Pattern Overview

**Overall:** Multi-platform plugin architecture with provider-agnostic AI command generation

**Key Characteristics:**
- Layered architecture with clear separation between UI, business logic, and external integrations
- Plugin-based design supporting VSCode extension and Electron-based CLI
- Strategy pattern for pluggable AI providers (OpenAI, Anthropic, xAI)
- Streaming-first architecture for real-time command generation feedback
- Webview-based UI for overlays and settings panels

## Layers

**Extension Core Layer:**
- Purpose: VSCode extension lifecycle and command registration
- Location: `src/extension.ts`
- Contains: Extension activation/deactivation, command handlers, disposables
- Depends on: TerminalManager, TerminalOverlay, SettingsPanel, Config
- Used by: VSCode editor process

**Terminal Management Layer:**
- Purpose: Terminal interaction, history tracking, command execution
- Location: `src/terminalManager.ts`
- Contains: History management, shell detection, command streaming
- Depends on: VSCode terminal API
- Used by: TerminalOverlay, StreamHandler, ContextBuilder

**UI/Webview Layer:**
- Purpose: User-facing interfaces for command generation and settings
- Location: `src/webview/terminalOverlay.ts`, `src/webview/settingsPanel.ts`
- Contains: Webview panel management, message handling, HTML rendering
- Depends on: Config, StreamHandler, TerminalManager, PromptInputUI
- Used by: Extension core

**AI Provider Layer:**
- Purpose: Encapsulate API communication with different LLM providers
- Location: `src/aiProviders/`
- Contains: OpenAIProvider, AnthropicProvider, XAIProvider classes
- Depends on: axios, openai SDK, @anthropic-ai/sdk
- Used by: StreamHandler

**Context Builder Layer:**
- Purpose: Assemble context for AI command generation (OS, shell, history, directory)
- Location: `src/contextBuilder.ts`
- Contains: Context assembly from terminal state and workspace
- Depends on: TerminalManager, VSCode API
- Used by: StreamHandler

**Configuration Layer:**
- Purpose: Manage extension settings and config validation
- Location: `src/config.ts`
- Contains: ConfigManager class with validation and utility methods
- Depends on: VSCode configuration API
- Used by: Most modules for accessing settings

## Data Flow

**Command Generation Flow:**

1. User presses Cmd+K in terminal
2. Extension activates `terminalAI.generateCommand` command
3. TerminalOverlay.show() creates webview panel
4. User enters prompt in webview
5. StreamHandler.generate() is invoked
6. StreamHandler creates appropriate AI provider instance
7. ContextBuilder assembles terminal context (shell, OS, history, directory)
8. Provider calls LLM API with system + user prompts
9. Stream chunks returned and fed to onChunk callback
10. Chunks streamed to terminal via TerminalManager.streamTextChunk()
11. Final command displayed in webview for review/execution
12. User executes or modifies command

**State Management:**
- Configuration state: VSCode workspace settings (extender-scoped)
- Terminal history: In-memory Map in TerminalManager (per-session)
- UI state: WebviewPanel instance variables (overlay and settings panels)
- Provider selection: Stored in TerminalOverlay and StreamHandler instances

## Key Abstractions

**AIProvider Interface:**
- Purpose: Abstract LLM communication behind common interface
- Examples: `src/aiProviders/openai.ts`, `src/aiProviders/anthropic.ts`, `src/aiProviders/xai.ts`
- Pattern: Strategy pattern with BaseAIProvider providing common prompt building

**CommandGenerationContext:**
- Purpose: Encapsulate all information needed for command generation
- Definition: `src/aiProviders/base.ts` (interface)
- Contains: OS name, shell type, working directory, terminal history, user prompt

**ExtensionConfig:**
- Purpose: Typed configuration object for all settings
- Definition: `src/config.ts`
- Contains: Provider selection, API keys, models for each provider, UI preferences

## Entry Points

**VSCode Extension Entry:**
- Location: `src/extension.ts`
- Triggers: VSCode activation event (onStartupFinished)
- Responsibilities: Register commands, initialize managers, setup listeners

**Command Handler:**
- Location: `extension.ts#handleGenerateCommand()`
- Triggers: Cmd+K keybinding when terminal is focused
- Responsibilities: Validate active terminal, show overlay, catch errors

**CLI Entry (Electron):**
- Location: `cli/bin/cli.js`
- Triggers: User runs `cmdk` command in shell
- Responsibilities: Start Electron process, manage daemon lifecycle

## Error Handling

**Strategy:** Try-catch with user-facing error messages via VSCode UI

**Patterns:**
- API validation errors → ConfigManager.validateConfig() returns validation result
- Provider errors → Caught in StreamHandler, sent to webview as error message
- Config errors → Displayed as VSCode error notifications
- Terminal errors → Logged to console, user shown notification

## Cross-Cutting Concerns

**Logging:** Console logging (console.log, console.error) throughout codebase

**Validation:**
- Config validation: `src/validators/apiValidator.ts`
- API key presence checks in BaseAIProvider.validateConfig()
- Provider availability checks in ConfigManager.getAvailableProviders()

**Authentication:**
- API keys stored in VSCode workspace settings
- Per-provider key management in ExtensionConfig
- No key logging or exposure in error messages

---

*Architecture analysis: 2026-02-21*
