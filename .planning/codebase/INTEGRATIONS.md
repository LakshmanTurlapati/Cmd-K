# External Integrations

**Analysis Date:** 2026-02-21

## APIs & External Services

**LLM Providers:**
- **OpenAI** - GPT-4, GPT-4o, GPT-4-turbo, GPT-3.5-turbo model access
  - SDK/Client: openai 4.77.0 (extension), 4.73.0 (cli)
  - Auth: API key via `terminalAI.openai.apiKey` (extension) or stored in electron-store (cli)
  - Implementation: `src/aiProviders/openai.ts` uses OpenAI client directly
  - Models configured via `terminalAI.openai.model` (extension) or stored config (cli)

- **Anthropic** - Claude Sonnet 4.5, Claude Opus 4.2, Claude 3.5 Sonnet, Claude 3 Opus
  - SDK/Client: @anthropic-ai/sdk 0.32.1
  - Auth: API key via `terminalAI.anthropic.apiKey` (extension) or stored in electron-store (cli)
  - Implementation: `src/aiProviders/anthropic.ts` uses Anthropic client directly
  - Models configured via `terminalAI.anthropic.model`
  - Supports streaming via `messages.create()` with `stream: true`

- **xAI (Grok)** - grok-beta, grok-4, grok-4-fast-reasoning, grok-code-fast-1, grok-3, grok-3-mini, grok-2-latest
  - SDK/Client: axios 1.7.9 (custom HTTP client)
  - Auth: API key via `terminalAI.xai.apiKey` (extension) or stored in electron-store (cli)
  - Implementation: `src/aiProviders/xai.ts` makes custom axios requests to `https://api.x.ai/v1/chat/completions`
  - Header format: `Authorization: Bearer {apiKey}`
  - Supports streaming via `stream: true` parameter with SSE (Server-Sent Events) response parsing

## Data Storage

**Databases:**
- None detected - application is stateless for command generation

**File Storage:**
- Local filesystem only - stores VSCode extension artifacts and configuration
- CLI stores process PID at `~/.cmdk.pid` for daemon management
- electron-store persists configuration to user's home directory

**Caching:**
- Terminal history cached in memory (not persistent)
  - Extension: stored in `TerminalManager` class (`src/terminalManager.ts`)
  - Limited to `maxHistoryLines` configuration (default 20, max 100)
  - Cleared on terminal close or extension deactivation

**Configuration Storage:**
- VSCode settings storage: `terminalAI.*` configuration namespace
- Electron-store: persists to platform-specific application data directory
  - Stores: provider selection, model selection, API keys for all providers

## Authentication & Identity

**Auth Provider:**
- Custom - Each AI provider manages its own API key authentication
- No centralized auth system or user accounts
- API keys stored locally in VSCode settings or electron-store
- No encrypted storage - keys stored as plain text in VSCode settings or electron-store JSON files

**API Key Management:**
- OpenAI: `sk-*` format keys
- Anthropic: `sk-ant-*` format keys
- xAI: `xai-*` format keys
- Each provider validated on first use via `validateConfig()` in `src/aiProviders/base.ts`

## Monitoring & Observability

**Error Tracking:**
- None detected - errors logged to console

**Logs:**
- Console-based logging
  - Extension: uses `console.log()` and `console.error()` (visible in VSCode debug console)
  - CLI: uses `console.log()` for user feedback
- Error messages displayed to user via VSCode notification system or CLI output

## CI/CD & Deployment

**Hosting:**
- Extension: VSCode Marketplace (planned, not yet published per README)
- CLI: Distributed via npm as standalone Electron application

**CI Pipeline:**
- None detected - no GitHub Actions, GitLab CI, or similar workflows found

**Build Commands:**
- Extension: `npm run package` builds for publication (webpack production mode)
- CLI: `npm start` runs Electron, `npm run build` uses electron-builder

## Environment Configuration

**Required env vars:**
- None required at runtime
- `SHELL` environment variable read at runtime for context (`process.env.SHELL`)
- API keys must be provided via VSCode settings or CLI configuration UI

**Secrets location:**
- VSCode settings: `~/.config/Code/User/settings.json` (Linux/macOS equivalent)
- electron-store: Platform-specific app data directories
- No environment variable support for secrets

## Webhooks & Callbacks

**Incoming:**
- None detected

**Outgoing:**
- None detected - application is read-only consumer of LLM APIs

## API Response Patterns

**OpenAI:**
- Request: `POST https://api.openai.com/v1/chat/completions`
- Response: JSON with `choices[0].message.content` containing generated command
- Streaming: SSE with `choices[0].delta.content` chunks

**Anthropic:**
- Request: `POST https://api.anthropic.com/v1/messages`
- Response: JSON with `content[0].text` containing generated command
- Streaming: Server-Sent Events with `content_block_delta` events containing `delta.text`

**xAI:**
- Request: `POST https://api.x.ai/v1/chat/completions` (OpenAI-compatible)
- Response: JSON with `choices[0].message.content` containing generated command
- Streaming: Server-Sent Events (custom parsing in `src/aiProviders/xai.ts`)
- Bearer token authentication in Authorization header

## Terminal Integration

**VSCode Extension:**
- Accesses active terminal via `vscode.window.activeTerminal`
- Reads terminal history from `TerminalManager` (in-memory cache)
- Injects commands via `terminal.sendText()` or shows confirmation dialog

**CLI Application:**
- Clipboard-based command injection: copies command to clipboard, then uses macOS AppleScript to paste
- Targets terminal applications: Terminal.app, iTerm2, Hyper, Alacritty, kitty, WezTerm
- Fallback: sends Cmd+V keystroke to whatever has focus

---

*Integration audit: 2026-02-21*
