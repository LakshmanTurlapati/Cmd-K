# Project Research Summary

**Project:** CMD+K v0.3.11 — Local LLM Providers (Ollama + LM Studio)
**Domain:** Local LLM provider integration into a cloud-first multi-provider AI terminal overlay
**Researched:** 2026-03-17
**Confidence:** HIGH

## Executive Summary

Adding Ollama and LM Studio to CMD+K is a surgical integration, not a rewrite. Both providers expose OpenAI-compatible `/v1/chat/completions` endpoints with SSE streaming and `data: [DONE]` sentinels — the exact same protocol the existing `openai_compat::stream()` adapter already handles for OpenAI, xAI, and OpenRouter. Zero new Rust crates and zero new streaming adapters are required. The scope is: extend the `Provider` enum with two new keyless variants, make the base URL configurable rather than static, add local model discovery via HTTP, and adapt the frontend to skip API key steps in favor of connection health checks. The integration touches about 8 files in well-defined ways, and every change is additive rather than replacing existing behavior.

The recommended approach is a 5-phase build ordered strictly by dependency: provider enum foundation first (unblocks everything), then model discovery (users need a model before streaming), then streaming integration (the URL-dynamic adapter change), then frontend store updates, and finally UI polish for settings and onboarding. The architecture research confirms all five phases are straightforward because both providers deliberately mimic the cloud API contract CMD+K already supports. The critical design rule is to use `provider.is_local()` for all branching rather than matching individual variants — this future-proofs against adding more local providers (llama.cpp, vLLM, etc.) later.

The primary risks are not technical complexity but implementation oversights: the API key gate in `stream_ai_response` and `submitQuery` will permanently block local providers unless explicitly bypassed, the 30-second cloud timeout will kill cold-start requests (local model loading takes 10–90 seconds), and the onboarding wizard has no path for keyless providers — it will ask users for an "Ollama API key" that does not exist. All three risks have clear, low-cost fixes identified in research. The one area requiring a judgment call at implementation time is whether to use `stream_options.include_usage` with an extended outer timeout or restructure to a per-chunk idle timeout for very long local generations.

## Key Findings

### Recommended Stack

No new dependencies are needed. The existing stack — `tauri-plugin-http` (reqwest), `eventsource-stream`, `futures-util`, `tokio`, `serde_json`, and `tauri-plugin-store` — already provides HTTP streaming, SSE parsing, JSON deserialization, and settings persistence. The integration is entirely within the application layer.

**Core technologies (unchanged, reused):**
- `reqwest` via `tauri-plugin-http`: HTTP client for all local provider calls (health, models, streaming)
- `eventsource-stream`: SSE parsing — identical format from Ollama and LM Studio `/v1/` endpoints
- `tauri-plugin-store` (`settings.json`): Persist configurable base URLs (`ollama_base_url`, `lmstudio_base_url`)
- `openai_compat::stream()`: The existing adapter reused with a dynamic URL parameter and conditional auth header

The only structural change is making `openai_compat::stream()` accept an explicit `api_url: &str` parameter instead of calling `provider.api_url()`, and conditionally omitting the `Authorization` header when `api_key` is empty.

### Expected Features

**Must have (table stakes):**
- Provider entries for Ollama and LM Studio in `PROVIDERS` list and `Provider` enum — without this users cannot select them
- Dynamic model discovery from local APIs (`/api/tags` for Ollama, `/v1/models` for LM Studio) — users expect to see their downloaded models, not a hardcoded list
- Connection status indicator replacing API key validation — local servers may not be running; green/red dot is expected UX
- No API key required — presenting a key input for Ollama/LM Studio is actively confusing and blocks the flow
- Free usage display — `pricing_available: false` path already exists; local models show token counts without dollar amounts
- Base URL configuration — defaults to `localhost:11434` and `localhost:1234` but must be editable for non-default ports or remote machines
- Streaming chat completions — reuse existing adapter; functionally identical to cloud providers
- Provider SVG icons — visual consistency with existing 5 providers

**Should have (differentiators):**
- Model metadata in labels (parameter size + quantization) — "Llama 3.2 3B Q4_K_M" is far more useful than `llama3.2:3b-instruct-q4_K_M`
- Auto-tier heuristic for model selector — use parameter count from metadata: 7B or fewer = fast, 8–20B = balanced, above 20B = capable
- Server-not-running guidance — "Ollama is not running. Start it with `ollama serve`" is far more useful than "Network error"
- Connection auto-retry on overlay open — detect server startup without requiring app restart or manual reconnect
- Longer timeout (120s) for local providers — model cold-start on consumer hardware takes 10–90 seconds

**Defer to v2+:**
- Context window detection via `POST /api/show` (Ollama) and `max_context_length` (LM Studio) — use 4096 default for v1
- Model metadata label enrichment — can ship with raw model IDs and improve labels in a follow-up
- Pre-warming models on provider selection — optimization, not blocker

### Architecture Approach

The integration follows a clean extension pattern on the existing provider abstraction. A `Provider::is_local() -> bool` method gates all branching, keeping the common paths (streaming, token tracking, usage stats) unchanged. A new `ProviderConfig { base_url: String }` struct stored in `settings.json` replaces the hard-coded `&'static str` returned by `api_url()` for local providers. The `AppState` struct gains a `provider_configs: Mutex<HashMap<String, ProviderConfig>>` field. Three new IPC commands (`check_local_provider`, `set_provider_config`, `get_provider_config`) expose local provider management to the frontend. The frontend replaces the API key validation gate with a connection status gate for local providers, and adapts `AccountTab` and the onboarding wizard to show URL config and connection status instead of key inputs.

**Major components:**
1. `Provider` enum (`providers/mod.rs`) — gains `Ollama` and `LMStudio` variants with `is_local()`, `default_base_url()`, `requires_api_key()`, and 120s `default_timeout_secs()`
2. `ProviderConfig` + `AppState` field — stores configurable base URLs, loaded from `settings.json` on startup, updated via IPC
3. `openai_compat::stream()` — modified to accept explicit `api_url: &str` and skip `Authorization` header when key is empty; all existing callers pass URL explicitly
4. `models::fetch_api_models()` — new branches for Ollama (`/api/tags`) and LM Studio (`/v1/models`) with new `OllamaTagsResponse` deserialization types
5. Frontend store (`store/index.ts`) — new `localProviderStatus` and `localProviderBaseUrl` state fields; `submitQuery` checks connection status for local providers instead of API key status
6. `AccountTab.tsx` + `StepApiKey.tsx` — conditional rendering based on `provider.local` flag: URL input + connection dot instead of API key input

### Critical Pitfalls

1. **API key gate blocks local providers entirely** — `stream_ai_response` unconditionally reads a keychain entry and fails with "No API key configured"; `submitQuery` gates on `apiKeyStatus !== "valid"`. Fix: add `Provider::requires_api_key()`, skip keychain read for local providers, auto-set `apiKeyStatus` to "valid" when health check succeeds.

2. **Onboarding has no path for keyless providers** — `StepApiKey` requires `apiKeyStatus === "valid"` to enable Next; a user selecting Ollama is permanently stuck at step 1. Fix: skip `StepApiKey` for local providers using the same conditional pattern as the Windows Accessibility skip; replace with a connection check step.

3. **Cold-start timeout kills first request** — the existing 30s timeout is fatal for cold local model loading (10–90s). Fix: `default_timeout_secs()` returns 120 for local providers. Note: the outer `tokio::time::timeout` wraps the entire stream, not per-chunk idle time — may need to raise to 300s or refactor to idle-timeout for very large models.

4. **Connection refused misdiagnosed as auth failure or network error** — `reqwest` connection-refused errors hit the generic "Check your internet connection" path. Fix: match on `err.is_connect()` before `handle_http_status` and emit provider-specific "not running" messages with actionable instructions.

5. **Wrong Ollama API endpoint silently breaks token tracking** — Ollama's native `/api/chat` uses NDJSON (not SSE) and different field names (`eval_count` vs `completion_tokens`). Fix: always use `/v1/chat/completions`; add a code comment explaining this choice to prevent future regressions.

6. **Model tier grouping unusable with dynamic local models** — all local models land in a flat unsorted list; 10+ installed models become unmanageable. Fix: auto-tier by parsing `parameter_size` from Ollama metadata (7B or fewer = fast, 8–20B = balanced, above 20B = capable); default to "balanced" when unknown.

## Implications for Roadmap

Based on the combined research, a 5-phase build ordered by hard dependency is recommended. Each phase has a clean, verifiable exit condition before the next phase begins.

### Phase 1: Provider Abstraction Foundation

**Rationale:** Everything else depends on the `Provider` enum having Ollama and LMStudio variants. Rust's exhaustive match enforcement means the codebase will not compile with partial enum additions — all match arms must be updated together. This phase also resolves the two most critical pitfalls (API key gate, hardcoded URLs) before any feature work begins.

**Delivers:** `Ollama` and `LMStudio` variants in `Provider` enum; `is_local()`, `requires_api_key()`, `default_base_url()` methods; `ProviderConfig` struct; `provider_configs` in `AppState`; `settings.json` persistence for base URLs; three new IPC commands registered; all existing `match provider` arms updated; conditional keychain skip in `stream_ai_response`; provider-specific connection error messages.

**Addresses:** Base URL configuration (table stakes), no API key required (table stakes)

**Avoids:** API key gate (Pitfall 1), hardcoded URLs (Pitfall 2), connection error misdiagnosis (Pitfall 4), exhaustive match panics

### Phase 2: Model Discovery

**Rationale:** Users must select a model before streaming can be invoked. Depends on Phase 1 for the `Provider` enum and `ProviderConfig`. This phase also resolves the model tier usability pitfall.

**Delivers:** New `OllamaTagsResponse` deserialization types; Ollama branch in `fetch_api_models()` calling `/api/tags`; LM Studio branch calling `/v1/models`; auto-tier heuristic based on parameter count from metadata; `curated_models()` returns empty vec for local providers; model labels composed from family, parameter size, and quantization.

**Addresses:** Dynamic model discovery (table stakes), model metadata in labels (differentiator), auto-tier grouping (differentiator)

**Avoids:** Model tier unusability (Pitfall 6)

### Phase 3: Streaming Integration

**Rationale:** The query path itself. Depends on Phase 1 (enum + config) and Phase 2 (model selection). The change to `openai_compat::stream()` is small but touches three call sites and must be done as an atomic change to keep the build passing.

**Delivers:** `openai_compat::stream()` accepts explicit `api_url: &str` parameter; all three call sites in `ai.rs` updated to pass URL; Ollama/LM Studio stream via `{base_url}/v1/chat/completions` with no Authorization header; 120s timeout for local providers; token usage tracking works unchanged via existing `stream_options.include_usage`; cost display returns `pricing_available: false` for local queries.

**Addresses:** Streaming chat completions (table stakes), longer timeout (differentiator), free usage display (table stakes)

**Avoids:** Cold-start timeout (Pitfall 3), wrong API endpoint (Pitfall 5)

### Phase 4: Frontend Store + Provider List

**Rationale:** Frontend wiring should connect to working IPC commands, not stubs. Depends on Phases 1–3.

**Delivers:** Ollama and LM Studio added to `PROVIDERS` array with `local: true` flag; `localProviderStatus` and `localProviderBaseUrl` state fields in Zustand store; `submitQuery` checks connection status for local providers; health check invoked on local provider selection; Ollama and LM Studio SVG icons in `ProviderIcon.tsx`.

**Addresses:** Provider entries in PROVIDERS list (table stakes), connection status (table stakes), provider icons (table stakes)

**Avoids:** `apiKeyStatus` gate permanently blocking local provider queries

### Phase 5: Settings + Onboarding UI

**Rationale:** UI polish phase. Depends on all backend integration being complete and testable. Resolves the onboarding pitfall that would leave users unable to complete first-run setup with a local provider selected.

**Delivers:** `AccountTab.tsx` conditionally renders base URL input + connection dot for local providers; `StepApiKey.tsx` skipped for local providers using the Windows Accessibility skip pattern; `OnboardingWizard.tsx` adds connection check step for local providers; `StepProviderSelect.tsx` visually distinguishes local vs. cloud providers; server-not-running guidance with actionable instructions per provider; usage stats show "Free (local)" instead of "$0.00".

**Addresses:** Connection status indicator (table stakes), server-not-running guidance (differentiator), onboarding adaptation

**Avoids:** Onboarding blocked on API key (Pitfall 2), "Free (local)" vs "$0.00" UX confusion

### Phase Ordering Rationale

- Phase 1 must be first because adding new `Provider` enum variants without updating all match arms breaks compilation immediately; it is an all-or-nothing change per Rust's exhaustiveness rules.
- Phase 2 before Phase 3 because the streaming command requires a model ID to be passed in — building model discovery first creates a testable end-to-end path from provider selection through model list before streaming is wired up.
- Phase 3 before Phases 4 and 5 because UI state changes should wire to known-working IPC commands rather than stub responses.
- Phases 4 and 5 are sequenced frontend-last because UI behavior is easier to validate against a working backend, and UI is the layer most likely to require iterative adjustment during implementation.

### Research Flags

Phases with well-documented patterns (skip research-phase):
- **Phase 1:** Mechanical enum extension; all code locations identified and verified by direct codebase inspection. The conditional keychain skip pattern is straightforward.
- **Phase 2:** Both API response formats are fully documented. `OllamaTagsResponse` struct fields confirmed by Ollama docs. `OpenAIModelsResponse` already exists in the codebase for LM Studio reuse.
- **Phase 3:** SSE format compatibility is HIGH confidence from official docs. The `openai_compat::stream()` signature change is minimal and all three call sites are known.
- **Phase 4:** Zustand store pattern is established; no novel patterns required.

Phases that may benefit from implementation-time investigation:
- **Phase 3 (timeout architecture):** The outer `tokio::time::timeout` wraps the entire stream rather than resetting on each token chunk. For large local models generating long responses slowly, 120s may be insufficient. The implementation phase should decide between: (a) raising to 300s for local, (b) adding per-chunk idle timeout logic, or (c) accepting the limitation in v1 with a user-facing note.
- **Phase 5 (onboarding wizard):** The conditional step-skip pattern exists (Windows Accessibility skip at lines 30–32 of `OnboardingWizard.tsx`) but the new "ConnectionCheck" step component is net-new. Verify that `apiKeyStatus` is set to "valid" when connection succeeds so all downstream `submitQuery` gates are satisfied.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Both providers' API formats verified via official docs; no new dependencies confirmed by direct Cargo.toml inspection |
| Features | HIGH | Table stakes derived from official API docs + direct codebase analysis; LM Studio `/v1/models` exact field mapping is MEDIUM (inferred from OpenAI-compat claim, not field-by-field live-verified) |
| Architecture | HIGH | Based on direct codebase analysis of all files that need changing; integration points are specific and code-level with line references |
| Pitfalls | HIGH | Most pitfalls traced to specific line numbers in the codebase; community sources for timeout values are MEDIUM but the 120s recommendation is conservative and safe |

**Overall confidence:** HIGH

### Gaps to Address

- **LM Studio `/v1/models` exact field mapping:** Confirmed as OpenAI-compat but individual fields (`quantization`, `arch`, `state`, `max_context_length`) should be verified against a real LM Studio instance during Phase 2 implementation before finalizing the deserialization struct.

- **Outer streaming timeout vs. idle timeout:** The `tokio::time::timeout` wrapping the entire `openai_compat::stream()` call does not reset on token arrival. For local models generating long responses slowly, this may cause spurious timeouts. Decision point at Phase 3: keep simple 120s (acceptable for v1) or refactor to per-chunk idle timeout.

- **`context_window_for_model()` prefix matching for local models:** The existing function uses a prefix table (e.g., `"gpt-4"`, `"claude-"`) to look up context windows. Ollama model names like `llama3.2:3b-instruct-q4_K_M` will not match any existing prefix and fall through to the 128K default. Safe as a fallback but may over-allocate context for small models. Optionally add common local model family prefixes (`llama`, `mistral`, `qwen`, `deepseek`, `phi`, `gemma`) as a post-Phase-3 enhancement.

## Sources

### Primary (HIGH confidence)
- [Ollama API documentation](https://github.com/ollama/ollama/blob/main/docs/api.md) — endpoint formats, authentication, streaming
- [Ollama OpenAI compatibility](https://docs.ollama.com/api/openai-compatibility) — SSE format, stream_options, /v1/ endpoints
- [Ollama /api/tags endpoint](https://docs.ollama.com/api/tags) — model listing format, parameter_size, quantization_level fields
- [Ollama health check issue #1378](https://github.com/ollama/ollama/issues/1378) — GET / returns "Ollama is running" confirmed
- [Ollama stream_options PR #6784](https://github.com/ollama/ollama/issues/5200) — include_usage support confirmed merged Dec 2024
- [LM Studio OpenAI compatibility endpoints](https://lmstudio.ai/docs/developer/openai-compat) — SSE format, stream_options, /v1/ endpoints
- [LM Studio model listing](https://lmstudio.ai/docs/developer/openai-compat/models) — /v1/models response format
- [LM Studio server docs](https://lmstudio.ai/docs/developer/core/server) — default port 1234, manual server start required
- CMD+K codebase — direct analysis of `providers/mod.rs`, `openai_compat.rs`, `ai.rs`, `models.rs`, `state.rs`, `keychain.rs`, `store/index.ts`, `AccountTab.tsx`, `OnboardingWizard.tsx`, `StepApiKey.tsx`

### Secondary (MEDIUM confidence)
- [LM Studio REST API v0 endpoints](https://lmstudio.ai/docs/developer/rest/endpoints) — model state, max_context_length fields (exact field mapping inferred from OpenAI-compat claim)
- [Cline Ollama timeout fix](https://localllm.in/blog/cline-ollama-timeout-fix) — 90–120s recommended for mid-range hardware cold starts
- [AnythingLLM Ollama troubleshooting](https://docs.useanything.com/ollama-connection-troubleshooting) — connection error patterns

### Tertiary (LOW confidence)
- None — all key claims are verified at HIGH or MEDIUM.

---
*Research completed: 2026-03-17*
*Ready for roadmap: yes*
