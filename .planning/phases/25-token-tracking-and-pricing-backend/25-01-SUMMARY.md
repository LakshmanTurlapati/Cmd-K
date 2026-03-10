---
phase: 25-token-tracking-and-pricing-backend
plan: 01
subsystem: api
tags: [token-tracking, pricing, streaming, rust, tauri]

requires:
  - phase: none
    provides: existing streaming adapters and AppState
provides:
  - TokenUsage, UsageEntry, UsageAccumulator types in state.rs
  - Token extraction from all 3 streaming adapters (OpenAI-compat, Anthropic, Gemini)
  - Session-scoped token accumulation in AppState.usage
  - Pricing data on all curated models ($/1M tokens)
  - openrouter_pricing cache field on AppState
affects: [25-02, 26-cost-estimation-ui]

tech-stack:
  added: []
  patterns: [adapter-returns-token-usage, session-scoped-accumulator]

key-files:
  created: []
  modified:
    - src-tauri/src/state.rs
    - src-tauri/src/commands/models.rs
    - src-tauri/src/commands/providers/openai_compat.rs
    - src-tauri/src/commands/providers/anthropic.rs
    - src-tauri/src/commands/providers/gemini.rs
    - src-tauri/src/commands/ai.rs

key-decisions:
  - "UsageAccumulator keys are (String, String) not (Provider, String) to keep state.rs decoupled from providers module"
  - "Adapters return TokenUsage with Option fields -- None values are silently skipped during accumulation"

patterns-established:
  - "Adapter return pattern: all streaming adapters return Result<TokenUsage, String> instead of Result<(), String>"
  - "Usage accumulation pattern: ai.rs captures TokenUsage from adapter and records to AppState.usage via Mutex"

requirements-completed: [TRAK-01, TRAK-02, TRAK-03, TRAK-04, PRIC-01]

duration: 8min
completed: 2026-03-10
---

# Phase 25 Plan 01: Token Tracking & Pricing Backend Summary

**Token usage extraction from all 3 streaming adapters with session accumulation and curated model pricing data**

## Performance

- **Duration:** 8 min
- **Started:** 2026-03-10T06:46:46Z
- **Completed:** 2026-03-10T06:55:00Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments
- Defined TokenUsage, UsageEntry, and UsageAccumulator types for session-scoped token tracking
- All 3 streaming adapters now extract and return token counts from provider-specific response formats
- ai.rs accumulates token usage into AppState after each query completes
- All 16 curated models across 4 providers have accurate $/1M token pricing

## Task Commits

Each task was committed atomically:

1. **Task 1: Define token/pricing types and extend AppState + ModelWithMeta** - `315f998` (feat)
2. **Task 2: Extract token usage from all 3 adapters and wire accumulation in ai.rs** - `3a92130` (feat)

## Files Created/Modified
- `src-tauri/src/state.rs` - TokenUsage, UsageEntry, UsageAccumulator structs + usage/openrouter_pricing on AppState
- `src-tauri/src/commands/models.rs` - input_price_per_m/output_price_per_m on ModelWithMeta + curated pricing data
- `src-tauri/src/commands/providers/openai_compat.rs` - stream_options.include_usage + final chunk usage parsing
- `src-tauri/src/commands/providers/anthropic.rs` - message_start/message_delta token usage extraction
- `src-tauri/src/commands/providers/gemini.rs` - usageMetadata parsing
- `src-tauri/src/commands/ai.rs` - State parameter + token accumulation after adapter call

## Decisions Made
- UsageAccumulator uses (String, String) keys instead of (Provider, String) to keep state.rs decoupled from the providers module
- Adapters return TokenUsage with Option<u64> fields -- providers that don't return usage data produce None which is silently skipped by the accumulator

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Token usage pipeline is complete from adapter extraction through accumulation
- Ready for Plan 25-02 (OpenRouter dynamic pricing + frontend cost display)
- openrouter_pricing field on AppState is ready for population

---
*Phase: 25-token-tracking-and-pricing-backend*
*Completed: 2026-03-10*
