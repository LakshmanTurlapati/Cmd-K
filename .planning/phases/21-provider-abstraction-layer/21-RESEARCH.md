# Phase 21: Provider Abstraction Layer - Research

**Researched:** 2026-03-09 (re-research)
**Domain:** Multi-provider AI streaming APIs (Rust/Tauri backend)
**Confidence:** HIGH

## Summary

This phase refactors the existing xAI-only AI backend into a provider-agnostic abstraction supporting 5 providers: OpenAI, Anthropic, Google Gemini, xAI, and OpenRouter. The existing codebase has a working SSE streaming pipeline (`eventsource_stream` + `futures_util::StreamExt`), keychain storage via `keyring` v3 (platform-native on both macOS and Windows), and the Tauri IPC channel pattern (`tauri::ipc::Channel<String>`) for token streaming. The work involves creating a `Provider` enum, 3 streaming adapters (OpenAI-compatible, Anthropic, Gemini), parameterizing keychain storage per provider, persisting provider/model selection in `settings.json`, and migrating existing v0.2.4 xAI keys on first launch.

All 5 providers have REST APIs with streaming support. OpenAI, xAI, and OpenRouter share the OpenAI-compatible SSE format (`data: JSON` with `choices[0].delta.content`, terminated by `data: [DONE]`). Anthropic uses its own event-based SSE protocol (`event: content_block_delta` with `delta.text`). Google Gemini uses `streamGenerateContent?alt=sse` returning `GenerateContentResponse` chunks with text in `candidates[0].content.parts[0].text`. No new Rust crates are needed -- the existing `Cargo.toml` dependencies cover everything.

**Primary recommendation:** Build a `Provider` enum with `to_api_url()`, `to_keychain_account()`, and `default_timeout()` methods, implement 3 adapter functions dispatched by provider, parameterize keychain by provider, persist provider/model in `settings.json` via `tauri-plugin-store`, and run xAI key migration on app setup.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Build a common internal message format in Rust; each provider adapter translates to/from its native API
- Frontend stays unchanged -- always sends the same IPC shape
- 3 streaming adapters: OpenAI-compatible (covers OpenAI, xAI, OpenRouter), Anthropic adapter, Google Gemini adapter
- Same system prompts across all providers; each adapter places the prompt in the correct API field (role:system for OpenAI-compat, top-level system field for Anthropic, systemInstruction for Gemini)
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
- API key validation at save time only (lightweight API call), no re-validation on provider switch
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
| PROV-01 | User can select AI provider from 5 options | Provider enum with 5 variants; selection persisted in settings.json via tauri-plugin-store |
| PROV-02 | User can store separate API key per provider in platform keychain | keyring crate with per-provider account names (e.g., 'openai_api_key') under shared service |
| PROV-03 | Existing xAI API key migrated automatically on upgrade | Migration function in app setup reads old 'xai_api_key', copies to new entry, sets xAI as default |
| PROV-04 | User can validate API key for any provider | Per-provider validation endpoints documented below; lightweight API call at save time |
| PROV-05 | User can see available models for selected provider | Hardcoded curated lists + API-fetched refresh; model list endpoints documented per provider |
| PROV-06 | AI responses stream in real-time from all 5 providers | 3 streaming adapters cover all 5 providers; SSE parsing patterns documented below |
| PROV-07 | Provider-specific error messages with troubleshooting hints | Error mapping from HTTP status codes to provider-named messages with console URLs |
</phase_requirements>

## Standard Stack

### Core (Already in Cargo.toml -- no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `keyring` | 3.x | Platform keychain access | Already used; apple-native + windows-native features enabled |
| `eventsource-stream` | 0.2 | SSE stream parsing | Already used for xAI streaming; works for OpenAI-compat and Anthropic |
| `futures-util` | 0.3 | StreamExt for async iteration | Already used for SSE stream consumption |
| `tauri-plugin-http` | 2.x (stream feature) | HTTP client (reqwest re-export) | Already used; provides `reqwest::Client` for all API calls |
| `tokio` | 1.x (time feature) | Async runtime + timeouts | Already used for `tokio::time::timeout` |
| `serde` / `serde_json` | 1.x | JSON serialization | Already used throughout |
| `tauri-plugin-store` | 2.x | Persistent JSON config | Already used for settings.json (hotkey, model, etc.) |

### Supporting (Already available)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tauri::ipc::Channel<String>` | Tauri 2.x | Token streaming to frontend | All streaming adapters emit tokens through this |
| `Store.load("settings.json")` | tauri-plugin-store TS | Frontend config persistence | Provider + model selection read/write |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual SSE parsing for Gemini | eventsource-stream | Gemini SSE format uses standard `data:` prefix with `alt=sse`; same crate works |
| Separate HTTP client per provider | Single reqwest::Client | One client is fine; per-request headers differ but client is reusable |
| async-trait for provider abstraction | Enum dispatch with match | Enum dispatch is simpler, no vtable overhead, all providers known at compile time |

**Installation:**
```bash
# No new dependencies needed -- all crates already in Cargo.toml
```

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/
├── ai.rs              # stream_ai_response (refactored: accepts provider param, dispatches)
├── providers/         # NEW: provider module
│   ├── mod.rs         # Provider enum, ProviderConfig, dispatch logic
│   ├── openai_compat.rs  # OpenAI-compatible adapter (OpenAI, xAI, OpenRouter)
│   ├── anthropic.rs      # Anthropic adapter
│   └── gemini.rs         # Google Gemini adapter
├── keychain.rs        # Parameterized: save_api_key(provider, key), get_api_key(provider), etc.
├── models.rs          # NEW: renamed from xai.rs -- validate_and_fetch_models(provider, key)
├── mod.rs             # Updated: register providers module
└── ...existing files...
```

### Pattern 1: Provider Enum with Data Methods
**What:** A Rust enum where each variant carries provider-specific config as methods, not associated data.
**When to use:** When all providers are known at compile time and dispatch is exhaustive.

```rust
// Source: Project convention (match existing Tauri command patterns)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    XAI,
    OpenRouter,
}

impl Provider {
    /// Keychain account name for this provider's API key
    pub fn keychain_account(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai_api_key",
            Provider::Anthropic => "anthropic_api_key",
            Provider::Gemini => "gemini_api_key",
            Provider::XAI => "xai_api_key",
            Provider::OpenRouter => "openrouter_api_key",
        }
    }

    /// Base URL for chat completions / message creation
    pub fn api_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1/chat/completions",
            Provider::Anthropic => "https://api.anthropic.com/v1/messages",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/models/",
            Provider::XAI => "https://api.x.ai/v1/chat/completions",
            Provider::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
        }
    }

    /// Default streaming timeout in seconds
    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            Provider::OpenAI => 30,
            Provider::Anthropic => 30,
            Provider::Gemini => 30,
            Provider::XAI => 10,
            Provider::OpenRouter => 30,
        }
    }

    /// Display name for error messages
    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Google Gemini",
            Provider::XAI => "xAI",
            Provider::OpenRouter => "OpenRouter",
        }
    }

    /// Console/dashboard URL for troubleshooting
    pub fn console_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "platform.openai.com",
            Provider::Anthropic => "console.anthropic.com",
            Provider::Gemini => "aistudio.google.com",
            Provider::XAI => "console.x.ai",
            Provider::OpenRouter => "openrouter.ai/keys",
        }
    }

    /// Which streaming adapter to use
    pub fn adapter_kind(&self) -> AdapterKind {
        match self {
            Provider::OpenAI | Provider::XAI | Provider::OpenRouter => AdapterKind::OpenAICompat,
            Provider::Anthropic => AdapterKind::Anthropic,
            Provider::Gemini => AdapterKind::Gemini,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AdapterKind {
    OpenAICompat,
    Anthropic,
    Gemini,
}
```

### Pattern 2: Streaming Adapter Dispatch
**What:** The refactored `stream_ai_response` accepts a provider parameter and dispatches to the correct adapter.
**When to use:** In the main AI command handler.

```rust
// In commands/ai.rs (refactored signature)
#[tauri::command]
pub async fn stream_ai_response(
    provider: Provider,       // NEW: explicit provider param
    query: String,
    model: String,
    context_json: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    // 1. Read API key from keychain using provider-specific account
    let account = provider.keychain_account();
    let entry = keyring::Entry::new(SERVICE, account)
        .map_err(|e| format!("{}: Keyring error: {}", provider.display_name(), e))?;
    let api_key = entry
        .get_password()
        .map_err(|_| format!("No {} API key configured. Open Settings to add one.", provider.display_name()))?;

    // 2. Parse context, build system prompt and user message (unchanged logic)
    // ...

    // 3. Dispatch to correct adapter
    let timeout = tokio::time::Duration::from_secs(provider.default_timeout_secs());
    match provider.adapter_kind() {
        AdapterKind::OpenAICompat => {
            openai_compat::stream(&provider, &api_key, &model, messages, &on_token, timeout).await
        }
        AdapterKind::Anthropic => {
            anthropic::stream(&api_key, &model, &system_prompt, messages, &on_token, timeout).await
        }
        AdapterKind::Gemini => {
            gemini::stream(&api_key, &model, &system_prompt, messages, &on_token, timeout).await
        }
    }
}
```

### Pattern 3: OpenAI-Compatible Adapter (covers OpenAI, xAI, OpenRouter)
**What:** Single adapter for all OpenAI-compatible APIs.
**When to use:** For OpenAI, xAI, and OpenRouter -- same SSE format, different base URLs.

```rust
// Source: OpenAI API docs (https://platform.openai.com/docs/api-reference/chat-streaming)
// In commands/providers/openai_compat.rs
pub async fn stream(
    provider: &Provider,
    api_key: &str,
    model: &str,
    messages: Vec<serde_json::Value>,  // includes system message in messages array
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = provider.api_url();

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "temperature": 0.1
    }).to_string();

    let mut request = client
        .post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    // OpenRouter requires additional headers
    if *provider == Provider::OpenRouter {
        request = request
            .header("HTTP-Referer", "https://cmdkapp.com")
            .header("X-Title", "CMD+K");
    }

    let response = request
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Network error: Check your internet connection."))?;

    // Check HTTP status with provider-specific error messages
    handle_http_status(provider, response.status().as_u16())?;

    // Parse SSE -- same format as existing xAI code
    let mut stream = response.bytes_stream().eventsource();
    tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data == "[DONE]" { break; }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(token) = chunk["choices"][0]["delta"]["content"].as_str() {
                            if !token.is_empty() {
                                on_token.send(token.to_string())
                                    .map_err(|e| format!("Channel error: {}", e))?;
                            }
                        }
                    }
                }
                Err(e) => return Err(format!("{}: Stream error: {}", provider.display_name(), e)),
            }
        }
        Ok(())
    }).await
    .map_err(|_| format!("{}: Request timed out. Try again.", provider.display_name()))?
}
```

### Pattern 4: Anthropic Adapter
**What:** Adapter for Anthropic's unique SSE event protocol.
**When to use:** For the Anthropic provider only.

```rust
// Source: Anthropic docs (https://docs.anthropic.com/en/api/messages-streaming)
// In commands/providers/anthropic.rs

// KEY DIFFERENCES from OpenAI-compat:
// 1. Auth header: "x-api-key" (NOT "Authorization: Bearer")
// 2. Required header: "anthropic-version: 2023-06-01"
// 3. System prompt: top-level "system" field (NOT in messages array)
// 4. Required field: "max_tokens" (must be specified)
// 5. SSE events: named events (event: content_block_delta), NOT just data lines
// 6. Text extraction: delta.text (NOT choices[0].delta.content)
// 7. No [DONE] sentinel -- stream ends with event: message_stop

pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,  // user/assistant only, NO system message
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,           // Top-level, not in messages
        "messages": messages,              // Only user/assistant roles
        "max_tokens": 4096,                // Required by Anthropic
        "stream": true
    }).to_string();

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)                    // NOT Bearer token
        .header("anthropic-version", "2023-06-01")       // Required version header
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| "Network error: Check your internet connection.".to_string())?;

    handle_http_status_anthropic(response.status().as_u16())?;

    // Anthropic SSE uses named events -- eventsource-stream handles this
    let mut stream = response.bytes_stream().eventsource();
    tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    // eventsource-stream provides event.event (the SSE event name)
                    // and event.data (the JSON payload)
                    if event.data.is_empty() || event.event == "ping" {
                        continue;
                    }
                    if event.event == "message_stop" {
                        break;
                    }
                    if event.event == "content_block_delta" {
                        if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&event.data) {
                            if chunk["delta"]["type"].as_str() == Some("text_delta") {
                                if let Some(text) = chunk["delta"]["text"].as_str() {
                                    if !text.is_empty() {
                                        on_token.send(text.to_string())
                                            .map_err(|e| format!("Channel error: {}", e))?;
                                    }
                                }
                            }
                        }
                    }
                    // Ignore: message_start, content_block_start, content_block_stop, message_delta
                }
                Err(e) => return Err(format!("Anthropic: Stream error: {}", e)),
            }
        }
        Ok(())
    }).await
    .map_err(|_| "Anthropic: Request timed out. Try again.".to_string())?
}
```

### Pattern 5: Google Gemini Adapter
**What:** Adapter for Google Gemini's REST streaming API.
**When to use:** For the Gemini provider only.

```rust
// Source: Google Gemini API docs (https://ai.google.dev/api/generate-content)
// In commands/providers/gemini.rs

// KEY DIFFERENCES from OpenAI-compat:
// 1. URL includes model name + ?alt=sse&key=API_KEY (API key in URL, not header)
// 2. Body format: { "contents": [{ "role": "user", "parts": [{ "text": "..." }] }] }
// 3. System prompt: top-level "systemInstruction" field with parts array
// 4. SSE data: candidates[0].content.parts[0].text (NOT choices[0].delta.content)
// 5. Role names: "user" and "model" (NOT "user" and "assistant")
// 6. No [DONE] -- stream ends when connection closes
// 7. Config in "generationConfig" object (NOT top-level temperature)

pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,  // needs conversion to Gemini format
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<(), String> {
    let client = reqwest::Client::new();

    // Gemini URL format: base/models/{model}:streamGenerateContent?alt=sse&key={key}
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
        model, api_key
    );

    // Convert messages from OpenAI format to Gemini format
    let contents: Vec<serde_json::Value> = messages.iter()
        .filter(|m| m["role"].as_str() != Some("system"))  // system handled separately
        .map(|m| {
            let role = match m["role"].as_str().unwrap_or("user") {
                "assistant" => "model",     // Gemini uses "model" not "assistant"
                other => other,
            };
            serde_json::json!({
                "role": role,
                "parts": [{ "text": m["content"].as_str().unwrap_or("") }]
            })
        })
        .collect();

    let body = serde_json::json!({
        "contents": contents,
        "systemInstruction": {
            "parts": [{ "text": system_prompt }]
        },
        "generationConfig": {
            "temperature": 0.1
        }
    }).to_string();

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| "Network error: Check your internet connection.".to_string())?;

    handle_http_status_gemini(response.status().as_u16())?;

    // Gemini with alt=sse returns standard SSE format
    let mut stream = response.bytes_stream().eventsource();
    tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if event.data.is_empty() { continue; }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        // Gemini nests text in candidates[0].content.parts[0].text
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
    }).await
    .map_err(|_| "Google Gemini: Request timed out. Try again.".to_string())?
}
```

### Pattern 6: Parameterized Keychain
**What:** Refactored keychain commands that accept a provider parameter.
**When to use:** Replacing the current hardcoded `ACCOUNT` constant.

```rust
// In commands/keychain.rs (refactored)
const SERVICE: &str = "com.lakshmanturlapati.cmd-k";

#[tauri::command]
pub fn save_api_key(provider: Provider, key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}

#[tauri::command]
pub fn get_api_key(provider: Provider) -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read from Keychain: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key(provider: Provider) -> Result<(), String> {
    let entry = Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry
        .delete_credential()
        .map_err(|e| format!("Failed to delete from Keychain: {}", e))
}
```

### Pattern 7: v0.2.4 Migration
**What:** One-time migration of existing xAI key to new provider-keyed entry.
**When to use:** In Tauri `.setup()` callback, runs on every app launch but is idempotent.

```rust
// In lib.rs setup or a new commands/migration.rs
fn migrate_v024_api_key() {
    let old_entry = match keyring::Entry::new(SERVICE, "xai_api_key") {
        Ok(e) => e,
        Err(_) => return,
    };

    // Check if old key exists
    let old_key = match old_entry.get_password() {
        Ok(k) => k,
        Err(_) => return, // No old key, nothing to migrate
    };

    // Check if new-format key already exists (migration already done)
    let new_entry = match keyring::Entry::new(SERVICE, "xai_api_key") {
        Ok(e) => e,
        Err(_) => return,
    };

    // The account name is the same for xAI ("xai_api_key"), so migration is about
    // ensuring the settings.json has provider set to "xai" and the key is accessible
    // through the new parameterized interface.
    //
    // Key action: write "provider": "xai" to settings.json if not already set
    // The keychain entry itself doesn't need copying since the account name hasn't changed.
}
```

**Important migration note:** Since the existing xAI keychain account is already `"xai_api_key"` and the new Provider::XAI also maps to `"xai_api_key"`, the actual keychain data does NOT need to be copied. The migration only needs to:
1. Check if an xAI key exists in keychain
2. If yes, write `"provider": "xai"` to `settings.json` (so the app defaults to xAI)
3. The key is already accessible through the new parameterized interface

### Anti-Patterns to Avoid
- **Trait objects for providers:** Don't use `Box<dyn Provider>` -- the enum is exhaustively known at compile time, making match dispatch simpler and more efficient
- **Per-provider HTTP clients:** Don't create separate `reqwest::Client` instances per provider -- one client with per-request headers is correct
- **System prompt in messages array for Anthropic/Gemini:** These providers use separate fields; putting system in messages will cause API errors
- **Hardcoded API key in URL for non-Gemini:** Only Gemini uses URL-param auth; OpenAI-compat and Anthropic use headers
- **Forgetting `max_tokens` for Anthropic:** Anthropic requires `max_tokens` in every request; omitting it causes 400 errors

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SSE parsing | Custom line-by-line parser | `eventsource-stream` crate | Handles reconnection, named events, multi-line data; already in deps |
| Keychain access | File-based key storage | `keyring` crate v3 | Platform-native (macOS Keychain, Windows Credential Manager); already in deps |
| Config persistence | Custom file I/O | `tauri-plugin-store` | Already used for settings.json; handles atomic writes, app data dir paths |
| HTTP client | Raw TCP/TLS | `tauri-plugin-http` (reqwest) | Already in deps with stream feature; handles TLS, redirects, timeouts |
| JSON serialization | Manual string building | `serde_json` | Already in deps; type-safe, handles escaping |

**Key insight:** The existing codebase already has every dependency needed. The refactoring is purely structural -- no new crates, just reorganization and generalization.

## Common Pitfalls

### Pitfall 1: Anthropic System Prompt Placement
**What goes wrong:** Putting the system prompt as a `{"role": "system", "content": "..."}` message in the messages array.
**Why it happens:** OpenAI convention bleeds into Anthropic code.
**How to avoid:** Anthropic requires a top-level `"system"` field in the request body. The messages array must only contain `"user"` and `"assistant"` roles.
**Warning signs:** Anthropic returns 400 with "messages: roles must alternate between user and assistant" or "system messages are not supported in messages".

### Pitfall 2: Gemini Role Naming
**What goes wrong:** Using `"assistant"` as a role name in Gemini API calls.
**Why it happens:** Copy-paste from OpenAI-format message building.
**How to avoid:** Gemini uses `"model"` instead of `"assistant"`. Convert roles when building the Gemini request body.
**Warning signs:** Gemini returns 400 with invalid role error.

### Pitfall 3: Gemini API Key Auth Method
**What goes wrong:** Sending the Gemini API key as a Bearer token in the Authorization header.
**Why it happens:** Assuming all APIs use Bearer auth like OpenAI.
**How to avoid:** Gemini uses URL query parameter: `?key=API_KEY`. No Authorization header.
**Warning signs:** 401 errors even with a valid Gemini key.

### Pitfall 4: Anthropic Auth Header Format
**What goes wrong:** Using `"Authorization: Bearer {key}"` for Anthropic.
**Why it happens:** Same as above -- OpenAI convention assumed.
**How to avoid:** Anthropic uses `"x-api-key: {key}"` header (no "Bearer" prefix). Also requires `"anthropic-version: 2023-06-01"`.
**Warning signs:** 401 authentication error with valid key.

### Pitfall 5: Missing max_tokens for Anthropic
**What goes wrong:** Not including `max_tokens` in the Anthropic request body.
**Why it happens:** OpenAI doesn't require it (defaults to model max).
**How to avoid:** Always include `"max_tokens": 4096` (or appropriate value) in Anthropic requests.
**Warning signs:** 400 error mentioning missing required field.

### Pitfall 6: Anthropic SSE Event Name Handling
**What goes wrong:** Treating Anthropic SSE as data-only events (like OpenAI's `data: {json}\n\n`).
**Why it happens:** Assuming all SSE streams work like OpenAI.
**How to avoid:** Anthropic SSE uses named events (`event: content_block_delta`). The `eventsource-stream` crate exposes `event.event` (the event name) alongside `event.data`. Filter on `event.event == "content_block_delta"` and extract `delta.text`.
**Warning signs:** Trying to parse `message_start` or `ping` events as content, getting None/empty tokens.

### Pitfall 7: OpenRouter Rate Headers Missing
**What goes wrong:** OpenRouter requests work but the app is not identified, preventing analytics/attribution.
**Why it happens:** Not reading the OpenRouter docs on required headers.
**How to avoid:** Include `HTTP-Referer` and `X-Title` headers on OpenRouter requests.
**Warning signs:** No analytics in OpenRouter dashboard; functionally works but misses attribution.

### Pitfall 8: Gemini Streaming URL Parameter
**What goes wrong:** Using `generateContent` endpoint instead of `streamGenerateContent` for streaming.
**Why it happens:** Not noticing Gemini has separate endpoints.
**How to avoid:** Use `streamGenerateContent?alt=sse` endpoint. The `alt=sse` parameter is required for SSE format.
**Warning signs:** Getting a single JSON response instead of streaming chunks.

### Pitfall 9: Frontend IPC Contract Breaking Change
**What goes wrong:** Changing the `stream_ai_response` IPC signature without updating the frontend `invoke` call.
**Why it happens:** Adding the `provider` parameter to the Rust command but forgetting the frontend.
**How to avoid:** Update the `submitQuery` function in `src/store/index.ts` to pass `provider` alongside `query`, `model`, `contextJson`, `history`, and `onToken`. The frontend must read the selected provider from settings.
**Warning signs:** Tauri invoke fails with "missing argument" error.

## Code Examples

### Provider-Specific Validation Endpoints

```rust
// Source: Official API docs for each provider
pub async fn validate_api_key(provider: &Provider, api_key: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    match provider {
        // OpenAI: GET /v1/models (list models validates key)
        Provider::OpenAI => {
            let resp = client.get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await.map_err(|_| "Network error".to_string())?;
            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                s => Err(format!("API error: {}", s)),
            }
        }
        // Anthropic: POST /v1/messages with max_tokens=1
        Provider::Anthropic => {
            let body = serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "hi"}]
            }).to_string();
            let resp = client.post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .body(body)
                .send().await.map_err(|_| "Network error".to_string())?;
            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                s => Err(format!("API error: {}", s)),
            }
        }
        // Gemini: GET /v1beta/models?key={key}
        Provider::Gemini => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key
            );
            let resp = client.get(&url)
                .send().await.map_err(|_| "Network error".to_string())?;
            match resp.status().as_u16() {
                200 => Ok(()),
                400 | 403 => Err("invalid_key".to_string()),
                s => Err(format!("API error: {}", s)),
            }
        }
        // xAI: existing pattern (GET /v1/models, fallback to POST /v1/chat/completions)
        Provider::XAI => {
            // Reuse existing validate_and_fetch_models logic
            let resp = client.get("https://api.x.ai/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await.map_err(|_| "Network error".to_string())?;
            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                404 => {
                    // Fallback: try chat completions
                    let body = serde_json::json!({
                        "model": "grok-3", "messages": [{"role":"user","content":"hi"}],
                        "max_tokens": 1
                    }).to_string();
                    let r = client.post("https://api.x.ai/v1/chat/completions")
                        .header("Authorization", format!("Bearer {}", api_key))
                        .header("Content-Type", "application/json")
                        .body(body).send().await.map_err(|_| "Network error".to_string())?;
                    match r.status().as_u16() {
                        200 => Ok(()),
                        401 => Err("invalid_key".to_string()),
                        s => Err(format!("API error: {}", s)),
                    }
                }
                s => Err(format!("API error: {}", s)),
            }
        }
        // OpenRouter: GET /api/v1/models
        Provider::OpenRouter => {
            let resp = client.get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await.map_err(|_| "Network error".to_string())?;
            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                s => Err(format!("API error: {}", s)),
            }
        }
    }
}
```

### Provider-Specific Error Messages

```rust
// Source: CONTEXT.md locked decision
fn handle_http_status(provider: &Provider, status: u16) -> Result<(), String> {
    match status {
        200 => Ok(()),
        401 => Err(format!(
            "{}: Authentication failed. Check your API key at {}.",
            provider.display_name(), provider.console_url()
        )),
        429 => Err(format!(
            "{}: Rate limited. Wait a moment and try again.",
            provider.display_name()
        )),
        _ => Err(format!(
            "{}: API error ({}). Try again.",
            provider.display_name(), status
        )),
    }
}
```

### Hardcoded Curated Model Lists

```rust
// Source: Provider documentation and model pages

pub fn curated_models(provider: &Provider) -> Vec<ModelWithMeta> {
    match provider {
        Provider::OpenAI => vec![
            ModelWithMeta { id: "gpt-4o".into(), label: "Balanced".into(), tier: "balanced".into() },
            ModelWithMeta { id: "gpt-4o-mini".into(), label: "Fast".into(), tier: "fast".into() },
            ModelWithMeta { id: "gpt-4.1".into(), label: "Most Capable".into(), tier: "capable".into() },
            ModelWithMeta { id: "gpt-4.1-mini".into(), label: "Fast".into(), tier: "fast".into() },
            ModelWithMeta { id: "gpt-4.1-nano".into(), label: "Fastest".into(), tier: "fast".into() },
        ],
        Provider::Anthropic => vec![
            ModelWithMeta { id: "claude-sonnet-4-20250514".into(), label: "Balanced".into(), tier: "balanced".into() },
            ModelWithMeta { id: "claude-haiku-3-5-20241022".into(), label: "Fast".into(), tier: "fast".into() },
            ModelWithMeta { id: "claude-opus-4-20250514".into(), label: "Most Capable".into(), tier: "capable".into() },
        ],
        Provider::Gemini => vec![
            ModelWithMeta { id: "gemini-2.0-flash".into(), label: "Fast".into(), tier: "fast".into() },
            ModelWithMeta { id: "gemini-2.5-pro-preview-06-05".into(), label: "Most Capable".into(), tier: "capable".into() },
            ModelWithMeta { id: "gemini-2.5-flash-preview-05-20".into(), label: "Balanced".into(), tier: "balanced".into() },
        ],
        Provider::XAI => vec![
            ModelWithMeta { id: "grok-3".into(), label: "Balanced".into(), tier: "balanced".into() },
            ModelWithMeta { id: "grok-3-mini".into(), label: "Fast".into(), tier: "fast".into() },
            ModelWithMeta { id: "grok-4".into(), label: "Most Capable".into(), tier: "capable".into() },
        ],
        Provider::OpenRouter => vec![], // OpenRouter fetches from API
    }
}
```

### Model Fetching Per Provider

```rust
// Source: Official API docs
pub async fn fetch_models(provider: &Provider, api_key: &str) -> Result<Vec<ModelWithMeta>, String> {
    let client = reqwest::Client::new();
    match provider {
        // OpenAI, xAI: GET /v1/models
        Provider::OpenAI => {
            let resp = client.get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await.map_err(|_| "Network error".to_string())?;
            // Parse response.data array, filter to chat models
            // ...
            todo!()
        }
        // OpenRouter: GET /api/v1/models, filter chat-capable
        Provider::OpenRouter => {
            let resp = client.get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send().await.map_err(|_| "Network error".to_string())?;
            // Parse response, filter to chat-capable models
            // ...
            todo!()
        }
        // Gemini: GET /v1beta/models?key={key}
        Provider::Gemini => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key
            );
            let resp = client.get(&url)
                .send().await.map_err(|_| "Network error".to_string())?;
            // Parse response, filter to generateContent-capable models
            // ...
            todo!()
        }
        // Anthropic: no public model list API -- use hardcoded only
        Provider::Anthropic => Ok(curated_models(provider)),
        // xAI: existing pattern
        Provider::XAI => {
            // Reuse existing validate_and_fetch_models logic
            todo!()
        }
    }
}
```

### Frontend IPC Update (store/index.ts)

```typescript
// Source: Existing pattern in src/store/index.ts
// The submitQuery function needs to pass provider to the backend

// In the invoke call (line ~475 of store/index.ts):
await invoke("stream_ai_response", {
  provider: selectedProvider,  // NEW: read from settings
  query,
  model: selectedModel,
  contextJson,
  history,
  onToken,
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| OpenAI-only API format | Each provider has its own REST API | 2023-2024 | Must handle 3 distinct formats |
| Bearer auth everywhere | Anthropic uses x-api-key, Gemini uses URL param | Always | Auth logic varies per provider |
| Single SSE format | Anthropic named events, Gemini alt=sse | Always | SSE parsing differs per adapter |
| `gpt-3.5-turbo` | `gpt-4o`, `gpt-4.1` | 2024-2025 | Model IDs change; hardcoded lists need periodic updates |

**Deprecated/outdated:**
- Anthropic's `v1/complete` endpoint (replaced by `v1/messages`)
- OpenAI's `text-davinci-003` and similar completion models (replaced by chat models)
- Google PaLM API (replaced by Gemini API)

## Open Questions

1. **Anthropic model list API**
   - What we know: Anthropic does not have a public REST endpoint to list available models
   - What's unclear: Whether they plan to add one
   - Recommendation: Use hardcoded curated list only for Anthropic; no "Refresh models" for this provider

2. **Gemini model ID stability**
   - What we know: Gemini model IDs include preview/date suffixes that change frequently
   - What's unclear: How often the stable model aliases update
   - Recommendation: Use stable aliases like `gemini-2.0-flash` in curated lists; accept that API-fetched lists will include preview models

3. **Temperature parameter compatibility**
   - What we know: All providers accept `temperature` but ranges differ slightly. OpenAI: 0-2, Anthropic: 0-1, Gemini: 0-2
   - What's unclear: Whether 0.1 works well across all providers
   - Recommendation: Use 0.1 for all (within valid range for all providers); this is Claude's discretion per CONTEXT.md

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None detected -- no test infrastructure exists |
| Config file | none -- see Wave 0 |
| Quick run command | N/A |
| Full suite command | N/A |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROV-01 | Provider selection persists | manual-only | Manual: select provider, restart app, verify selection | N/A |
| PROV-02 | Per-provider keychain storage | manual-only | Manual: save key for each provider, verify retrieval | N/A |
| PROV-03 | v0.2.4 xAI key migration | manual-only | Manual: install v0.2.4, add key, upgrade to v0.2.6, verify | N/A |
| PROV-04 | API key validation per provider | manual-only | Manual: enter valid/invalid keys, verify messages | N/A |
| PROV-05 | Available models per provider | manual-only | Manual: select provider, verify model list appears | N/A |
| PROV-06 | Real-time streaming all providers | manual-only | Manual: generate command with each provider, verify streaming | N/A |
| PROV-07 | Provider-specific error messages | manual-only | Manual: trigger errors, verify provider name in message | N/A |

**Note:** All requirements involve external API calls and platform-specific keychain operations, making unit testing impractical without mocking infrastructure. Testing is manual for this phase.

### Sampling Rate
- **Per task commit:** Manual smoke test -- generate a command with the provider being implemented
- **Per wave merge:** Test all 5 providers end-to-end (key save, validate, stream, error)
- **Phase gate:** All 5 providers streaming successfully + migration verified

### Wave 0 Gaps
None -- no test infrastructure to set up for this phase (all manual testing).

## Sources

### Primary (HIGH confidence)
- Anthropic Messages API streaming docs -- SSE event types, request format, auth headers (https://docs.anthropic.com/en/api/messages-streaming -> https://platform.claude.com/docs/en/api/messages-streaming)
- Google Gemini API generateContent reference -- streamGenerateContent endpoint, URL format, systemInstruction field (https://ai.google.dev/api/generate-content)
- OpenAI Chat Completions API streaming -- SSE format, [DONE] sentinel (https://platform.openai.com/docs/api-reference/chat-streaming)
- OpenRouter API docs -- required headers, /api/v1/models endpoint, OpenAI compatibility (https://openrouter.ai/docs/api/reference/overview)
- Existing codebase: `commands/ai.rs`, `commands/keychain.rs`, `commands/xai.rs`, `src/store/index.ts`

### Secondary (MEDIUM confidence)
- keyring crate v3 docs -- platform support, feature flags (https://docs.rs/keyring)
- eventsource-stream crate -- named event support via `.event` field

### Tertiary (LOW confidence)
- Curated model lists -- model IDs based on current knowledge; may need updating before release
- Gemini preview model IDs -- change frequently; use stable aliases

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates already in use, no new dependencies
- Architecture: HIGH -- patterns derived directly from existing codebase + verified API docs
- Pitfalls: HIGH -- each pitfall verified against official API documentation
- Model lists: MEDIUM -- model IDs are current but may change before release

**Research date:** 2026-03-09
**Valid until:** 2026-04-09 (30 days -- API formats are stable; model IDs may need refresh)
