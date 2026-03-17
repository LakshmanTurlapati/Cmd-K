# Architecture: Local LLM Provider Integration (Ollama + LM Studio)

**Domain:** Local LLM provider integration into existing multi-provider AI terminal overlay
**Researched:** 2026-03-17
**Confidence:** HIGH -- based on direct codebase analysis + official API documentation

## Executive Summary

Ollama and LM Studio both expose OpenAI-compatible `/v1/chat/completions` endpoints with SSE streaming. This is the same protocol the existing `openai_compat::stream()` adapter handles for OpenAI, xAI, and OpenRouter. The integration requires zero new streaming adapters. The main architectural changes are: (1) the `Provider` enum gains two keyless, URL-configurable variants, (2) model discovery calls local HTTP endpoints instead of cloud APIs, (3) a new connection health check system determines reachability before streaming, and (4) the frontend gains local-provider-specific UI for base URL configuration and connection status.

## Recommended Architecture

### Integration Strategy: Reuse OpenAI-Compatible Adapter

Both Ollama (`localhost:11434/v1/chat/completions`) and LM Studio (`localhost:1234/v1/chat/completions`) speak the same SSE streaming protocol already handled by `providers/openai_compat.rs`. The existing adapter accepts a `Provider` reference and calls `provider.api_url()` for the endpoint URL. The path to integration is:

1. Add `Ollama` and `LMStudio` variants to the `Provider` enum
2. Make `api_url()` return a dynamic URL (stored in config) instead of a `&'static str`
3. Skip API key retrieval for local providers (they ignore authentication)
4. Reuse `openai_compat::stream()` with a dummy/empty API key

### Component Boundaries

| Component | Responsibility | New/Modified | Communicates With |
|-----------|---------------|--------------|-------------------|
| `Provider` enum | Provider identity, adapter dispatch, URL resolution | **Modified** -- add Ollama, LMStudio variants | All provider-aware code |
| `ProviderConfig` (new) | Store per-provider base URLs, persist to settings.json | **New** | Provider, AppState, frontend |
| `openai_compat::stream()` | SSE streaming for OpenAI-compat APIs | **Modified** -- accept dynamic URL, optional API key | AI command handler |
| `health::check_provider()` (new) | HTTP health check for local providers | **New** | Frontend (via IPC), streaming pre-flight |
| `models::fetch_models()` | Model listing per provider | **Modified** -- add Ollama /api/tags + LM Studio /v1/models branches | Frontend model dropdown |
| `models::validate_api_key()` | Key validation per provider | **Modified** -- local providers validate via health check instead | Frontend onboarding/settings |
| `ai::stream_ai_response()` | Main AI dispatch | **Modified** -- read config URL, skip keychain for local | Frontend submitQuery |
| `keychain.rs` | API key CRUD | **No change** -- local providers skip keychain entirely | Account settings |
| `state::AppState` | Shared Tauri state | **Modified** -- add provider_configs field | All IPC commands |
| `PROVIDERS` (frontend) | Provider list for UI | **Modified** -- add Ollama, LM Studio entries | Onboarding, Settings |
| `AccountTab.tsx` | Provider selection + API key entry | **Modified** -- show URL config instead of key input for local | Settings panel |
| `StepApiKey.tsx` | Onboarding key entry | **Modified** -- skip or show URL config for local | Onboarding wizard |
| `ProviderIcon.tsx` | SVG icons per provider | **Modified** -- add Ollama + LM Studio icons | All provider UI |
| `store/index.ts` | Zustand state + submitQuery | **Modified** -- handle keyless providers in validation checks | Entire frontend |

## Data Flow Changes

### 1. Model Discovery Flow

**Current flow (cloud providers):**
```
Frontend selects provider
  -> invoke("fetch_models", { provider, apiKey })
    -> Rust reads API key from keychain
    -> HTTP GET to cloud /v1/models endpoint
    -> Parse OpenAI-format response { data: [{ id }] }
    -> Merge curated + API models
    -> Return ModelWithMeta[]
```

**New flow (local providers):**
```
Frontend selects Ollama/LMStudio
  -> invoke("fetch_models", { provider, apiKey: "" })
    -> Rust reads base_url from ProviderConfig (settings.json)
    -> For Ollama:
         GET {base_url}/api/tags
         Parse { models: [{ name, details: { parameter_size, family } }] }
         Map to ModelWithMeta with tier="" (no tiers for local)
    -> For LM Studio:
         GET {base_url}/v1/models
         Parse OpenAI-format { data: [{ id }] }
         Map to ModelWithMeta with tier=""
    -> Return ModelWithMeta[] (no curated list, all dynamic)
```

**Key differences:**
- No API key needed (pass empty string or skip keychain lookup)
- Ollama has its own `/api/tags` response format (not OpenAI-format) with richer metadata (parameter_size, quantization, family)
- LM Studio uses standard OpenAI `/v1/models` format
- No curated model list -- all models are dynamic (user controls what they download)
- Model labels should include parameter size and quantization info where available (e.g., "llama3.2:3b-q4_0" -> "Llama 3.2 3B Q4_0")

### 2. Connection Health Check Flow (New)

**Purpose:** Local servers may not be running. The UI needs to show connection status before the user tries to generate.

```
Frontend mounts/provider changes
  -> invoke("check_local_provider", { provider })
    -> Rust reads base_url from ProviderConfig
    -> For Ollama:
         GET {base_url}/  (returns 200 "Ollama is running")
    -> For LM Studio:
         GET {base_url}/v1/models  (returns 200 with model list)
    -> Return { connected: bool, model_count: u32 }
```

**Health check characteristics:**
- Ollama: `GET http://localhost:11434/` returns `200 OK` with body "Ollama is running"
- LM Studio: `GET http://localhost:1234/v1/models` returns `200 OK` with model list (double-duty as health check + model discovery)
- Timeout: 2 seconds (local server should respond near-instantly)
- Called on: provider selection change, settings mount, overlay open (if local provider active)
- Result drives UI: green dot = connected, red dot = not running, amber = unreachable

### 3. Streaming Flow

**Current flow:**
```
submitQuery()
  -> invoke("stream_ai_response", { provider, query, model, contextJson, history, onToken })
    -> Read API key from keychain
    -> Build messages array
    -> Dispatch to openai_compat::stream(provider, api_key, model, messages, on_token, timeout)
      -> POST to provider.api_url() with Bearer auth
      -> SSE stream: data: {JSON} chunks with choices[0].delta.content
      -> Final chunk: usage { prompt_tokens, completion_tokens }
    -> Record token usage
```

**New flow (local providers):**
```
submitQuery()
  -> invoke("stream_ai_response", { provider, query, model, contextJson, history, onToken })
    -> Skip keychain read (local provider)
    -> Read base_url from ProviderConfig in AppState
    -> Build messages array (identical)
    -> Dispatch to openai_compat::stream(provider, "", model, messages, on_token, timeout)
      -> POST to {base_url}/v1/chat/completions with empty/dummy Bearer
      -> SSE stream: identical format (both are OpenAI-compatible)
      -> Final chunk: usage { prompt_tokens, completion_tokens }
         (Ollama supports stream_options.include_usage)
         (LM Studio 0.3.18+ supports stream_options.include_usage)
    -> Record token usage (cost = None for local providers, $0.00)
```

**Key differences:**
- No API key needed -- pass empty string as Bearer token (both servers ignore it)
- URL comes from config instead of hardcoded constant
- Token usage tracking still works (local servers return prompt_tokens/completion_tokens)
- Cost estimation: local models have no pricing, `pricing_available: false`
- Timeout should be longer for local (models may need loading time): 60s default

### 4. Cost Tracking Flow

**No pricing for local models.** The existing cost display already handles `pricing_available: false` gracefully (shows "$---" with "pricing unavailable" note). Local providers will:
- Still track token counts (input/output) via the existing UsageAccumulator
- Return `None` for `estimated_cost` in usage stats
- Not appear in curated_models_pricing()
- Sparkline bars show `$0` / no bar for local queries (existing behavior for unpriced models)

## Detailed Component Changes

### Backend (Rust)

#### 1. `providers/mod.rs` -- Provider Enum Extension

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    #[serde(rename = "xai")]
    XAI,
    OpenRouter,
    Ollama,       // NEW
    LMStudio,     // NEW
}
```

New methods needed:
- `is_local(&self) -> bool` -- returns true for Ollama/LMStudio
- `default_base_url(&self) -> &'static str` -- "http://localhost:11434" / "http://localhost:1234"
- `display_name()` -- "Ollama" / "LM Studio"
- `config_key(&self) -> &'static str` -- "ollama" / "lmstudio" (settings.json key)

Modified methods:
- `api_url()` -- CANNOT return `&'static str` for local providers (URL is configurable). Two options:
  - **Option A (recommended):** Keep `api_url()` for cloud, add `api_url_with_config(config: &ProviderConfig) -> String` that appends `/v1/chat/completions`
  - **Option B:** Change signature to `api_url(&self, configs: &HashMap<String, ProviderConfig>) -> String` (breaks existing callers)
- `keychain_account()` -- return a sentinel/empty string for local (never used)
- `adapter_kind()` -- return `AdapterKind::OpenAICompat` for both Ollama and LMStudio
- `default_timeout_secs()` -- return 60 for local providers (model loading latency)
- `console_url()` -- return "ollama.com" / "lmstudio.ai"

#### 2. `ProviderConfig` -- New Configuration Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub base_url: String,
}

impl ProviderConfig {
    pub fn chat_completions_url(&self) -> String {
        format!("{}/v1/chat/completions", self.base_url.trim_end_matches('/'))
    }
}
```

Stored in `settings.json` as:
```json
{
  "providerConfigs": {
    "ollama": { "base_url": "http://localhost:11434" },
    "lmstudio": { "base_url": "http://localhost:1234" }
  }
}
```

Default values loaded on first use.

#### 3. `state::AppState` -- Config Storage

```rust
pub struct AppState {
    // ... existing fields ...
    /// Per-provider configuration (base URLs for local providers).
    /// Loaded from settings.json on startup, updated via IPC.
    pub provider_configs: Mutex<HashMap<String, ProviderConfig>>,
}
```

#### 4. `ai::stream_ai_response()` -- Keyless Dispatch

The main change is conditional keychain lookup:

```rust
// For local providers: skip keychain, use empty API key
let api_key = if provider.is_local() {
    String::new()
} else {
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keyring error: {}", e))?;
    entry.get_password().map_err(|_| {
        format!("No {} API key configured.", provider.display_name())
    })?
};
```

And dynamic URL resolution:

```rust
// Get API URL: static for cloud, config-based for local
let api_url = if provider.is_local() {
    let configs = state.provider_configs.lock().unwrap();
    let config = configs.get(provider.config_key())
        .cloned()
        .unwrap_or_else(|| ProviderConfig {
            base_url: provider.default_base_url().to_string(),
        });
    config.chat_completions_url()
} else {
    provider.api_url().to_string()
};
```

#### 5. `openai_compat::stream()` -- Accept Dynamic URL

Change the function signature to accept an explicit URL instead of calling `provider.api_url()`:

```rust
pub async fn stream(
    provider: &Provider,
    api_url: &str,      // NEW: explicit URL instead of provider.api_url()
    api_key: &str,
    model: &str,
    messages: Vec<serde_json::Value>,
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<TokenUsage, String> {
    // ... existing code, but use api_url parameter instead of provider.api_url()
    let mut request = client
        .post(api_url)  // Use parameter
        .header("Content-Type", "application/json");

    // Only add auth header if key is non-empty
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    // OpenRouter-specific headers (unchanged)
    if *provider == Provider::OpenRouter {
        request = request
            .header("HTTP-Referer", "https://cmdkapp.com")
            .header("X-Title", "CMD+K");
    }
    // ... rest unchanged
}
```

This also requires updating the three call sites in `ai.rs` (OpenAICompat, Anthropic, Gemini branches).

#### 6. `models::fetch_models()` -- Local Model Discovery

New branches in `fetch_api_models()`:

```rust
Provider::Ollama => {
    let configs = state.provider_configs.lock().unwrap();
    let base_url = configs.get("ollama")
        .map(|c| c.base_url.clone())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    drop(configs);

    let resp = client
        .get(format!("{}/api/tags", base_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| "Cannot reach Ollama. Is it running?".to_string())?;

    let parsed: OllamaTagsResponse =
        resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed.models.into_iter().map(|m| {
        let label = format_ollama_label(&m.name, &m.details);
        ModelWithMeta {
            id: m.name,
            label,
            tier: String::new(),
            input_price_per_m: None,
            output_price_per_m: None,
        }
    }).collect())
}

Provider::LMStudio => {
    // LM Studio uses standard OpenAI /v1/models format
    let configs = state.provider_configs.lock().unwrap();
    let base_url = configs.get("lmstudio")
        .map(|c| c.base_url.clone())
        .unwrap_or_else(|| "http://localhost:1234".to_string());
    drop(configs);

    let resp = client
        .get(format!("{}/v1/models", base_url))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|_| "Cannot reach LM Studio. Is the server running?".to_string())?;

    let parsed: OpenAIModelsResponse =
        resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed.data.into_iter().map(|m| ModelWithMeta {
        label: m.id.clone(),
        id: m.id,
        tier: String::new(),
        input_price_per_m: None,
        output_price_per_m: None,
    }).collect())
}
```

New response types for Ollama:

```rust
#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
    details: OllamaModelDetails,
}

#[derive(Deserialize)]
struct OllamaModelDetails {
    parameter_size: Option<String>,
    family: Option<String>,
    quantization_level: Option<String>,
}
```

#### 7. `models::validate_api_key()` -- Health Check for Local

Local providers replace API key validation with a health/connectivity check. The `validate_api_key` function needs a `state` parameter added to access provider configs:

```rust
Provider::Ollama => {
    let configs = state.provider_configs.lock().unwrap();
    let base_url = configs.get("ollama")
        .map(|c| c.base_url.clone())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    drop(configs);

    let resp = client
        .get(&base_url)
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map_err(|_| "connection_failed".to_string())?;

    match resp.status().as_u16() {
        200 => Ok(()),
        _ => Err("connection_failed".to_string()),
    }
}

Provider::LMStudio => {
    let configs = state.provider_configs.lock().unwrap();
    let base_url = configs.get("lmstudio")
        .map(|c| c.base_url.clone())
        .unwrap_or_else(|| "http://localhost:1234".to_string());
    drop(configs);

    let resp = client
        .get(format!("{}/v1/models", base_url))
        .timeout(std::time::Duration::from_secs(3))
        .send()
        .await
        .map_err(|_| "connection_failed".to_string())?;

    match resp.status().as_u16() {
        200 => Ok(()),
        _ => Err("connection_failed".to_string()),
    }
}
```

#### 8. New IPC Commands

```rust
/// Check if a local provider is reachable.
#[tauri::command]
pub async fn check_local_provider(
    provider: Provider,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<LocalProviderStatus, String>

/// Update base URL for a provider.
#[tauri::command]
pub async fn set_provider_config(
    provider: Provider,
    base_url: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<(), String>

/// Get current config for a provider.
#[tauri::command]
pub fn get_provider_config(
    provider: Provider,
    state: tauri::State<'_, crate::state::AppState>,
) -> ProviderConfig
```

### Frontend (TypeScript/React)

#### 1. `store/index.ts` -- PROVIDERS Array

```typescript
export const PROVIDERS = [
  { id: "openai", name: "OpenAI", local: false },
  { id: "anthropic", name: "Anthropic", local: false },
  { id: "gemini", name: "Google Gemini", local: false },
  { id: "xai", name: "xAI", local: false },
  { id: "openrouter", name: "OpenRouter", local: false },
  { id: "ollama", name: "Ollama", local: true },
  { id: "lmstudio", name: "LM Studio", local: true },
] as const;
```

#### 2. `store/index.ts` -- submitQuery Changes

The `submitQuery` function currently checks `apiKeyStatus !== "valid"` before submitting. For local providers, the validation check should be against connection health rather than API key:

```typescript
// Current: blocks on missing API key
if (currentState.apiKeyStatus !== "valid") {
  set({ streamError: "No API key configured..." });
  return;
}

// New: local providers use connection check, not API key
const provider = PROVIDERS.find(p => p.id === currentState.selectedProvider);
if (provider?.local) {
  if (currentState.localProviderStatus !== "connected") {
    set({ streamError: "Cannot reach local server. Is it running?" });
    return;
  }
} else {
  if (currentState.apiKeyStatus !== "valid") {
    set({ streamError: "No API key configured..." });
    return;
  }
}
```

New store fields:
```typescript
localProviderStatus: "unknown" | "checking" | "connected" | "disconnected";
localProviderBaseUrl: string;
setLocalProviderStatus: (status) => void;
setLocalProviderBaseUrl: (url: string) => void;
```

#### 3. `AccountTab.tsx` -- Conditional UI

When a local provider is selected, replace the API key input with:
- Base URL input (pre-filled with default, editable)
- Connection status indicator (green/red dot)
- "Test Connection" button
- Model count badge when connected

#### 4. `StepApiKey.tsx` -- Onboarding Adaptation

When a local provider is selected during onboarding:
- Show base URL field instead of API key field
- Show connection test instead of key validation
- Auto-proceed if health check succeeds

#### 5. `ProviderIcon.tsx` -- New Icons

Add SVG path data for Ollama and LM Studio icons.

## Patterns to Follow

### Pattern 1: Provider.is_local() Guard
**What:** All provider-specific branching should use `provider.is_local()` rather than matching individual variants.
**When:** Any code path that differs between cloud and local providers.
**Why:** If more local providers are added later (e.g., llama.cpp server, vLLM), the guards automatically include them.
**Example:**
```rust
let api_key = if provider.is_local() {
    String::new()
} else {
    keychain_lookup(provider)?
};
```

### Pattern 2: Config Defaults with Override
**What:** ProviderConfig provides defaults that the user can override. Never require configuration before first use.
**When:** Any access to provider base URL.
**Why:** Zero-friction first run. If Ollama is on default port, it should "just work" on provider selection.
**Example:**
```rust
let config = configs.get(provider.config_key())
    .cloned()
    .unwrap_or_else(|| ProviderConfig {
        base_url: provider.default_base_url().to_string(),
    });
```

### Pattern 3: Graceful Degradation on Connection Failure
**What:** Connection failures show clear status, never crash. Model list returns empty vec, not error.
**When:** Any local provider HTTP call.
**Why:** Local servers are expected to be offline sometimes. The UI should guide the user to start them.
**Example:**
```rust
let models = fetch_api_models(&provider, &api_key, &state)
    .await
    .unwrap_or_default();  // Already exists in codebase
```

### Pattern 4: Reuse validate_api_key for Local Connection Check
**What:** Rather than creating a separate health check flow, local providers treat `validate_api_key` as a connection test. The frontend already calls this on provider selection.
**When:** User selects Ollama or LM Studio in the provider dropdown.
**Why:** Minimizes frontend changes. The existing flow (select provider -> validate -> fetch models -> show models) maps cleanly to (select provider -> check connection -> fetch models -> show models).

## Anti-Patterns to Avoid

### Anti-Pattern 1: New Streaming Adapter
**What:** Creating a separate `ollama.rs` or `lmstudio.rs` streaming adapter.
**Why bad:** Both use identical OpenAI-compat SSE format. Would duplicate ~100 lines of `openai_compat.rs`.
**Instead:** Reuse `openai_compat::stream()` with dynamic URL parameter.

### Anti-Pattern 2: Storing Base URL in Keychain
**What:** Using the existing keychain infrastructure to store base URLs.
**Why bad:** Keychain is for secrets. Base URLs are not secrets and should be visible/editable.
**Instead:** Store in `settings.json` via Tauri plugin-store, same as other preferences.

### Anti-Pattern 3: Polling Health Checks
**What:** Setting up a timer to poll local server health every N seconds.
**Why bad:** Unnecessary battery/CPU usage. The app is an overlay, not a monitoring dashboard.
**Instead:** Check health on-demand: (1) when provider is selected, (2) when settings mount, (3) before streaming starts. Frontend can add a manual "refresh" button.

### Anti-Pattern 4: Hardcoding localhost URLs
**What:** Using literal `"http://localhost:11434"` throughout the codebase.
**Why bad:** Users may run Ollama on a different machine (network GPU server) or port.
**Instead:** Always read from ProviderConfig with default fallback.

### Anti-Pattern 5: Curated Model Lists for Local Providers
**What:** Maintaining hardcoded model lists for Ollama/LM Studio like cloud providers.
**Why bad:** Local model availability is entirely user-controlled. A curated list would be perpetually wrong.
**Instead:** 100% dynamic model discovery from local API. No curated entries, no tier tags.

## Scalability Considerations

| Concern | Current (5 cloud) | After (5 cloud + 2 local) |
|---------|--------------------|---------------------------|
| Provider enum size | 5 variants | 7 variants -- negligible |
| Keychain entries | 5 max | Still 5 (local providers skip keychain) |
| Config persistence | settings.json (provider, model) | settings.json adds providerConfigs map |
| Model list memory | ~50 models cached | +0-100 local models (dynamic) |
| Network calls | Cloud APIs only | +localhost HTTP (sub-1ms latency) |
| Token tracking | Works for all 5 | Works for all 7 (no cost for local) |

## Suggested Build Order

Based on dependency analysis of the existing architecture:

### Phase 1: Provider Enum + Config Foundation
1. Add `Ollama` and `LMStudio` to `Provider` enum with all trait methods
2. Add `ProviderConfig` struct with `base_url`
3. Add `provider_configs` to `AppState`
4. Add `is_local()` helper method
5. Load/save configs from `settings.json` on startup
6. Register new IPC commands (`check_local_provider`, `set_provider_config`, `get_provider_config`)

**Why first:** Everything else depends on the Provider enum having the new variants. Until this compiles, nothing else can build.

### Phase 2: Model Discovery
1. Add `OllamaTagsResponse` and related deserialization types
2. Add Ollama branch in `fetch_api_models()` -- `/api/tags` parsing
3. Add LM Studio branch in `fetch_api_models()` -- reuse `OpenAIModelsResponse`
4. Add Ollama/LMStudio branches in `validate_api_key()` (health-check-as-validation)
5. Wire `validate_api_key` to accept `state` parameter (signature change)

**Why second:** Model discovery is needed before streaming (user must select a model).

### Phase 3: Streaming Integration
1. Modify `openai_compat::stream()` to accept explicit URL parameter
2. Update all 3 call sites in `ai::stream_ai_response()` to pass URL
3. Add local provider branch in `stream_ai_response()` (skip keychain, read config URL)
4. Set 60s timeout for local providers (model loading latency)

**Why third:** Streaming depends on Phase 1 (enum) and Phase 2 (model selection).

### Phase 4: Frontend -- Store + Provider List
1. Add Ollama/LM Studio to `PROVIDERS` array with `local: true`
2. Add `localProviderStatus`, `localProviderBaseUrl` to Zustand store
3. Update `submitQuery()` to handle keyless providers
4. Add health check invocation on local provider selection

**Why fourth:** Frontend changes depend on all backend IPC commands being available.

### Phase 5: Frontend -- Settings + Onboarding UI
1. Add Ollama/LM Studio SVG icons to `ProviderIcon.tsx`
2. Modify `AccountTab.tsx` for conditional URL config vs. API key input
3. Add connection status indicator (green/red dot)
4. Modify `StepApiKey.tsx` for local provider onboarding flow
5. Modify `StepProviderSelect.tsx` to visually distinguish local vs. cloud providers

**Why last:** UI polish depends on all backend integration being complete and testable.

## Sources

- Ollama OpenAI Compatibility docs: https://docs.ollama.com/api/openai-compatibility (HIGH confidence)
- Ollama /api/tags endpoint docs: https://docs.ollama.com/api/tags (HIGH confidence)
- Ollama health check (root GET): https://github.com/ollama/ollama/issues/1378 (HIGH confidence)
- Ollama streaming usage: https://docs.ollama.com/api/usage (HIGH confidence)
- LM Studio OpenAI Compatibility docs: https://lmstudio.ai/docs/developer/openai-compat (HIGH confidence)
- LM Studio models listing: https://lmstudio.ai/docs/developer/openai-compat/models (HIGH confidence)
- LM Studio REST API v0: https://lmstudio.ai/docs/developer/rest/endpoints (MEDIUM confidence)
- Existing codebase: Direct analysis of Provider enum, openai_compat adapter, models.rs, ai.rs, state.rs, store/index.ts, AccountTab.tsx, StepApiKey.tsx (HIGH confidence)
