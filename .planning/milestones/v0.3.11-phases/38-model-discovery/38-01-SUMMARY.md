---
phase: 38-model-discovery
plan: 01
subsystem: api
tags: [ollama, lmstudio, model-discovery, reqwest, serde, local-llm]

# Dependency graph
requires:
  - phase: 37-provider-foundation
    provides: Provider enum with is_local(), get_provider_base_url(), validate_api_key health checks, fetch_models IPC plumbing
provides:
  - Ollama /api/tags model discovery with OllamaTagsResponse deserialization
  - LM Studio /v1/models model discovery reusing OpenAIModelsResponse
  - Parameter size parsing (parse_param_size_billions) for B/M suffixes
  - Auto-tiering (tier_from_param_size) with <7B=fast, 7-30B=balanced, >30B=capable
  - ModelTab refresh-on-mount for local providers
affects: [39-command-generation, model-selection, local-providers]

# Tech tracking
tech-stack:
  added: []
  patterns: [parameter-size-tiering, provider-specific-deserialization, refresh-on-mount]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/models.rs
    - src/components/Settings/ModelTab.tsx

key-decisions:
  - "Raw model names as labels (no pretty-printing) per locked decision"
  - "LM Studio models get empty tier (All Models section) since OpenAI-compat endpoint lacks parameter_size"
  - "2-second timeout for local API calls matching validate_api_key pattern"

patterns-established:
  - "Provider-specific deserialization structs mapping to common ModelWithMeta output"
  - "parse_param_size_billions for numeric extraction from size strings with B/M suffixes"
  - "tier_from_param_size for auto-categorizing models by parameter count thresholds"

requirements-completed: [LMOD-01, LMOD-02, LMOD-03]

# Metrics
duration: 5min
completed: 2026-03-17
---

# Phase 38 Plan 01: Model Discovery Summary

**Dynamic model discovery from Ollama /api/tags and LM Studio /v1/models with auto-tiering by parameter size and refresh-on-mount in ModelTab**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-17T19:23:17Z
- **Completed:** 2026-03-17T19:28:33Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Ollama models fetched from /api/tags with full metadata (parameter_size, quantization_level, family)
- LM Studio models fetched from /v1/models reusing existing OpenAIModelsResponse struct
- Auto-tiering assigns Ollama models to Fast/Balanced/Most Capable based on parameter size
- ModelTab refreshes model list on each tab open for local providers, catching new installs/unloads
- 5 unit tests covering param parsing, tier assignment, and JSON deserialization

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Ollama response structs, param size parser, and unit tests** - `83b3ccb` (feat)
2. **Task 2: Wire Ollama and LM Studio match arms into fetch_api_models** - `cff9ed5` (feat)
3. **Task 3: Add model refresh on ModelTab mount for local providers** - `3f8e590` (feat)

## Files Created/Modified
- `src-tauri/src/commands/models.rs` - Added OllamaTagsResponse/OllamaModel/OllamaModelDetails structs, parse_param_size_billions and tier_from_param_size helpers, separate Ollama and LMStudio match arms in fetch_api_models, threaded app_handle for base URL resolution, unit tests
- `src/components/Settings/ModelTab.tsx` - Added refresh-on-mount useEffect for local providers, setModels store selector, ModelWithMeta import, moved isLocal derivation before hooks

## Decisions Made
- Raw model names used as labels (no pretty-printing) per locked decision in CONTEXT.md
- LM Studio models assigned empty tier since OpenAI-compat /v1/models lacks parameter_size metadata
- Reused existing OpenAIModelsResponse struct for LM Studio (same format, no new structs needed)
- 2-second timeout for local API requests matching existing validate_api_key pattern
- Model refresh scoped to local providers only on ModelTab mount (cloud providers already refresh via AccountTab)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Model discovery pipeline fully functional for both local providers
- Selected model ID flows through to store for Phase 39 command generation
- All tier grouping works with existing TIER_ORDER display in ModelTab
- Manual verification needed with running Ollama/LM Studio instances

## Self-Check: PASSED

All files exist. All commit hashes verified.

---
*Phase: 38-model-discovery*
*Completed: 2026-03-17*
