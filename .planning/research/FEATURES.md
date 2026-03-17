# Feature Landscape: Local LLM Providers (Ollama + LM Studio)

**Domain:** Local LLM provider integration for cross-platform AI terminal command overlay
**Researched:** 2026-03-17
**Confidence:** HIGH (verified against official API docs and existing codebase)

## Table Stakes

Features users expect when they see "Ollama" or "LM Studio" in a provider list.
Missing any of these = product feels broken or amateur.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Provider entries in PROVIDERS list | Users must select Ollama/LM Studio in onboarding and Settings dropdown | Low | Add to frontend `PROVIDERS` array, backend `Provider` enum. Two new entries with `isLocal: true` flag. |
| Auto-discovered model list | Both tools expose model listing APIs; users expect to see their downloaded models, not a hardcoded list | Medium | Ollama: `GET /api/tags` returns `{models: [{name, details: {parameter_size, quantization_level}}]}`. LM Studio: `GET /v1/models` returns OpenAI-format `{data: [{id, state, max_context_length, quantization}]}`. No curated list -- dynamic only. |
| Connection status indicator | Local servers may not be running; users need clear "connected" / "not running" feedback instead of API key validation | Medium | Replaces the API key validation flow entirely. Poll `GET http://localhost:11434` (Ollama returns "Ollama is running" with 200) or `GET http://localhost:1234/v1/models` (LM Studio returns model list on success). Show green/red connection dot in AccountTab and onboarding. |
| No API key required | Local providers run without authentication; showing a key input field is confusing and wrong | Low | Skip keychain storage entirely for local providers. Skip onboarding `StepApiKey` when Ollama/LM Studio selected. Mark as "ready" when connection succeeds instead of when key validates. Ollama's OpenAI-compat endpoint accepts any bearer token (or none). |
| Free usage (no cost tracking) | Local inference has no per-token cost; showing "$0.00" or pricing columns is misleading | Low | Set `input_price_per_m: None`, `output_price_per_m: None` on all local models. Existing `pricing_available: false` indicator in ModelTab already handles this. Token counting still works (useful for context awareness) but cost display shows "N/A". |
| Base URL configuration | Users may run servers on non-default ports, Docker containers, or remote machines | Medium | Defaults: `http://localhost:11434` (Ollama), `http://localhost:1234` (LM Studio). Editable in Settings AccountTab for local providers. Persist to `settings.json` via Tauri plugin-store. URL used by connection check, model discovery, and streaming. |
| Streaming chat completions | Users expect identical real-time token streaming as cloud providers | Low | Both Ollama and LM Studio support OpenAI-compatible `POST /v1/chat/completions` with `stream: true` and SSE format (`data: {JSON}` chunks, `data: [DONE]` sentinel). Reuse existing `openai_compat::stream` adapter -- just change the target URL. Both support `stream_options: { include_usage: true }`. |
| Provider SVG icons | All 5 existing providers have inline SVG icons in onboarding and settings. Missing icons for new providers breaks visual consistency. | Low | Add inline SVG paths for Ollama (llama silhouette) and LM Studio (LM logo). Same pattern as existing `ProviderIcon` component with switch-on-provider-id. |

## Differentiators

Features that set CMD+K's local provider experience apart. Not expected by all users,
but valued by power users who are already running local models.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Model metadata in label (parameter size + quantization) | Ollama's `/api/tags` returns `details.parameter_size` ("7B", "13B") and `details.quantization_level` ("Q4_K_M"). Shows users meaningful info about model capability instead of opaque IDs like "llama3.1:latest". | Low | Compose label as "Llama 3.1 8B (Q4_K_M)" or "Qwen 2.5 7B Q5_K_M". LM Studio also returns `quantization` and `arch` fields in `/v1/models`. Helps users pick the right model for their hardware. |
| Connection auto-retry on overlay open | If user starts Ollama after CMD+K is already running, the next Cmd+K press should auto-detect the server without requiring a settings visit or app restart | Low | Re-check connection status on each overlay show() or settings panel open. Don't require manual "reconnect" button. Cache status in AppState for instant display, but background-verify on interaction. |
| Server-not-running guidance | When connection fails, show actionable instructions instead of a generic error | Low | Ollama: "Ollama is not running. Start it with `ollama serve` in your terminal." LM Studio: "LM Studio server is not running. Open LM Studio and start the local server." Provider-specific messages are far more helpful than "Connection failed." |
| Graceful token usage extraction | Track input/output tokens for local models so the usage stats tab shows query count and token breakdown (even without cost) | Low | Already implemented: `openai_compat::stream` extracts `usage.prompt_tokens` and `usage.completion_tokens` from the final SSE chunk when `stream_options.include_usage` is set. Both Ollama and LM Studio support this field. Works out of the box. |
| Context window awareness for smart truncation | `context_window_for_model()` drives smart terminal context truncation. Local models need their context lengths known for optimal context budgeting. | Medium | Ollama: `POST /api/show` returns `model_info.{family}.context_length` (architecture max). LM Studio: `max_context_length` in `/v1/models` response per model. Cache on model discovery. Fallback: 4096 tokens for unknown models. |
| Longer timeout for local models | Consumer hardware is 5-30x slower than cloud APIs, especially on first request when model must load into memory (10-30s on Ollama) | Low | Set `default_timeout_secs` to 120s for Ollama/LM Studio (vs. 10-30s for cloud). Ollama auto-loads models on first request, which adds significant latency the first time. Subsequent requests are much faster. |

## Anti-Features

Features to explicitly NOT build. These add complexity without matching CMD+K's scope.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Model download/pull from within CMD+K | CMD+K is a command overlay, not a model manager. Downloading multi-GB models is a long background task that doesn't fit the overlay UX. Both Ollama and LM Studio have their own model management UIs. | Show "No models found" with guidance: "Pull a model with `ollama pull llama3.1` in your terminal" or "Download a model in LM Studio's Discover tab." |
| Model loading/unloading controls | Both servers handle model memory management automatically. Ollama auto-loads on first request and unloads after idle timeout (default 5 min). LM Studio loads models on API request. Exposing load/unload controls adds complexity for zero user benefit. | Let the local server handle it. First request will be slower if model isn't loaded; the 120s timeout accommodates this. |
| Custom Modelfile / model parameters | Ollama supports Modelfiles for custom system prompts, temperature, etc. This is a power-user feature that belongs in Ollama's own tooling and conflicts with CMD+K's own system prompt injection. | CMD+K injects its own terminal/assistant mode system prompts. Temperature is hardcoded at 0.1 for consistent command generation. No per-model customization needed. |
| GPU/VRAM monitoring | Showing GPU utilization adds platform-specific complexity (NVML on NVIDIA, ROCm on AMD, Metal on Apple Silicon) and is irrelevant to CMD+K's job of generating terminal commands. | Out of scope. Users who monitor GPU usage have nvidia-smi, radeontop, or Activity Monitor. |
| Remote server discovery (mDNS/Bonjour/SSDP) | Auto-discovering Ollama/LM Studio instances on the local network adds significant complexity (cross-platform mDNS, firewall traversal, security concerns) for a niche use case. | Base URL text field covers this entirely. Users running remote servers can type `http://192.168.1.100:11434`. Simple, explicit, secure. |
| Tier grouping (Fast/Balanced/Capable) for local models | Cloud providers have well-known model tiers. Local model naming varies wildly across quantizations, finetunes, and custom Modelfiles. Tier assignment would require maintaining mappings for hundreds of model variants that change constantly. | Show all local models in a flat alphabetical list without tier grouping. Cloud providers keep their existing tier grouping. The model name + parameter size + quantization gives users enough info to choose. |
| Offline-only mode toggle | A separate "offline mode" creates a confusing mental model. The app should just work regardless of whether the selected provider is local or cloud. | Provider selection IS the offline choice. Select Ollama = offline. Select OpenAI = online. No separate toggle. No airplane mode icon. |
| Ollama native API (/api/chat) streaming | Ollama's native API uses NDJSON streaming (not SSE). Supporting both would mean maintaining a second streaming adapter just for Ollama. | Use Ollama's OpenAI-compatible endpoint `/v1/chat/completions` exclusively. It uses the same SSE format as all other providers. No reason to support the native NDJSON format. |

## Feature Dependencies

```
Provider enum extension -----> All provider-dispatched code paths:
    |                              keychain.rs    (skip for local)
    |                              models.rs      (new fetch branch)
    |                              ai.rs          (URL override, skip key)
    |                              AccountTab.tsx (URL input, connection dot)
    |                              StepApiKey.tsx (skip step for local)
    |                              StepProviderSelect.tsx (new entries)
    |                              ModelTab.tsx   (flat list, no tiers)
    |
    +--> Base URL config (settings.json) ---> Connection check (health endpoint)
                                          |-> Model discovery (list models API)
                                          |-> Streaming (chat completions API)
    |
    +--> Connection check ---> Model discovery (only if connected)
                           |-> "Ready" state (replaces "valid" apiKeyStatus)
    |
    +--> No keychain dependency (local providers skip key storage entirely)

Context window for local models ---> Smart truncation in terminal/context.rs
                                 --> Requires /api/show (Ollama) or /v1/models metadata (LM Studio)
                                 --> Can defer: use 4096 default until implemented
```

## Existing Provider Abstraction Integration Points

### Backend (Rust)

| File | Current Pattern | Local Provider Change |
|------|----------------|----------------------|
| `providers/mod.rs` | `Provider` enum with 5 variants, `api_url()` returns `&'static str`, `keychain_account()`, `adapter_kind()`, `display_name()`, `console_url()`, `default_timeout_secs()` | Add `Ollama` and `LMStudio` variants. `api_url()` must change to support dynamic URLs (from settings) -- either add a method that accepts a base URL parameter, or return a default that callers override. `keychain_account()` returns a no-op value. `adapter_kind()` returns `OpenAICompat`. `console_url()` could point to `localhost:11434` / `localhost:1234`. `default_timeout_secs()` returns 120. |
| `keychain.rs` | `save_api_key(provider, key)`, `get_api_key(provider)`, `delete_api_key(provider)` all take `Provider` | No keychain operations for local providers. Frontend simply never calls these for Ollama/LM Studio. Backend functions can return `Ok(None)` or `Ok(())` for local providers without touching keyring. |
| `models.rs` | `curated_models()` returns hardcoded tier-tagged list; `fetch_api_models()` fetches from cloud; `validate_api_key()` hits cloud endpoints | Local providers: `curated_models()` returns empty vec (no curated list -- all dynamic). `fetch_api_models()` calls local server's model list endpoint (`/api/tags` for Ollama, `/v1/models` for LM Studio). New `check_local_connection(provider, base_url)` Tauri command replaces `validate_api_key()` for local providers. |
| `ai.rs` | `stream_ai_response()` reads API key from keychain, dispatches to adapter via `provider.adapter_kind()` | For local providers: skip keychain read. Pass "ollama" as dummy bearer token (Ollama ignores auth). Construct chat completions URL from base URL config: `{base_url}/v1/chat/completions`. Existing `openai_compat::stream` needs URL parameter instead of using `provider.api_url()`. |
| `providers/openai_compat.rs` | `stream()` uses `provider.api_url()` for the POST URL, sets `Authorization: Bearer {api_key}` header | Must accept `url: &str` parameter instead of deriving from `provider.api_url()`. For local providers, skip or use dummy auth header. For cloud providers, pass existing static URL. Minimal change: add `url` parameter to `stream()`, all callers pass it. |
| `usage.rs` | Token tracking with pricing lookup from curated + OpenRouter cache | Works as-is. Local models have no curated pricing, no OpenRouter cache match, so `pricing_available: false` and `estimated_cost: None`. Token counts still tracked and displayed. |
| `state.rs` | `AppState` struct with various Mutex fields | Optionally add `local_connection_status: Mutex<HashMap<String, bool>>` for caching Ollama/LM Studio connection state. Alternatively, let frontend re-check on each interaction (simpler). |

### Frontend (TypeScript/React)

| File | Current Pattern | Local Provider Change |
|------|----------------|----------------------|
| `store/index.ts` | `PROVIDERS` array (5 entries), `apiKeyStatus` drives submission guard, `submitQuery` checks `apiKeyStatus !== "valid"` | Add Ollama/LM Studio to PROVIDERS with `isLocal: true` flag. Add `connectionStatus: "unknown" \| "checking" \| "connected" \| "disconnected"` state. `submitQuery` must check `connectionStatus === "connected"` for local providers instead of `apiKeyStatus === "valid"`. Or: treat "connected" as equivalent to `apiKeyStatus = "valid"` by setting apiKeyStatus to "valid" when connection succeeds (simpler, fewer code paths). |
| `StepProviderSelect.tsx` | Renders all PROVIDERS as selectable cards | Add Ollama/LM Studio cards with icons. No logic change needed. |
| `StepApiKey.tsx` | Shows API key input, validates against cloud API | Skip this step entirely for local providers. Onboarding wizard should jump from provider select to connection check (inline in model select step or new step). |
| `OnboardingWizard.tsx` | Steps: Provider -> ApiKey -> Model -> Done | For local providers: Provider -> ConnectionCheck -> Model -> Done. Skip ApiKey step. Connection check can be embedded in the model select step (show "Connecting..." then model list or error). |
| `AccountTab.tsx` | Provider dropdown + API key input + validation indicator (green check / red X) | For local providers: hide key input field entirely. Show base URL text input + connection status dot (green = connected, red = not running). Show connection error message with guidance. "Edit URL" with default pre-filled. |
| `ModelTab.tsx` | Tier-grouped model list (Fast/Balanced/Capable sections), pricing column | For local providers: flat model list (no tier groupings since all models have `tier: ""`). Show parameter size + quantization in model label. Hide pricing column (or show "Free"). |
| `ProviderIcon.tsx` | Switch on provider ID returning inline SVG paths | Add `case "ollama":` and `case "lmstudio":` with appropriate SVG paths. |

## MVP Recommendation

Prioritize (in implementation order):

1. **Provider enum + frontend PROVIDERS extension** -- Foundation. Unblocks all other features. Add `Ollama` and `LMStudio` to both Rust `Provider` enum and TypeScript `PROVIDERS` array.
2. **Connection check command** -- New `check_local_connection(provider, base_url)` Tauri command. Hits health endpoint and returns connected/disconnected. Replaces `validate_api_key` for local providers.
3. **Base URL config** -- Settings persistence for Ollama/LM Studio base URLs in `settings.json`. Defaults: `http://localhost:11434`, `http://localhost:1234`.
4. **Model discovery** -- Call `/api/tags` (Ollama) or `/v1/models` (LM Studio) to populate model list. Map to `ModelWithMeta` with descriptive labels.
5. **Streaming via openai_compat adapter** -- Modify `openai_compat::stream` to accept URL parameter. Construct `{base_url}/v1/chat/completions` for local providers. Skip auth or use dummy token.
6. **Onboarding flow adaptation** -- Skip StepApiKey for local providers. Show connection status in model selection step.
7. **Settings UI adaptation** -- AccountTab shows URL input + connection dot for local providers instead of API key input.
8. **Provider icons** -- Add Ollama and LM Studio SVG icons.
9. **Longer timeout** -- Set 120s default for local providers.

Defer to later enhancement:
- **Context window detection** -- Use 4096 default initially. Add `/api/show` call for Ollama and use `max_context_length` from LM Studio's model list later.
- **Model metadata enrichment** -- Can ship with raw model IDs initially, enhance labels with parameter_size + quantization in follow-up.

## Key API Endpoints Summary

### Ollama (default: http://localhost:11434)

| Purpose | Endpoint | Method | Response Format |
|---------|----------|--------|----------------|
| Health check | `/` | GET | Returns "Ollama is running" (text, HTTP 200) |
| Version | `/api/version` | GET | `{"version": "0.5.x"}` |
| List models (native) | `/api/tags` | GET | `{models: [{name, model, size, details: {parameter_size, quantization_level, family}}]}` |
| List models (OpenAI) | `/v1/models` | GET | `{data: [{id, object}]}` |
| Chat completion | `/v1/chat/completions` | POST | SSE streaming: `data: {"choices":[{"delta":{"content":"..."}}]}`, `data: [DONE]`. Supports `stream_options.include_usage`. |
| Model details | `/api/show` | POST | Body: `{name: "model"}`. Returns `model_info.{family}.context_length`, parameters, license. |
| Auth | None | - | No authentication required. OpenAI-compat accepts any bearer token or none. |

### LM Studio (default: http://localhost:1234)

| Purpose | Endpoint | Method | Response Format |
|---------|----------|--------|----------------|
| Health check | `/v1/models` | GET | Returns model list on success (doubles as health + model discovery). |
| List models (OpenAI) | `/v1/models` | GET | `{data: [{id, object, type, publisher, arch, quantization, state, max_context_length}]}` |
| Chat completion | `/v1/chat/completions` | POST | Standard OpenAI SSE streaming format. Same as Ollama's OpenAI-compat endpoint. |
| Model state | In `/v1/models` response | - | `state` field: "loaded" or "not-loaded". `max_context_length` per model. |
| Auth | None | - | No authentication required. |

## Ollama vs LM Studio: Integration Differences

| Aspect | Ollama | LM Studio |
|--------|--------|-----------|
| Health check | Dedicated `GET /` endpoint (fast, lightweight) | Use `GET /v1/models` (returns full model list) |
| Model list format | Native `/api/tags` has richer metadata (parameter_size, quantization_level, family). OpenAI-compat `/v1/models` is minimal. | `/v1/models` includes `max_context_length`, `state` (loaded/not-loaded), `quantization`, `arch` |
| Context window discovery | `POST /api/show` with model name returns `model_info.{family}.context_length` (architecture max). Default runtime is 4096 unless `num_ctx` set. | `max_context_length` in `/v1/models` per model entry |
| Model auto-loading | Auto-loads model on first chat request. First request adds 10-30s latency. | Must have model loaded (user does this in LM Studio UI or via `/api/v1/models/load`) |
| Server auto-start | Ollama runs as a system service on macOS/Linux (auto-starts). On Windows, runs on login. | Must be started manually (LM Studio app must be open and server enabled) |
| Server management | CLI: `ollama serve`, system service | GUI: toggle in LM Studio app, CLI: `lms server start` |

## Sources

- [Ollama API Introduction](https://docs.ollama.com/api/introduction) -- HIGH confidence
- [Ollama OpenAI Compatibility](https://docs.ollama.com/api/openai-compatibility) -- HIGH confidence
- [Ollama /api/tags endpoint](https://docs.ollama.com/api/tags) -- HIGH confidence
- [Ollama Context Length docs](https://docs.ollama.com/context-length) -- HIGH confidence
- [Ollama Show Model Details](https://docs.ollama.com/api-reference/show-model-details) -- HIGH confidence
- [Ollama GitHub](https://github.com/ollama/ollama) -- HIGH confidence
- [LM Studio Developer Docs](https://lmstudio.ai/docs/developer) -- HIGH confidence
- [LM Studio REST API](https://lmstudio.ai/docs/developer/rest) -- HIGH confidence
- [LM Studio OpenAI Compatibility](https://lmstudio.ai/docs/developer/openai-compat) -- HIGH confidence
- [LM Studio Server Docs](https://lmstudio.ai/docs/developer/core/server) -- HIGH confidence
- [LM Studio /v1/models Context Length](https://lmstudio.ai/docs/typescript/model-info/get-context-length) -- HIGH confidence
- Existing codebase: `providers/mod.rs`, `models.rs`, `ai.rs`, `openai_compat.rs`, `keychain.rs`, `state.rs`, `store/index.ts`, `AccountTab.tsx`, `ModelTab.tsx`, `StepProviderSelect.tsx`, `StepApiKey.tsx` -- direct inspection
