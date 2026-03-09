# Phase 21: Provider Abstraction Layer - Research

**Researched:** 2026-03-09
**Domain:** Multi-provider AI streaming APIs (Rust/Tauri backend)
**Confidence:** HIGH

## Summary

This phase refactors the existing xAI-only AI backend into a provider-agnostic abstraction supporting 5 providers: OpenAI, Anthropic, Google Gemini, xAI, and OpenRouter. The existing codebase already has a working SSE streaming pipeline (`eventsource_stream` + `futures_util::StreamExt`), keychain storage via `keyring`, and the Tauri IPC channel pattern for token streaming. The primary work is creating a provider enum, 3 streaming adapters (OpenAI-compatible, Anthropic, Gemini), parameterizing keychain storage, persisting provider/model selection, and migrating existing xAI keys.

All 5 providers have REST APIs with streaming support. OpenAI, xAI, and OpenRouter share the same OpenAI-compatible SSE format (data: JSON with `choices[0].delta.content`, terminated by `data: [DONE]`). Anthropic uses its own SSE event types (`content_block_delta` with `text_delta`). Google Gemini uses `streamGenerateContent?alt=sse` returning `GenerateContentResponse` chunks. No new Rust crates are needed beyond what already exists in `Cargo.toml`.

**Primary recommendation:** Build a `Provider` enum and trait-like dispatch pattern with 3 adapter functions (OpenAI-compatible, Anthropic, Gemini), parameterize keychain by provider, persist provider/model in `settings.json` via `tauri-plugin-store`, and run migration on first launch.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Build a common internal message format in Rust; each provider adapter translates to/from its native API
- Frontend stays unchanged -- always sends the same IPC shape
- 3 streaming adapters: OpenAI-compatible (covers OpenAI, xAI, OpenRouter), Anthropic adapter, Google Gemini adapter
- Same system prompts across all providers; each adapter places the prompt in the correct API field
- Per-provider default timeouts (e.g., 10s for fast models, 30s for reasoning models) -- no user-facing timeout setting
- All implementations must work cross-platform (macOS + Windows)
- Separate keychain account per provider (e.g., account='openai_api_key', 'anthropic_api_key', etc.) under the same service name 'com.lakshmanturlapati.cmd-k'
- Uses `keyring` crate -- works on both macOS Keychain and Windows Credential Manager
- Provider and model selection persisted in Tauri config file (JSON in app data dir), accessible from both Rust and frontend
- Frontend sends provider as an explicit IPC parameter to stream_ai_response alongside model, query, context, and history
- Read-on-first-launch migration: check if old 'xai_api_key' entry exists, copy to new provider-keyed entry, set xAI as default, leave old entry as backup
- Hybrid model lists: hardcoded curated lists per provider + "Refresh models" capability from provider APIs
- Curated models get tier tags (Fast, Balanced, Most Capable) -- hardcoded tier mapping
- OpenRouter: model list fetched from /api/v1/models endpoint, filtered to chat-capable
- API key validation at save time only (lightweight API call when user enters/saves key)
- Provider-specific error messages with provider name + actionable hint
- No automatic retry on rate limits
- Mid-stream errors: keep partial response visible and append error indicator
- No pre-check connectivity -- let API calls fail naturally

### Claude's Discretion
- Provider enum structure and what data each variant carries
- Temperature and parameter differences across providers
- OpenRouter-specific headers (HTTP-Referer, app name)
- Exact adapter code organization (separate files vs single module)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROV-01 | User can select their AI provider from OpenAI, Anthropic, Google Gemini, xAI, or OpenRouter | Provider enum, settings.json persistence, IPC parameter |
| PROV-02 | User can store a separate API key per provider in the platform keychain | Parameterized keyring::Entry with provider-specific account names |
| PROV-03 | Existing xAI API key is migrated automatically on upgrade from v0.2.4 | First-launch migration function reading old 'xai_api_key' entry |
| PROV-04 | User can validate their API key for any provider before saving | Per-provider validation endpoints (GET /models or minimal chat completion) |
| PROV-05 | User can see available models for their selected provider | Hardcoded curated lists + API fetch per provider |
| PROV-06 | AI responses stream in real-time from all 5 providers | 3 streaming adapters (OpenAI-compat, Anthropic, Gemini) |
| PROV-07 | Provider-specific error messages show the correct provider name and troubleshooting hints | Provider name injected into error strings with actionable URLs |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| keyring | 3.6.x | Platform keychain (macOS Keychain, Windows Credential Manager) | Already in Cargo.toml with platform features |
| tauri-plugin-http (reqwest) | 2.x | HTTP client for all provider API calls | Already in use, handles streaming via bytes_stream() |
| eventsource-stream | 0.2.x | SSE event parsing for OpenAI-compat and Anthropic | Already in use for xAI streaming |
| futures-util | 0.3.x | StreamExt for async SSE consumption | Already in use |
| tokio | 1.x | Async runtime, timeout wrapping | Already in use |
| serde/serde_json | 1.x | JSON serialization for API bodies and responses | Already in use |
| tauri-plugin-store | 2.x | Persistent JSON config (settings.json) | Already in use for hotkey, model selection |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @tauri-apps/plugin-store | 2.x (JS) | Frontend reads/writes settings.json | Provider/model persistence from frontend |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| eventsource-stream for Gemini | Manual line parsing | Gemini SSE is standard SSE with `alt=sse`; eventsource-stream handles it fine |
| Separate HTTP clients per provider | Single reqwest::Client | Single client is sufficient; all providers use HTTPS + JSON |

**Installation:**
No new crates needed. All dependencies already in `Cargo.toml`.

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/
  providers/
    mod.rs           # Provider enum, ProviderConfig, dispatch functions
    openai_compat.rs # OpenAI/xAI/OpenRouter streaming adapter
    anthropic.rs     # Anthropic Messages API streaming adapter
    gemini.rs        # Google Gemini streamGenerateContent adapter
  ai.rs              # Refactored: accepts provider param, dispatches to adapter
  keychain.rs        # Refactored: parameterized by provider account name
  xai.rs             # Removed or gutted (logic moves to providers/)
  mod.rs             # Updated module declarations
```

### Pattern 1: Provider Enum with Config
**What:** A Rust enum representing each provider, with associated configuration (base URL, auth header format, default models, timeout).
**When to use:** Everywhere a provider-specific behavior diverges.
**Example:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    XAI,
    OpenRouter,
}

impl Provider {
    /// Keychain account name for this provider's API key
    pub fn keychain_account(&self) -> &str {
        match self {
            Provider::OpenAI => "openai_api_key",
            Provider::Anthropic => "anthropic_api_key",
            Provider::Gemini => "gemini_api_key",
            Provider::XAI => "xai_api_key",
            Provider::OpenRouter => "openrouter_api_key",
        }
    }

    /// Base URL for chat completions
    pub fn base_url(&self) -> &str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1",
            Provider::Anthropic => "https://api.anthropic.com/v1",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Provider::XAI => "https://api.x.ai/v1",
            Provider::OpenRouter => "https://openrouter.ai/api/v1",
        }
    }

    /// Display name for error messages
    pub fn display_name(&self) -> &str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Google Gemini",
            Provider::XAI => "xAI",
            Provider::OpenRouter => "OpenRouter",
        }
    }

    /// Default streaming timeout in seconds
    pub fn timeout_secs(&self) -> u64 {
        match self {
            Provider::OpenAI => 30,
            Provider::Anthropic => 30,
            Provider::Gemini => 30,
            Provider::XAI => 10,
            Provider::OpenRouter => 30,
        }
    }

    /// Which streaming adapter to use
    pub fn adapter(&self) -> StreamAdapter {
        match self {
            Provider::OpenAI | Provider::XAI | Provider::OpenRouter => StreamAdapter::OpenAICompat,
            Provider::Anthropic => StreamAdapter::Anthropic,
            Provider::Gemini => StreamAdapter::Gemini,
        }
    }
}

pub enum StreamAdapter {
    OpenAICompat,
    Anthropic,
    Gemini,
}
```

### Pattern 2: Adapter Dispatch in stream_ai_response
**What:** The refactored `stream_ai_response` command accepts a `provider` string parameter, resolves it to the enum, reads the correct keychain entry, and dispatches to the right adapter function.
**When to use:** The main IPC entry point for AI streaming.
**Example:**
```rust
#[tauri::command]
pub async fn stream_ai_response(
    provider: String,     // NEW: "openai", "anthropic", "gemini", "xai", "openrouter"
    query: String,
    model: String,
    context_json: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let provider = Provider::from_str(&provider)?;
    let api_key = read_provider_key(&provider)?;
    let (system_prompt, user_message) = build_prompts(&query, &context_json, &history)?;

    match provider.adapter() {
        StreamAdapter::OpenAICompat => {
            openai_compat::stream(&provider, &api_key, &model, system_prompt, user_message, history, on_token).await
        }
        StreamAdapter::Anthropic => {
            anthropic::stream(&api_key, &model, system_prompt, user_message, history, on_token).await
        }
        StreamAdapter::Gemini => {
            gemini::stream(&api_key, &model, system_prompt, user_message, history, on_token).await
        }
    }
}
```

### Pattern 3: Parameterized Keychain
**What:** Keychain functions accept a provider parameter to determine the account name.
**When to use:** All keychain operations (save, get, delete).
**Example:**
```rust
const SERVICE: &str = "com.lakshmanturlapati.cmd-k";

#[tauri::command]
pub fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let provider = Provider::from_str(&provider)?;
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry.set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}
```

### Anti-Patterns to Avoid
- **Monolithic adapter file:** Don't put all 3 streaming adapters in `ai.rs`. Separate files keep each adapter independently maintainable.
- **Provider-specific logic in frontend:** Don't let the frontend know about API formats. It sends `provider: "anthropic"` and gets back tokens -- same shape always.
- **Hardcoding URLs in adapter functions:** Put base URLs in the Provider enum so they're centralized and testable.
- **Re-creating reqwest::Client per request:** Create one client and reuse it. The existing code already creates a new client per call, which is fine for now but could be optimized later.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE parsing | Custom line-by-line parser | `eventsource-stream` crate | Handles reconnection, event names, multi-line data fields |
| Keychain access | Platform-specific APIs | `keyring` crate v3 | Already handles macOS Keychain + Windows Credential Manager |
| JSON config persistence | Custom file I/O | `tauri-plugin-store` | Already integrated, handles atomic writes, accessible from both Rust and JS |
| HTTP client | Raw TCP/TLS | `tauri-plugin-http`'s reqwest | Already configured with stream feature |

**Key insight:** The existing codebase already has all the infrastructure needed. This phase is about refactoring and parameterizing, not adding new capabilities.

## Common Pitfalls

### Pitfall 1: Anthropic SSE Event Format Differs from OpenAI
**What goes wrong:** Treating Anthropic's SSE stream like OpenAI's (looking for `choices[0].delta.content` and `[DONE]`).
**Why it happens:** Anthropic uses named SSE events (`event: content_block_delta`) with different JSON structure (`delta.text` inside `text_delta` type).
**How to avoid:** The Anthropic adapter must:
1. Check the SSE event name (not just data)
2. Parse `content_block_delta` events specifically
3. Extract text from `delta.text` where `delta.type == "text_delta"`
4. Stop on `message_stop` event (not `[DONE]`)
**Warning signs:** Empty responses or parse errors from Anthropic.

### Pitfall 2: Gemini API Authentication Differs
**What goes wrong:** Sending Gemini API key as a Bearer token in the Authorization header.
**Why it happens:** All other providers use `Authorization: Bearer <key>`. Gemini uses `?key=<key>` query parameter.
**How to avoid:** The Gemini adapter must append the API key as a query parameter, not a header.
**Warning signs:** 401 errors from Gemini despite valid key.

### Pitfall 3: Anthropic Requires anthropic-version Header
**What goes wrong:** Omitting the `anthropic-version` header, getting 400 errors.
**Why it happens:** Anthropic requires `anthropic-version: 2023-06-01` on every request. Other providers don't have this.
**How to avoid:** Anthropic adapter always includes `x-api-key` (NOT `Authorization: Bearer`), `anthropic-version: 2023-06-01`, and `content-type: application/json`.
**Warning signs:** 400 Bad Request from Anthropic API.

### Pitfall 4: Anthropic System Prompt is Top-Level, Not in Messages
**What goes wrong:** Putting the system prompt as a `{"role": "system", "content": "..."}` message for Anthropic.
**Why it happens:** OpenAI-compatible APIs use system role in messages array. Anthropic uses a top-level `system` field.
**How to avoid:** Anthropic adapter builds body as:
```json
{
  "model": "...",
  "max_tokens": 4096,
  "system": "system prompt here",
  "messages": [{"role": "user", "content": "..."}],
  "stream": true
}
```
**Warning signs:** System prompt being ignored or errors from Anthropic.

### Pitfall 5: Gemini Message Format Differs
**What goes wrong:** Sending `{"role": "user", "content": "text"}` to Gemini.
**Why it happens:** Gemini uses `contents` array with `parts` sub-array: `{"role": "user", "parts": [{"text": "..."}]}`. System prompt goes in `systemInstruction.parts.text`.
**How to avoid:** Gemini adapter translates the internal message format to Gemini's `contents` format. Also note Gemini uses `"role": "model"` instead of `"role": "assistant"`.
**Warning signs:** 400 errors from Gemini API.

### Pitfall 6: Anthropic Requires max_tokens
**What goes wrong:** Omitting `max_tokens` in Anthropic request body.
**Why it happens:** OpenAI-compatible APIs have sensible defaults. Anthropic requires `max_tokens` explicitly.
**How to avoid:** Always include `max_tokens: 4096` (or appropriate value) in Anthropic requests.
**Warning signs:** 400 error: "max_tokens is required".

### Pitfall 7: Migration Race Condition
**What goes wrong:** Migration runs multiple times or conflicts with user actions.
**Why it happens:** Migration checks for old key on every launch without recording completion.
**How to avoid:** Record migration completion in `settings.json` (e.g., `"migration_v026_done": true`). Check this flag before running migration.
**Warning signs:** Duplicate keychain entries or unexpected provider defaults.

### Pitfall 8: Gemini SSE Chunk Format
**What goes wrong:** Trying to parse Gemini SSE chunks as OpenAI format.
**Why it happens:** Gemini returns `GenerateContentResponse` objects, not OpenAI-style delta chunks.
**How to avoid:** Extract text from `candidates[0].content.parts[0].text` in Gemini SSE chunks.
**Warning signs:** Null/empty tokens from Gemini streaming.

## Code Examples

### OpenAI-Compatible Streaming Adapter (covers OpenAI, xAI, OpenRouter)
```rust
// Source: Existing ai.rs pattern + OpenAI/xAI API docs
pub async fn stream(
    provider: &Provider,
    api_key: &str,
    model: &str,
    system_prompt: String,
    user_message: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let mut messages: Vec<serde_json::Value> = vec![
        serde_json::json!({"role": "system", "content": system_prompt}),
    ];
    for msg in &history {
        messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    messages.push(serde_json::json!({"role": "user", "content": user_message}));

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "temperature": 0.1
    });

    let client = reqwest::Client::new();
    let mut request = client
        .post(format!("{}/chat/completions", provider.base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    // OpenRouter-specific headers
    if *provider == Provider::OpenRouter {
        request = request
            .header("HTTP-Referer", "https://github.com/cmd-k-app")
            .header("X-Title", "CMD+K");
    }

    let response = request.body(body.to_string()).send().await
        .map_err(|e| format!("{}: Network error: Check your internet connection.", provider.display_name()))?;

    // ... status check and SSE parsing (same as existing ai.rs) ...
    Ok(())
}
```

### Anthropic Streaming Adapter
```rust
// Source: https://platform.claude.com/docs/en/api/messages-streaming
pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: String,
    user_message: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    // NOTE: No system role in messages -- system is top-level
    for msg in &history {
        messages.push(serde_json::json!({"role": msg.role, "content": msg.content}));
    }
    messages.push(serde_json::json!({"role": "user", "content": user_message}));

    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": system_prompt,  // Top-level, not in messages
        "messages": messages,
        "stream": true,
        "temperature": 0.1
    });

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)                    // NOT Authorization: Bearer
        .header("anthropic-version", "2023-06-01")        // Required
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await
        .map_err(|e| format!("Anthropic: Network error: Check your internet connection."))?;

    // Parse SSE: look for event: content_block_delta with text_delta
    let mut stream = response.bytes_stream().eventsource();
    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                // ev.event contains the SSE event name
                if ev.event == "message_stop" {
                    break;
                }
                if ev.event == "content_block_delta" {
                    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                        if data["delta"]["type"] == "text_delta" {
                            if let Some(text) = data["delta"]["text"].as_str() {
                                if !text.is_empty() {
                                    on_token.send(text.to_string())
                                        .map_err(|e| format!("Channel error: {}", e))?;
                                }
                            }
                        }
                    }
                }
                // Ignore ping, message_start, content_block_start, content_block_stop, message_delta
            }
            Err(e) => return Err(format!("Anthropic: Stream error: {}", e)),
        }
    }
    Ok(())
}
```

### Gemini Streaming Adapter
```rust
// Source: https://ai.google.dev/api/generate-content
pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: String,
    user_message: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let mut contents: Vec<serde_json::Value> = Vec::new();
    for msg in &history {
        let role = if msg.role == "assistant" { "model" } else { &msg.role };
        contents.push(serde_json::json!({"role": role, "parts": [{"text": msg.content}]}));
    }
    contents.push(serde_json::json!({"role": "user", "parts": [{"text": user_message}]}));

    let body = serde_json::json!({
        "contents": contents,
        "systemInstruction": {
            "parts": [{"text": system_prompt}]
        }
    });

    // Gemini: API key as query param, model in URL path
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body.to_string())
        .send().await
        .map_err(|e| format!("Google Gemini: Network error: Check your internet connection."))?;

    // Parse SSE: each chunk is a GenerateContentResponse
    let mut stream = response.bytes_stream().eventsource();
    while let Some(event) = stream.next().await {
        match event {
            Ok(ev) => {
                if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&ev.data) {
                    if let Some(text) = chunk["candidates"][0]["content"]["parts"][0]["text"].as_str() {
                        if !text.is_empty() {
                            on_token.send(text.to_string())
                                .map_err(|e| format!("Channel error: {}", e))?;
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Google Gemini: Stream error: {}", e)),
        }
    }
    Ok(())
}
```

### Key Validation Per Provider
```rust
// Validation strategy per provider:
// - OpenAI, xAI: GET /v1/models (returns model list, validates key)
// - Anthropic: GET /v1/models with x-api-key + anthropic-version headers
// - Gemini: GET /v1beta/models?key=<key>
// - OpenRouter: GET /api/v1/models (returns model list, validates key)
//
// All return model lists on success (200), 401 on invalid key.
// Fallback: minimal chat completion call if /models returns 404.
```

### Migration Function
```rust
pub fn migrate_v024_key() -> Result<(), String> {
    let old_entry = keyring::Entry::new(SERVICE, "xai_api_key")
        .map_err(|e| format!("Migration error: {}", e))?;

    match old_entry.get_password() {
        Ok(key) => {
            // Key exists -- copy to new location (xai provider uses same account name)
            // The old entry remains as backup (per decision)
            eprintln!("[migration] Found existing xAI API key, preserving as default provider");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            eprintln!("[migration] No existing xAI key found, skipping migration");
            Ok(())
        }
        Err(e) => Err(format!("Migration error: {}", e)),
    }
}
```

### Curated Model Lists (Hardcoded Defaults)
```rust
pub fn default_models(provider: &Provider) -> Vec<ModelWithMeta> {
    match provider {
        Provider::OpenAI => vec![
            ModelWithMeta { id: "gpt-4.1".into(), label: "Most Capable".into(), tier: "capable".into() },
            ModelWithMeta { id: "gpt-4.1-mini".into(), label: "Balanced".into(), tier: "balanced".into() },
            ModelWithMeta { id: "gpt-4.1-nano".into(), label: "Fast".into(), tier: "fast".into() },
        ],
        Provider::Anthropic => vec![
            ModelWithMeta { id: "claude-sonnet-4-20250514".into(), label: "Most Capable".into(), tier: "capable".into() },
            ModelWithMeta { id: "claude-haiku-4-20250514".into(), label: "Fast".into(), tier: "fast".into() },
        ],
        Provider::Gemini => vec![
            ModelWithMeta { id: "gemini-2.5-pro".into(), label: "Most Capable".into(), tier: "capable".into() },
            ModelWithMeta { id: "gemini-2.5-flash".into(), label: "Balanced".into(), tier: "balanced".into() },
        ],
        Provider::XAI => vec![
            ModelWithMeta { id: "grok-3".into(), label: "Balanced".into(), tier: "balanced".into() },
            ModelWithMeta { id: "grok-3-mini".into(), label: "Fast".into(), tier: "fast".into() },
        ],
        Provider::OpenRouter => vec![], // Always fetched from API
    }
}
```

## Provider API Reference

### Endpoint Summary

| Provider | Chat Endpoint | Models Endpoint | Auth | System Prompt |
|----------|---------------|-----------------|------|---------------|
| OpenAI | POST /v1/chat/completions | GET /v1/models | `Authorization: Bearer <key>` | `role: "system"` in messages |
| Anthropic | POST /v1/messages | GET /v1/models | `x-api-key: <key>` + `anthropic-version: 2023-06-01` | Top-level `system` field |
| Gemini | POST /v1beta/models/{model}:streamGenerateContent?alt=sse&key={key} | GET /v1beta/models?key={key} | API key as query param | `systemInstruction.parts[].text` |
| xAI | POST /v1/chat/completions | GET /v1/models | `Authorization: Bearer <key>` | `role: "system"` in messages |
| OpenRouter | POST /api/v1/chat/completions | GET /api/v1/models | `Authorization: Bearer <key>` | `role: "system"` in messages |

### SSE Format Summary

| Provider | Event Names | Token Location | Stream End Signal |
|----------|-------------|----------------|-------------------|
| OpenAI/xAI/OpenRouter | None (data-only) | `choices[0].delta.content` | `data: [DONE]` |
| Anthropic | `content_block_delta`, `message_stop`, etc. | `delta.text` (when `delta.type == "text_delta"`) | `event: message_stop` |
| Gemini | Standard SSE data events | `candidates[0].content.parts[0].text` | Stream closes naturally |

### Temperature Defaults
All providers: `temperature: 0.1` (matching existing xAI behavior). All support `temperature` parameter with similar 0.0-2.0 range.

### Additional Headers
- **OpenRouter:** Optional `HTTP-Referer` (app URL) and `X-Title` (app name) for attribution/rankings
- **Anthropic:** Required `anthropic-version: 2023-06-01`

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| xAI-only hardcoded | Multi-provider abstraction | This phase | 5 providers supported |
| Single keychain account | Per-provider keychain accounts | This phase | Separate keys per provider |
| `XaiModelWithMeta` struct | `ModelWithMeta` (generic) | This phase | Provider-agnostic model metadata |
| No migration logic | Read-on-first-launch migration | This phase | Smooth upgrade for existing users |

## Open Questions

1. **Exact curated model IDs at ship time**
   - What we know: Model IDs for current top-tier models from each provider
   - What's unclear: Providers release new models frequently; IDs may change by ship date
   - Recommendation: Use current best-known IDs, the "Refresh models" capability handles updates

2. **OpenRouter model filtering**
   - What we know: `/api/v1/models` returns hundreds of models
   - What's unclear: Exact filtering criteria for "chat-capable" models
   - Recommendation: Filter by checking model `type` or `architecture` fields; show all but tag curated ones

3. **Anthropic max_tokens optimal value**
   - What we know: Anthropic requires explicit max_tokens; 4096 is a safe default
   - What's unclear: Whether different Claude models have different optimal max_tokens
   - Recommendation: Use 4096 for all Anthropic models (CMD+K generates short responses)

## Sources

### Primary (HIGH confidence)
- Anthropic Streaming Messages API - [platform.claude.com/docs/en/api/messages-streaming](https://platform.claude.com/docs/en/api/messages-streaming) - Full SSE event format, request/response examples
- Google Gemini generateContent API - [ai.google.dev/api/generate-content](https://ai.google.dev/api/generate-content) - REST endpoint, streaming format, systemInstruction
- OpenRouter API Reference - [openrouter.ai/docs/api/api-reference/chat/send-chat-completion-request](https://openrouter.ai/docs/api/api-reference/chat/send-chat-completion-request) - OpenAI-compatible format, optional headers
- keyring crate docs - [docs.rs/keyring](https://docs.rs/keyring) - Multi-account support, platform backends

### Secondary (MEDIUM confidence)
- OpenAI Chat Completions API - [platform.openai.com/docs/api-reference/chat](https://platform.openai.com/docs/api-reference/chat) - Standard SSE format verified via WebSearch
- Anthropic Models List - [docs.anthropic.com/en/api/models-list](https://docs.anthropic.com/en/api/models-list) - GET /v1/models for key validation
- Google Gemini Models API - [ai.google.dev/api/models](https://ai.google.dev/api/models) - GET /v1beta/models for key validation

### Tertiary (LOW confidence)
- Specific model IDs (gpt-4.1, claude-sonnet-4, gemini-2.5-pro) - These are current as of March 2026 but may change before ship

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in use, no new dependencies
- Architecture: HIGH - Clear adapter pattern, well-understood API differences from official docs
- Pitfalls: HIGH - Verified API format differences from official documentation
- Model lists: LOW - Specific model IDs may change; mitigated by refresh capability

**Research date:** 2026-03-09
**Valid until:** 2026-04-09 (stable APIs, but model IDs may shift)
