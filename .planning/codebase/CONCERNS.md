# Codebase Concerns

**Analysis Date:** 2026-02-21

## Tech Debt

**API Key Storage (Extension):**
- Issue: VSCode extension stores API keys as plain text in settings (not encrypted by default)
- Files: `extension/src/config.ts`, `extension/src/webview/settingsPanel.ts`
- Impact: API keys exposed if VSCode settings backup is compromised. Highly sensitive credentials stored insecurely.
- Fix approach: Implement VSCode SecretStorage API to encrypt API keys. Requires using `context.secrets.store()` and `context.secrets.get()` instead of plain `config.update()`.

**API Key Storage (CLI - Electron):**
- Issue: electron-store package stores configuration (including API keys) as plain JSON file on disk
- Files: `cli/main.js` (lines 8-14)
- Impact: API keys persist unencrypted in filesystem. Anyone with filesystem access can read keys.
- Fix approach: Encrypt sensitive data before storing. Use OS keychain (keytar package) for secure credential storage.

**No Input Validation on Command Context:**
- Issue: Terminal history is collected and sent directly to AI without sanitization
- Files: `extension/src/terminalManager.ts` (lines 38-41), `extension/src/contextBuilder.ts` (lines 19-27)
- Impact: Sensitive information in terminal history (passwords, tokens, secrets) is sent to external AI APIs
- Fix approach: Add filtering rules to strip common secrets (API keys, tokens, passwords) from history before sending

**Unhandled Edge Cases in Stream Processing:**
- Issue: Stream error handling is incomplete. xAI provider silently skips invalid JSON chunks
- Files: `extension/src/aiProviders/xai.ts` (lines 90-99)
- Impact: Malformed responses could result in incomplete command generation without user awareness
- Fix approach: Log skipped chunks and notify user of partial responses. Implement timeout handling for stuck streams.

**Inconsistent Error Messages:**
- Issue: Different providers return different error formats; message text passed directly from API to user
- Files: `extension/src/aiProviders/openai.ts`, `extension/src/aiProviders/anthropic.ts`, `extension/src/aiProviders/xai.ts`
- Impact: User confusion with inconsistent error reporting. Potential exposure of internal API details.
- Fix approach: Standardize all error messages to user-friendly format. Never expose raw API response text.

**Configuration Hard-Coded Model Names:**
- Issue: Model names are duplicated across files and hard-coded as string arrays
- Files: `extension/src/config.ts` (lines 119-147), `cli/ai.js` (lines 115-134)
- Impact: Difficult to maintain. If models deprecate, changes needed in multiple places. Models may go offline.
- Fix approach: Create centralized model registry. Validate models at startup. Implement fallback to available models.

**Terminal History Identifier Fragile:**
- Issue: Terminal ID uses terminal name + processId, which can be unreliable across terminal instances
- Files: `extension/src/terminalManager.ts` (lines 101-105)
- Impact: History may leak between terminals with same name. Memory leaks possible if terminals aren't properly cleaned up.
- Fix approach: Use vscode.Terminal object identity or UUID instead of process ID. Implement Map.WeakMap for auto-cleanup.

---

## Known Bugs

**Anthropic Provider Not Following Own Pattern:**
- Issue: Anthropic provider sends system prompt as user message instead of using system field
- Files: `extension/src/aiProviders/anthropic.ts` (lines 20-28)
- Symptoms: System instructions not prioritized. Inconsistent behavior with OpenAI/xAI providers.
- Trigger: Any Anthropic command generation
- Workaround: Switch to OpenAI or xAI provider. Anthropic API still works because system context is in first message.

**Race Condition in Stream Handler:**
- Issue: StreamHandler doesn't validate that currentTerminal still exists when stream completes
- Files: `extension/src/webview/terminalOverlay.ts` (lines 237-251)
- Trigger: User closes terminal while command is generating
- Symptoms: Chunks stream to closed terminal, postMessage to disposed panel
- Workaround: Don't close terminal during generation

**Missing Message User Parameter in Anthropic:**
- Issue: Anthropic API requires system field in messages.create(), but code passes system prompt as user content
- Files: `extension/src/aiProviders/anthropic.ts` (lines 20-28, 50-60)
- Trigger: Using Anthropic provider
- Symptoms: Works but system instructions treated as user message, reducing effectiveness
- Workaround: None - code works despite being wrong pattern

---

## Security Considerations

**Arbitrary Command Execution Risk:**
- Risk: Generated commands are executed directly without validation. User could generate destructive commands (rm -rf, format disk, etc.)
- Files: `extension/src/terminalManager.ts` (lines 87-91), `cli/main.js` (lines 182-222)
- Current mitigation: Review mode enabled by default in extension. User sees command before execution.
- Recommendations:
  - Implement command allowlist for "execute immediately" mode (e.g., only safe commands)
  - Add warnings for high-risk commands (rm, dd, format, mkfs, etc.)
  - Implement undo/history to recover from mistakes
  - Consider sandboxing execution in CLI version

**API Key Exposure in Logs:**
- Risk: API keys logged in error messages or console output
- Files: Potentially in error handlers across all AI provider files
- Current mitigation: Error messages don't explicitly log keys, but API error responses might contain request details
- Recommendations:
  - Implement API request interceptor to redact auth headers from logs
  - Review all console.error() calls to ensure no PII

**Terminal Context Leakage:**
- Risk: Terminal history sent to external APIs may contain credentials, tokens, secrets
- Files: `extension/src/contextBuilder.ts` (line 19), `cli/ai.js` (lines 80-89)
- Current mitigation: User consent implied by using the feature. README mentions reviewing commands.
- Recommendations:
  - Implement automatic filtering of common secret patterns (API keys, tokens, passwords)
  - Add warning when history contains likely secrets
  - Provide "anonymous mode" that excludes history
  - Allow user-defined filters for custom secret patterns

**OSScript Injection in CLI:**
- Risk: Command string passed to osascript without escaping could allow injection if command comes from untrusted source
- Files: `cli/main.js` (lines 211)
- Current mitigation: Command comes from user input or AI, sanitized by AI system prompt
- Recommendations:
  - Use AppleScript parameter passing instead of string interpolation
  - Implement proper argument escaping

---

## Performance Bottlenecks

**Terminal History Growth Unbounded During Session:**
- Problem: Each terminal accumulates history. If extension stays active for long session, memory usage grows.
- Files: `extension/src/terminalManager.ts` (lines 9-10, 30-31)
- Cause: maxHistorySize only limits per-terminal. If user has many terminals, total memory usage is maxHistorySize * numTerminals.
- Improvement path: Implement global memory cap. Implement LRU eviction across all terminals. Periodically trim old entries.

**API Validation Called Every Time Settings Saved:**
- Problem: Settings panel validates all three providers on every save by making API calls
- Files: `extension/src/webview/settingsPanel.ts` (lines 100, 122-171)
- Cause: _validateProviders() makes 3 HTTP requests even if only one API key changed
- Improvement path: Only validate changed providers. Cache validation results for N minutes.

**No Caching of Model Lists:**
- Problem: ConfigManager.getModelsForProvider() called repeatedly but always returns hardcoded arrays
- Files: `extension/src/config.ts` (lines 119-147)
- Cause: UI queries model list on provider change, each query re-allocates array
- Improvement path: Cache model lists in ConfigManager. Refresh on startup only.

**Streaming Not Cancelled When Panel Closes:**
- Problem: If user closes overlay while command streaming, stream continues and chunks still emitted
- Files: `extension/src/webview/terminalOverlay.ts` (lines 59-72)
- Cause: No AbortController or cancellation mechanism
- Improvement path: Implement AbortController. Cancel stream when panel disposed.

---

## Fragile Areas

**Webview HTML File Loading:**
- Files: `extension/src/webview/terminalOverlay.ts` (lines 269-284), `extension/src/webview/settingsPanel.ts` (lines 173-188)
- Why fragile: Hard-coded file paths with no validation. If HTML files move or missing, extension silently fails to load UI
- Safe modification: Add existence check. Throw meaningful error if files missing. Consider inlining HTML for robustness.
- Test coverage: No tests for webview initialization. Need tests that verify HTML can be loaded.

**Stream Parser Regex Patterns:**
- Files: `extension/src/aiProviders/openai.ts` (line 83), `extension/src/aiProviders/xai.ts` (line 123), `extension/src/aiProviders/anthropic.ts` (line 84)
- Why fragile: Same regex `/```[\w]*\n?/g` used in 3 places. If markdown format changes, all need updating. Regex doesn't handle nested backticks.
- Safe modification: Extract to shared utility function. Test edge cases (triple-backticks with whitespace, missing newline, multiple blocks).
- Test coverage: No unit tests for cleanCommand() methods. Need tests for various markdown formats.

**Terminal Shell Detection:**
- Files: `extension/src/terminalManager.ts` (lines 107-127)
- Why fragile: Detects shell by checking if path string includes shell name (case-insensitive). Fragile to path variations, custom shells, symlinks.
- Safe modification: Use full path comparison. Check for executable vs. string inclusion. Handle PowerShell variants (pwsh, pwsh-preview).
- Test coverage: No tests for edge cases (pwsh vs powershell, custom shell paths).

**Disposables Not Always Cleaned:**
- Files: `extension/src/extension.ts` (lines 46-51), multiple webview classes
- Why fragile: Disposables arrays managed manually. Easy to forget to push new subscriptions. If exception thrown, cleanup skipped.
- Safe modification: Use try-finally blocks. Consider helper function to push disposables. Review all event listeners.
- Test coverage: No tests for cleanup on extension deactivate.

---

## Scaling Limits

**VSCode Extension Single Instance:**
- Current capacity: One extension instance per VSCode window. All terminals in window share same context.
- Limit: Memory is shared across all terminals. Large number of open terminals (50+) with history will consume significant memory.
- Scaling path: Implement session-based history cleanup. Implement off-memory storage (database) for old history. Implement per-project history limits.

**API Rate Limits Not Handled:**
- Current capacity: Each AI provider has rate limits (OpenAI: 3,500 requests/min, Anthropic: 50 requests/min depending on plan).
- Limit: No throttling or queue. If user generates commands rapidly, will hit rate limits.
- Scaling path: Implement request queue. Implement exponential backoff on 429 responses. Cache frequently generated commands.

**CLI Overlay Not Multi-Display Aware:**
- Current capacity: Window centered on primary display. Multi-monitor setups may have overlay appear on wrong screen.
- Limit: Works on single screen. Multi-display usage is problematic.
- Scaling path: Detect active monitor. Center on monitor with keyboard focus. Remember last used monitor.

---

## Dependencies at Risk

**OpenAI SDK Version Mismatch:**
- Risk: Extension uses openai@^4.77.0, CLI uses openai@^4.73.0. Different major versions could have breaking API differences.
- Impact: If OpenAI SDK updates, behavior may diverge between tools.
- Migration plan: Lock both to same minor version. Implement compatibility tests.

**Electron 28 (CLI):**
- Risk: Electron 28 (released 2024) not actively maintained. Current stable is 32+.
- Impact: Security vulnerabilities not patched. Missing OS-level integrations.
- Migration plan: Update to Electron 32+. Review breaking changes in Preload context isolation.

**Node Module Vulnerable Dependencies:**
- Risk: Transitive dependencies not audited. electron-store has no built-in encryption. axios < 1.8 has known vulnerabilities.
- Impact: Potential security holes from deep dependencies.
- Migration plan: Run `npm audit`. Use npm audit fix. Consider dependency scanner in CI.

**VSCode API Stability:**
- Risk: Extension uses VSCode 1.85.0+ APIs. Future VSCode versions could deprecate used APIs.
- Impact: Extension could break on VSCode update.
- Migration plan: Monitor VSCode release notes. Add deprecation checks. Test on VSCode pre-release versions.

---

## Missing Critical Features

**No Command History or Favorites:**
- Problem: Generated commands aren't saved. User must regenerate frequently-used commands.
- Blocks: User workflows that would benefit from command library

**No Undo/History for Executed Commands:**
- Problem: Commands executed immediately can't be undone. User can't see what was executed.
- Blocks: Safety. Users can't track what automated the terminal.

**No Multi-Line Command Support:**
- Problem: AI generates command, user can only use single line. Complex multi-step workflows need manual coordination.
- Blocks: Advanced automation workflows

**No Command Explanation Mode:**
- Problem: User generates command but doesn't understand what it does. Risk of executing dangerous commands.
- Blocks: Educational use case. Safety for beginners.

**No Offline Mode:**
- Problem: Extension requires internet to generate commands. Can't work when offline.
- Blocks: Offline usage, air-gapped environments

---

## Test Coverage Gaps

**No Unit Tests:**
- What's not tested: All TypeScript code in extension. All JavaScript code in CLI.
- Files: All files in `extension/src/`, `cli/*.js`
- Risk: High. Refactoring breaks nothing without tests. Hard to catch regressions. New contributors can't verify changes.
- Priority: High

**No Integration Tests:**
- What's not tested: VSCode extension webview communication. Terminal context building. AI provider integration.
- Files: `extension/src/webview/`, `extension/src/streaming/`, `extension/src/aiProviders/`
- Risk: High. Webview-to-extension IPC is critical path. Hard to debug without tests.
- Priority: High

**No E2E Tests:**
- What's not tested: Full flow from CMD+K keypress to command execution. CLI overlay activation to command injection.
- Files: Extension full flow, CLI full flow
- Risk: Very High. Users experience these flows. Breaking changes undetected.
- Priority: High

**Missing Edge Case Tests:**
- What's not tested: Empty terminal history, malformed API responses, network timeouts, closed terminals during generation, concurrent generations
- Risk: Medium. Edge cases cause real user issues.
- Priority: Medium

**No Security Tests:**
- What's not tested: API key not exposed in logs, terminal history contains secrets, command injection prevention
- Files: All files with API interaction
- Risk: Very High. Security bugs have real-world impact.
- Priority: Critical

---

*Concerns audit: 2026-02-21*
