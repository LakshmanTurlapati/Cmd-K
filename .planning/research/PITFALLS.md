# Pitfalls Research

**Domain:** Adding local LLM providers (Ollama, LM Studio) to a cloud-first multi-provider AI terminal overlay
**Researched:** 2026-03-17
**Confidence:** HIGH

## Critical Pitfalls

### Pitfall 1: API Key Gate Blocks Local Providers Entirely

**What goes wrong:**
The `stream_ai_response` command unconditionally reads an API key from the OS Keychain on line 206-213 of `ai.rs` and fails with "No API key configured" if none is found. Ollama and LM Studio require no authentication. Every call to `save_api_key`, `get_api_key`, `validate_api_key`, and `delete_api_key` assumes a keychain entry exists per provider. The frontend's `submitQuery` in `store/index.ts` also gates on `apiKeyStatus !== "valid"` (line 446) before allowing any query, meaning local providers will be permanently blocked unless this gate is bypassed.

**Why it happens:**
The entire auth flow was designed for cloud providers where an API key is the entry requirement. `Provider::keychain_account()` returns a `&'static str` for each provider, and every method on `Provider` assumes a key exists. This is deeply embedded -- the onboarding wizard's `StepApiKey` requires a valid key to proceed (line 113: `canProceed = apiKeyStatus === "valid"`).

**How to avoid:**
Introduce a `Provider::requires_api_key()` method returning `bool` (false for Ollama/LM Studio). In `stream_ai_response`, skip the keychain read for local providers and pass an empty string as the api_key (Ollama ignores the Authorization header; LM Studio likewise). In the frontend, auto-set `apiKeyStatus` to "valid" for local providers or introduce a new status like "not_required". The `validate_api_key` command should return Ok immediately for local providers (validate connectivity instead via a health check).

**Warning signs:**
- "No API key configured" error when trying to use Ollama/LM Studio
- Users unable to proceed past onboarding step 1 with local provider selected
- `apiKeyStatus` stuck at "unknown" for local providers

**Phase to address:**
Provider Abstraction phase (first phase) -- this is the foundational change that unblocks everything else.

---

### Pitfall 2: Hardcoded Remote URLs Make Local Providers Unreachable

**What goes wrong:**
`Provider::api_url()` returns `&'static str` for each provider (line 40-48 of `providers/mod.rs`). Ollama uses `http://localhost:11434/v1/chat/completions` and LM Studio uses `http://localhost:1234/v1/chat/completions`. These are not only different from cloud URLs, but users may need to change the port or use a remote machine IP. A static string cannot accommodate configurable base URLs.

**Why it happens:**
Cloud providers have permanent, well-known API endpoints. Local providers run on user-controlled machines where the port might be customized, the server might be on another host, or the default port might be occupied by another service.

**How to avoid:**
For Ollama and LM Studio, store the base URL in `settings.json` (Tauri store) with defaults of `http://localhost:11434` and `http://localhost:1234`. Change `api_url()` to either accept a base URL parameter or have `stream_ai_response` construct the full URL from stored settings for local providers. Do NOT change the existing cloud providers' static URLs -- only add the dynamic path for local providers. The endpoint path (`/v1/chat/completions`) is constant and can be appended at request time.

**Warning signs:**
- Connection refused errors on non-standard ports
- Users running Ollama on a remote machine unable to connect
- "Network error" messages when Ollama is running but on a different port

**Phase to address:**
Provider Abstraction phase -- base URL configuration is needed before streaming can work.

---

### Pitfall 3: Model Cold-Start Timeout Kills First Request

**What goes wrong:**
Ollama unloads models from GPU/RAM after 5 minutes of inactivity by default. When a model is cold (not loaded), the first request must load it into memory before generating tokens. For a 7B model this takes 5-15 seconds; for 13B+ models, 30-90+ seconds. The existing `Provider::default_timeout_secs()` returns 10s for xAI and 30s for everything else. A 30-second timeout will kill legitimate cold-start requests for medium and large models, producing "Request timed out. Try again." errors that look like the provider is broken.

**Why it happens:**
Cloud APIs never have cold-start delays because the model is always loaded server-side. The 30-second timeout is generous for network latency but far too short for local model loading. Users will not understand why the first request fails but retries succeed (because the model is now loaded after the first attempt triggered loading).

**How to avoid:**
Set local provider timeouts to 120 seconds minimum (covers loading 13B models on mid-range hardware). This is implemented in `Provider::default_timeout_secs()` which feeds the `tokio::time::timeout` wrapper in `stream_ai_response`. For LM Studio, models are typically loaded manually by the user before querying, so 120s is conservative but safe. Do NOT try to detect cold-start and adjust dynamically -- that adds complexity for marginal benefit.

**Warning signs:**
- First query after idle period always fails with timeout
- Retrying the same query immediately succeeds
- Users report "it works on second try every time"

**Phase to address:**
Streaming Adapter phase -- timeout values must be set correctly when building the request.

---

### Pitfall 4: Token Usage Tracking Breaks on Wrong API Endpoint

**What goes wrong:**
The `openai_compat::stream` adapter (line 74-78 of `openai_compat.rs`) extracts token usage from the final streaming chunk via `chunk["usage"]["prompt_tokens"]` and `chunk["usage"]["completion_tokens"]`. This works perfectly with Ollama's OpenAI-compatible `/v1/chat/completions` endpoint, which returns the same field names and supports `stream_options: { include_usage: true }` (confirmed merged in PR #6784). However, if someone accidentally uses Ollama's *native* `/api/chat` endpoint, the response is newline-delimited JSON (not SSE), uses different field names (`prompt_eval_count` / `eval_count`), and the `eventsource_stream` parser will fail silently.

**Why it happens:**
Ollama's documentation prominently features the native `/api/chat` endpoint. The OpenAI-compatible `/v1/` path is a secondary feature. Developers searching for "Ollama API" will find the native endpoint first.

**How to avoid:**
Always use the `/v1/chat/completions` endpoint for both Ollama and LM Studio -- this is the one compatible with the existing `openai_compat::stream` adapter without any code changes. The existing `stream_options: { include_usage: true }` in the request body (line 25 of `openai_compat.rs`) works with both providers' OpenAI-compat layers. Document this choice prominently in code comments so future developers do not "optimize" by switching to the native API.

**Warning signs:**
- Token counts always showing 0 or None for local providers
- Parsing errors or empty streams in adapter logs
- `eval_count` field names appearing in debug output instead of `completion_tokens`

**Phase to address:**
Streaming Adapter phase -- verify endpoint path is `/v1/chat/completions` for both providers.

---

### Pitfall 5: Onboarding Flow Has No Path for Keyless Providers

**What goes wrong:**
The onboarding wizard steps are: Provider (step 0) -> API Key (step 1) -> Model (step 2) -> Accessibility (step 3) -> Done (step 4). The `StepApiKey` component requires `apiKeyStatus === "valid"` to enable the Next button. If a user selects Ollama in step 0, they hit step 1 which shows "Enter your Ollama API key" -- a concept that does not exist. They cannot proceed. Even if they use "Skip this step", `apiKeyStatus` remains "unknown", and `submitQuery` refuses to send any query (line 446 of store/index.ts).

**Why it happens:**
The onboarding was designed assuming every provider needs an API key. The 5-step linear wizard has no conditional branching based on provider type. The only existing conditional skip is for Accessibility on Windows (line 31 of OnboardingWizard.tsx).

**How to avoid:**
When a local provider is selected in step 0, skip step 1 entirely using the same pattern as the Windows Accessibility skip (line 30-32 of `OnboardingWizard.tsx`). Replace with a "Connection" step for local providers that validates connectivity to the local server (HTTP GET to the base URL -- Ollama returns "Ollama is running" on GET /). On success, set `apiKeyStatus` to "valid" to satisfy the downstream gate. The `PROVIDERS` array in `store/index.ts` needs to be extended with Ollama and LM Studio entries.

**Warning signs:**
- "Enter your Ollama API key" text appearing in onboarding
- Users unable to get past step 1 with local provider selected
- "No API key configured" error on first query attempt after completing onboarding

**Phase to address:**
UI Integration phase -- onboarding and settings panels must handle keyless providers.

---

### Pitfall 6: Connection Refused Misdiagnosed as Auth Failure

**What goes wrong:**
Cloud providers are always reachable (barring network outage). Local providers can be: not installed, installed but not running, running but no models loaded, running with models loaded. The current `handle_http_status` function (line 91-109 of `providers/mod.rs`) handles 401 as "Authentication failed. Check your API key." and all other status codes generically. But connection refused errors from `reqwest` (the server is not running) never reach `handle_http_status` -- they are caught by the `.send().await.map_err()` call which produces "Network error: Check your internet connection." Neither message is helpful for local providers.

**Why it happens:**
Cloud providers never produce connection refused errors because DNS resolution succeeds and TLS handshake happens first. The error handling was designed for cloud failure modes (401 invalid key, 429 rate limit, 500 server error). Local providers introduce a new failure mode: the server process is simply not running.

**How to avoid:**
For local providers, match on the `reqwest::Error` kind before calling `handle_http_status`. If the error is a connection error (`.is_connect()`), produce a provider-specific message: "Ollama is not running. Start it with `ollama serve` and try again." or "LM Studio server is not running. Start it in LM Studio's Developer tab." For non-connection errors, fall through to the existing generic handling. Critically, never show "Check your API key" for providers that do not use API keys.

**Warning signs:**
- "Network error: Check your internet connection" when Ollama is simply not started
- "Authentication failed" for a provider that needs no auth
- Users thinking their internet is down when the issue is a local process

**Phase to address:**
Provider Abstraction phase (error handling) and UI Integration phase (status indicator).

---

### Pitfall 7: Model Tier Grouping Unusable With Dynamic Local Models

**What goes wrong:**
The existing model system uses three hardcoded tiers: "fast", "balanced", "capable" (line 12 of `models.rs`). Cloud providers have curated lists where each model is manually assigned a tier. Ollama and LM Studio models are dynamically discovered via their APIs, and the list changes as users pull/remove models. These local models have no tier assignment and wildly varying capabilities (a 1B quantized model is very different from a 70B full-precision model). Putting all local models into an empty-tier uncategorized bucket makes the model selector unusable when users have 10+ models installed.

**Why it happens:**
Tier assignment was a manual process during `curated_models()` curation. There is no automated way to classify an arbitrary model like `deepseek-r1:8b-distill-q4_K_M` into a meaningful tier.

**How to avoid:**
Use parameter count as a heuristic for auto-tiering: models with 7B or fewer parameters map to "fast", 8B-20B to "balanced", greater than 20B to "capable". Ollama's `/api/tags` response includes model details with `parameter_size` field (e.g., "8.0B"). For models where parameter count cannot be determined from metadata, parse it from the model name (most Ollama model names include the parameter count like `llama3.2:3b`). Default to "balanced" if no signal is available.

**Warning signs:**
- All local models appearing in a single unsorted list
- Users unable to find the model they want among many entries
- No visual distinction between a 1B and 70B model in the dropdown

**Phase to address:**
Model Discovery phase -- auto-tiering logic should be part of the model fetch pipeline.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Storing base URL as static str in Provider enum | No settings persistence needed | Cannot configure custom ports or remote hosts for local providers | Never for local providers -- they must have configurable base URLs |
| Reusing `apiKeyStatus: "valid"` for local providers instead of new state | No new frontend state fields | Confuses "has valid key" with "server is reachable"; makes debugging harder | MVP only -- replace with proper `connectionStatus` in a follow-up |
| Skipping auto-tier for local models | Ship faster, all models in flat list | Unusable model selector with many installed models | Acceptable if users typically have fewer than 5 local models |
| Hardcoding 120s timeout for all local requests | Covers cold-start for large models | Wastes time on genuine failures (server crashed mid-request) | Acceptable -- cold-start detection adds complexity for marginal benefit |
| Not pre-warming models on provider selection | No background HTTP activity | First query always slow if model is cold | Acceptable for v1 -- pre-warming is an optimization |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Ollama native API vs /v1/ compat | Using `/api/chat` (ndjson format, different field names `eval_count`) | Always use `/v1/chat/completions` (SSE, OpenAI-compatible `completion_tokens`) |
| Ollama model names | Assuming model IDs are simple names (e.g., `llama3`) | Use the full model name including tag from `/api/tags` response (e.g., `llama3.2:3b-instruct-q4_K_M`) |
| LM Studio server lifecycle | Assuming server auto-starts like Ollama on macOS | LM Studio requires user to manually start the local server in the Developer tab; it does not auto-start on app launch |
| Ollama platform lifecycle | Assuming same lifecycle on all OSes | macOS: menu bar app (auto-starts). Windows: background service. Linux: systemd service. None has a tray icon by default. |
| `stream_options` support | Worrying that `stream_options: { include_usage: true }` is unsupported | Both Ollama (PR #6784, merged) and LM Studio support this in their /v1/ compat layer |
| CORS from WebView | Making HTTP requests to localhost from the frontend JavaScript | CMD+K already makes all provider requests from Rust (`reqwest`) which bypasses CORS entirely -- do NOT move requests to frontend |
| Authorization header to local providers | Sending `Bearer <empty>` header to Ollama | Skip the Authorization header entirely for local providers; Ollama ignores it but it is unnecessary noise in logs |
| Context window lookup | `context_window_for_model()` has no prefix matches for local model names like `llama3.2:3b` | Add prefix patterns for common local model families (llama, mistral, qwen, deepseek, phi, gemma) or use the 128K default as a safe fallback |
| Provider enum exhaustive matches | Adding Ollama/LMStudio to `Provider` enum without updating ALL match arms | Every `match provider` block in `mod.rs`, `ai.rs`, `models.rs`, `keychain.rs` must handle the new variants or use `_` wildcard |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Model loading on every query | 10-60 second delay on first query after 5 min idle | Document Ollama `OLLAMA_KEEP_ALIVE` setting; consider adding app-level tip on first timeout | Every time user waits > 5 minutes between queries |
| Large model on limited hardware | Tokens stream at 1-2 tokens/sec; overlay appears frozen | Show streaming text character by character (already done); the real risk is that users think the app is broken | When users load 70B+ models on 8GB RAM machines |
| Model list fetch on every settings open | HTTP call to local server blocks UI thread | Cache model list in AppState with a 30-second TTL; invalidate on provider change | Not a scale issue but a UX responsiveness issue |
| Concurrent requests to Ollama | Ollama queues requests by default (single inference slot) | The app already disables submit during streaming; no concurrent request issue | Only if architecture changes to allow parallel requests |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Base URL defaults to non-localhost | Anyone on network can intercept prompts and responses | Default base URL must be `http://localhost:PORT`; if user configures non-localhost, show a warning that traffic is unencrypted |
| Storing base URL with embedded credentials | `http://user:pass@host:port` persists in plaintext `settings.json` | Validate base URL format; reject URLs containing `@` (userinfo component) |
| Sending terminal context to local models unfiltered | Local models are less reliable at following system prompt constraints; could echo sensitive env vars | The existing `filter_sensitive()` pipeline in `ai.rs` line 139 runs regardless of provider; no change needed |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No indication that Ollama/LM Studio is not running | User submits query, waits 30+ seconds, gets cryptic error | Show connection status indicator (green/red dot) in overlay when local provider is selected |
| Identical onboarding for cloud and local providers | User asked for API key that does not exist | Conditional onboarding: local providers get "Connection Check" step instead of "API Key" step |
| Cost display shows $0.00 for local models | Technically correct but misleading -- suggests tracking is broken | Show "Free (local)" text instead of "$0.00" for local providers in usage stats |
| Model names are opaque (e.g., `qwen2.5:7b-instruct-q5_K_M`) | Users do not know which model to pick | Parse model names into structured labels: family, parameter count, quantization as secondary info |
| No visual distinction between cloud and local providers in selector | Users cannot tell which providers need internet vs work offline | Add "Local" badge or section header in the provider selector; group local providers separately |
| Slow token generation not communicated | User thinks the app is frozen during slow local inference | The existing streaming display handles this (tokens appear one by one), but an initial "Model loading..." state for cold starts would help |

## "Looks Done But Isn't" Checklist

- [ ] **Streaming works with real model:** Verify both Ollama AND LM Studio actually stream tokens via SSE -- test with a real loaded model, not just a health check
- [ ] **Token tracking in streaming:** Confirm `stream_options: { include_usage: true }` returns `prompt_tokens`/`completion_tokens` in the final SSE chunk for both providers
- [ ] **Model list populates:** Verify `/api/tags` (Ollama) and `/v1/models` (LM Studio) both parse correctly and populate the model dropdown with correct model IDs
- [ ] **Connection refused handling:** Verify that stopping Ollama/LM Studio produces a user-friendly error, not a generic "Network error" or "Check your API key"
- [ ] **Cold-start timeout:** Test with a model NOT currently loaded (`ollama stop <model>` first) -- verify the request survives the model loading phase without timing out
- [ ] **Onboarding path:** Walk through the full onboarding wizard selecting Ollama -- verify no step blocks on API key entry
- [ ] **Settings provider switch:** Switch from a cloud provider to Ollama and back in Settings -- verify state (apiKeyStatus, models, base URL) resets correctly for each type
- [ ] **Cross-platform localhost:** Test on macOS, Windows (including WSL), AND Linux -- verify `http://localhost:11434` resolves correctly on all three
- [ ] **Usage display:** Verify local provider queries show "Free (local)" rather than "$0.00" or a broken pricing indicator
- [ ] **Model selector with many models:** Install 10+ models in Ollama and verify the model selector remains usable (scrollable, grouped by tier)
- [ ] **Provider enum exhaustiveness:** Compile and verify all `match provider` blocks handle Ollama and LMStudio variants without panics

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| API key gate blocking local providers | LOW | Add `requires_api_key()` method, skip keychain read, update frontend gate condition |
| Hardcoded URLs | LOW | Add base_url to settings store, construct URL at request time for local providers |
| Cold-start timeout | LOW | Change `default_timeout_secs()` return value for local provider variants to 120 |
| Token tracking None values | LOW | Verify `/v1/` endpoint is used and `stream_options` is in request body; existing adapter handles the format |
| Model tier grouping | MEDIUM | Parse parameter count from model metadata or name, build auto-tier heuristic |
| Onboarding flow | MEDIUM | Add conditional step skip logic (same pattern as Windows Accessibility skip), create ConnectionStep component |
| Connection error messages | MEDIUM | Match on `reqwest::Error::is_connect()`, produce provider-specific "not running" message |
| Cost display | LOW | Check if provider is local, show "Free (local)" text in usage stats UI |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| API key gate (Pitfall 1) | Provider Abstraction | Ollama selected, query submitted without any key -- succeeds |
| Hardcoded URLs (Pitfall 2) | Provider Abstraction | Custom port configured in settings, connection succeeds |
| Cold-start timeout (Pitfall 3) | Streaming Adapter | Cold model (not in `ollama ps`) queried, completes without timeout error |
| Wrong API endpoint (Pitfall 4) | Streaming Adapter | Token counts appear in usage stats after local provider query |
| Onboarding flow (Pitfall 5) | UI Integration | Fresh install, select Ollama, complete onboarding without API key prompt |
| Connection error messages (Pitfall 6) | Provider Abstraction + UI | Ollama stopped, error says "not running" not "check your API key" |
| Model tier grouping (Pitfall 7) | Model Discovery | 10+ Ollama models installed, all appear grouped by tier in model selector |

## Sources

### Verified via Official Documentation (HIGH confidence)
- [Ollama OpenAI Compatibility Docs](https://docs.ollama.com/api/openai-compatibility) -- /v1/ endpoint, SSE streaming, stream_options support confirmed
- [Ollama Usage/Token Docs](https://docs.ollama.com/api/usage) -- token fields in final streaming chunk
- [Ollama /api/tags Docs](https://docs.ollama.com/api/tags) -- model listing endpoint, response includes parameter_size in details
- [Ollama FAQ](https://docs.ollama.com/faq) -- OLLAMA_HOST, OLLAMA_ORIGINS, OLLAMA_KEEP_ALIVE, platform-specific configuration
- [Ollama stream_options PR #6784](https://github.com/ollama/ollama/issues/5200) -- confirmed merged and released
- [Ollama cold-start issue #1769](https://github.com/ollama/ollama/issues/1769) -- model loading delay on first request
- [Ollama timeout issue #4350](https://github.com/ollama/ollama/issues/4350) -- configurable model loading timeout requests
- [Ollama CORS + Tauri issue #2291](https://github.com/ollama/ollama/issues/2291) -- tauri:// origin not supported, Rust-side requests bypass CORS
- [LM Studio OpenAI Compat Docs](https://lmstudio.ai/docs/developer/openai-compat) -- /v1/chat/completions, SSE format with data: prefix and [DONE] sentinel, stream_options.include_usage support
- [LM Studio Models Endpoint](https://lmstudio.ai/docs/developer/openai-compat/models) -- /v1/models response format with model metadata
- [LM Studio Server Docs](https://lmstudio.ai/docs/developer/core/server) -- default port 1234, manual server start required

### CMD+K Codebase Analysis (HIGH confidence)
- `src-tauri/src/commands/providers/mod.rs` -- Provider enum, api_url(), keychain_account(), handle_http_status(), adapter_kind()
- `src-tauri/src/commands/ai.rs` -- stream_ai_response command, keychain read on line 206-213, message building
- `src-tauri/src/commands/providers/openai_compat.rs` -- SSE streaming, stream_options in request body, usage extraction
- `src-tauri/src/commands/models.rs` -- curated_models(), ModelWithMeta tier field, fetch_models(), validate_api_key()
- `src-tauri/src/state.rs` -- TokenUsage, UsageAccumulator, AppState
- `src-tauri/src/commands/usage.rs` -- get_usage_stats, pricing lookup, cost calculation
- `src-tauri/src/commands/keychain.rs` -- save/get/delete_api_key, all require Provider with keychain_account()
- `src-tauri/src/terminal/context.rs` -- context_window_for_model() prefix matching, CONTEXT_WINDOWS table
- `src/store/index.ts` -- PROVIDERS array, apiKeyStatus gate in submitQuery, streaming flow
- `src/components/Onboarding/OnboardingWizard.tsx` -- 5-step wizard, Windows Accessibility skip pattern
- `src/components/Onboarding/StepApiKey.tsx` -- canProceed requires valid API key
- `src/components/Onboarding/StepProviderSelect.tsx` -- provider selection from PROVIDERS array

### Community Sources (MEDIUM confidence)
- [Ollama keep-alive configuration](https://markaicode.com/ollama-keep-alive-memory-management/) -- OLLAMA_KEEP_ALIVE=-1 for permanent loading
- [Cline Ollama timeout fix](https://localllm.in/blog/cline-ollama-timeout-fix) -- recommended 90-120s timeout for mid-range hardware
- [AnythingLLM Ollama troubleshooting](https://docs.useanything.com/ollama-connection-troubleshooting) -- connection debugging patterns

---
*Pitfalls research for: Adding local LLM providers (Ollama, LM Studio) to CMD+K cloud-first multi-provider system*
*Researched: 2026-03-17*
