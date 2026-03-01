# Phase 4: AI Command Generation - Research

**Researched:** 2026-02-22
**Domain:** xAI Grok SSE streaming via Rust Tauri IPC Channel + React token display
**Confidence:** HIGH (core stack), MEDIUM (stream feature dependency), HIGH (UI patterns)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Send ALL available context to xAI but condensed: CWD, shell type, last N lines of terminal output (not full buffer), running process name, app name, console last line (if browser with DevTools)
- Terminal output truncated to last ~20-30 lines (most recent = most relevant)
- System prompt includes macOS and shell type (e.g., zsh) so AI generates platform-appropriate commands
- Two distinct system prompts:
  - Terminal mode (shell detected, including editors with integrated terminals): strict command-only output. AI returns executable command(s) and nothing else. Compound commands (pipes, && chains) allowed.
  - Assistant mode (no shell -- browser, Finder, WhatsApp, Notion, etc.): concise conversational responses. 2-3 sentences max. Still useful, not a chatbot.
- Mode selection rule: if `appContext.terminal.shell_type` is present, use terminal mode. Otherwise, assistant mode.
- Console last line included in prompt when browser with DevTools is open (helps debug JS errors)
- Running process name included in prompt (e.g., if node server is running, AI can suggest Ctrl+C first)
- Terminal prompt instructs AI to prefer common POSIX tools (grep, find, sed, awk) over modern alternatives (rg, fd, jq) for maximum compatibility
- System prompt template stored in Tauri plugin-store (JSON). Editable by power users manually (no Settings UI for it).
- Fixed low temperature (0.1-0.3) for deterministic, reliable command generation
- Respect user's model selection from Settings (Phase 2)
- Session memory: up to 7 turns within a single overlay session. History cleared when overlay closes.
- Context captured once on overlay open, reused for all turns in the session (no re-detection on follow-ups)
- API calls routed through Rust backend (Tauri IPC), not frontend fetch. API key stays in backend.
- Character-by-character streaming as tokens arrive from xAI SSE response
- Monospace font with syntax highlighting (no heavy code block chrome -- clean, not a full code editor widget)
- Block cursor (full character width) blinks at the end of streaming text while response is generating. Disappears when streaming completes.
- Always monospace, even in assistant mode
- Adaptive formatting: code blocks for commands (terminal mode), plain monospace text for conversational answers (assistant mode). Both streamed the same way.
- Auto-copy to clipboard when streaming completes (no visual feedback on auto-copy)
- Click-to-copy on the output area: clicking the output re-copies to clipboard and shows a small "Copied to clipboard" indicator on the bottom-right of the output area (only on click, not on auto-copy)
- Subtle hover effect on the output area to hint clickability
- Input field transforms into output display on submit. Same area, different content. Minimal, tight.
- First Escape: toggles back to input mode with the previous query text restored (for editing/refining)
- Second Escape: closes the overlay
- Follow-up queries replace the previous response (not chat-style scroll). Only latest response visible.
- Input disabled while streaming. User must wait for completion before submitting follow-up.
- No explicit regenerate button. User retypes or refines as a follow-up.
- Badge (zsh/Chrome/etc.) stays visible below the output area
- Overlay grows vertically to fit multi-line output, capped at ~60% screen height (scroll beyond that)
- Auto-copy happens in both terminal and assistant modes
- Placeholder text: "Ask anything..."
- No API key configured: inline error in the output area ("No API key configured. Open Settings to add one.")
- API errors (rate limit, auth failure, server error): shown in the output area with muted/red styling
- Timeout: spinner shown while waiting, hard timeout (~10 seconds) cancels and shows "Request timed out. Try again."
- Empty input submit: brief shake animation on the input field (no-op, no API call)

### Claude's Discretion

- Exact system prompt wording and template structure
- Syntax highlighting implementation (library choice or CSS-based)
- SSE parsing and streaming architecture details
- Exact hover effect styling
- Timeout duration (somewhere around 10s, exact value flexible)
- How session memory messages are structured in the API call

### Deferred Ideas (OUT OF SCOPE)

- Direct terminal command injection (auto-paste into active terminal) -- Phase 6, already in roadmap
- Phase 6 should auto-inject alongside clipboard copy, not as an alternative to it
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| AICG-01 | User can type natural language and receive a terminal command via xAI (Grok) | xAI /v1/chat/completions endpoint documented; Rust command pattern established; keychain API key retrieval already implemented |
| AICG-02 | Command generation streams in real-time as the response is generated | Tauri ipc::Channel<String> pattern confirmed; eventsource-stream crate for SSE parsing; tauri-plugin-http stream feature needed |
</phase_requirements>

---

## Summary

Phase 4 connects the existing overlay UI and terminal context pipeline to xAI's chat completions API, streaming tokens back to the frontend via Tauri's IPC Channel. The project already has most plumbing in place: API key in Keychain (`keychain.rs`), model selection in `settings.json`, app context detection producing `AppContext` in the Zustand store, and the `tauri-plugin-http` reqwest client already used for the `/v1/models` call.

The key architectural decision is how SSE streaming flows from Rust to React. The confirmed pattern is: Rust Tauri command receives `tauri::ipc::Channel<String>`, calls the xAI API with `stream: true`, parses SSE `data:` lines using `eventsource-stream` (or manual line-splitting as a fallback), and sends each token string through the channel. The TypeScript frontend creates a `Channel<string>` from `@tauri-apps/api/core`, sets `onmessage`, and passes it as an argument to `invoke()`. Tokens are appended to state as they arrive, driving character-by-character rendering.

The UI transformation -- input field becoming output display on submit, Escape cycling between states, two-mode rendering (terminal command vs assistant text), block cursor during streaming, click-to-copy, auto-copy on complete -- is entirely frontend Zustand + React state management, with no new Rust surface needed beyond the single streaming command. Syntax highlighting is best handled with pure CSS (monospace + color tokens via Tailwind classes), avoiding heavy bundle cost from Shiki or Prism given the confined use case (single-line or short multi-line shell commands).

**Primary recommendation:** Implement `stream_ai_response` Tauri command using `tauri::ipc::Channel<String>` + `eventsource-stream` crate for SSE parsing. Enable `stream` feature on `tauri-plugin-http`. All token streaming flows through the Tauri IPC channel; no frontend fetch or WebSocket needed.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `tauri::ipc::Channel<T>` | Tauri 2.x (already in use) | Stream tokens from Rust command to TypeScript frontend | Official Tauri v2 recommended mechanism for streaming data to frontend; ordered delivery guaranteed |
| `eventsource-stream` | 0.2.3 | Parse xAI SSE `data:` lines from reqwest `bytes_stream()` | Thin wrapper, no retry logic (we don't need retry), uses nom parser; fits the "parse only" use case |
| `tauri-plugin-http` | 2.x (already in use) | reqwest re-export for HTTP calls from Rust | Already integrated; needs `stream` feature enabled |
| `futures-util` | 0.3 (transitive) | `StreamExt::next()` for async SSE iteration | Required by eventsource-stream; already in dependency tree via tauri |
| Zustand (already in use) | 5.x | Frontend state: streaming text, mode, turn history | Already the state manager; extend with streaming fields |
| `@tauri-apps/api/core` `Channel` | 2.x (already in use) | Create frontend channel instance for receiving tokens | Official API; already imported in codebase |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri-plugin-store` | 2.x (already in use) | Read `selectedModel` and system prompt templates from `settings.json` | Rust side reads model preference at request time; template read once on demand |
| `keyring` | 3.x (already in use) | Read API key from macOS Keychain | Already used in `keychain.rs`; re-use exact same pattern |
| CSS Tailwind classes | (zero new deps) | Syntax highlighting visual effect for shell output | Monospace + minimal color classes only; no library needed |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `eventsource-stream` | Manual line-split on `bytes_stream()` bytes | Manual split is ~20 lines of Rust, avoids new dep, but must handle partial chunks across network packets correctly -- eventsource-stream handles this edge case |
| `eventsource-stream` | `reqwest-eventsource` | reqwest-eventsource adds retry logic we don't want; eventsource-stream is the lower-level primitive it uses anyway |
| CSS-only highlighting | `shiki` npm package | Shiki full bundle is ~1.2MB gzipped; web bundle ~695KB; unacceptable for a fast overlay that must feel instant |
| CSS-only highlighting | `prism-react-renderer` | Adds ~50KB; not worth it for monospace terminal output of 1-3 lines |
| Tauri IPC Channel | Tauri event emit per token | Channel has ordered delivery guarantee and lower overhead than event system for high-frequency sends |
| Tauri IPC Channel | Frontend fetch with native SSE | Would expose API key to JS layer; violates architecture decision |

**Installation (new Cargo additions):**
```toml
# In src-tauri/Cargo.toml -- change existing tauri-plugin-http entry:
tauri-plugin-http = { version = "2", features = ["stream"] }

# New crate:
eventsource-stream = "0.2"
futures-util = "0.3"
```

No new npm packages required.

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/src/commands/
├── ai.rs          # NEW: stream_ai_response Tauri command + context building + SSE parsing
├── xai.rs         # existing: validate_and_fetch_models (unchanged)
├── keychain.rs    # existing: get_api_key (unchanged)
└── ...

src/
├── store/index.ts          # extend: streamingText, isStreaming, displayMode, turnHistory
├── components/
│   ├── ResultsArea.tsx     # replace placeholder with streaming output renderer
│   ├── CommandInput.tsx    # extend: disable during streaming, Escape state machine
│   └── Overlay.tsx         # extend: height cap + scroll at 60vh
└── hooks/
    └── useKeyboard.ts      # extend: Escape state machine (input -> output -> close)
```

### Pattern 1: Tauri IPC Channel for Token Streaming

**What:** Rust command accepts `tauri::ipc::Channel<String>` parameter. Sends each token string through it as SSE chunks arrive. Frontend creates `Channel<string>`, sets `onmessage`, passes to invoke.

**When to use:** Any time Rust needs to push incremental data to the frontend during a long-running async command.

**Example (Rust side):**
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/#channels
use tauri::ipc::Channel;

#[tauri::command]
pub async fn stream_ai_response(
    query: String,
    on_token: Channel<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // ... build request, start SSE ...
    while let Some(event) = stream.next().await {
        match event {
            Ok(eventsource_stream::Event::Message(msg)) => {
                if msg.data == "[DONE]" { break; }
                // Parse JSON chunk, extract delta.content
                if let Some(token) = extract_token(&msg.data) {
                    on_token.send(token).map_err(|e| e.to_string())?;
                }
            }
            Err(e) => return Err(e.to_string()),
            _ => {}
        }
    }
    Ok(())
}
```

**Example (TypeScript side):**
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/#channels
import { invoke, Channel } from '@tauri-apps/api/core';

const onToken = new Channel<string>();
onToken.onmessage = (token: string) => {
  useOverlayStore.getState().appendToken(token);
};

await invoke('stream_ai_response', { query, onToken });
// After invoke resolves: streaming complete
```

### Pattern 2: SSE Parsing with eventsource-stream

**What:** Wrap reqwest `bytes_stream()` with `.eventsource()` trait method. Yields `Result<Event, EventStreamError>` where `Event` has `.data` field containing raw JSON string from xAI.

**When to use:** Any time you consume an HTTP SSE endpoint from Rust without retry logic.

**Example:**
```rust
// Source: https://docs.rs/eventsource-stream/0.2.3
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

let response = client
    .post("https://api.x.ai/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .body(request_body)
    .send()
    .await
    .map_err(|e| format!("Network error: {}", e))?;

let mut stream = response.bytes_stream().eventsource();

while let Some(event) = stream.next().await {
    match event {
        Ok(event) => {
            if event.data == "[DONE]" { break; }
            // event.data is the raw JSON string from xAI
        }
        Err(e) => { /* handle parse error */ }
    }
}
```

### Pattern 3: xAI Request Body with Session Memory

**What:** Build messages array with system prompt + up to 7 turns of history + current user query. Two system prompts: terminal mode (command-only) vs assistant mode (conversational). Read model from plugin-store.

**When to use:** Every AI request. Session history stored in Zustand; cleared on overlay close.

**Example request body:**
```json
{
  "model": "grok-3",
  "messages": [
    { "role": "system", "content": "<terminal or assistant system prompt>" },
    { "role": "user", "content": "<previous user query 1>" },
    { "role": "assistant", "content": "<previous response 1>" },
    { "role": "user", "content": "<current query with context>" }
  ],
  "stream": true,
  "temperature": 0.1
}
```

**User message format (terminal mode):**
```
App: Terminal
Shell: zsh
CWD: /Users/name/projects/myapp
Running: node
Terminal output (last 20 lines):
<truncated visible_output>

Task: find all PDFs modified this week
```

### Pattern 4: Zustand State Machine for UI Modes

**What:** Extend existing Zustand store with streaming state. UI has three sub-states in command mode: `input` (showing textarea), `streaming` (showing output area, input disabled), `result` (streaming complete, showing output area). Escape transitions: `result` -> `input` (restore query), `input` -> close overlay.

**When to use:** This is the core UI state machine for Phase 4.

**New store fields:**
```typescript
// Add to OverlayState in store/index.ts
streamingText: string;       // accumulated response tokens
isStreaming: boolean;         // true while SSE in flight
displayMode: 'input' | 'streaming' | 'result';
previousQuery: string;        // saved for Escape-to-edit
turnHistory: { role: 'user' | 'assistant'; content: string }[];
abortController: AbortController | null; // for timeout/cancel
```

### Anti-Patterns to Avoid

- **Fetching xAI from TypeScript/JS directly:** Exposes API key in JS memory. The architecture decision is clear: API key stays in Rust/Keychain only.
- **Using Tauri event system for token streaming:** `app.emit()` per token has higher overhead and no ordering guarantee compared to `ipc::Channel`. Use Channel.
- **Re-detecting app context on follow-up queries:** Context is captured once on overlay open and reused for all turns. Do not call `get_app_context` again on submit.
- **Blocking the Tauri command thread during streaming:** The command must be `async`. SSE streaming is inherently async; blocking would freeze the UI.
- **Including full terminal output buffer in prompt:** Truncate to last 20-30 lines. Sending full buffer wastes tokens and may hit context limits.
- **Loading Shiki or Prism:** Bundle cost is unacceptable for a fast overlay. Use CSS-only monospace + Tailwind color classes.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE line parsing from byte stream | Custom parser splitting on `\n\n` and `data: ` | `eventsource-stream` 0.2.3 | Partial chunk boundary handling is subtle -- chunks can arrive mid-line; the crate handles buffering correctly |
| Streaming to frontend | Custom WebSocket or long-poll | `tauri::ipc::Channel<T>` | Official Tauri v2 API; handles serialization, ordering, and IPC correctly |
| Clipboard write | navigator.clipboard or Rust NSPasteboard FFI | `@tauri-apps/plugin-opener` or `navigator.clipboard.writeText()` | `navigator.clipboard` works in Tauri webview for write operations; no new plugin needed |
| Token accumulation animation | requestAnimationFrame loop | Direct React state append | Appending to `streamingText` on each `onmessage` call and React batching is sufficient; no animation loop needed |

**Key insight:** The hard problem here is not AI integration (xAI is OpenAI-compatible) -- it is correctly piping SSE bytes through the Tauri IPC boundary without deadlocking, dropping chunks, or exposing the API key. The Channel + eventsource-stream combination solves all three.

---

## Common Pitfalls

### Pitfall 1: tauri-plugin-http Missing `stream` Feature

**What goes wrong:** `bytes_stream()` is not available on `reqwest::Response` because the stream reqwest feature is not enabled. Compile error: "method not found in `reqwest::Response`".

**Why it happens:** `tauri-plugin-http` ships with `default-features = false` on reqwest; stream is an opt-in feature (`stream = ["reqwest/stream"]`).

**How to avoid:** Add `features = ["stream"]` to the `tauri-plugin-http` dependency in `Cargo.toml`:
```toml
tauri-plugin-http = { version = "2", features = ["stream"] }
```

**Warning signs:** Compile-time error mentioning `bytes_stream` not found.

### Pitfall 2: `futures-util` Not in Direct Dependencies

**What goes wrong:** `stream.next().await` fails to compile because `StreamExt` is not in scope.

**Why it happens:** `futures-util` is a transitive dependency but not directly declared; Cargo may not expose it reliably.

**How to avoid:** Add `futures-util = "0.3"` explicitly to `Cargo.toml` and `use futures_util::StreamExt;` in the AI command file.

**Warning signs:** `next()` not found on stream type at compile time.

### Pitfall 3: Channel Serialization Constraint

**What goes wrong:** `Channel<MyCustomStruct>` fails because the type doesn't implement `IpcResponse`.

**Why it happens:** `Channel<T>` requires `T: IpcResponse`, which is auto-implemented for types that implement `serde::Serialize`. Custom struct needs `#[derive(Serialize)]`.

**How to avoid:** Use `Channel<String>` (always satisfies `IpcResponse`) and do JSON extraction in Rust before sending. Send only the extracted token string, not the full chunk struct.

**Warning signs:** Trait bound error on `Channel<T>` at compile time.

### Pitfall 4: `[DONE]` Sentinel vs JSON Parse Error

**What goes wrong:** Code tries to `serde_json::from_str` on `"[DONE]"` and panics or returns an error, terminating the stream prematurely with an error state.

**Why it happens:** xAI's final SSE message has `data: [DONE]` which is not valid JSON.

**How to avoid:** Check `if event.data == "[DONE]" { break; }` BEFORE attempting JSON parse.

**Warning signs:** Stream ends with a JSON parse error logged instead of clean completion.

### Pitfall 5: Stale Context Across Overlay Sessions

**What goes wrong:** Context from a previous overlay open leaks into a new session (user opens overlay over Terminal, closes, opens over Chrome -- AI still gets terminal context).

**Why it happens:** `appContext` in Zustand is not cleared on overlay open. Already fixed in Phase 3: `appContext: null` is reset in `show()`.

**How to avoid:** Verify `show()` in store already sets `appContext: null`. Session turn history (`turnHistory`) must ALSO be cleared in `show()` -- this is new Phase 4 state that must be included in the reset.

**Warning signs:** AI references context from a different app or previous session content.

### Pitfall 6: Escape Key Conflict with Streaming State Machine

**What goes wrong:** Pressing Escape during streaming aborts the channel but the UI gets stuck in `streaming` mode instead of returning to `input`.

**Why it happens:** The existing `useKeyboard.ts` just calls `hide_overlay` + `hide()` on Escape. With Phase 4's two-Escape behavior (first Escape: result -> input; second Escape: close), the keyboard hook needs to be aware of `displayMode`.

**How to avoid:** Extend `useKeyboard.ts` to read `displayMode` from the store. First Escape when `displayMode === 'result'` or `'streaming'` cancels streaming and transitions to `input`. Second Escape (when already at `input`) closes the overlay. Cancel streaming by calling a store action that sets `isStreaming: false` -- the channel's invoke promise resolving handles cleanup.

**Warning signs:** Escape doesn't work as expected; overlay stuck in streaming or result state.

### Pitfall 7: Model Not Available at Runtime in Rust

**What goes wrong:** The Rust `stream_ai_response` command needs to know the selected model, but the frontend stores it in Zustand -- Rust has no access to Zustand.

**Why it happens:** Frontend state lives in JavaScript. Rust commands receive data only via IPC arguments or plugin-store/Keychain reads.

**How to avoid:** Two options (pick one):
1. Pass `model: String` as a command argument from the frontend (read `selectedModel` from Zustand before invoking).
2. Read `selectedModel` from `settings.json` via `tauri_plugin_store::StoreExt` in Rust.

Option 1 is simpler and consistent with how `query` is passed. **Recommendation: pass model as argument.**

**Warning signs:** Hardcoded model string in Rust command.

### Pitfall 8: `tauri_plugin_http::reqwest` Re-Export Limitations

**What goes wrong:** The `json()` builder method is not available on `RequestBuilder` (discovered in Phase 2/xai.rs comments). The `bytes_stream()` method on `Response` is also gated on the stream feature.

**Why it happens:** tauri-plugin-http re-exports reqwest with selective features enabled. Already known from Phase 2; `xai.rs` uses `.body(serde_json_string)` + `.bytes()` pattern.

**How to avoid:** Continue the established pattern: serialize with `serde_json::json!(...).to_string()`, set `.body()`, add `Content-Type: application/json` header. For streaming, enable stream feature and use `.bytes_stream()`.

**Warning signs:** "method not found" errors on request builder or response.

---

## Code Examples

Verified patterns from official sources:

### Channel: Rust Command Signature
```rust
// Source: https://v2.tauri.app/develop/calling-frontend/#channels
use tauri::ipc::Channel;

#[tauri::command]
pub async fn stream_ai_response(
    query: String,
    model: String,
    on_token: Channel<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // 1. Read API key from keychain
    // 2. Build messages array (system prompt + history + user query with context)
    // 3. POST to https://api.x.ai/v1/chat/completions with stream:true
    // 4. Parse SSE stream, send tokens via on_token.send(token)?
    // 5. Return Ok(()) when stream ends -- signals invoke() resolved
    Ok(())
}
```

### Channel: TypeScript Frontend Invocation
```typescript
// Source: https://v2.tauri.app/develop/calling-frontend/#channels
import { invoke, Channel } from '@tauri-apps/api/core';

const onToken = new Channel<string>();
onToken.onmessage = (token: string) => {
  useOverlayStore.getState().appendToken(token);
};

try {
  await invoke('stream_ai_response', {
    query: inputValue,
    model: selectedModel ?? 'grok-3',
    onToken,
  });
  // Stream complete
  useOverlayStore.getState().setIsStreaming(false);
  useOverlayStore.getState().autoClipboardCopy();
} catch (err) {
  useOverlayStore.getState().setStreamError(String(err));
}
```

### xAI Request Body Construction (Rust)
```rust
// Source: confirmed from existing xai.rs pattern in codebase + xAI docs
let body = serde_json::json!({
    "model": model,
    "messages": messages,  // Vec<{role, content}>
    "stream": true,
    "temperature": 0.1
}).to_string();

let response = client
    .post("https://api.x.ai/v1/chat/completions")
    .header("Authorization", format!("Bearer {}", api_key))
    .header("Content-Type", "application/json")
    .body(body)
    .send()
    .await
    .map_err(|e| format!("Network error: {}", e))?;
```

### SSE Stream Parsing (Rust)
```rust
// Source: https://docs.rs/eventsource-stream/0.2.3
use eventsource_stream::Eventsource;
use futures_util::StreamExt;

let mut stream = response.bytes_stream().eventsource();

while let Some(event) = stream.next().await {
    match event {
        Ok(event) => {
            let data = event.data;
            if data == "[DONE]" {
                break;
            }
            // Parse JSON to extract delta.content
            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&data) {
                if let Some(token) = chunk["choices"][0]["delta"]["content"].as_str() {
                    if !token.is_empty() {
                        on_token.send(token.to_string())
                            .map_err(|e| format!("Channel error: {}", e))?;
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("SSE parse error: {}", e));
        }
    }
}
```

### CSS-Only Syntax Highlighting (TypeScript/JSX)
```tsx
// No library needed. Monospace + Tailwind for terminal command display.
// Shell keywords: pipes (|), redirects (>, >>), && and || operators
// Highlight via regex split + span wrapping at render time

function HighlightedCommand({ text }: { text: string }) {
  // Simple: render as monospace with subtle token coloring
  // Full approach: split on shell operators and wrap spans with Tailwind colors
  return (
    <pre className="font-mono text-sm text-white/90 whitespace-pre-wrap break-words">
      {text}
    </pre>
  );
}
```

### Context Message Builder (Rust)
```rust
// Terminal mode user message format (per CONTEXT.md decisions)
fn build_user_message(query: &str, ctx: &AppContext) -> String {
    let mut parts = Vec::new();

    if let Some(name) = &ctx.app_name {
        parts.push(format!("App: {}", name));
    }
    if let Some(terminal) = &ctx.terminal {
        if let Some(shell) = &terminal.shell_type {
            parts.push(format!("Shell: {}", shell));
        }
        if let Some(cwd) = &terminal.cwd {
            parts.push(format!("CWD: {}", cwd));
        }
        if let Some(proc) = &terminal.running_process {
            parts.push(format!("Running: {}", proc));
        }
        if let Some(output) = &terminal.visible_output {
            // Truncate to last 20-30 lines
            let lines: Vec<&str> = output.lines().collect();
            let start = lines.len().saturating_sub(25);
            let truncated = lines[start..].join("\n");
            parts.push(format!("Terminal output (last {} lines):\n{}", lines[start..].len(), truncated));
        }
    }
    if ctx.console_detected {
        if let Some(line) = &ctx.console_last_line {
            parts.push(format!("Console last line: {}", line));
        }
    }

    parts.push(format!("\nTask: {}", query));
    parts.join("\n")
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Frontend fetch for AI APIs | Rust-side HTTP via tauri-plugin-http | Tauri 2.0 stable (Oct 2024) | API key security; no CORS issues |
| `app.emit()` per token | `tauri::ipc::Channel<T>` | Tauri 2.0 | Ordered, lower-overhead streaming |
| reqwest directly | `tauri-plugin-http` re-export | Tauri 2.x plugin model | Consistent with Tauri security model |
| GET /v1/models for validation | POST /v1/chat/completions fallback | Already implemented in Phase 2 | Robust API key validation pattern already in codebase |
| /v1/chat/completions | /v1/responses (newer endpoint) | xAI late 2024/2025 | The new endpoint is marked as current but /v1/chat/completions still works and is simpler for our use case |

**Deprecated/outdated:**
- `app.emit()` per token: Works but not recommended for high-frequency streaming; use Channel instead.
- xAI `/v1/chat/completions`: Marked "deprecated" in xAI docs in favor of `/v1/responses`, but it still works and is the simpler OpenAI-compatible format. The new `/v1/responses` endpoint uses different field names (`input` instead of `messages`, `previous_response_id` for threading) and is not worth migrating to for this use case.

---

## Open Questions

1. **Can `tauri-plugin-http` stream feature be enabled without breaking existing reqwest usage in `xai.rs`?**
   - What we know: `tauri-plugin-http` declares `stream = ["reqwest/stream"]` as opt-in feature; existing code uses `.bytes()` (non-streaming). The stream feature only adds the `bytes_stream()` method; it should not break `.bytes()`.
   - What's unclear: Whether adding the feature flag causes any compilation conflict with existing pinned crate versions (e.g., the `time` crate pinned to 0.3.36 from Phase 1).
   - Recommendation: Enable `features = ["stream"]` on `tauri-plugin-http` and run `cargo build` as the first verification step.

2. **Does `eventsource-stream` work with `tauri-plugin-http::reqwest` re-export?**
   - What we know: `eventsource-stream` 0.2.3 depends on `futures-core ^0.3` and `nom ^7.1`. It provides the `Eventsource` trait on any `Stream<Item = Result<impl AsRef<[u8]>, E>>`. The `bytes_stream()` method from reqwest returns exactly this type.
   - What's unclear: Whether the reqwest re-export exposes the same `Response` type that `bytes_stream()` returns, compatible with eventsource-stream's trait implementation.
   - Recommendation: Alternative if compatibility fails -- manually parse SSE by splitting accumulated byte chunks on `\n\ndata: ` boundaries. This is 25-30 lines of Rust and avoids the crate dependency entirely.

3. **Timeout implementation for ~10 second hard timeout**
   - What we know: `tokio::time::timeout()` wraps any future. The streaming loop is `async`. Wrapping the SSE loop in `tokio::time::timeout(Duration::from_secs(10), async { ... })` provides a hard cut-off.
   - What's unclear: Whether the existing project uses tokio directly or only via tauri's async runtime. Tauri 2 uses tokio internally; `tokio::time::timeout` should be available.
   - Recommendation: Add `tokio = { version = "1", features = ["time"] }` to Cargo.toml if not already transitive. Use `tokio::time::timeout` in the streaming command.

4. **Clipboard write from TypeScript**
   - What we know: `navigator.clipboard.writeText()` is a standard Web API. Tauri webviews on macOS have clipboard write access.
   - What's unclear: Whether Tauri's NSPanel webview has clipboard permissions without additional Tauri capability configuration.
   - Recommendation: Attempt `navigator.clipboard.writeText(text)` first. If it fails in the NSPanel context, fall back to invoking a new Rust command that calls `NSPasteboard` directly (a small addition to `commands/window.rs` or a new `commands/clipboard.rs`).

---

## Sources

### Primary (HIGH confidence)
- Tauri v2 official docs, Calling Frontend / Channels: https://v2.tauri.app/develop/calling-frontend/ -- Channel API, TypeScript usage, Rust send pattern
- Tauri v2 official docs, Calling Rust from Frontend: https://v2.tauri.app/develop/calling-rust/ -- Channel parameter in commands
- `eventsource-stream` 0.2.3 docs.rs: https://docs.rs/eventsource-stream/ -- Eventsource trait, Event type, bytes_stream integration
- xAI SSE streaming docs: https://docs.x.ai/docs/guides/streaming-response -- Confirmed SSE chunk format, `[DONE]` terminator, `delta.content` path
- xAI chat completions API: https://docs.x.ai/docs/guides/chat-completions -- Endpoint URL, messages format, temperature, stream parameter
- Existing codebase `src-tauri/src/commands/xai.rs` -- Established pattern for reqwest body construction without `.json()` method
- Existing codebase `src-tauri/src/commands/keychain.rs` -- API key read pattern
- Existing codebase `src/store/index.ts` -- Current Zustand store structure, `AppContext` type
- `tauri-plugin-http` v2 Cargo.toml: https://github.com/tauri-apps/tauri-plugin-http/blob/v2/Cargo.toml -- Confirmed `stream` feature flag name

### Secondary (MEDIUM confidence)
- `tauri::ipc::Channel` struct docs: https://docs.rs/tauri/latest/tauri/ipc/struct.Channel.html -- `IpcResponse` trait constraint, `send()` method
- tauri-plugin-http configuration deepwiki: https://deepwiki.com/tauri-apps/tauri-plugin-http/4-configuration -- Feature matrix, stream feature confirmed
- WebSearch: Tauri SSE streaming discussion (github.com/tauri-apps/tauri/discussions/6613) -- Confirmed Channel as recommended streaming mechanism

### Tertiary (LOW confidence)
- xAI API reference: https://docs.x.ai/docs/api-reference -- Endpoint list only, no parameter details visible
- WebSearch: tauri-plugin-http reqwest stream feature -- Multiple sources confirm stream feature exists but direct Cargo.toml inspection was the authoritative check

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- xAI API format verified from official docs; Channel API from official Tauri docs; eventsource-stream from docs.rs; all other libs already in codebase
- Architecture: HIGH -- Tauri Channel pattern is documented and confirmed; SSE format from official xAI docs; Zustand state machine is extension of existing pattern
- Pitfalls: HIGH for stream feature, [DONE] sentinel, and Channel serialization (all verified from official sources or existing codebase patterns); MEDIUM for clipboard and timeout (standard APIs but NSPanel context untested)

**Research date:** 2026-02-22
**Valid until:** 2026-03-22 (stable APIs; xAI endpoint deprecation note should be re-checked if work extends past April 2026)
