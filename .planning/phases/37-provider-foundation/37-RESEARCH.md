# Phase 37: Provider Foundation - Research

**Researched:** 2026-03-17
**Domain:** Extending Rust Provider enum + frontend for Ollama/LM Studio: keyless auth, configurable URLs, health checks, connection status
**Confidence:** HIGH

## Summary

Phase 37 adds Ollama and LM Studio as new Provider enum variants with three fundamental differences from existing cloud providers: (1) no API key required, (2) configurable base URLs instead of hardcoded endpoints, and (3) connection health checks that replace API key validation. The existing `openai_compat::stream()` adapter, SSE parsing, and token usage extraction work identically for local providers. The work is structural: adding match arms to 7 existing dispatch points in the Provider enum, bypassing the keychain in `stream_ai_response`, introducing a health check IPC command, and surfacing connection status through the same checkmark indicator used for API key validation.

The codebase has been analyzed in detail. There are exactly 7 match blocks in `providers/mod.rs` that need new arms, 1 unconditional keychain read in `ai.rs` (lines 206-213) that must become conditional, 1 hardcoded `provider.api_url()` call in `openai_compat.rs` (line 32) that must accept dynamic URLs, and 1 frontend gate in `store/index.ts` (line 446) that blocks queries on `apiKeyStatus !== "valid"`. No new Rust crates are required. The existing `tauri-plugin-store` (already used for hotkey config and provider selection) stores base URLs. The existing `tauri-plugin-http`/`reqwest` client handles all local HTTP calls.

**Primary recommendation:** Extend Provider enum with `is_local()` and `requires_api_key()` helper methods; repurpose `validate_api_key` as health-check-as-validation for local providers (returns Ok when server reachable, which maps cleanly to the existing `apiKeyStatus` flow on the frontend).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Store base URLs in Tauri preferences store (not keychain -- URLs are not secrets)
- Show default URL as placeholder text (localhost:11434 for Ollama, localhost:1234 for LM Studio) -- input is empty unless user customizes
- Accept both `host:port` and `http://host:port` formats -- app normalizes internally (prepend http:// if no protocol)
- Check server health on provider selection in settings AND on overlay open
- 2-second timeout for health check -- localhost responds in <100ms if running
- If server unreachable when overlay opens, allow typing anyway -- error on submit, not disabled input
- Health result surfaces as the same checkmark indicator used for API key validation
- 3 distinct error states: "Server not running" / "No models loaded" / "Request failed -- [details]"
- No start hints in error messages (no "run ollama serve") -- just status
- Error messages use same styling as existing cloud provider errors
- Mixed alphabetically with cloud providers -- no "Local" group header
- No visual distinction (no "(Local)" suffix) -- treat equally, icons are the differentiator

### Claude's Discretion
- Exact Provider enum method implementations (keychain_account returning a no-op or sentinel)
- How to handle api_url() returning dynamic URLs (currently &'static str -- needs refactor for configurable base URLs)
- Whether to add is_local() / requires_api_key() helper methods on Provider
- Internal architecture for health check IPC command

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LPROV-01 | User can select Ollama as an AI provider in settings and onboarding | Provider enum extension, PROVIDERS array extension, ProviderIcon SVG, StepProviderSelect rendering |
| LPROV-02 | User can select LM Studio as an AI provider in settings and onboarding | Same as LPROV-01 for LMStudio variant |
| LPROV-03 | Ollama and LM Studio require no API key -- keyless auth bypass in backend and frontend | `requires_api_key()` helper, conditional keychain bypass in ai.rs, conditional auth header in openai_compat.rs, apiKeyStatus mapping for local providers |
| LPROV-04 | User can configure base URL for each local provider (defaults: localhost:11434 for Ollama, localhost:1234 for LM Studio) | `tauri-plugin-store` settings.json storage, `default_base_url()` method, URL normalization, AccountTab conditional UI |
| LPROV-05 | App checks local provider health (Ollama GET /, LM Studio GET /v1/models) and surfaces connection status | Health check via `validate_api_key` repurposing, 2-second timeout, reqwest error kind matching, checkmark indicator reuse |
| LPROV-06 | Provider-specific error messages differentiate "server not running" from "model not loaded" from network errors | `reqwest::Error::is_connect()` matching, `handle_http_status` extension for local providers, 3-state error mapping |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri-plugin-store | 2.x (already in Cargo.toml) | Persist base URLs in settings.json | Already used for hotkey config, provider selection, onboarding state |
| tauri-plugin-http (reqwest) | 2.x (already in Cargo.toml) | HTTP client for health checks and streaming | Already powers all cloud provider API calls |
| eventsource-stream | 0.2 (already in Cargo.toml) | SSE parsing for streaming responses | Already used by openai_compat adapter |
| tokio | 1.x (already in Cargo.toml) | Async runtime + timeout wrapper | Already wraps streaming in timeout |
| @tauri-apps/plugin-store | (already in package.json) | Frontend settings store access | Already used in AccountTab, OnboardingWizard, StepProviderSelect |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde/serde_json | 1.x (already in Cargo.toml) | JSON serialization for health check responses | Already used everywhere |
| keyring | 3.x (already in Cargo.toml) | API key storage (skipped for local providers) | Only for cloud providers -- local providers bypass |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tauri-plugin-store for URLs | AppState in-memory only | Would lose persistence across app restarts -- unacceptable per LPROV-04 |
| Separate health.rs module | Health check in models.rs | Either works; keeping in models.rs alongside validate_api_key minimizes file count |
| New HealthStatus struct return | Reusing Ok/Err from validate_api_key | Reusing validate_api_key maps directly to existing apiKeyStatus flow -- zero frontend state refactoring |

**No new dependencies. Zero Cargo.toml or package.json changes needed.**

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/commands/
  providers/
    mod.rs              # Provider enum + 2 new variants, 3 new helper methods
    openai_compat.rs    # Accept dynamic URL param, conditional auth header
  ai.rs                 # Conditional keychain bypass, dynamic URL resolution
  models.rs             # validate_api_key health-check branches, curated_models empty vec
  keychain.rs           # No changes (local providers skip keychain entirely)
src/
  store/index.ts        # PROVIDERS array + local flag, submitQuery bypass
  components/
    Settings/AccountTab.tsx  # Conditional URL input vs API key input
    icons/ProviderIcon.tsx   # 2 new SVG icon entries
    Onboarding/
      StepProviderSelect.tsx # Renders from PROVIDERS (automatic)
      StepApiKey.tsx         # No changes this phase (onboarding adaptation is Phase 40)
```

### Pattern 1: Validate-as-Health-Check
**What:** Local providers repurpose the existing `validate_api_key` IPC command to perform a health check instead of key validation. The frontend already calls this on provider selection change and maps the result to `apiKeyStatus`.
**When to use:** Always for LPROV-05 -- reuse existing flow.
**Why:** Zero new frontend state fields needed. The existing flow (select provider -> validate -> checkmark or X) maps directly to (select provider -> health check -> checkmark or X). The backend returns `Ok(())` when server reachable, `Err("connection_failed")` when not.
**Example:**
```rust
// In validate_api_key match block
Provider::Ollama => {
    let base_url = get_base_url_from_store(app_handle, "ollama")?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&base_url).send().await;
    match resp {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(_) => Err("Server not running".to_string()),
        Err(e) if e.is_connect() => Err("Server not running".to_string()),
        Err(e) => Err(format!("Request failed -- {}", e)),
    }
}
```

### Pattern 2: Provider.is_local() Guard
**What:** All code paths that differ between cloud and local providers branch on `provider.is_local()` rather than matching individual Ollama/LMStudio variants.
**When to use:** Every conditional: keychain bypass, URL resolution, auth header, error messages.
**Why:** Future-proofs for additional local providers (llama.cpp server, vLLM, text-generation-webui). Adding a new local provider only requires adding to the `is_local()` match.
**Example:**
```rust
impl Provider {
    pub fn is_local(&self) -> bool {
        matches!(self, Provider::Ollama | Provider::LMStudio)
    }
    pub fn requires_api_key(&self) -> bool {
        !self.is_local()
    }
}
```

### Pattern 3: Dynamic URL with Static Fallback
**What:** `api_url()` stays `&'static str` for cloud providers. Local providers read from settings store at call time via a separate `get_base_url()` function.
**When to use:** In `stream_ai_response` and `validate_api_key` when building the request URL.
**Why:** Avoids changing the `api_url()` return type (which would break all 5 existing cloud provider callers). The URL resolution happens once at the top of each command, not inside the streaming adapter.
**Example:**
```rust
// In stream_ai_response
let (api_url, api_key) = if provider.is_local() {
    let base = get_provider_base_url(&app_handle, &provider)?;
    let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));
    (url, String::new())
} else {
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keyring error: {}", e))?;
    let key = entry.get_password().map_err(|_| {
        format!("No {} API key configured.", provider.display_name())
    })?;
    (provider.api_url().to_string(), key)
};
```

### Pattern 4: URL Normalization
**What:** Accept both `host:port` and `http://host:port` from user input, normalize to always include protocol.
**When to use:** When saving base URL to settings store.
**Why:** Users will type `localhost:11434` without thinking about protocol. Reqwest requires a protocol prefix.
**Example:**
```rust
fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}
```

### Anti-Patterns to Avoid
- **Storing base URLs in keychain:** URLs are not secrets. Use settings.json via tauri-plugin-store.
- **Creating separate streaming adapters:** Both Ollama and LM Studio use identical OpenAI-compat SSE format. Reuse `openai_compat::stream()`.
- **Polling health checks on a timer:** Check on-demand (provider selection, overlay open). No background polling.
- **Hardcoding localhost throughout:** Always read from settings store with default fallback.
- **Changing api_url() return type to String:** This breaks all cloud provider callers. Keep static for cloud, separate function for local.
- **Adding curated model lists for local providers:** Local models are 100% dynamic. Return empty vec from `curated_models()`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP client for health checks | Custom TCP socket check | `reqwest::Client` with 2-second timeout | Already available, handles connection refused properly |
| Settings persistence | Custom JSON file read/write | `tauri-plugin-store` (settings.json) | Already initialized, used throughout app, handles atomic writes |
| SSE streaming for local providers | New adapter or NDJSON parser | Existing `openai_compat::stream()` | Both providers use identical OpenAI-compat SSE format |
| URL validation | Regex-based URL parser | `url::Url::parse()` or simple prefix check | reqwest will catch invalid URLs on request; normalize protocol prefix only |
| Provider selection UI | New component for local providers | Existing PROVIDERS array + AccountTab | Adding entries to PROVIDERS array auto-populates all dropdowns |

**Key insight:** This phase is about extending 7 existing match arms and adding 3 conditional branches (keychain, URL, auth header). No new abstractions needed.

## Common Pitfalls

### Pitfall 1: API Key Gate Blocks Local Providers
**What goes wrong:** `stream_ai_response` unconditionally reads API key from keychain (ai.rs lines 206-213). Fails with "No API key configured" for Ollama/LM Studio. Frontend `submitQuery` gates on `apiKeyStatus !== "valid"` (store/index.ts line 446), blocking all queries.
**Why it happens:** Entire auth flow assumes every provider has an API key.
**How to avoid:** Add `if provider.is_local() { String::new() } else { keychain_lookup() }` in ai.rs. On frontend, when local provider is selected and health check succeeds, set `apiKeyStatus` to "valid" to satisfy the existing gate.
**Warning signs:** "No API key configured" error for local providers. Users stuck on onboarding step 1.

### Pitfall 2: Hardcoded provider.api_url() Returns &'static str
**What goes wrong:** `openai_compat::stream()` calls `provider.api_url()` on line 32, which returns a compile-time string. Local provider URLs are user-configurable and not known at compile time.
**Why it happens:** Cloud providers have permanent, well-known endpoints. The return type was chosen for zero-allocation performance.
**How to avoid:** Change `openai_compat::stream()` signature to accept `api_url: &str` as an explicit parameter. Resolve URL before calling the adapter, not inside it. Update all 3 call sites in ai.rs.
**Warning signs:** Compilation error if attempting to return a String from api_url().

### Pitfall 3: Connection Refused Misdiagnosed as Auth Error
**What goes wrong:** `handle_http_status` (mod.rs lines 91-109) handles 401 as "Authentication failed. Check your API key." Connection refused errors from reqwest never reach `handle_http_status` -- they are caught by `.send().await.map_err()` which produces "Network error."
**Why it happens:** Cloud providers never produce connection refused. Error handling was designed for cloud failure modes.
**How to avoid:** For local providers, match on `reqwest::Error` kind before `handle_http_status`. If `error.is_connect()`, return "Server not running". This implements the 3-state error requirement (LPROV-06).
**Warning signs:** "Network error: Check your internet connection" when Ollama is simply not started.

### Pitfall 4: validate_api_key Needs AppHandle for Store Access
**What goes wrong:** The current `validate_api_key` signature is `(provider: Provider, api_key: String) -> Result<(), String>`. Local providers need to read base URL from settings store, which requires `tauri::AppHandle` or `tauri::State<AppState>`.
**Why it happens:** Cloud providers validate with a static URL + the provided key. No state access needed.
**How to avoid:** Add `app_handle: tauri::AppHandle` parameter to `validate_api_key`. Update the frontend `invoke` call to not pass `apiKey` for local providers (or pass empty string). The `app_handle` is automatically injected by Tauri for `#[tauri::command]` functions.
**Warning signs:** Compilation error about missing state parameter. Cannot read settings store inside validate_api_key.

### Pitfall 5: Provider Enum Exhaustive Matches
**What goes wrong:** Adding `Ollama` and `LMStudio` to the Provider enum without updating ALL match arms causes compilation errors in `keychain_account()`, `api_url()`, `default_timeout_secs()`, `display_name()`, `console_url()`, `adapter_kind()`, and `handle_http_status()`.
**Why it happens:** Rust enforces exhaustive matching. Every `match provider` block must handle the new variants.
**How to avoid:** Update all 7 match blocks in mod.rs before any other changes. Use `cargo check` early to catch missing arms. The compiler will list every location.
**Warning signs:** Dozens of "non-exhaustive patterns" compilation errors.

### Pitfall 6: AccountTab Calls get_api_key for Local Providers
**What goes wrong:** `AccountTab.tsx` useEffect (line 31-60) calls `invoke("get_api_key", { provider })` on mount and provider change. For local providers, this calls `keyring::Entry::new()` with a meaningless keychain account, which may error on some platforms.
**Why it happens:** The checkStoredKey flow assumes every provider has a keychain entry.
**How to avoid:** Guard the keychain check with the `local` flag from PROVIDERS. For local providers, skip keychain and instead call `validate_api_key` (which performs a health check for local).
**Warning signs:** Keychain errors in console for Ollama/LM Studio. `apiKeyStatus` stuck at "unknown".

### Pitfall 7: openai_compat.rs Always Sends Authorization Header
**What goes wrong:** Line 33 of `openai_compat.rs` unconditionally adds `.header("Authorization", format!("Bearer {}", api_key))`. For local providers, this sends `Bearer ` (empty string) which both Ollama and LM Studio ignore, but it is unnecessary noise.
**Why it happens:** All current providers require auth headers.
**How to avoid:** Conditionally add auth header only when `api_key` is non-empty: `if !api_key.is_empty() { request = request.header("Authorization", ...); }`. This is a clean no-op change that works for both cloud and local providers.
**Warning signs:** Not a runtime bug (servers ignore empty bearer), but clutters request logs.

## Code Examples

### Provider Enum Extension
```rust
// Source: Direct analysis of src-tauri/src/commands/providers/mod.rs

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
    #[serde(rename = "lmstudio")]
    LMStudio,     // NEW
}

impl Provider {
    /// Whether this provider runs locally (no API key, configurable URL).
    pub fn is_local(&self) -> bool {
        matches!(self, Provider::Ollama | Provider::LMStudio)
    }

    /// Whether this provider requires an API key for authentication.
    pub fn requires_api_key(&self) -> bool {
        !self.is_local()
    }

    /// Default base URL for local providers. Empty for cloud providers.
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::Ollama => "http://localhost:11434",
            Provider::LMStudio => "http://localhost:1234",
            _ => "",
        }
    }

    /// Settings store key for this provider's base URL.
    pub fn base_url_store_key(&self) -> &'static str {
        match self {
            Provider::Ollama => "ollama_base_url",
            Provider::LMStudio => "lmstudio_base_url",
            _ => "",
        }
    }

    // Existing methods extended with new match arms:

    pub fn keychain_account(&self) -> &'static str {
        match self {
            // ... existing 5 arms ...
            Provider::Ollama | Provider::LMStudio => "", // Never used -- is_local() guards
        }
    }

    pub fn api_url(&self) -> &'static str {
        match self {
            // ... existing 5 arms unchanged ...
            // Local providers use dynamic URLs resolved at call time
            Provider::Ollama => "http://localhost:11434/v1/chat/completions",
            Provider::LMStudio => "http://localhost:1234/v1/chat/completions",
        }
    }

    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            Provider::XAI => 10,
            Provider::Ollama | Provider::LMStudio => 120, // Model cold-start
            _ => 30,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            // ... existing 5 arms ...
            Provider::Ollama => "Ollama",
            Provider::LMStudio => "LM Studio",
        }
    }

    pub fn console_url(&self) -> &'static str {
        match self {
            // ... existing 5 arms ...
            Provider::Ollama => "ollama.com",
            Provider::LMStudio => "lmstudio.ai",
        }
    }

    pub fn adapter_kind(&self) -> AdapterKind {
        match self {
            Provider::OpenAI | Provider::XAI | Provider::OpenRouter
                | Provider::Ollama | Provider::LMStudio => AdapterKind::OpenAICompat,
            Provider::Anthropic => AdapterKind::Anthropic,
            Provider::Gemini => AdapterKind::Gemini,
        }
    }
}
```

### Modified openai_compat::stream() Signature
```rust
// Source: Direct analysis of src-tauri/src/commands/providers/openai_compat.rs

pub async fn stream(
    provider: &Provider,
    api_url: &str,       // NEW: explicit URL replaces provider.api_url()
    api_key: &str,
    model: &str,
    messages: Vec<serde_json::Value>,
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<TokenUsage, String> {
    // ... body unchanged except:
    let mut request = client
        .post(api_url)  // Was: provider.api_url()
        .header("Content-Type", "application/json");

    // Conditionally add auth header (local providers have empty api_key)
    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }
    // ... rest unchanged
}
```

### Conditional Keychain Bypass in stream_ai_response
```rust
// Source: Direct analysis of src-tauri/src/commands/ai.rs lines 206-213

// Resolve API URL and key based on provider type
let (api_url, api_key) = if provider.is_local() {
    let base = get_provider_base_url(&app_handle, &provider);
    let url = format!("{}/v1/chat/completions", base.trim_end_matches('/'));
    (url, String::new())
} else {
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keyring error: {}", e))?;
    let key = entry.get_password().map_err(|_| {
        format!("No {} API key configured. Open Settings to add one.", provider.display_name())
    })?;
    (provider.api_url().to_string(), key)
};
```

### Health Check via validate_api_key
```rust
// Source: Direct analysis of src-tauri/src/commands/models.rs

// In validate_api_key match block:
Provider::Ollama => {
    let base_url = get_provider_base_url(&app_handle, &provider);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    match client.get(&base_url).send().await {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(_) => Err("Server not running".to_string()),
        Err(e) if e.is_connect() => Err("Server not running".to_string()),
        Err(e) if e.is_timeout() => Err("Server not running".to_string()),
        Err(e) => Err(format!("Request failed -- {}", e)),
    }
}

Provider::LMStudio => {
    let base_url = get_provider_base_url(&app_handle, &provider);
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;
    match client.get(format!("{}/v1/models", base_url)).send().await {
        Ok(r) if r.status().is_success() => Ok(()),
        Ok(_) => Err("Server not running".to_string()),
        Err(e) if e.is_connect() => Err("Server not running".to_string()),
        Err(e) if e.is_timeout() => Err("Server not running".to_string()),
        Err(e) => Err(format!("Request failed -- {}", e)),
    }
}
```

### Reading Base URL from Store
```rust
// Source: Pattern from src-tauri/src/commands/updater.rs lines 37-43

/// Read the configured base URL for a local provider from settings.json.
/// Falls back to the provider's default base URL if not configured.
fn get_provider_base_url(app_handle: &tauri::AppHandle, provider: &Provider) -> String {
    use tauri_plugin_store::StoreExt;
    let key = provider.base_url_store_key();
    app_handle
        .store("settings.json")
        .ok()
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| provider.default_base_url().to_string())
}
```

### Frontend PROVIDERS Array
```typescript
// Source: Direct analysis of src/store/index.ts

export const PROVIDERS = [
  { id: "anthropic", name: "Anthropic", local: false },
  { id: "gemini", name: "Google Gemini", local: false },
  { id: "lmstudio", name: "LM Studio", local: true },
  { id: "ollama", name: "Ollama", local: true },
  { id: "openai", name: "OpenAI", local: false },
  { id: "openrouter", name: "OpenRouter", local: false },
  { id: "xai", name: "xAI", local: false },
] as const;
// Note: Alphabetical per user decision (mixed, no grouping).
```

### AccountTab Conditional Rendering
```typescript
// Source: Direct analysis of src/components/Settings/AccountTab.tsx

// Inside AccountTab, after provider dropdown:
const currentProvider = PROVIDERS.find(p => p.id === selectedProvider);

{currentProvider?.local ? (
  // Local provider: show base URL input + connection status
  <div className="flex flex-col gap-1.5">
    <p className="text-white/40 text-xs uppercase tracking-wider">Server URL</p>
    <div className="flex items-center gap-2">
      <input
        type="text"
        value={baseUrlInput}
        onChange={(e) => setBaseUrlInput(e.target.value)}
        placeholder={selectedProvider === "ollama" ? "localhost:11434" : "localhost:1234"}
        className="..." // Same styling as API key input
      />
      {/* Status indicator -- same Check/X/Loader as API key validation */}
    </div>
  </div>
) : (
  // Cloud provider: show API key input (existing code)
  <div>...</div>
)}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Every provider has a keychain entry | `is_local()` guard skips keychain | This phase | Local providers bypass entire keychain flow |
| `api_url()` returns `&'static str` | Stream adapter accepts explicit URL param | This phase | Enables configurable base URLs without breaking existing callers |
| `validate_api_key` only validates keys | Repurposed as health check for local providers | This phase | Frontend flow unchanged -- just maps health to "valid"/"invalid" |
| `openai_compat::stream` always sends auth | Conditionally omits Authorization header | This phase | Clean request to local servers with no extraneous headers |

**Not deprecated but extended:**
- Provider enum: gains 2 variants, 3 new methods. All existing variants unchanged.
- PROVIDERS array: gains 2 entries with `local: true` flag. All existing entries unchanged.
- AppState: no changes this phase (base URLs read from store, not cached in state).

## Open Questions

1. **AppHandle access in validate_api_key**
   - What we know: `validate_api_key` currently has signature `(provider: Provider, api_key: String)`. Local providers need to read base URL from settings store, which requires `AppHandle`.
   - What's unclear: Whether to add `app_handle: tauri::AppHandle` (auto-injected by Tauri) or `state: tauri::State<'_, AppState>` (storing URLs in AppState).
   - Recommendation: Use `app_handle: tauri::AppHandle`. Tauri auto-injects it for `#[tauri::command]` functions. Reading directly from store avoids adding new AppState fields. The `updater.rs` pattern (line 37-43) already demonstrates this approach. Note: existing `fetch_models` already has `state: tauri::State<'_, AppState>` so either approach works.

2. **LMStudio serde rename**
   - What we know: `Provider::LMStudio` needs a serde rename for JSON deserialization from the frontend. The frontend sends `"lmstudio"` (lowercase) which serde_rename_all("lowercase") would serialize as `"lmstudio"`. The two-word "LM Studio" display name is handled by `display_name()`, not serde.
   - What's unclear: Whether `#[serde(rename_all = "lowercase")]` handles `LMStudio` -> `"lmstudio"` correctly.
   - Recommendation: Add explicit `#[serde(rename = "lmstudio")]` on the variant to be safe, matching the pattern used for `#[serde(rename = "xai")]` on XAI.

3. **"No models loaded" error state detection**
   - What we know: LPROV-06 requires distinguishing "server not running" from "no models loaded". For Ollama, `GET /` returns 200 even with no models. For LM Studio, `GET /v1/models` returns 200 with empty `data: []` when no models are loaded.
   - What's unclear: Whether "no models loaded" is detectable purely from the health check response, or requires a separate models fetch.
   - Recommendation: For this phase, health check covers "server running" vs "server not running". The "no models loaded" state is detectable when the overlay opens and `fetch_models` returns an empty list. Surface this as a message in the model dropdown, not in the health check. Full 3-state error handling happens at query time when `stream_ai_response` gets a model-not-found error from the provider.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust unit tests) |
| Config file | Cargo.toml (existing) |
| Quick run command | `cargo check --lib` |
| Full suite command | `cargo test --lib` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LPROV-01 | Ollama variant in Provider enum with all method implementations | unit | `cargo test --lib` | No -- Wave 0 |
| LPROV-02 | LMStudio variant in Provider enum with all method implementations | unit | `cargo test --lib` | No -- Wave 0 |
| LPROV-03 | `is_local()` returns true, `requires_api_key()` returns false for Ollama/LMStudio | unit | `cargo test --lib` | No -- Wave 0 |
| LPROV-04 | Base URL default values, URL normalization, store key correctness | unit | `cargo test --lib` | No -- Wave 0 |
| LPROV-05 | Health check endpoint paths per provider | unit | `cargo test --lib` | No -- Wave 0 |
| LPROV-06 | Error message mapping (connect error -> "Server not running") | unit | `cargo test --lib` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo check --lib` (Linux compilation)
- **Per wave merge:** `cargo test --lib` (all unit tests)
- **Phase gate:** `cargo test --lib` green + manual verification of provider selection in running app

### Wave 0 Gaps
- [ ] Provider enum unit tests for `is_local()`, `requires_api_key()`, `default_base_url()`, `base_url_store_key()` in `providers/mod.rs`
- [ ] URL normalization unit tests for `normalize_base_url()` function
- [ ] `display_name()`, `console_url()`, `adapter_kind()`, `default_timeout_secs()` correctness for new variants

## Surgical Change List

### Files to Modify

| File | Change | Scope |
|------|--------|-------|
| `src-tauri/src/commands/providers/mod.rs` | Add `Ollama`, `LMStudio` variants. Add `is_local()`, `requires_api_key()`, `default_base_url()`, `base_url_store_key()`. Extend all 7 match blocks. | ~60 lines added |
| `src-tauri/src/commands/providers/openai_compat.rs` | Change signature to accept `api_url: &str`. Conditionally add auth header. | ~5 lines changed |
| `src-tauri/src/commands/ai.rs` | Conditional keychain bypass. Dynamic URL resolution. Pass explicit `api_url` to `openai_compat::stream()`. | ~15 lines changed |
| `src-tauri/src/commands/models.rs` | Add `validate_api_key` health-check branches for Ollama/LMStudio. Add `curated_models()` empty vec arms. Add `app_handle` parameter to `validate_api_key`. | ~40 lines added |
| `src-tauri/src/lib.rs` | No new IPC commands needed (validate_api_key already registered). | 0 lines changed |
| `src/store/index.ts` | Extend PROVIDERS with `local` flag. Guard `submitQuery` apiKey check for local. | ~10 lines changed |
| `src/components/Settings/AccountTab.tsx` | Conditional rendering: URL input + connection status for local providers. | ~50 lines changed |
| `src/components/icons/ProviderIcon.tsx` | Add Ollama and LM Studio SVG icon data. | ~20 lines added |

### Files NOT to Change

| File | Why Unchanged |
|------|--------------|
| `src-tauri/src/state.rs` | No new AppState fields. URLs read from store, not cached. |
| `src-tauri/src/commands/keychain.rs` | Local providers skip keychain entirely. Existing code untouched. |
| `src-tauri/src/commands/usage.rs` | `pricing_available: false` path already handles free providers. |
| `src-tauri/src/commands/providers/anthropic.rs` | Unrelated adapter. |
| `src-tauri/src/commands/providers/gemini.rs` | Unrelated adapter. |
| `src/components/Onboarding/StepApiKey.tsx` | Onboarding adaptation is Phase 40 scope. |
| `src/components/Onboarding/StepProviderSelect.tsx` | Renders from PROVIDERS array -- automatic once array is extended. |
| `src/components/Onboarding/OnboardingWizard.tsx` | Step skip logic is Phase 40 scope. |

## Sources

### Primary (HIGH confidence)
- Direct codebase analysis: `providers/mod.rs` (7 match blocks), `openai_compat.rs` (signature + auth header), `ai.rs` (keychain read lines 206-213), `models.rs` (validate_api_key + curated_models), `store/index.ts` (PROVIDERS array + submitQuery gate), `AccountTab.tsx` (provider-specific UI), `lib.rs` (IPC registration)
- Ollama OpenAI Compatibility docs: `https://docs.ollama.com/api/openai-compatibility` -- /v1/ endpoint, SSE streaming
- Ollama health check: `https://github.com/ollama/ollama/issues/1378` -- GET / returns "Ollama is running"
- LM Studio OpenAI Compatibility docs: `https://lmstudio.ai/docs/developer/openai-compat` -- /v1/ endpoints

### Secondary (MEDIUM confidence)
- Research files: `.planning/research/STACK.md`, `ARCHITECTURE.md`, `PITFALLS.md` -- extensive pre-research verified against official docs
- LM Studio server docs: `https://lmstudio.ai/docs/developer/core/server` -- default port 1234

### Tertiary (LOW confidence)
- None -- all findings verified against codebase and official documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies needed, all existing deps verified in Cargo.toml
- Architecture: HIGH -- all 8 source files read and analyzed, exact line numbers identified
- Pitfalls: HIGH -- 7 pitfalls identified from direct code analysis with specific line references

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable -- extends existing enum with well-understood patterns)
