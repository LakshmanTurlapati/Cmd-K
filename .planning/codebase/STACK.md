# Technology Stack

**Analysis Date:** 2026-02-21

## Languages

**Primary:**
- TypeScript 5.3.3 - VSCode extension (`/extension/src/**/*.ts`)
- JavaScript (Node.js) - CLI implementation (`/cli/**/*.js`)
- Electron framework uses CommonJS

**Secondary:**
- HTML/CSS - VSCode webview UI and Electron renderer

## Runtime

**Environment:**
- Node.js 20.x (specified in extension devDependencies)
- VSCode 1.85.0+ (extension engine requirement)
- Electron 28.0.0 (CLI standalone application)

**Package Manager:**
- npm 7+ (inferred from package-lock.json)
- Lockfile: `package-lock.json` present in both `/extension` and `/cli`

## Frameworks

**Core:**
- VSCode Extension API 1.85.0 - Powers the extension implementation
- Electron 28.0.0 - CLI standalone overlay application (`/cli/main.js`)
- Webpack 5.89.0 - Bundles extension TypeScript code

**Testing:**
- VSCode Test CLI 0.0.4 - VSCode extension testing framework
- VSCode Test Electron 2.3.8 - Electron-based test runner
- Mocha (implied via @types/mocha 20.x) - Test framework

**Build/Dev:**
- Webpack 5.89.0 with ts-loader 9.5.1 - TypeScript compilation and bundling
- ESLint 8.56.0 with @typescript-eslint plugins - Code linting
- @typescript-eslint/eslint-plugin 6.15.0 - TS-specific linting rules
- @typescript-eslint/parser 6.15.0 - TS parsing for ESLint

## Key Dependencies

**Critical:**
- openai 4.77.0 (extension), 4.73.0 (cli) - OpenAI API client for GPT models
- @anthropic-ai/sdk 0.32.1 - Anthropic API client for Claude models
- axios 1.7.9 - HTTP client for xAI (Grok) API calls via custom request handling

**Infrastructure:**
- electron-store 8.2.0 - Persistent configuration storage for CLI (`/cli/main.js`)
- vscode (npm package) - VSCode API type definitions and development

**Types:**
- @types/vscode 1.85.0 - VSCode extension type definitions
- @types/node 20.x - Node.js type definitions

## Configuration

**Environment:**
- Configuration stored in VSCode settings (extension) via `terminalAI.*` namespace
- Configuration stored in electron-store (CLI) at `~/.cmdk.pid` for process management and `apiKeys` object for credentials
- No .env file present - API keys managed through VSCode settings or Electron storage

**Build:**
- `webpack.config.js` - Bundles extension from `src/extension.ts` to `dist/extension.js`
- `tsconfig.json` - TypeScript compilation targeting ES2020, strict mode enabled
- ESLint configuration implied from scripts, likely `.eslintrc.json` or embedded config

## Platform Requirements

**Development:**
- Node.js 20.x
- npm 7+
- VSCode 1.85.0 or later (for extension development)
- TypeScript 5.3.3

**Production:**
- Extension: VSCode 1.85.0+ on macOS, Windows, or Linux
- CLI: Electron-based standalone app (macOS focus based on code patterns like `app.dock.hide()`)

## Build Output

- Extension: `dist/extension.js` (webpack bundled from TypeScript)
- CLI: Direct Node.js/Electron execution of `main.js` via `bin/cli.js`

---

*Stack analysis: 2026-02-21*
