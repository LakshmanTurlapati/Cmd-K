---
phase: 37-provider-foundation
plan: 01
subsystem: api
tags: [ollama, lmstudio, local-llm, openai-compat, health-check, provider-enum]

# Dependency graph
requires:
  - phase: 20-multi-provider
    provides: Provider enum with 5 cloud variants and AdapterKind streaming dispatch
provides:
  - Provider::Ollama and Provider::LMStudio enum variants with serde support
  - is_local() and requires_api_key() helper methods for provider type detection
  - get_provider_base_url() for reading configurable URLs from tauri-plugin-store
  - normalize_base_url() for user-supplied URL normalization
  - Parameterized openai_compat::stream() with explicit api_url and conditional auth
  - Keychain bypass in stream_ai_response for local providers
  - Health check validation with 3 error states (Server not running, No models loaded, Request failed)
affects: [37-02-frontend-wiring, 38-model-discovery, 39-local-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: [is_local() guard pattern for local vs cloud provider branching, health-check-as-validation for keyless providers]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/providers/mod.rs
    - src-tauri/src/commands/providers/openai_compat.rs
    - src-tauri/src/commands/ai.rs
    - src-tauri/src/commands/models.rs

key-decisions:
  - "Use provider.is_local() guard pattern for all local vs cloud branching"
  - "Health checks replace API key validation for local providers with 3 distinct error states"
  - "120-second streaming timeout for local providers (cold-start model loading)"
  - "Empty api_key string used as sentinel to skip Authorization header"

patterns-established:
  - "is_local() guard: all provider-conditional logic checks provider.is_local() before keychain/URL decisions"
  - "Health-check-as-validation: local providers use reachability + model list checks instead of API key validation"
  - "Parameterized streaming URL: openai_compat::stream receives explicit api_url rather than deriving from Provider"

requirements-completed: [LPROV-01, LPROV-02, LPROV-03, LPROV-04, LPROV-05, LPROV-06]

# Metrics
duration: 8min
completed: 2026-03-17
---

# Phase 37 Plan 01: Provider Foundation Backend Summary

**Ollama/LMStudio Provider variants with keyless auth bypass, configurable base URLs from settings store, health-check validation with 3 error states, and parameterized OpenAI-compat streaming**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-17T18:22:20Z
- **Completed:** 2026-03-17T18:30:46Z
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- Extended Provider enum from 5 to 7 variants with full match coverage across all helper methods
- Implemented keychain bypass for local providers with dynamic URL resolution from tauri-plugin-store settings
- Added health check validation distinguishing 3 error states: server not running, no models loaded, request failed
- Parameterized openai_compat::stream to accept explicit URL and conditionally omit auth header
- All 88 unit tests pass (9 new provider tests + 79 existing)

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend Provider enum with Ollama/LMStudio variants and helper methods** - `795543c` (feat)
2. **Task 2: Parameterize openai_compat::stream URL and conditionally skip auth header** - `74a6601` (feat)
3. **Task 3: Add health check validation with 3 error states and curated_models arms** - `5fa9f0a` (feat)

## Files Created/Modified
- `src-tauri/src/commands/providers/mod.rs` - Provider enum with Ollama/LMStudio, 4 new methods, URL utilities, 9 unit tests
- `src-tauri/src/commands/providers/openai_compat.rs` - stream() with explicit api_url param and conditional auth header
- `src-tauri/src/commands/ai.rs` - stream_ai_response with is_local() keychain bypass and dynamic URL resolution
- `src-tauri/src/commands/models.rs` - validate_api_key health checks, empty curated/fetch model stubs for local providers

## Decisions Made
- Used `provider.is_local()` guard pattern for all local vs cloud provider branching (clean, extensible)
- Health checks replace API key validation for local providers with 3 distinct error states for clear UI feedback
- Set 120-second streaming timeout for local providers to accommodate cold-start model loading
- Empty api_key string used as sentinel to skip Authorization header (avoid Option complexity in existing signatures)
- Used `resp.bytes()` + `serde_json::from_slice()` instead of `.json()` because tauri_plugin_http::reqwest::Response lacks `.json()` method
- Prefixed `_app_handle` in `fetch_models` since Phase 38 will use it for model discovery

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed Response::json() not available in tauri_plugin_http::reqwest**
- **Found during:** Task 3 (health check implementation)
- **Issue:** `tauri_plugin_http::reqwest::Response` does not expose `.json()` method like standalone reqwest
- **Fix:** Used `resp.bytes().await` + `serde_json::from_slice()` pattern consistent with existing code in fetch_api_models
- **Files modified:** src-tauri/src/commands/models.rs
- **Verification:** `cargo check --lib` passes cleanly
- **Committed in:** 5fa9f0a (Task 3 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Trivial API difference between tauri_plugin_http::reqwest and standalone reqwest. Used the project's existing pattern. No scope creep.

## Issues Encountered
None beyond the deviation above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All backend plumbing complete for Ollama and LM Studio providers
- Frontend wiring (Plan 02) can now invoke existing IPC commands with `provider: "ollama"` or `provider: "lmstudio"`
- Health check validation ready for Settings panel integration
- Phase 38 model discovery will use the `_app_handle` parameter already added to `fetch_models`

## Self-Check: PASSED

All 4 modified files verified present. All 3 task commit hashes verified in git log.

---
*Phase: 37-provider-foundation*
*Completed: 2026-03-17*
