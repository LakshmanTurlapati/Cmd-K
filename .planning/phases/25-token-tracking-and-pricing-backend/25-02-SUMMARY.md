---
phase: 25-token-tracking-and-pricing-backend
plan: 02
subsystem: api
tags: [token-tracking, pricing, ipc, rust, tauri, openrouter]

requires:
  - phase: 25-token-tracking-and-pricing-backend
    provides: TokenUsage, UsageEntry, UsageAccumulator types and AppState.usage/openrouter_pricing fields
provides:
  - get_usage_stats IPC command returning per-model token counts with estimated costs
  - reset_usage IPC command clearing session usage data
  - curated_models_pricing() helper for cost lookup across all providers
  - OpenRouter dynamic pricing cached from API response
affects: [26-cost-estimation-ui]

tech-stack:
  added: []
  patterns: [curated-plus-dynamic-pricing-lookup, per-million-token-pricing-format]

key-files:
  created:
    - src-tauri/src/commands/usage.rs
  modified:
    - src-tauri/src/commands/models.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Cost calculation uses two-tier pricing: curated models first, then OpenRouter dynamic pricing as fallback"
  - "OpenRouter pricing strings parsed to f64 and converted to $/1M tokens format for consistency with curated pricing"

patterns-established:
  - "Pricing lookup pattern: curated_models_pricing() HashMap checked first, then openrouter_pricing cache"
  - "Graceful degradation: models without pricing return pricing_available: false and estimated_cost: None"

requirements-completed: [PRIC-02, PRIC-03]

duration: 4min
completed: 2026-03-10
---

# Phase 25 Plan 02: Usage Stats IPC & OpenRouter Pricing Summary

**get_usage_stats and reset_usage IPC commands with two-tier cost calculation using curated + OpenRouter dynamic pricing**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-10T08:53:33Z
- **Completed:** 2026-03-10T08:57:30Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Created get_usage_stats IPC command returning per-model token counts with estimated costs and session total
- Created reset_usage IPC command to clear all session usage data
- Added curated_models_pricing() helper aggregating pricing from all 4 non-OpenRouter providers (16 models)
- OpenRouter pricing fetched from API response and cached in AppState for dynamic cost calculation

## Task Commits

Each task was committed atomically:

1. **Task 1: Create usage.rs with get_usage_stats and reset_usage IPC commands** - `9c20063` (feat)
2. **Task 2: Fetch and cache OpenRouter pricing from API response** - `cc7cf4c` (feat)

## Files Created/Modified
- `src-tauri/src/commands/usage.rs` - UsageStatEntry, UsageStatsResponse types; get_usage_stats and reset_usage commands
- `src-tauri/src/commands/models.rs` - curated_models_pricing() helper, OpenRouterPricing struct, pricing cache in fetch_models
- `src-tauri/src/commands/mod.rs` - Added usage module
- `src-tauri/src/lib.rs` - Registered get_usage_stats and reset_usage in invoke_handler

## Decisions Made
- Cost calculation uses two-tier pricing: curated models checked first, then OpenRouter dynamic pricing as fallback
- OpenRouter pricing strings parsed to f64 and multiplied by 1M to match curated per-million-token format
- session_total_cost is None when zero models have pricing data (not 0.0)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Both IPC commands ready for frontend consumption in Phase 26
- Frontend can call get_usage_stats to display session cost breakdown
- Frontend can call reset_usage to clear session stats
- OpenRouter pricing automatically cached when user loads model list

---
*Phase: 25-token-tracking-and-pricing-backend*
*Completed: 2026-03-10*
