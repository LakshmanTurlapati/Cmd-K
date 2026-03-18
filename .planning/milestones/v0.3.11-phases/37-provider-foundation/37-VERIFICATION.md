---
phase: 37-provider-foundation
verified: 2026-03-17T19:15:00Z
status: human_needed
score: 7/7 must-haves verified
re_verification: false
human_verification:
  - test: "Verify provider dropdown renders Ollama and LM Studio with distinct icons"
    expected: "Both providers appear in the dropdown alongside cloud providers, each with a unique SVG icon rendered correctly"
    why_human: "SVG path data is present in ProviderIcon.tsx but visual correctness (shape, appearance) cannot be verified without rendering"
  - test: "Select Ollama in Settings — verify Server URL label and health check indicator"
    expected: "API Key section replaced by Server URL input, placeholder shows 'localhost:11434', red X or green checkmark appears depending on server state"
    why_human: "Conditional rendering verified in code but real toggle behavior requires running app"
  - test: "Select LM Studio in Settings — verify placeholder and health check"
    expected: "Placeholder shows 'localhost:1234', same health indicator behavior as Ollama"
    why_human: "Cannot verify live health check without running app and local server"
  - test: "Open overlay with Ollama selected — verify input is usable immediately"
    expected: "Input is interactive from first keystroke; health check status icon appears/updates after a moment without blocking input"
    why_human: "Fire-and-forget async behavior, race condition handling, and UX responsiveness cannot be verified statically"
  - test: "Switch from local provider to cloud provider — verify API Key input returns"
    expected: "Switching from Ollama/LM Studio to xAI shows API Key label and password input instead of Server URL"
    why_human: "State toggle correctness requires live interaction test"
---

# Phase 37: Provider Foundation Verification Report

**Phase Goal:** Users can select Ollama or LM Studio as a provider, configure connection URLs, and see whether the local server is reachable
**Verified:** 2026-03-17T19:15:00Z
**Status:** human_needed — all automated checks pass, 5 UI behaviors require live app verification
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Ollama and LMStudio variants exist in Provider enum and compile without errors | VERIFIED | `mod.rs` lines 17-19 have `Ollama,` and `#[serde(rename = "lmstudio")] LMStudio,`; 9 provider unit tests all pass |
| 2 | `is_local()` returns true for Ollama and LMStudio, false for all others | VERIFIED | `mod.rs` lines 32-34, confirmed by `test_is_local` unit test passing |
| 3 | `stream_ai_response` bypasses keychain for local providers and uses dynamic URL | VERIFIED | `ai.rs` lines 207-224: `if provider.is_local()` branch resolves URL from `get_provider_base_url()` and returns `String::new()` for key |
| 4 | `openai_compat::stream` accepts explicit `api_url` parameter and conditionally omits auth header | VERIFIED | `openai_compat.rs` lines 15-38: `api_url: &str` second param, `.post(api_url)`, `if !api_key.is_empty()` guards Authorization header |
| 5 | `validate_api_key` performs health checks for Ollama (GET /) and LM Studio (GET /v1/models) | VERIFIED | `models.rs` lines 235-303: full health check implementation with Ollama checking `GET /` then `GET /api/tags`, LM Studio checking `GET /v1/models` |
| 6 | Health check errors distinguish 3 states: server-not-running, no-models-loaded, request-failed | VERIFIED | `models.rs`: `"Server not running"` (connect/timeout), `"No models loaded"` (empty array), `"Request failed -- {}"` (other errors) |
| 7 | Base URL read from tauri-plugin-store with fallback to default | VERIFIED | `mod.rs` lines 164-177: `get_provider_base_url()` reads from store via `StoreExt`, falls back to `provider.default_base_url()` |

**Score:** 7/7 truths verified (backend)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/commands/providers/mod.rs` | Provider enum with Ollama/LMStudio variants and helper methods | VERIFIED | 255 lines; `Ollama`, `LMStudio`, `is_local()`, `requires_api_key()`, `default_base_url()`, `base_url_store_key()`, `normalize_base_url()`, `get_provider_base_url()`, 9 unit tests |
| `src-tauri/src/commands/providers/openai_compat.rs` | OpenAI-compat streaming with explicit URL parameter | VERIFIED | 102 lines; `api_url: &str` in signature, `.post(api_url)`, conditional auth header |
| `src-tauri/src/commands/ai.rs` | Conditional keychain bypass for local providers | VERIFIED | `if provider.is_local()` block at lines 207-224, passes `&api_url` to `openai_compat::stream` at line 315 |
| `src-tauri/src/commands/models.rs` | Health check validation with 3 error states and empty curated models for local providers | VERIFIED | `Provider::Ollama` arm at line 235, `Provider::LMStudio` arm at line 272; `curated_models` returns `vec![]` for both at line 93 |
| `src/store/index.ts` | PROVIDERS array with local flag, submitQuery bypass, overlay-open health check | VERIFIED | Lines 4-12: 7 entries with `local` flag; line 473: `!isLocalProvider &&` guard; lines 342-362: silent fire-and-forget health check in `show()` |
| `src/components/Settings/AccountTab.tsx` | Conditional URL input vs API key input for local providers | VERIFIED | Line 26: `isLocal` computed; line 320: `{isLocal ? ...Server URL... : ...API Key...}` conditional rendering; `"Server URL"` label at line 324 |
| `src/components/icons/ProviderIcon.tsx` | SVG icon data for Ollama and LM Studio | VERIFIED | Lines 42-54: `ollama` and `lmstudio` keys in `ICON_DATA` with `viewBox` and `paths` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src-tauri/src/commands/ai.rs` | `providers/mod.rs` | `provider.is_local()` guard for keychain bypass | WIRED | `if provider.is_local()` at line 207 |
| `src-tauri/src/commands/ai.rs` | `providers/openai_compat.rs` | passes explicit `api_url` to `stream()` | WIRED | `providers::openai_compat::stream(&provider, &api_url, &api_key, ...)` at line 314-316 |
| `src-tauri/src/commands/models.rs` | `providers/mod.rs` | `Provider::Ollama` and `Provider::LMStudio` match arms in `validate_api_key` | WIRED | `Provider::Ollama =>` at line 235, `Provider::LMStudio =>` at line 272, both calling `get_provider_base_url(&app_handle, &provider)` |
| `src/components/Settings/AccountTab.tsx` | `src/store/index.ts` | `PROVIDERS` array `local` flag determines conditional rendering | WIRED | `currentProvider?.local ?? false` at line 26, drives `isLocal ?` at line 320 |
| `src/store/index.ts` | `src-tauri/src/commands/ai.rs` | `submitQuery` invokes `stream_ai_response` | WIRED | `await invoke("stream_ai_response", { provider: state.selectedProvider, ... })` |
| `src/components/Settings/AccountTab.tsx` | `src-tauri/src/commands/models.rs` | `validate_api_key` performs health check for local providers | WIRED | `await invoke("validate_api_key", { provider: currentProv, apiKey: "" })` at lines 55 and 115 |
| `src/store/index.ts (show)` | `src-tauri/src/commands/models.rs` | overlay open triggers `validate_api_key` for local providers | WIRED | Fire-and-forget IIFE at lines 344-362 calls `invoke("validate_api_key", ...)` |

### Requirements Coverage

| Requirement | Source Plans | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| LPROV-01 | 37-01, 37-02 | User can select Ollama as an AI provider in settings and onboarding | SATISFIED | `{ id: "ollama", name: "Ollama", local: true }` in PROVIDERS; renders in AccountTab dropdown via `PROVIDERS.map()` |
| LPROV-02 | 37-01, 37-02 | User can select LM Studio as an AI provider in settings and onboarding | SATISFIED | `{ id: "lmstudio", name: "LM Studio", local: true }` in PROVIDERS; renders in AccountTab dropdown |
| LPROV-03 | 37-01, 37-02 | Ollama and LM Studio require no API key — keyless auth bypass in backend and frontend | SATISFIED | Backend: `if provider.is_local()` skips keychain, passes `String::new()` as api_key; Frontend: `submitQuery` bypass with `!isLocalProvider &&` guard |
| LPROV-04 | 37-01, 37-02 | User can configure base URL for each local provider (defaults: localhost:11434 for Ollama, localhost:1234 for LM Studio) | SATISFIED | Backend: `default_base_url()` returns correct defaults; `get_provider_base_url()` reads from store with fallback; Frontend: `AccountTab` saves URL to `ollama_base_url`/`lmstudio_base_url` store keys |
| LPROV-05 | 37-01, 37-02 | App checks local provider health and surfaces connection status | SATISFIED | `validate_api_key` performs reachability checks; AccountTab shows validating/valid/invalid status indicator; overlay `show()` runs silent health check |
| LPROV-06 | 37-01, 37-02 | Provider-specific error messages differentiate "server not running" from "model not loaded" from network errors | SATISFIED | `models.rs`: `"Server not running"` (connect/timeout errors), `"No models loaded"` (empty model array), `"Request failed -- {details}"` (other HTTP errors) |

All 6 LPROV requirements satisfied. No orphaned requirements found (REQUIREMENTS.md maps only LPROV-01 through LPROV-06 to Phase 37).

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `models.rs` | 555 | Comment: "Dynamic model discovery is Phase 38. Return empty for now." | Info | Intentional deferral per phase scope — `fetch_api_models` returns `Ok(vec![])` for local providers until Phase 38 |

No blockers or warnings found. The Phase 38 deferral comment is by design and not a stub in the blocking sense — it is the correct behavior for this phase boundary.

### Human Verification Required

#### 1. Provider Dropdown with Icons

**Test:** Run `cargo tauri dev`, open Settings, click the Provider dropdown
**Expected:** Ollama and LM Studio appear in the alphabetically-sorted list alongside cloud providers; each shows a distinct SVG icon (llama silhouette for Ollama, chip icon for LM Studio)
**Why human:** SVG path data exists in `ProviderIcon.tsx` but visual correctness of icon shapes requires live rendering

#### 2. Server URL Input for Local Providers

**Test:** Select "Ollama" from the provider dropdown in Settings
**Expected:** The "API Key" section is replaced by a "Server URL" label with a text input; placeholder reads "localhost:11434"; a checkmark or X appears reflecting server reachability
**Why human:** Conditional rendering (`isLocal ?`) is verified in code but live toggle behavior and visual layout require running app

#### 3. LM Studio URL Configuration

**Test:** Select "LM Studio" from the provider dropdown in Settings
**Expected:** Placeholder reads "localhost:1234"; same health indicator behavior; typing a custom URL triggers debounced save and health re-check
**Why human:** Debounced save (800ms) and health re-trigger need live interaction to confirm correct timing

#### 4. Overlay-Open Health Check (Non-Blocking)

**Test:** With Ollama selected, close Settings and open the overlay with the hotkey
**Expected:** Input is immediately usable (not disabled, not in loading state); after a moment, the status indicator updates to show server reachability; the user can type and submit without waiting
**Why human:** Fire-and-forget async timing, input focus behavior, and non-blocking UX cannot be verified statically

#### 5. Cloud Provider Restore on Switch

**Test:** Select "Ollama", then switch to "xAI"
**Expected:** The Server URL input disappears; the API Key section with password input and reveal toggle reappears
**Why human:** State cleanup (`setBaseUrlInput("")`, `setApiKeyStatus("unknown")`) is in code but live switching order and visual restoration require running app

### Gaps Summary

No gaps found. All automated checks pass:

- All 9 Rust unit tests pass (`cargo test --lib -- providers::tests`)
- TypeScript compiles with zero errors (`npx tsc --noEmit`)
- All 6 commit hashes verified in git log (795543c, 74a6601, 5fa9f0a, c6e667e, 83c843f, a537435)
- All 7 artifacts exist, are substantive, and are wired to their consumers
- All 7 key links confirmed by source inspection
- All 6 LPROV requirements have concrete implementation evidence

The phase requires human verification for 5 UI behaviors that are correct in code but cannot be confirmed without a running app.

---

_Verified: 2026-03-17T19:15:00Z_
_Verifier: Claude (gsd-verifier)_
