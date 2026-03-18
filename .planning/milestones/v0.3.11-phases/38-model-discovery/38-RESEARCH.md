# Phase 38: Model Discovery - Research

**Researched:** 2026-03-17
**Domain:** Local LLM model discovery (Ollama /api/tags, LM Studio /v1/models) and display in existing model picker
**Confidence:** HIGH

## Summary

This phase implements dynamic model discovery for Ollama and LM Studio, replacing the current `vec![]` placeholder in `fetch_api_models()`. The two providers use different API response formats: Ollama returns a custom `{models: [...]}` JSON from `/api/tags` with rich metadata (parameter_size, quantization_level, family), while LM Studio returns an OpenAI-compatible `{data: [...]}` JSON from `/v1/models` with its own extended fields (quantization, arch, type, max_context_length). The existing `fetch_models` IPC command already passes `app_handle` (currently unused `_app_handle`) and `state`, so the plumbing is in place to read base URLs and add provider-specific parsing branches.

The key architectural insight is that the existing `validate_api_key` function already fetches and inspects both `/api/tags` (Ollama) and `/v1/models` (LM Studio) responses during health checks -- it just discards the model data. Phase 38 adds proper parsing of these same responses in `fetch_api_models()`, reusing the same HTTP client pattern and base URL resolution (`get_provider_base_url`). No new IPC commands are needed. The frontend already calls `fetch_models` after successful health check validation and feeds the result into the model picker, which already handles tier grouping and the "All Models" section.

**Primary recommendation:** Add Ollama and LMStudio match arms to `fetch_api_models()` with provider-specific deserialization structs, auto-tier by parameter size using the locked decision rules (<7B = fast, 7-30B = balanced, >30B = capable), and use raw model names as labels (no pretty-printing).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Display raw model names as-is for both providers -- no parsing or pretty-printing
- Ollama: show the full tag name (e.g., `llama3.2:3b-instruct-q4_K_M`) exactly as returned by `/api/tags`
- LM Studio: show the full model ID (e.g., `lmstudio-community/Meta-Llama-3.1-8B-Instruct-GGUF`) exactly as returned by `/v1/models`
- The `label` field in `ModelWithMeta` equals the raw name/ID
- Auto-tier by parameter size: <7B = Fast, 7-30B = Balanced, >30B = Most Capable
- Ollama provides `details.parameter_size` (e.g., "3B", "7B", "70B") -- parse the numeric value for tiering
- When parameter size is unknown (LM Studio or missing field), put in "All Models" section with empty tier string
- Fetch model list on provider selection AND each time the settings Model tab opens
- Same behavior as cloud providers but more frequent to catch model installs/unloads
- No manual refresh button -- tab re-open is the refresh trigger
- Keep current "No models found -- check that models are loaded" message (from Phase 37)
- No provider-specific hints -- consistent with Phase 37 "no start hints" decision
- If fetch fails, graceful degradation to empty list

### Claude's Discretion
- Exact JSON response parsing structure for Ollama `/api/tags` and LM Studio `/v1/models`
- How to extract numeric param size from Ollama's `details.parameter_size` string
- Whether to filter out embedding-only models from LM Studio results
- HTTP timeout for model list fetch (reuse existing patterns)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LMOD-01 | App auto-discovers available models from Ollama via /api/tags endpoint | Ollama `/api/tags` response format verified, deserialization structs defined, integration point in `fetch_api_models()` Ollama match arm identified |
| LMOD-02 | App auto-discovers available models from LM Studio via /v1/models endpoint | LM Studio `/v1/models` OpenAI-compat response format verified, can reuse existing `OpenAIModelsResponse` struct (with LM Studio extensions for filtering), integration point in same `fetch_api_models()` function |
| LMOD-03 | Model list displays metadata where available (parameter size, quantization level for Ollama models) | Ollama `details.parameter_size` and `details.quantization_level` fields confirmed in API docs; auto-tiering by numeric param size extraction; labels show raw names per locked decision |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| reqwest | (existing in Cargo.toml) | HTTP GET to local model listing APIs | Already used for all provider HTTP calls including validate_api_key health checks |
| serde / serde_json | (existing in Cargo.toml) | Deserialize Ollama and LM Studio JSON responses | Already used for OpenAIModelsResponse, GeminiModelsResponse, OpenRouterModelsResponse |
| tauri-plugin-store | (existing in Cargo.toml) | Read base_url from settings.json via get_provider_base_url | Already used in validate_api_key for local providers |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| regex (NOT needed) | - | Parameter size parsing | NOT needed -- simple string manipulation with trim_end_matches and parse::<f64> suffices for strings like "3B", "7B", "70B", "4.3B" |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Ollama `/api/tags` (native) | Ollama `/v1/models` (OpenAI-compat) | `/api/tags` has richer metadata (parameter_size, quantization_level, family) -- use native endpoint for LMOD-03 metadata requirement |
| Separate LM Studio REST API `/api/v1/models` | OpenAI-compat `/v1/models` | REST API has more fields (params_string, loaded_instances, capabilities) but requires a different port config; `/v1/models` is simpler and already used for health checks |

**No new dependencies required.** All work uses existing crate dependencies.

## Architecture Patterns

### Recommended Change Structure
```
src-tauri/src/commands/
  models.rs        # Add OllamaTagsResponse structs + Ollama/LMStudio match arms in fetch_api_models()
                   # Thread app_handle into fetch_api_models() for base URL resolution
                   # Add parse_param_size() helper for tier assignment
```

### Pattern 1: Provider-Specific Deserialization with Common Output
**What:** Each local provider has its own response struct but maps to the same `ModelWithMeta` output.
**When to use:** Ollama `/api/tags` returns `{models: [{name, details: {parameter_size, ...}}]}` while LM Studio `/v1/models` returns `{data: [{id, ...}]}`.
**Example:**
```rust
// Ollama-specific response types (new)
#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    details: OllamaModelDetails,
}

#[derive(Deserialize, Default)]
struct OllamaModelDetails {
    parameter_size: Option<String>,   // e.g., "3B", "7B", "70B", "4.3B"
    quantization_level: Option<String>, // e.g., "Q4_K_M", "Q5_0"
    family: Option<String>,           // e.g., "llama", "gemma", "qwen2"
}

// LM Studio: reuse existing OpenAIModelsResponse { data: Vec<OpenAIModel { id }> }
// The existing struct already captures the `id` field which is all we need
```
**Source:** Ollama /api/tags docs (https://docs.ollama.com/api/tags), LM Studio OpenAI-compat docs (https://lmstudio.ai/docs/developer/openai-compat/models)

### Pattern 2: Parameter Size Parsing for Auto-Tiering
**What:** Extract numeric value from Ollama's `details.parameter_size` string for tier assignment.
**When to use:** Mapping Ollama models to Fast/Balanced/Most Capable tiers.
**Example:**
```rust
/// Parse parameter size string like "3B", "7B", "70B", "4.3B", "1.5B" into
/// a numeric value in billions. Returns None if parsing fails.
fn parse_param_size_billions(s: &str) -> Option<f64> {
    let trimmed = s.trim().to_uppercase();
    let numeric_part = trimmed.trim_end_matches('B');
    numeric_part.parse::<f64>().ok()
}

/// Assign tier based on parameter count: <7B=fast, 7-30B=balanced, >30B=capable
fn tier_from_param_size(param_size: Option<&str>) -> String {
    match param_size.and_then(parse_param_size_billions) {
        Some(b) if b < 7.0 => "fast".to_string(),
        Some(b) if b <= 30.0 => "balanced".to_string(),
        Some(_) => "capable".to_string(),
        None => String::new(), // Unknown size -> "All Models" section
    }
}
```

### Pattern 3: Base URL Resolution via app_handle
**What:** Use existing `get_provider_base_url()` to resolve configurable base URLs.
**When to use:** Every local provider HTTP call in `fetch_api_models()`.
**Example:**
```rust
// In fetch_api_models(), thread app_handle from fetch_models:
Provider::Ollama => {
    let base_url = super::providers::get_provider_base_url(app_handle, &Provider::Ollama);
    let tags_url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    // ... HTTP GET and parse
}
```
**Key detail:** `fetch_models` already has `_app_handle: tauri::AppHandle` -- just rename to `app_handle` and pass into `fetch_api_models()`.

### Pattern 4: Timeout Reuse from Health Check
**What:** Use the same 2-second timeout used in `validate_api_key` for local providers.
**When to use:** Model list fetches from local servers.
**Why:** Local servers should respond near-instantly (sub-10ms). A 2-second timeout catches stale/crashed servers without blocking the UI.
**Example:**
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(2))
    .build()
    .map_err(|e| e.to_string())?;
```

### Anti-Patterns to Avoid
- **Separate IPC command for local model fetching:** The existing `fetch_models` / `fetch_api_models` pipeline handles this. Adding a new `fetch_local_models` command would duplicate merge logic and force frontend changes.
- **Caching model lists in AppState:** Local model lists change frequently (user installs/unloads models). Always fetch fresh on each call. The frontend already handles the lifecycle (fetch on provider select, on tab open).
- **Pretty-printing model names:** Locked decision says raw names as-is. Do not parse "llama3.2:3b-instruct-q4_K_M" into "Llama 3.2 3B Instruct Q4_K_M".
- **Filtering Ollama models by family/type:** Ollama `/api/tags` returns all pulled models. There are no embedding-only models in Ollama's model library that would appear in `/api/tags` -- all are chat-capable.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Base URL resolution | Custom config reading logic | `get_provider_base_url(app_handle, provider)` | Already exists in `providers/mod.rs`, handles default fallback, reads from settings.json store |
| HTTP client with timeout | New reqwest::Client per call | `reqwest::Client::builder().timeout(Duration::from_secs(2)).build()` | Same pattern as `validate_api_key` for local providers |
| OpenAI-format model list parsing | Custom LM Studio response parser | Existing `OpenAIModelsResponse` struct | LM Studio's `/v1/models` returns standard OpenAI `{data: [{id}]}` format; the existing struct captures what we need |
| Tier-grouped model display | Custom frontend tier rendering | Existing `TIER_ORDER` + "All Models" section in `ModelTab.tsx` | Already handles fast/balanced/capable grouping plus uncategorized models in "All Models" |

**Key insight:** Phase 38 is almost entirely backend work in a single file (`models.rs`). The frontend model picker already works with tier grouping and empty tier strings -- no frontend changes are needed beyond what Phase 37 already wired up.

## Common Pitfalls

### Pitfall 1: LM Studio Embedding Models in Model List
**What goes wrong:** LM Studio `/v1/models` may return embedding models alongside LLMs. Embedding models cannot do chat completions and will fail at streaming time.
**Why it happens:** LM Studio lists all loaded/available models regardless of type. The response includes a `type` field distinguishing "llm" from "embedding".
**How to avoid:** Filter by checking if model ID or type indicates an embedding model. However, the basic `OpenAIModelsResponse` struct only captures `id` -- it does not capture the `type` field. Option A: Extend the struct to capture `type` and filter. Option B: Accept all models (user knows what they loaded). Recommendation: Filter out known embedding identifiers if the `type` field is not available in the OpenAI-compat format. In practice, LM Studio's OpenAI-compat `/v1/models` endpoint includes a minimal response; the `type` field may not be present in the OpenAI-compat response (it IS present in the REST API `/api/v1/models`). Accept all models and let failures surface at streaming time as a practical approach.
**Warning signs:** Users see embedding model entries in their model picker that fail when selected.

### Pitfall 2: Missing `details` in Ollama Response
**What goes wrong:** Ollama's `/api/tags` may return models with empty or missing `details` block (e.g., custom Modelfiles, very old model versions).
**Why it happens:** The `details` object depends on model metadata being present in the GGUF file. Custom models or imports may lack it.
**How to avoid:** Use `#[serde(default)]` on the `details` field and `Option<String>` for all detail sub-fields. When `parameter_size` is None, assign empty tier string so the model appears in "All Models" section.
**Warning signs:** Deserialization panic or model silently omitted from list.

### Pitfall 3: fetch_api_models Needs app_handle for Base URL
**What goes wrong:** `fetch_api_models()` currently has no access to `app_handle`, but `get_provider_base_url()` requires it.
**Why it happens:** The function signature only takes `(provider, api_key, state)`. Cloud providers use hardcoded URLs so never needed `app_handle`.
**How to avoid:** Add `app_handle: &tauri::AppHandle` parameter to `fetch_api_models()`. Update the single call site in `fetch_models()` to pass `&_app_handle` (renamed to `&app_handle`). This is a minor signature change.
**Warning signs:** Compilation error when trying to call `get_provider_base_url` inside `fetch_api_models`.

### Pitfall 4: Model Tab Refresh Timing
**What goes wrong:** User installs a model in Ollama while settings are open, but the model list doesn't update.
**Why it happens:** The model list is fetched on provider selection change but NOT on tab re-open within the same settings session.
**How to avoid:** The locked decision says "Fetch model list on provider selection AND each time the settings Model tab opens." The frontend `ModelTab` component mounts/unmounts on tab switch, so its `useEffect` hook fires on each open. However, the model fetch currently happens in `AccountTab` (on provider change), not in `ModelTab`. The fetch needs to also happen in `ModelTab` mount or via a shared effect that runs on tab navigation.
**Warning signs:** Stale model list after installing/unloading models without changing providers.

### Pitfall 5: Parameter Size String Edge Cases
**What goes wrong:** Parameter size parsing fails on unexpected formats.
**Why it happens:** Ollama's `parameter_size` field is free-form text. Observed values: "3B", "7B", "70B", "4.3B", "1.5B", "236B". Could potentially include "M" suffix for small models (e.g., "270M").
**How to avoid:** Handle both "B" (billions) and "M" (millions) suffixes. For "M" suffix, divide by 1000 to get billions equivalent. Return None for unparseable values.
**Warning signs:** Small models (270M params) classified as >30B or zero-B.

## Code Examples

Verified patterns from existing codebase and official API documentation:

### Ollama fetch_api_models Branch
```rust
// Source: Ollama /api/tags docs (https://docs.ollama.com/api/tags)
// Integration point: models.rs fetch_api_models() Ollama match arm
Provider::Ollama => {
    let base_url = super::providers::get_provider_base_url(app_handle, &Provider::Ollama);
    let tags_url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&tags_url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
    let parsed: OllamaTagsResponse =
        serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed
        .models
        .into_iter()
        .map(|m| {
            let tier = tier_from_param_size(m.details.parameter_size.as_deref());
            ModelWithMeta {
                id: m.name.clone(),
                label: m.name, // Raw name per locked decision
                tier,
                input_price_per_m: None,
                output_price_per_m: None,
            }
        })
        .collect())
}
```

### LM Studio fetch_api_models Branch
```rust
// Source: LM Studio /v1/models docs (https://lmstudio.ai/docs/developer/openai-compat/models)
// Integration point: models.rs fetch_api_models() LMStudio match arm
Provider::LMStudio => {
    let base_url = super::providers::get_provider_base_url(app_handle, &Provider::LMStudio);
    let models_url = format!("{}/v1/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(&models_url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
    let parsed: OpenAIModelsResponse =
        serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

    Ok(parsed
        .data
        .into_iter()
        .map(|m| ModelWithMeta {
            label: m.id.clone(), // Raw ID per locked decision
            id: m.id,
            tier: String::new(), // LM Studio has no parameter_size in OpenAI-compat format
            input_price_per_m: None,
            output_price_per_m: None,
        })
        .collect())
}
```

### Parameter Size Parser
```rust
// Parse "3B" -> 3.0, "7B" -> 7.0, "70B" -> 70.0, "4.3B" -> 4.3, "270M" -> 0.27
fn parse_param_size_billions(s: &str) -> Option<f64> {
    let trimmed = s.trim().to_uppercase();
    if let Some(numeric) = trimmed.strip_suffix('B') {
        numeric.parse::<f64>().ok()
    } else if let Some(numeric) = trimmed.strip_suffix('M') {
        numeric.parse::<f64>().ok().map(|v| v / 1000.0)
    } else {
        None
    }
}

fn tier_from_param_size(param_size: Option<&str>) -> String {
    match param_size.and_then(parse_param_size_billions) {
        Some(b) if b < 7.0 => "fast".into(),
        Some(b) if b <= 30.0 => "balanced".into(),
        Some(_) => "capable".into(),
        None => String::new(),
    }
}
```

### fetch_api_models Signature Change
```rust
// BEFORE (current):
async fn fetch_api_models(
    provider: &Provider,
    api_key: &str,
    state: &tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<ModelWithMeta>, String> {

// AFTER (add app_handle parameter):
async fn fetch_api_models(
    provider: &Provider,
    api_key: &str,
    state: &tauri::State<'_, crate::state::AppState>,
    app_handle: &tauri::AppHandle,
) -> Result<Vec<ModelWithMeta>, String> {
```

### Model Tab Refresh on Open (Frontend)
```typescript
// In ModelTab.tsx -- add model fetch on mount to catch new model installs
// Source: existing pattern from AccountTab.tsx useEffect on selectedProvider
useEffect(() => {
  if (!isLocal) return; // Cloud providers already fetch in AccountTab
  const refreshModels = async () => {
    try {
      const models = await invoke<ModelWithMeta[]>(
        "fetch_models",
        { provider: selectedProvider, apiKey: "" }
      );
      setModels(models);
    } catch {
      // Graceful degradation -- keep existing list
    }
  };
  refreshModels();
}, []); // On mount = on tab open
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hardcoded model lists for local providers | Dynamic discovery from local API endpoints | Phase 38 (now) | Models always reflect what user has installed |
| Ollama `/v1/models` (OpenAI-compat, minimal) | Ollama `/api/tags` (native, rich metadata) | Decision in Phase 38 | Enables parameter_size and quantization display for LMOD-03 |
| Flat "All Models" section for local providers (FEATURES.md anti-feature) | Auto-tier by parameter size into Fast/Balanced/Capable | Decision in Phase 38 CONTEXT.md | Overrides initial FEATURES.md recommendation; better UX for users with many models |

**Important context evolution:** The initial FEATURES.md research (done during milestone planning) recommended flat alphabetical lists with no tier grouping for local models. The Phase 38 CONTEXT.md discussion overrode this with auto-tiering by parameter size. The locked decision in CONTEXT.md takes precedence.

## Open Questions

1. **LM Studio embedding model filtering**
   - What we know: LM Studio's REST API `/api/v1/models` includes a `type` field ("llm" vs "embedding"). The OpenAI-compat `/v1/models` endpoint may or may not include this field.
   - What's unclear: Whether the OpenAI-compat response includes the `type` field. If not, there is no reliable way to filter embedding models without checking another endpoint.
   - Recommendation: Accept all models from `/v1/models`. Embedding models in LM Studio are unlikely to appear in the OpenAI-compat endpoint when JIT loading is off. If they do appear, they will fail gracefully at streaming time. This is Claude's Discretion per CONTEXT.md -- recommend not filtering.

2. **Model tab refresh for cloud providers**
   - What we know: The locked decision says "fetch on provider selection AND each time Model tab opens." Currently, cloud providers fetch models in AccountTab on provider change, not on Model tab mount.
   - What's unclear: Whether this decision applies to cloud providers too or only local providers.
   - Recommendation: Add the refresh-on-mount behavior to ModelTab for ALL providers (consistent behavior). The fetch is fast and non-destructive. However, this may be out of phase scope since cloud providers already work. Keep the refresh for local providers only if scope must be tight.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust unit tests (cargo test) |
| Config file | Standard Cargo.toml test configuration |
| Quick run command | `cargo test --lib -p cmd-k -- --nocapture` |
| Full suite command | `cargo test --lib -p cmd-k` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| LMOD-01 | Ollama `/api/tags` response parsed into ModelWithMeta vec | unit | `cargo test --lib -p cmd-k -- ollama_tags --nocapture` | No -- Wave 0 |
| LMOD-02 | LM Studio `/v1/models` response parsed into ModelWithMeta vec | unit | `cargo test --lib -p cmd-k -- lmstudio_models --nocapture` | No -- Wave 0 |
| LMOD-03 | Parameter size parsing and tier assignment from Ollama metadata | unit | `cargo test --lib -p cmd-k -- param_size --nocapture` | No -- Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib -p cmd-k`
- **Per wave merge:** `cargo test --lib -p cmd-k` + `cargo check`
- **Phase gate:** Full suite green + manual test with running Ollama/LM Studio instances

### Wave 0 Gaps
- [ ] `parse_param_size_billions` unit tests -- covers tier_from_param_size logic for "3B", "7B", "70B", "4.3B", "270M", empty, garbage input
- [ ] `OllamaTagsResponse` deserialization tests -- verified against official API docs example JSON
- [ ] `OllamaModel` with missing/empty `details` field -- graceful default handling
- [ ] LM Studio `OpenAIModelsResponse` parsing -- verify existing struct handles LM Studio's response correctly

## Sources

### Primary (HIGH confidence)
- [Ollama /api/tags endpoint](https://docs.ollama.com/api/tags) -- Response format with `{models: [{name, details: {parameter_size, quantization_level, family}}]}`, verified against official docs
- [LM Studio /v1/models](https://lmstudio.ai/docs/developer/openai-compat/models) -- OpenAI-compat model listing endpoint
- [LM Studio REST API /api/v1/models](https://lmstudio.ai/docs/developer/rest/list) -- Extended model listing with `type`, `params_string`, `quantization`, `max_context_length` fields
- Existing codebase: `models.rs` lines 8-19 (ModelWithMeta), 93 (empty vec for Ollama/LMStudio), 356-376 (fetch_models with _app_handle), 380-559 (fetch_api_models with provider match arms), 306-316 (OpenAIModelsResponse/OpenAIModel structs)
- Existing codebase: `providers/mod.rs` lines 164-177 (get_provider_base_url reads from settings.json store)
- Existing codebase: `models.rs` lines 236-270 (validate_api_key Ollama branch already fetches /api/tags and inspects models array)

### Secondary (MEDIUM confidence)
- [LM Studio OpenAI Compatibility](https://lmstudio.ai/docs/developer/openai-compat) -- Endpoint listing confirms /v1/models availability
- [Ollama OpenAI Compatibility](https://docs.ollama.com/api/openai-compatibility) -- Confirms /v1/models also available but with less metadata than /api/tags

### Tertiary (LOW confidence)
- LM Studio `/v1/models` exact field list in OpenAI-compat mode -- official docs lack detailed response schema for this endpoint. The REST API `/api/v1/models` has detailed schema but that is a different endpoint. Assumed the OpenAI-compat `/v1/models` returns at minimum `{data: [{id, object}]}` matching OpenAI format, which is what the existing `OpenAIModelsResponse` struct captures.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies, all existing crate features
- Architecture: HIGH -- direct codebase inspection, clear integration point in `fetch_api_models()` match block
- Pitfalls: HIGH -- identified from codebase analysis (missing app_handle, serde defaults) and API doc review (embedding models, parameter_size edge cases)

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (30 days -- Ollama and LM Studio APIs are stable)
