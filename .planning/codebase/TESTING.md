# Testing Patterns

**Analysis Date:** 2026-02-21

## Test Framework

**Runner:**
- VSCode Test Runner: `@vscode/test-cli` and `@vscode/test-electron`
- No other testing frameworks configured (Jest, Vitest, Mocha not used)
- Config: Not explicitly configured; uses VSCode testing defaults

**Assertion Library:**
- None installed or used in codebase
- VSCode provides native testing via terminal output validation

**Run Commands:**
```bash
npm run compile-tests        # Compile TypeScript test files
npm run watch-tests          # Watch mode for test compilation
npm run pretest              # Run linting and compilation before tests
npm test                     # Run all tests via vscode-test
```

## Test File Organization

**Location:**
- No test files found in codebase
- Pattern expected: co-located or separate `tests/` directory (not implemented)

**Naming:**
- No naming convention established (no tests exist)

**Structure:**
- Would follow VSCode test pattern if tests were added
- Tests would be compiled to `out/` directory via tsconfig

## Current Testing Status

**Coverage:** Not detected - no test files in codebase

**Test Types:**
- Unit tests: Not implemented
- Integration tests: Not implemented
- E2E tests: Not implemented

The codebase currently has zero test coverage. Extension functionality is validated manually through VSCode extension host execution.

## Areas Requiring Tests

**Critical paths without test coverage:**

**1. Configuration Management (`config.ts`)**
- `ConfigManager.getConfig()`: Reads workspace configuration
- `ConfigManager.validateConfig()`: Validates API key presence by provider
- `ConfigManager.getAvailableProviders()`: Filters configured providers
- Should test: config reading, validation logic, provider filtering

**2. AI Provider Implementations (`aiProviders/*.ts`)**

**OpenAI (`openai.ts`):**
- `generateCommand()`: API call to OpenAI, response parsing, error handling
- `generateCommandStream()`: Streaming response handling, chunk accumulation
- `cleanCommand()`: Markdown code block removal, quote stripping
- Should test: happy path generation, API errors (401, network), malformed responses, streaming chunk handling

**Anthropic (`anthropic.ts`):**
- `generateCommand()`: API call to Anthropic with message format
- `generateCommandStream()`: Event-based streaming (content_block_delta events)
- `cleanCommand()`: Same as OpenAI
- Should test: message formatting, event filtering, stream completion

**xAI (`xai.ts`):**
- `generateCommand()`: HTTP POST to custom endpoint
- `generateCommandStream()`: Server-Sent Events (SSE) format parsing
- Response format parsing: `data: {json}\n` format, `[DONE]` sentinel
- Should test: SSE parsing, JSON parse errors (with try-catch skip), stream event handling

**3. Terminal Manager (`terminalManager.ts`)**
- `captureCommand()`: Terminal ID generation, history management, size limiting
- `detectShell()`: Shell path parsing, platform fallbacks
- `getHistory()`: History retrieval by terminal ID
- Should test: command capture with size limits, shell detection on all platforms, history isolation between terminals

**4. API Validator (`validators/apiValidator.ts`)**
- `validateOpenAI()`: API key validation via test request
- `validateAnthropic()`: API key validation via test request
- `validateXAI()`: API key validation via HTTP POST
- Should test: valid/invalid keys, network errors, different error status codes

**5. Context Builder (`contextBuilder.ts`)**
- `buildContext()`: OS name mapping, shell detection, workspace path resolution
- OS detection: darwin -> macOS, win32 -> Windows, linux -> Linux
- Should test: OS name mapping, default working directory fallback

**6. Stream Handler (`streaming/streamHandler.ts`)**
- `generate()`: Provider selection, config merging, error handling
- Provider/model override logic
- Should test: provider selection priority (override > config default), API key validation before generation

**7. UI Components (`ui/promptInput.ts`)**
- `showPrompt()`: Input box handling, event disposal
- `showCommandConfirmation()`: Quick pick item selection
- `withProgress()`: Progress notification wrapping
- Should test: input cancellation, selection handling, progress completion

**8. Webview Overlay (`webview/terminalOverlay.ts`)**
- `show()`: Panel creation, HTML loading, message handling
- `_handleMessage()`: Command routing, provider changes
- `_generateCommand()`: Stream setup, chunk handling, error messaging
- Should test: message dispatching, panel disposal, error recovery

## Suggested Testing Approach

**Unit Test Structure (if implemented):**
```typescript
// Example structure for config.test.ts
import * as assert from 'assert';
import { ConfigManager } from '../src/config';

describe('ConfigManager', () => {
  describe('validateConfig', () => {
    it('should return valid=true when OpenAI key is configured', () => {
      const config = {
        provider: 'openai',
        openai: { apiKey: 'sk-test', model: 'gpt-4o' },
        anthropic: { apiKey: '', model: 'claude-sonnet-4-5-20250929' },
        xai: { apiKey: '', model: 'grok-beta' },
        maxHistoryLines: 20,
        showPreviewBox: true
      };
      const result = ConfigManager.validateConfig(config);
      assert.strictEqual(result.valid, true);
    });

    it('should return valid=false when OpenAI key is missing', () => {
      const config = {
        provider: 'openai',
        openai: { apiKey: '', model: 'gpt-4o' },
        // ... other providers
      };
      const result = ConfigManager.validateConfig(config);
      assert.strictEqual(result.valid, false);
      assert(result.message?.includes('OpenAI'));
    });
  });
});
```

**Integration Test Structure (if implemented):**
```typescript
// Example structure for providers.integration.test.ts
import * as assert from 'assert';
import { OpenAIProvider } from '../src/aiProviders/openai';
import { CommandGenerationContext } from '../src/aiProviders/base';

describe('OpenAIProvider Integration', () => {
  it('should clean markdown code blocks from response', () => {
    const provider = new OpenAIProvider('sk-test', 'gpt-4o');
    const context: CommandGenerationContext = {
      os: 'macOS',
      shell: 'zsh',
      workingDirectory: '/Users/test',
      terminalHistory: [],
      userPrompt: 'list files'
    };
    // Note: This would need mocking to avoid real API calls
    // const command = await provider.generateCommand(context);
    // assert(command && !command.includes('```'));
  });
});
```

## Mocking Strategy (if tests were added)

**What to Mock:**
- VSCode API calls (`vscode.workspace.getConfiguration`)
- External API calls (OpenAI, Anthropic, xAI)
- File system operations (`fs.readFileSync`)
- Terminal/process operations

**What NOT to Mock:**
- Class constructors and internal method calls
- Configuration validation logic
- Command cleaning/parsing logic
- Error message formatting

## Testing Gaps by Severity

**Critical (No coverage):**
- AI provider streaming response handling (especially xAI SSE parsing)
- API error handling and user-facing messages
- Terminal history isolation and size limits

**High (No coverage):**
- Shell detection across platforms
- Configuration validation logic
- Context building with workspace resolution

**Medium (No coverage):**
- Webview message routing
- Provider selection with overrides
- UI input and confirmation flows

---

*Testing analysis: 2026-02-21*

**Key Note:** No tests currently exist. The codebase is validated only through manual testing in VSCode extension host. All identified areas above require test implementation for production quality.
