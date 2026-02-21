# Codebase Structure

**Analysis Date:** 2026-02-21

## Directory Layout

```
CMD + K/
├── extension/                          # VSCode extension implementation
│   ├── src/                            # TypeScript source
│   │   ├── extension.ts                # Entry point, activation/deactivation
│   │   ├── config.ts                   # Configuration management
│   │   ├── terminalManager.ts          # Terminal interaction and history
│   │   ├── contextBuilder.ts           # Build AI context from terminal state
│   │   ├── aiProviders/                # AI provider implementations
│   │   │   ├── base.ts                 # Abstract base + interfaces
│   │   │   ├── openai.ts               # OpenAI implementation
│   │   │   ├── anthropic.ts            # Anthropic implementation
│   │   │   └── xai.ts                  # xAI implementation
│   │   ├── webview/                    # VSCode webview components
│   │   │   ├── terminalOverlay.ts      # Terminal command overlay UI
│   │   │   ├── settingsPanel.ts        # Settings configuration panel
│   │   │   └── html/                   # HTML files for webviews
│   │   │       ├── overlay.html        # Overlay UI markup
│   │   │       └── settings.html       # Settings UI markup
│   │   ├── streaming/                  # Streaming logic
│   │   │   └── streamHandler.ts        # Handle AI response streaming
│   │   ├── ui/                         # Input UI components
│   │   │   └── promptInput.ts          # Prompt input dialog
│   │   └── validators/                 # Input validation
│   │       └── apiValidator.ts         # API configuration validation
│   ├── dist/                           # Built extension (generated)
│   ├── icons/                          # VSCode extension icons
│   ├── package.json                    # Extension dependencies
│   └── webpack.config.js               # Webpack build config
│
├── cli/                                # Standalone Electron CLI implementation
│   ├── main.js                         # Electron main process
│   ├── preload.js                      # IPC preload script
│   ├── ai.js                           # AI generation logic (shared)
│   ├── renderer/                       # Electron renderer process
│   │   ├── index.html                  # UI entry point
│   │   ├── css/                        # Styles
│   │   └── js/                         # Renderer logic
│   ├── bin/                            # CLI executables
│   │   └── cli.js                      # Command-line interface
│   └── package.json                    # CLI dependencies
│
└── .planning/                          # Planning documents
    └── codebase/                       # Architecture analysis
        ├── ARCHITECTURE.md
        ├── STRUCTURE.md
        ├── STACK.md
        ├── INTEGRATIONS.md
        ├── CONVENTIONS.md
        ├── TESTING.md
        └── CONCERNS.md
```

## Directory Purposes

**extension/src:**
- Purpose: Main VSCode extension implementation
- Contains: TypeScript modules for extension lifecycle, terminal interaction, AI providers
- Key files: `extension.ts` (entry), `config.ts` (settings), `terminalManager.ts` (terminal ops)

**extension/src/aiProviders:**
- Purpose: Pluggable AI provider implementations
- Contains: OpenAI, Anthropic, xAI provider classes
- Pattern: All extend BaseAIProvider, implement generateCommand() and generateCommandStream()

**extension/src/webview:**
- Purpose: VSCode webview UI components
- Contains: Panel managers and HTML markup for overlays and settings
- Key files: `terminalOverlay.ts` (main UI), `settingsPanel.ts` (config UI)

**extension/src/streaming:**
- Purpose: Streaming command generation
- Contains: StreamHandler for managing AI response streaming
- Key files: `streamHandler.ts` (orchestrates streaming from providers)

**extension/dist:**
- Purpose: Built extension output
- Generated: Yes (webpack output)
- Committed: No

**cli:**
- Purpose: Standalone Electron-based CLI overlay
- Contains: Electron main/preload/renderer, AI integration, daemon management
- Architecture: Separate from VSCode extension, runs as independent process

## Key File Locations

**Entry Points:**
- `extension/src/extension.ts`: VSCode extension activation and command registration
- `cli/main.js`: Electron application main process
- `cli/bin/cli.js`: CLI executable (node entry point for `cmdk` command)

**Configuration:**
- `extension/src/config.ts`: All extension configuration (ExtensionConfig class)
- `extension/package.json`: VSCode extension manifest and dependencies
- `cli/package.json`: CLI Electron dependencies

**Core Logic:**
- `extension/src/terminalManager.ts`: Terminal history and shell detection
- `extension/src/contextBuilder.ts`: Assemble AI context
- `extension/src/aiProviders/`: AI provider implementations
- `extension/src/streaming/streamHandler.ts`: Streaming orchestration
- `cli/ai.js`: AI generation logic for CLI

**Testing:**
- `extension/` has test infrastructure in `package.json` but no test files visible
- Tests would run: `npm run pretest` followed by `npm test`

**Styling/UI:**
- `extension/src/webview/html/`: HTML markup for webviews
- `cli/renderer/`: Electron renderer HTML/CSS

## Naming Conventions

**Files:**
- TypeScript modules: camelCase (e.g., `terminalManager.ts`, `contextBuilder.ts`)
- Provider classes: Named pattern `[Provider]Provider` (e.g., `OpenAIProvider`)
- HTML files: descriptive names (e.g., `overlay.html`, `settings.html`)
- Directories: camelCase for logical grouping (e.g., `aiProviders`, `webview`)

**Directories:**
- Feature/concern grouping: `aiProviders`, `webview`, `streaming`, `validators`, `ui`
- Lowercase plural for collections of related items

**Classes:**
- PascalCase: `TerminalManager`, `ConfigManager`, `StreamHandler`, `OpenAIProvider`

**Functions/Methods:**
- camelCase: `generateCommand()`, `buildContext()`, `captureCommand()`, `detectShell()`

**Interfaces:**
- PascalCase prefixed with capital I or descriptive name: `TerminalInfo`, `CommandGenerationContext`, `AIProvider`, `ExtensionConfig`

## Where to Add New Code

**New Feature (command generation):**
- Primary code: `extension/src/` (new module if significant)
- Terminal interaction: `extension/src/terminalManager.ts`
- AI integration: `extension/src/aiProviders/` (if new provider)
- Streaming: `extension/src/streaming/streamHandler.ts`
- UI: `extension/src/webview/` (new webview or update existing)

**New AI Provider:**
- Implementation: `extension/src/aiProviders/[provider].ts`
- Extend: `BaseAIProvider` from `extension/src/aiProviders/base.ts`
- Register: Add case in `StreamHandler.generate()` switch statement
- Config: Add type/interface to `ExtensionConfig` in `extension/src/config.ts`
- Update: Add provider to `package.json` configuration schema

**New UI Component:**
- Input dialogs: `extension/src/ui/[component].ts`
- Webviews: `extension/src/webview/[panel].ts` with corresponding HTML in `extension/src/webview/html/`
- Message handling: Add case in `_handleMessage()` method of webview class

**Utilities/Helpers:**
- Shared terminal logic: `extension/src/terminalManager.ts`
- Context assembly: `extension/src/contextBuilder.ts`
- Configuration: `extension/src/config.ts`
- Validation: `extension/src/validators/[validator].ts`

**CLI-specific code:**
- Electron main: `cli/main.js`
- Renderer process: `cli/renderer/`
- AI generation: `cli/ai.js` (shared with main)

## Special Directories

**extension/dist:**
- Purpose: Webpack-compiled output (extension.js and dependencies)
- Generated: Yes (build artifact from `npm run package`)
- Committed: No (in .gitignore)

**extension/icons:**
- Purpose: VSCode extension icon assets
- Generated: No
- Committed: Yes

**cli/node_modules:**
- Purpose: npm dependencies for CLI
- Generated: Yes (from package.json)
- Committed: No

**extension/node_modules:**
- Purpose: npm dependencies for VSCode extension
- Generated: Yes (from package.json)
- Committed: No

---

*Structure analysis: 2026-02-21*
