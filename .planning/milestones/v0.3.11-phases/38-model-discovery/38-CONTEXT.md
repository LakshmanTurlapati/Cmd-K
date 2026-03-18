# Phase 38: Model Discovery - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-discover locally installed models from Ollama (`/api/tags`) and LM Studio (`/v1/models`) and display them in the model picker with metadata. Streaming and command generation with selected models is Phase 39.

</domain>

<decisions>
## Implementation Decisions

### Model label formatting
- Display raw model names as-is for both providers — no parsing or pretty-printing
- Ollama: show the full tag name (e.g., `llama3.2:3b-instruct-q4_K_M`) exactly as returned by `/api/tags`
- LM Studio: show the full model ID (e.g., `lmstudio-community/Meta-Llama-3.1-8B-Instruct-GGUF`) exactly as returned by `/v1/models`
- The `label` field in `ModelWithMeta` equals the raw name/ID

### Tier grouping for local models
- Auto-tier by parameter size: <7B → Fast, 7-30B → Balanced, >30B → Most Capable
- Ollama provides `details.parameter_size` (e.g., "3B", "7B", "70B") — parse the numeric value for tiering
- When parameter size is unknown (LM Studio doesn't always provide it, or field missing), put in "All Models" section with empty tier string
- Existing tier group display (Fast/Balanced/Most Capable + All Models) works unchanged

### Model refresh behavior
- Fetch model list on provider selection AND each time the settings Model tab opens
- Same behavior as cloud providers but more frequent to catch model installs/unloads
- No manual refresh button — tab re-open is the refresh trigger

### Empty/error states
- Keep current "No models found — check that models are loaded" message (from Phase 37 fix)
- No provider-specific hints — consistent with Phase 37 "no start hints" decision
- If fetch fails (server went down between health check and model list), graceful degradation to empty list

### Claude's Discretion
- Exact JSON response parsing structure for Ollama `/api/tags` and LM Studio `/v1/models`
- How to extract numeric param size from Ollama's `details.parameter_size` string
- Whether to filter out embedding-only models from LM Studio results
- HTTP timeout for model list fetch (reuse existing patterns)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Backend model fetching
- `src-tauri/src/commands/models.rs` — `fetch_models` IPC command (line 356), `fetch_api_models` (line 380), `ModelWithMeta` struct (line 8), `curated_models` returns empty vec for Ollama/LMStudio (line 93). Also contains `validate_api_key` which already calls both Ollama `/api/tags` and LM Studio `/v1/models` for health checks (can reuse response parsing)
- `src-tauri/src/commands/providers/mod.rs` — Provider enum with `is_local()`, `default_base_url()`, `normalize_base_url()` helpers

### Frontend model display
- `src/components/Settings/ModelTab.tsx` — Model picker with tier grouping (TIER_ORDER constant), `isEnabled` gate, `renderModelButton`, and current local provider empty state messages
- `src/store/index.ts` — `availableModels` state, `setAvailableModels`, `fetchModels` IPC call pattern

### Research findings
- `.planning/research/FEATURES.md` — Ollama `/api/tags` response format, LM Studio `/v1/models` response format, model metadata fields
- `.planning/research/ARCHITECTURE.md` — Integration architecture for model discovery

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `fetch_api_models()` in models.rs — Pattern for provider-specific model fetching with match arms. Add Ollama and LMStudio arms.
- `ModelWithMeta` struct — Already has `id`, `label`, `tier`, optional pricing. Local models use `tier` for auto-tiering, `None` pricing.
- `OpenAIModelsResponse` — Existing deserialization struct for `/v1/models` format. LM Studio uses same format.
- `validate_api_key` health check — Already fetches `/api/tags` (Ollama) and `/v1/models` (LM Studio) for health checks. Could potentially reuse the response body for model discovery to avoid a duplicate HTTP call.

### Established Patterns
- `fetch_models` merges curated + API models — local providers have empty curated list, so all models come from API
- Cloud providers use `reqwest::Client::new()` with auth headers — local providers skip auth (Phase 37 pattern)
- Model list is fetched in frontend via `invoke("fetch_models", { provider, apiKey })` — local providers pass empty string for apiKey

### Integration Points
- `fetch_api_models()` match block — add `Provider::Ollama` and `Provider::LMStudio` arms (currently matched with `Provider::OpenRouter` returning `vec![]`)
- `fetch_models` — reads base URL from settings store for local providers (same pattern as `stream_ai_response` in ai.rs)
- Frontend `ModelTab` — `isEnabled` already checks `apiKeyStatus === "valid" && availableModels.length > 0`, so populating `availableModels` auto-enables the picker

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches within the decisions above.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 38-model-discovery*
*Context gathered: 2026-03-17*
