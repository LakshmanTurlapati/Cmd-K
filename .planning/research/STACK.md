# Technology Stack: Local LLM Providers (Ollama + LM Studio)

**Project:** CMD+K v0.3.11
**Researched:** 2026-03-17
**Scope:** Additions/changes needed for Ollama and LM Studio local provider support only

## Executive Summary

Both Ollama and LM Studio expose OpenAI-compatible `/v1/chat/completions` endpoints with SSE streaming, `data: [DONE]` sentinels, and `stream_options: { include_usage: true }` support. This means **the existing `openai_compat::stream()` adapter can be reused with zero modifications to its streaming logic**. The work is structural: extending the `Provider` enum, adding keyless auth flow, dynamic model discovery via `/v1/models`, connection health checking, and configurable base URLs.

## Critical Integration Fact: OpenAI-Compat Adapter Reuse

**Confidence: HIGH** (verified via Ollama official docs, LM Studio official docs)

Both local providers use the exact same SSE streaming format as the existing `openai_compat::stream()`:

| Feature | OpenAI/xAI/OpenRouter | Ollama `/v1/` | LM Studio `/v1/` |
|---------|----------------------|---------------|-------------------|
| Endpoint | `/v1/chat/completions` | `/v1/chat/completions` | `/v1/chat/completions` |
| SSE format | `data: {JSON}` chunks | `data: {JSON}` chunks | `data: {JSON}` chunks |
| Content path | `choices[0].delta.content` | `choices[0].delta.content` | `choices[0].delta.content` |
| Stream termination | `data: [DONE]` | `data: [DONE]` | `data: [DONE]` |
| `stream_options.include_usage` | Yes | Yes (since Dec 2024, PR #6784) | Yes (since v0.3.18) |
| Usage path | `usage.prompt_tokens` / `usage.completion_tokens` | Same | Same |
| Auth header | `Authorization: Bearer <key>` | None required | None required |

**What this means:** The `openai_compat::stream()` function in `src-tauri/src/commands/providers/openai_compat.rs` handles SSE parsing, token extraction, and `[DONE]` detection. All of that works identically for local providers. The only change needed is passing a dynamic URL and conditionally omitting the `Authorization` header.

## Recommended Stack Changes

### No New Rust Crates Required

**Confidence: HIGH**

The existing dependency set already provides everything needed:

| Existing Dependency | Used For | Reuse for Local Providers |
|---------------------|----------|--------------------------|
| `tauri-plugin-http` (reqwest) | HTTP client for streaming | Same client for local HTTP requests |
| `eventsource-stream` | SSE parsing | Same SSE format from local servers |
| `futures-util` | Stream combinators | Same async streaming |
| `tokio` | Async runtime + timeouts | Same timeout handling |
| `serde` / `serde_json` | JSON ser/de | Same response format |
| `tauri-plugin-store` | Settings persistence | Store base URLs and provider config |

**Zero new Cargo dependencies.** The existing stack handles HTTP streaming, SSE parsing, and JSON deserialization, which is everything needed to talk to local LLM servers.

### Rust Backend Changes Required

#### 1. Extend Provider Enum

```rust
// src-tauri/src/commands/providers/mod.rs
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    XAI,
    OpenRouter,
    Ollama,    // NEW
    LMStudio,  // NEW
}
```

Each match arm needs updating across `keychain_account()`, `api_url()`, `display_name()`, `console_url()`, `adapter_kind()`, `default_timeout_secs()`.

Key differences from cloud providers:

| Method | Ollama | LM Studio | Notes |
|--------|--------|-----------|-------|
| `keychain_account()` | N/A - no API key | N/A - no API key | Local providers do not use keychain |
| `api_url()` | `http://localhost:11434/v1/chat/completions` | `http://localhost:1234/v1/chat/completions` | Must be configurable, not hardcoded `&'static str` |
| `display_name()` | `"Ollama"` | `"LM Studio"` | Straightforward |
| `console_url()` | `"ollama.com"` | `"lmstudio.ai"` | Link to project site, not a key console |
| `adapter_kind()` | `AdapterKind::OpenAICompat` | `AdapterKind::OpenAICompat` | Same adapter as OpenAI/xAI/OpenRouter |
| `default_timeout_secs()` | `120` | `120` | Local models are slower; need longer timeout |

#### 2. Introduce Local Provider Distinction: No API Key Required

The current `stream_ai_response` command unconditionally reads an API key from the keychain (line 206-213 of `ai.rs`). Local providers must skip this.

**Recommended: `Provider::requires_api_key()` method**

```rust
impl Provider {
    pub fn requires_api_key(&self) -> bool {
        match self {
            Provider::Ollama | Provider::LMStudio => false,
            _ => true,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Provider::Ollama | Provider::LMStudio)
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::Ollama => "http://localhost:11434",
            Provider::LMStudio => "http://localhost:1234",
            _ => "", // Cloud providers use api_url() directly
        }
    }
}
```

Then in `stream_ai_response`:
```rust
let api_key = if provider.requires_api_key() {
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())...;
    entry.get_password()...
} else {
    String::new() // No auth needed
};
```

And in `openai_compat::stream()`, conditionally add the Authorization header:
```rust
if !api_key.is_empty() {
    request = request.header("Authorization", format!("Bearer {}", api_key));
}
```

#### 3. Dynamic Base URL via Settings Store

The current `Provider::api_url()` returns `&'static str`. Local providers need user-configurable base URLs (users may run Ollama on a different machine, or use a non-default port).

**Recommended: Store base URLs in `tauri-plugin-store` (settings.json)**

Settings keys:
- `ollama_base_url` -- default: `http://localhost:11434`
- `lmstudio_base_url` -- default: `http://localhost:1234`

The `api_url()` method stays static for the default, but `stream_ai_response` and `fetch_models` read the configured URL from the store at call time. This avoids making `Provider::api_url()` async or state-dependent.

#### 4. Model Discovery Endpoints

Both providers expose `/v1/models` with the standard OpenAI response format:

```json
{
  "object": "list",
  "data": [
    { "id": "llama3.2:latest", "object": "model", "created": 1234567890, "owned_by": "library" }
  ]
}
```

**Confidence: HIGH** (Ollama docs confirm ToListCompletion conversion; LM Studio docs confirm OpenAI-compat /v1/models)

The existing `OpenAIModelsResponse` and `OpenAIModel` structs in `models.rs` (lines 236-243) already parse this format. The `fetch_api_models` function needs a new match arm for `Provider::Ollama` and `Provider::LMStudio` that:

1. GETs `{base_url}/v1/models` (no auth header)
2. Parses with existing `OpenAIModelsResponse` struct
3. Returns all models (no filtering needed -- local models are all relevant)
4. Sets `tier: String::new()` (no tier grouping for local models)
5. Sets pricing to `None` (local = free, no cost tracking)

**No curated model list for local providers.** Unlike cloud providers, local model lists are fully dynamic (user downloads what they want). The `curated_models()` function returns `vec![]` for both.

#### 5. Connection Health Check

**Ollama:** `GET /` returns `"Ollama is running"` with HTTP 200. Confidence: HIGH (official docs + GitHub issue #1378).

**LM Studio:** `GET /v1/models` returning HTTP 200 indicates the server is running. No dedicated health endpoint exists, but the models endpoint serves the same purpose. Confidence: MEDIUM (no official health endpoint documented; using /v1/models as proxy).

**Recommended: New Tauri command `check_provider_health`**

```rust
#[tauri::command]
pub async fn check_provider_health(provider: Provider, base_url: String) -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;

    let url = match provider {
        Provider::Ollama => base_url.clone(),  // GET / returns "Ollama is running"
        Provider::LMStudio => format!("{}/v1/models", base_url),
        _ => return Err("Not a local provider".into()),
    };

    match client.get(&url).send().await {
        Ok(resp) => Ok(resp.status().is_success()),
        Err(_) => Ok(false),  // Connection refused = not running
    }
}
```

3-second timeout to prevent UI blocking. Returns `false` on connection refused (not an error -- expected state when server is off).

#### 6. Token Usage for Local Providers

Both Ollama and LM Studio support `stream_options: { include_usage: true }` on `/v1/chat/completions`. The existing `openai_compat::stream()` already extracts `usage.prompt_tokens` and `usage.completion_tokens` from the final SSE chunk. This works unchanged.

**Cost estimation:** Local providers have no per-token cost. The existing `UsageStatsResponse` already handles this gracefully -- when no pricing is found in `curated_models_pricing()` or `openrouter_pricing`, it returns `pricing_available: false` and `estimated_cost: None`. The frontend already displays a pricing-unavailable indicator for this case. No changes needed.

**Token tracking still valuable:** Even without cost, users want to see token counts to understand model context usage. The existing accumulator handles this.

### TypeScript Frontend Changes Required

#### 1. Extend PROVIDERS Array

```typescript
// src/store/index.ts
export const PROVIDERS = [
  { id: "openai", name: "OpenAI" },
  { id: "anthropic", name: "Anthropic" },
  { id: "gemini", name: "Google Gemini" },
  { id: "xai", name: "xAI" },
  { id: "openrouter", name: "OpenRouter" },
  { id: "ollama", name: "Ollama", local: true },      // NEW
  { id: "lmstudio", name: "LM Studio", local: true },  // NEW
] as const;
```

The `local: true` flag lets UI components conditionally:
- Hide API key input for local providers
- Show connection status indicator instead
- Show base URL configuration input
- Hide pricing column in usage stats

#### 2. Account Tab Adaptation

The `AccountTab.tsx` currently shows an API key input for every provider. For local providers, it should show:
- Base URL input (editable, with default pre-filled)
- Connection status indicator (green/red dot + "Connected" / "Not running")
- "Test Connection" button that calls `check_provider_health`
- Link to download/install instructions if not detected

No new npm packages needed. The existing `invoke()` and `Store` from `@tauri-apps/plugin-store` handle everything.

#### 3. Provider Icon SVGs

Two new SVG paths needed in `ProviderIcon.tsx` for Ollama and LM Studio logos. Pattern matches existing inline SVG approach (no external assets).

#### 4. Validation Flow Adaptation

The current `validate_api_key` command is called in `AccountTab.tsx` to verify API keys. For local providers, replace this with `check_provider_health`. The `apiKeyStatus` state can be repurposed: `"valid"` means server is reachable and has models loaded.

## What NOT to Add

| Anti-Addition | Why Not |
|---------------|---------|
| New streaming adapter | Both use OpenAI-compat SSE format -- reuse existing adapter |
| Ollama native `/api/chat` endpoint | `/v1/chat/completions` is strictly more compatible, same features |
| ollama-rs or similar Rust SDK crate | Adds dependency for what is 3 HTTP calls (health, models, chat) |
| LM Studio SDK crate | Same reasoning -- raw HTTP via existing reqwest is simpler |
| Model download/management | Out of scope -- users manage models via Ollama CLI or LM Studio GUI |
| GPU/VRAM monitoring | Out of scope -- local runtime concerns, not CMD+K's job |
| Auto-detection of running servers via background polling | Wasteful; check on provider selection + query time |
| New keychain entries for local providers | No API key = no keychain interaction |
| Cost estimation for local models | Local = free; existing `pricing_available: false` path handles this |

## API Endpoint Reference

### Ollama

| Purpose | Method | Path | Auth | Default Base |
|---------|--------|------|------|-------------|
| Health check | GET | `/` | None | `http://localhost:11434` |
| List models | GET | `/v1/models` | None | `http://localhost:11434` |
| Chat completions | POST | `/v1/chat/completions` | None | `http://localhost:11434` |
| Native model list | GET | `/api/tags` | None | `http://localhost:11434` |

Use `/v1/models` over `/api/tags` because it matches the existing `OpenAIModelsResponse` struct. The native `/api/tags` returns a different format (`{ models: [...] }` with `name`, `modified_at`, `size`, `digest`, `details` fields) that would require a separate deserializer.

### LM Studio

| Purpose | Method | Path | Auth | Default Base |
|---------|--------|------|------|-------------|
| Health check | GET | `/v1/models` | None | `http://localhost:1234` |
| List models | GET | `/v1/models` | None | `http://localhost:1234` |
| Chat completions | POST | `/v1/chat/completions` | None | `http://localhost:1234` |

No dedicated health endpoint; use `/v1/models` (returns 200 if server is running, connection refused if not).

## Architecture Integration Map

```
Existing flow (cloud providers):
  Frontend -> invoke("stream_ai_response") -> keychain lookup -> openai_compat::stream(static_url, api_key, ...) -> SSE parse

New flow (local providers):
  Frontend -> invoke("stream_ai_response") -> skip keychain -> openai_compat::stream(dynamic_url, empty_key, ...) -> SSE parse (identical)

New flow (model discovery):
  Frontend -> invoke("fetch_models") -> GET {base_url}/v1/models -> parse OpenAIModelsResponse -> return Vec<ModelWithMeta>

New flow (health check):
  Frontend -> invoke("check_provider_health") -> GET {base_url}[/v1/models] -> bool
```

The streaming path is identical after URL resolution and auth bypass. The `openai_compat::stream()` function does not need to know whether it is talking to OpenAI's cloud or a local Ollama instance.

## Modifications to Existing Code (Surgical List)

### Must Change

| File | What Changes | Why |
|------|-------------|-----|
| `providers/mod.rs` | Add `Ollama`, `LMStudio` to `Provider` enum; add match arms for all methods; add `requires_api_key()`, `is_local()`, `default_base_url()` methods | Core provider identity |
| `providers/openai_compat.rs` | Accept dynamic URL parameter instead of `provider.api_url()`; conditionally add auth header when key is non-empty | Support keyless + non-static URLs |
| `commands/ai.rs` | Conditional keychain lookup via `requires_api_key()`; read base URL from store for local providers; build full URL | Entry point for streaming |
| `commands/models.rs` | Add `fetch_api_models` arms for Ollama/LMStudio; `curated_models()` returns `vec![]` for local; add `check_provider_health` command | Model discovery + health check |
| `src/store/index.ts` | Add Ollama/LMStudio to PROVIDERS array with `local` flag | UI provider list |
| `AccountTab.tsx` | Conditional rendering: base URL + connection status for local providers; API key for cloud providers | Provider-specific settings UI |
| `StepProviderSelect.tsx` | Show Ollama/LMStudio as selectable options | Onboarding flow |
| `ProviderIcon.tsx` | Add Ollama and LM Studio SVG paths | Provider branding |

### No Changes Needed

| File | Why Unchanged |
|------|--------------|
| `state.rs` | `UsageAccumulator` works as-is for local providers |
| `commands/keychain.rs` | Local providers skip keychain entirely; existing code untouched |
| `commands/usage.rs` | `pricing_available: false` path already handles free providers |
| `providers/anthropic.rs` | Unrelated adapter |
| `providers/gemini.rs` | Unrelated adapter |

### New Files (Optional)

A `check_provider_health` command can go in `commands/models.rs` alongside `fetch_models` and `validate_api_key`, since they share the same concern (provider connectivity). Alternatively, a small `commands/health.rs` keeps concerns cleanly separated. Either works; no strong preference.

## Configuration Storage

Using existing `tauri-plugin-store` with `settings.json`:

| Key | Type | Default | Purpose |
|-----|------|---------|---------|
| `ollama_base_url` | `string` | `http://localhost:11434` | Ollama server address |
| `lmstudio_base_url` | `string` | `http://localhost:1234` | LM Studio server address |
| `selectedProvider` | `string` | (existing) | Now also accepts `"ollama"` / `"lmstudio"` |

## Timeout Considerations

Local models are significantly slower than cloud APIs, especially on CPU-only machines.

| Phase | Cloud Providers | Local Providers | Rationale |
|-------|----------------|-----------------|-----------|
| Health check | N/A | 3 seconds | Fast fail for connection check |
| First token | 10-30 seconds | 120 seconds | Local model loading can be slow (cold start) |
| Streaming | 30 seconds total | 120 seconds total | Larger local models generate slowly |

The `default_timeout_secs()` method should return `120` for local providers.

**Timeout architecture concern:** The current implementation in `openai_compat.rs` wraps the *entire* streaming operation with `tokio::time::timeout` (line 57). For local models generating long responses, a 120-second total timeout may not be enough. The stream itself is active (tokens arriving), but the outer timeout does not reset on each token. Consider either: (a) increasing to 300 seconds for local, or (b) restructuring to a per-chunk idle timeout. Flag this for implementation-phase decision.

## Confidence Assessment

| Claim | Confidence | Source |
|-------|-----------|--------|
| Ollama uses SSE with `data: [DONE]` on `/v1/` | HIGH | Ollama OpenAI compatibility docs, Ollama blog |
| Ollama supports `stream_options.include_usage` | HIGH | GitHub PR #6784 merged Dec 2024 |
| Ollama health via `GET /` returning 200 | HIGH | GitHub issue #1378, Ollama FAQ |
| Ollama `/v1/models` returns OpenAI format | HIGH | Ollama OpenAI compatibility docs |
| LM Studio uses SSE on `/v1/chat/completions` | HIGH | LM Studio OpenAI compat docs |
| LM Studio supports `stream_options.include_usage` | HIGH | LM Studio docs (added v0.3.18) |
| LM Studio `/v1/models` returns OpenAI format | MEDIUM | LM Studio docs list it as OpenAI-compat endpoint; exact field mapping inferred |
| LM Studio has no dedicated health endpoint | MEDIUM | Not in REST API docs; using `/v1/models` as proxy |
| No new Rust crates needed | HIGH | All HTTP/SSE/JSON deps already in Cargo.toml |
| Existing token tracking works unchanged | HIGH | Verified in codebase: `pricing_available: false` path exists in usage.rs |

## Sources

- [Ollama API documentation](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Ollama OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility)
- [Ollama streaming docs](https://docs.ollama.com/api/streaming)
- [Ollama health check issue #1378](https://github.com/ollama/ollama/issues/1378)
- [Ollama stream_options PR #6784](https://github.com/ollama/ollama/issues/5200)
- [LM Studio OpenAI compatibility endpoints](https://lmstudio.ai/docs/developer/openai-compat)
- [LM Studio REST API v0 endpoints](https://lmstudio.ai/docs/developer/rest/endpoints)
- [LM Studio model listing](https://lmstudio.ai/docs/developer/openai-compat/models)
- [LM Studio developer docs](https://lmstudio.ai/docs/developer)
- [LM Studio server docs](https://lmstudio.ai/docs/developer/core/server)
