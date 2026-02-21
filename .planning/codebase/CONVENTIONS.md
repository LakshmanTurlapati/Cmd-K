# Coding Conventions

**Analysis Date:** 2026-02-21

## Naming Patterns

**Files:**
- PascalCase for classes and main exports: `ConfigManager.ts`, `TerminalManager.ts`, `OpenAIProvider.ts`
- camelCase for utility/interface files: `promptInput.ts`, `contextBuilder.ts`, `streamHandler.ts`
- snake_case for directories grouping related functionality: `aiProviders/`, `webview/`, `validators/`

**Functions:**
- camelCase for all methods and functions: `getConfig()`, `validateConfig()`, `generateCommand()`
- Private methods prefixed with underscore: `_getHtmlContent()`, `_handleMessage()`, `_generateCommand()`
- Async functions follow camelCase: `async generateCommandStream()`

**Variables:**
- camelCase for all variables: `terminalHistory`, `maxHistorySize`, `userPrompt`
- Constants use UPPERCASE_SNAKE_CASE: `CONFIG_SECTION`, `apiBaseUrl` (when static/readonly properties)
- Private class properties use underscore prefix: `_panel`, `_disposables`

**Types & Interfaces:**
- PascalCase for all interfaces and types: `AIProviderConfig`, `CommandGenerationContext`, `PromptResult`
- Include suffix for clarity: `*Config`, `*Manager`, `*Provider`, `*Handler`

## Code Style

**Formatting:**
- No Prettier configuration detected, following manual TypeScript conventions
- 2-space indentation observed throughout
- Single quotes for strings: `'openai'`, `'anthropic'`
- Trailing commas in multiline objects and arrays (not enforced)

**Linting:**
- ESLint configured in `extension/.eslintrc.json`
- TypeScript-ESLint parser with warnings (not errors) for:
  - `@typescript-eslint/naming-convention` - warn
  - `@typescript-eslint/semi` - warn
  - `curly` - warn
  - `eqeqeq` - warn
  - `no-throw-literal` - warn
- Semicolons optional (rule set to off)

## Import Organization

**Order:**
1. External packages: `import * as vscode from 'vscode'`
2. Node modules: `import * as path from 'path'`, `import * as fs from 'fs'`
3. SDK imports: `import OpenAI from 'openai'`, `import Anthropic from '@anthropic-ai/sdk'`
4. Local imports: `import { ConfigManager } from '../config'`

**Path Aliases:**
- No path aliases configured in tsconfig (uses relative paths)
- Relative imports used throughout: `'../config'`, `'./base'`, `'../webview/settingsPanel'`

## Error Handling

**Patterns:**
- Try-catch blocks wrap async operations returning Promise results
- Error objects destructured with optional chaining: `error?.message`, `error?.status`
- Specific error codes checked before generic messages: `error.status === 401` for auth errors
- Custom Error thrown with descriptive messages: `throw new Error('OpenAI API key is invalid...')`
- Provider-specific error handling: OpenAI checks `error.status`, Anthropic checks `error.status`, xAI checks `error.response?.status`

**Example patterns from `openai.ts`:**
```typescript
catch (error: any) {
  if (error.status === 401) {
    throw new Error('OpenAI API key is invalid. Please check your settings.');
  }
  throw new Error(`OpenAI Error: ${error.message || 'Unknown error occurred'}`);
}
```

**Example pattern from `xai.ts`:**
```typescript
catch (error: any) {
  if (error.response?.status === 401) {
    throw new Error('xAI API key is invalid. Please check your settings.');
  }
  const errorMessage = error.response?.data?.error?.message || error.message || 'Unknown error occurred';
  throw new Error(`xAI Error: ${errorMessage}`);
}
```

## Logging

**Framework:** Native `console` API (no logging library)

**Patterns:**
- Minimal logging in codebase
- `console.log()` only for activation confirmation: `console.log('CMD+K extension is now active')`
- `console.error()` for exception logging: `console.error('CMD+K Error:', error)`
- Errors logged to console after user-facing notification

**Observed in `extension.ts`:**
```typescript
console.log('CMD+K extension is now active');
// ...
vscode.window.showErrorMessage(`CMD+K Error: ${error.message || 'Unknown error'}`);
console.error('CMD+K Error:', error);
```

## Comments

**When to Comment:**
- JSDoc blocks for public methods with parameters and return types
- Single-line comments for non-obvious logic blocks
- Comments explain the "why", not the "what"

**JSDoc/TSDoc Pattern:**
```typescript
/**
 * Gets current extension configuration
 */
static getConfig(): ExtensionConfig {
  // implementation
}

/**
 * Validates that the selected provider is configured
 */
static validateConfig(config: ExtensionConfig): { valid: boolean; message?: string } {
  // implementation
}

/**
 * Captures command when it's sent to terminal
 */
captureCommand(terminal: vscode.Terminal, command: string): void {
  // implementation
}
```

## Function Design

**Size:** Most functions under 50 lines, longest are provider implementations (80-100 lines for streaming)

**Parameters:**
- Constructor injection for dependencies: `constructor(private terminalManager: TerminalManager)`
- Method callbacks passed as function parameters: `onChunk: (chunk: string) => void`
- Optional parameters last: `buildContext(terminal: vscode.Terminal, userPrompt: string, workspacePath?: string)`

**Return Values:**
- Functions returning Promise<T> for async operations
- Union types for action responses: `'review' | 'execute' | 'cancel'`
- Nullable returns with explicit null: `Promise<PromptResult | null>`
- Tuple-like returns via objects: `{ valid: boolean; message?: string }`

## Module Design

**Exports:**
- Named class exports: `export class ConfigManager`, `export class TerminalManager`
- Type exports for interfaces: `export interface AIProviderConfig`, `export type AIProviderType`
- Static utility classes (no instances): `ConfigManager` has all static methods
- Factory/service classes instantiated: `TerminalManager`, `StreamHandler`

**Barrel Files:**
- Not used - direct imports from individual files
- Each file imports only what it needs from other modules

**Architecture Notes:**
- Abstract base class pattern: `BaseAIProvider` defines interface, subclasses implement
- Configuration manager pattern: `ConfigManager` centralizes all VSCode settings access
- Strategy pattern: Provider swapping via `AIProviderType` union
- Observer pattern: VSCode event listeners in extension activation

---

*Convention analysis: 2026-02-21*
