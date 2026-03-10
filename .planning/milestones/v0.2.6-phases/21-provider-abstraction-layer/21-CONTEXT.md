# Phase 21: Provider Abstraction Layer - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Rust backend with provider enum, per-provider streaming, parameterized keychain, and v0.2.4 migration. Users can generate commands from any of the 5 supported AI providers (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) with correct streaming, key storage, and error handling. Frontend UI for provider selection is Phase 22.

</domain>

<decisions>
## Implementation Decisions

### Provider API Normalization
- Build a common internal message format in Rust; each provider adapter translates to/from its native API
- Frontend stays unchanged — always sends the same IPC shape
- 3 streaming adapters: OpenAI-compatible (covers OpenAI, xAI, OpenRouter), Anthropic adapter, Google Gemini adapter
- Same system prompts across all providers; each adapter places the prompt in the correct API field (role:system for OpenAI-compat, top-level system field for Anthropic)
- Per-provider default timeouts (e.g., 10s for fast models, 30s for reasoning models) — no user-facing timeout setting
- All implementations must work cross-platform (macOS + Windows)

### Keychain Storage
- Separate keychain account per provider (e.g., account='openai_api_key', 'anthropic_api_key', etc.) under the same service name 'com.lakshmanturlapati.cmd-k'
- Uses `keyring` crate — works on both macOS Keychain and Windows Credential Manager
- Provider and model selection persisted in Tauri config file (JSON in app data dir), accessible from both Rust and frontend
- Frontend sends provider as an explicit IPC parameter to stream_ai_response alongside model, query, context, and history

### v0.2.4 Migration
- Read-on-first-launch: on first launch of v0.2.6, check if old 'xai_api_key' entry exists
- If yes, copy it to the new provider-keyed entry, set xAI as default provider
- Leave the old entry in place as backup
- Existing users see their xAI key preserved automatically with xAI as default

### Model List Sourcing
- Hybrid approach: ship hardcoded curated model lists per provider as defaults, plus a "Refresh models" capability that fetches from provider APIs
- Curated models get tier tags (Fast, Balanced, Most Capable) — hardcoded tier mapping based on known model characteristics
- All available models shown in a general/uncategorized section for power users, in addition to tiered curated models
- OpenRouter: model list fetched from their /api/v1/models endpoint, filtered to chat-capable models (this is the point of OpenRouter — access to many models)
- API key validation happens at save time only (lightweight API call when user enters/saves key), no re-validation on provider switch

### Error Handling
- Provider-specific error messages: include provider name + actionable hint (e.g., "Anthropic: Authentication failed. Check your API key at console.anthropic.com.")
- No automatic retry on rate limits — show "[Provider]: Rate limited. Wait a moment and try again."
- Mid-stream errors: keep partial response visible and append error indicator — user sees what was generated so far
- No pre-check connectivity — let API calls fail naturally, show "Network error: Check your internet connection."

### Claude's Discretion
- Provider enum structure and what data each variant carries
- Temperature and parameter differences across providers
- OpenRouter-specific headers (HTTP-Referer, app name)
- Exact adapter code organization (separate files vs single module)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `keyring` crate: Already used in `commands/keychain.rs` for platform keychain access — reuse for all providers
- `eventsource_stream` + `futures_util::StreamExt`: SSE streaming in `commands/ai.rs` — reuse for OpenAI-compatible adapter
- `tauri_plugin_http::reqwest`: HTTP client already in use — reuse for all provider API calls
- `tauri::ipc::Channel<String>`: Token streaming channel pattern — reuse across all adapters
- System prompt templates: `TERMINAL_SYSTEM_PROMPT_TEMPLATE` and `ASSISTANT_SYSTEM_PROMPT` in `ai.rs` — keep as-is, adapters format them per API

### Established Patterns
- Tauri command pattern: `#[tauri::command] pub async fn` with `Result<(), String>` return — follow for new/modified commands
- SSE parsing: `response.bytes_stream().eventsource()` with `tokio::time::timeout` — base pattern for OpenAI-compat adapter
- Model metadata: `XaiModelWithMeta { id, label }` struct — generalize to `ModelWithMeta` for all providers
- Frontend pre-caps history via `turnLimit` — Rust receives pre-capped history, no Rust-side capping

### Integration Points
- `commands/mod.rs`: Register new/modified command modules
- `commands/ai.rs`: Refactor `stream_ai_response` to accept provider param and dispatch to correct adapter
- `commands/keychain.rs`: Parameterize with provider-specific account names
- `commands/xai.rs`: Generalize `validate_and_fetch_models` to work per-provider
- `lib.rs`: Update Tauri command handler registration
- Frontend store (`src/store/index.ts`): Will need provider state (Phase 22 scope, but IPC contract defined here)

</code_context>

<specifics>
## Specific Ideas

- Cross-platform compatibility is a hard constraint — all provider code must work on both macOS and Windows
- The user wants all models shown (not just curated ones) so power users can pick any model their provider supports
- OpenRouter model list should be API-fetched since its whole value is access to many models

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 21-provider-abstraction-layer*
*Context gathered: 2026-03-09*
