# Phase 25: Token Tracking & Pricing Backend - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Every AI query records input/output token counts and the app can calculate estimated cost using per-model pricing. Covers token extraction from all 3 streaming adapters, session-scoped accumulation in Rust state, bundled + dynamic pricing data, and IPC commands for frontend consumption. Frontend display is Phase 26.

</domain>

<decisions>
## Implementation Decisions

### Pricing data structure
- Colocate pricing with `curated_models()` in `models.rs` — add `input_price_per_m: Option<f64>` and `output_price_per_m: Option<f64>` fields to `ModelWithMeta`
- Prices stored as $/1M tokens (e.g., 2.50 means $2.50 per million tokens)
- Hardcode accurate pricing for ALL known curated models at time of writing — update manually in future releases
- Non-curated API-fetched models without known pricing get `None` — frontend shows "pricing unavailable"

### Token extraction approach
- Change adapter return type from `Result<(), String>` to `Result<TokenUsage, String>` where `TokenUsage` has `input_tokens: Option<u64>`, `output_tokens: Option<u64>`
- Send `stream_options: { include_usage: true }` for ALL OpenAI-compatible providers (OpenAI, xAI, OpenRouter) — providers that don't support it ignore unknown fields
- Anthropic: extract from `message_start` (input_tokens) and `message_delta` (output_tokens) events
- Gemini: extract from `usageMetadata.promptTokenCount` / `usageMetadata.candidatesTokenCount`
- When a provider doesn't return token counts, succeed silently with `None` — no warnings, no user impact

### Usage accumulation
- Add `usage: Mutex<UsageAccumulator>` field to `AppState` — consistent with existing session-scoped pattern
- Group by provider+model: `HashMap<(Provider, model_id), UsageEntry>` where `UsageEntry` has `total_input_tokens`, `total_output_tokens`, `query_count`
- `stream_ai_response` in `ai.rs` accumulates token counts into `AppState.usage` after each adapter call

### IPC commands
- `get_usage_stats` returns `Vec<UsageStatEntry>` with fields: `provider`, `model`, `input_tokens`, `output_tokens`, `estimated_cost: Option<f64>`, `pricing_available: bool`
- Backend also computes and returns `session_total_cost: Option<f64>` — avoids floating-point discrepancies across Rust/TS
- `reset_usage` clears ALL usage stats (single command, no per-provider granularity)

### OpenRouter pricing fetch
- Fetch pricing when user selects OpenRouter as provider — parse pricing fields from the same `/api/v1/models` response already being fetched for model list
- Cache in `AppState` as `openrouter_pricing: Mutex<HashMap<String, (f64, f64)>>` — session-scoped, refreshed on each provider select
- Convert OpenRouter's per-token strings to $/1M format at fetch time — one format everywhere
- Graceful degradation if API unreachable: models still work, tokens tracked, cost shows "pricing unavailable"

### Claude's Discretion
- Exact `TokenUsage` and `UsageAccumulator` struct designs
- How to wire `AppState` into `stream_ai_response` (via `tauri::State` parameter)
- OpenRouter pricing field parsing details
- Error handling internals within adapters

</decisions>

<specifics>
## Specific Ideas

- All curated models should have accurate, real pricing data at time of implementation — not placeholder values
- OpenRouter pricing conversion: multiply per-token string by 1,000,000 to get $/1M format

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ModelWithMeta` struct in `models.rs`: extend with pricing fields — already serialized to frontend
- `curated_models()` in `models.rs`: natural home for hardcoded pricing data
- `OpenRouterModelsResponse` / `OpenRouterModel` in `models.rs`: already deserializes from their API, add pricing fields
- `AppState` in `state.rs`: add usage accumulator field, follows existing Mutex pattern

### Established Patterns
- Streaming adapters in `providers/`: all return `Result<(), String>`, will change to `Result<TokenUsage, String>`
- `stream_ai_response` in `ai.rs` dispatches via `provider.adapter_kind()` match — accumulation happens after match
- `AppState::default()` initializes all fields — new usage field starts empty
- IPC commands pattern: `#[tauri::command]` async fns with `tauri::State<AppState>` parameter

### Integration Points
- `openai_compat::stream()`: add `stream_options.include_usage` to request body, parse final chunk usage
- `anthropic::stream()`: parse `message_start` and `message_delta` events (currently in `_ => {}` catch-all)
- `gemini::stream()`: parse `usageMetadata` from chunks (currently only extracts text)
- `fetch_api_models()` for OpenRouter: extend `OpenRouterModel` to capture pricing fields
- `lib.rs:100`: `AppState::default()` — new usage field auto-included

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 25-token-tracking-and-pricing-backend*
*Context gathered: 2026-03-10*
