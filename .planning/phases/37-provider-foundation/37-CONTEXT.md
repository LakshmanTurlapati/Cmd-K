# Phase 37: Provider Foundation - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Add Ollama and LM Studio as new Provider enum variants. Users can select them, configure base URLs, and see connection status (checkmark when server reachable). No API key required. Health checks distinguish "server not running" from "no models loaded" from request errors. This phase covers the backend provider abstraction and the minimum frontend wiring to make selection functional — full settings UI and onboarding adaptation are Phase 40.

</domain>

<decisions>
## Implementation Decisions

### URL storage & persistence
- Store base URLs in Tauri preferences store (not keychain — URLs are not secrets)
- Show default URL as placeholder text (localhost:11434 for Ollama, localhost:1234 for LM Studio) — input is empty unless user customizes
- Accept both `host:port` and `http://host:port` formats — app normalizes internally (prepend http:// if no protocol)

### Health check timing
- Check server health on provider selection in settings AND on overlay open
- 2-second timeout for health check — localhost responds in <100ms if running
- If server unreachable when overlay opens, allow typing anyway — error on submit, not disabled input
- Health result surfaces as the same checkmark indicator used for API key validation

### Error messaging
- 3 distinct error states: "Server not running" / "No models loaded" / "Request failed — [details]"
- No start hints in error messages (no "run ollama serve") — just status
- Error messages use same styling as existing cloud provider errors

### Provider ordering in UI
- Mixed alphabetically with cloud providers — no "Local" group header
- No visual distinction (no "(Local)" suffix) — treat equally, icons are the differentiator

### Claude's Discretion
- Exact Provider enum method implementations (keychain_account returning a no-op or sentinel)
- How to handle api_url() returning dynamic URLs (currently &'static str — needs refactor for configurable base URLs)
- Whether to add is_local() / requires_api_key() helper methods on Provider
- Internal architecture for health check IPC command

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Provider abstraction (backend)
- `src-tauri/src/commands/providers/mod.rs` — Provider enum with 7 match blocks to extend (keychain_account, api_url, default_timeout_secs, display_name, console_url, adapter_kind) + handle_http_status fn
- `src-tauri/src/commands/providers/openai_compat.rs` — OpenAI-compat streaming adapter (both local providers use this). Currently hardcodes provider.api_url() and always sends Bearer auth header
- `src-tauri/src/commands/keychain.rs` — save/get/delete_api_key functions that unconditionally call provider.keychain_account()
- `src-tauri/src/commands/models.rs` — curated_models() with hardcoded per-provider model lists. Local providers return empty (dynamic-only, Phase 38)
- `src-tauri/src/commands/ai.rs` — stream_ai_response entry point that reads API key from keychain before streaming

### Frontend provider flow
- `src/components/Onboarding/StepProviderSelect.tsx` — Provider selection step in onboarding wizard
- `src/components/Onboarding/StepApiKey.tsx` — API key entry step (must be skipped for local providers, Phase 40)
- `src/components/Settings/AccountTab.tsx` — Provider switching and API key management in settings
- `src/store/index.ts` — Zustand store with provider state, apiKeyStatus, model selection

### Research findings
- `.planning/research/STACK.md` — API compatibility details, health check endpoints
- `.planning/research/FEATURES.md` — Feature scope, UX patterns for local providers
- `.planning/research/ARCHITECTURE.md` — Integration architecture, component boundaries
- `.planning/research/PITFALLS.md` — 7 critical pitfalls to avoid

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `openai_compat::stream()` — Handles SSE streaming for OpenAI, xAI, OpenRouter. Both Ollama and LM Studio use identical format. Needs URL parameterization and optional auth header.
- `handle_http_status()` — Provider-aware error messages. Extend with local-specific cases (connection refused, no models).
- `ModelWithMeta` struct — Already has optional pricing fields. Local models can set None for prices.
- `AdapterKind::OpenAICompat` — Both local providers map to this adapter.

### Established Patterns
- Provider enum is the central dispatch point — every provider behavior branches on match arms
- Keychain access is unconditional in ai.rs — must add bypass before streaming
- `curated_models()` returns empty vec for unknown providers — clean no-op for local providers
- Frontend PROVIDERS array in store defines the provider list — alphabetical insertion point

### Integration Points
- `stream_ai_response` in ai.rs — Must conditionally skip keychain read for local providers
- `validate_api_key` IPC command — Must handle local providers (health check instead of key validation)
- Zustand store `apiKeyStatus` — Local providers need "valid" mapped from health check success
- Tauri preferences store — Already available via tauri-plugin-store, used for hotkey config

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

*Phase: 37-provider-foundation*
*Context gathered: 2026-03-17*
