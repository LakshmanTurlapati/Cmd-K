---
phase: 04-ai-command-generation
plan: 01
subsystem: api
tags: [rust, tauri, xai, grok, sse, streaming, keychain, ipc-channel, eventsource-stream, tokio]

# Dependency graph
requires:
  - phase: 02-settings-configuration
    provides: API key stored in Keychain via keychain.rs (SERVICE/ACCOUNT constants)
  - phase: 03-terminal-context-reading
    provides: AppContext struct (app_name, terminal, console_detected, console_last_line) returned by get_app_context IPC command

provides:
  - stream_ai_response Tauri IPC command that streams xAI /v1/chat/completions tokens via Channel<String>
  - Two-mode AI: terminal mode (command-only) vs assistant mode (conversational) based on shell_type presence
  - Context-aware user message builder using AppContext (CWD, shell, output, running process, console line)
  - Session history support (up to 7 turns = 14 messages) passed as ChatMessage array
  - 10-second hard timeout on SSE streaming loop via tokio::time::timeout

affects: [04-02-frontend-streaming-ui, 04-03-clipboard-copy]

# Tech tracking
tech-stack:
  added:
    - eventsource-stream = "0.2" (SSE line parsing from reqwest bytes_stream)
    - futures-util = "0.3" (StreamExt::next() for async SSE iteration)
    - tokio = { version = "1", features = ["time"] } (hard timeout via tokio::time::timeout)
    - tauri-plugin-http stream feature enabled (bytes_stream() on Response)
  patterns:
    - Channel<String> for Rust-to-frontend token streaming (tauri::ipc::Channel)
    - [DONE] sentinel check before JSON parse in SSE loop
    - Lightweight AppContextView deserialization (only fields needed for prompt building)
    - const system prompt strings with {shell_type} placeholder replacement

key-files:
  created:
    - src-tauri/src/commands/ai.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/Cargo.lock
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Use ACCOUNT='xai_api_key' to match keychain.rs exactly (plan had wrong value 'api-key' - auto-corrected)"
  - "Deserialize context_json into lightweight AppContextView struct instead of frontend-mirrored type to avoid coupling"
  - "fallback AppContextView on JSON parse failure preserves assistant mode behavior rather than returning an error"
  - "History capped at last 14 messages (7 user + 7 assistant) via saturating_sub(14)"

patterns-established:
  - "Pattern: API key always read from Keychain inside Rust command, never passed as IPC argument"
  - "Pattern: [DONE] sentinel checked before serde_json::from_str to avoid JSON parse error on stream termination"
  - "Pattern: Two-mode system prompt selection driven by terminal.shell_type Option presence"
  - "Pattern: context_json String parameter for AppContext IPC boundary (avoids Deserialize coupling to Rust struct)"

requirements-completed: [AICG-01, AICG-02]

# Metrics
duration: 8min
completed: 2026-02-23
---

# Phase 4 Plan 01: AI Command Generation - Rust Backend Summary

**Rust stream_ai_response command using eventsource-stream SSE parsing and Tauri IPC Channel<String>, with two-mode system prompts (terminal vs assistant) driven by AppContext context_json**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-23T02:08:21Z
- **Completed:** 2026-02-23T02:16:00Z
- **Tasks:** 1
- **Files modified:** 5 (1 created, 4 modified)

## Accomplishments

- Created `src-tauri/src/commands/ai.rs` with the full `stream_ai_response` Tauri async command
- API key retrieved from macOS Keychain using the same SERVICE/ACCOUNT constants as `keychain.rs`
- Two distinct system prompts: terminal mode (strict command-only, includes {shell_type}) and assistant mode (2-3 sentences max)
- Context-aware user message includes App, Shell, CWD, Running process, last 25 lines of terminal output, and browser console line
- SSE stream parsed via `eventsource-stream` with `[DONE]` sentinel checked before JSON parse
- Tokens forwarded one-by-one via `tauri::ipc::Channel<String>` to frontend
- Hard 10-second timeout wraps the entire SSE streaming loop via `tokio::time::timeout`
- `cargo check` and `cargo clippy` pass with zero errors; all 12 warnings are pre-existing in ObjC FFI code

## Task Commits

Each task was committed atomically:

1. **Task 1: Add streaming deps and create ai.rs with stream_ai_response** - `c7d7ddc` (feat)

**Plan metadata:** see below

## Files Created/Modified

- `src-tauri/src/commands/ai.rs` - stream_ai_response Tauri command with SSE parsing, two-mode system prompts, context builder
- `src-tauri/Cargo.toml` - Added eventsource-stream, futures-util, tokio[time]; enabled stream feature on tauri-plugin-http
- `src-tauri/Cargo.lock` - Updated lock file with 4 new resolved crates
- `src-tauri/src/commands/mod.rs` - Added `pub mod ai;`
- `src-tauri/src/lib.rs` - Imported `stream_ai_response` and added to `generate_handler![]`

## Decisions Made

- The plan specified keychain account `"api-key"` but the actual `keychain.rs` uses `"xai_api_key"`. Used the correct value `"xai_api_key"` to match the existing keychain implementation (auto-corrected, Rule 1 bug fix).
- `context_json: String` passed as raw JSON string rather than a deserialized type. This keeps the IPC boundary simple and avoids requiring `serde::Deserialize` on the existing `AppContext` type in `terminal/mod.rs` (which only derives `Serialize`). Deserialized into a lightweight `AppContextView` inside the command.
- On `context_json` parse failure, fallback to an empty `AppContextView` (assistant mode, no context) rather than returning an error. This keeps the AI functional even if context detection fails.
- History messages capped via `saturating_sub(14)` on the slice index rather than a separate truncation step.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Used correct Keychain account name `xai_api_key` instead of plan's `api-key`**
- **Found during:** Task 1 (reading keychain.rs during implementation)
- **Issue:** Plan specified account `"api-key"` but keychain.rs uses `const ACCOUNT: &str = "xai_api_key"`. Using the wrong account name would cause the API key lookup to fail at runtime with "No API key configured".
- **Fix:** Used the actual constant value `"xai_api_key"` from keychain.rs in ai.rs
- **Files modified:** src-tauri/src/commands/ai.rs
- **Verification:** cargo check passes; constant matches keychain.rs ACCOUNT field exactly
- **Committed in:** c7d7ddc (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug)
**Impact on plan:** Essential correctness fix. Without this, every AI invocation would fail with a keychain miss. No scope creep.

## Issues Encountered

None - compilation succeeded on first `cargo check` run with zero errors.

## User Setup Required

None - no external service configuration required beyond the API key already set up in Phase 2.

## Next Phase Readiness

- `stream_ai_response` command is registered and callable from TypeScript via `invoke('stream_ai_response', { query, model, contextJson, history, onToken })`
- Frontend needs to: create `Channel<string>`, set `onmessage` to append tokens to state, and invoke the command on form submit
- `AppContext` must be serialized to JSON string before passing as `context_json` argument
- `turnHistory` array (cleared on overlay close) passed as `history: ChatMessage[]`
- Plan 02 wires the frontend streaming UI to this command

---
*Phase: 04-ai-command-generation*
*Completed: 2026-02-23*
