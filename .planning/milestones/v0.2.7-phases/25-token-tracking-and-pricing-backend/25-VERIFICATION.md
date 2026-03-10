---
phase: 25-token-tracking-and-pricing-backend
verified: 2026-03-10T09:15:00Z
status: passed
score: 10/10 must-haves verified
---

# Phase 25: Token Tracking & Pricing Backend Verification Report

**Phase Goal:** Extract token usage from all 3 streaming adapters, accumulate in session state, and provide bundled + dynamic pricing data with IPC commands for frontend consumption.
**Verified:** 2026-03-10T09:15:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | After an OpenAI-compat streaming query completes, token counts are recorded in AppState | VERIFIED | `openai_compat.rs:55` initializes `TokenUsage::default()`, lines 75-78 extract `prompt_tokens`/`completion_tokens` from usage chunk, returns `Ok(token_usage)` at line 91. `ai.rs:342-344` accumulates via `state.usage.lock()`. |
| 2 | After an Anthropic streaming query completes, token counts are recorded in AppState | VERIFIED | `anthropic.rs:55` initializes `TokenUsage::default()`, line 75 extracts `input_tokens` from `message_start`, line 80 extracts `output_tokens` from `message_delta`, returns `Ok(token_usage)` at line 101. `ai.rs:342-344` accumulates. |
| 3 | After a Gemini streaming query completes, token counts are recorded in AppState | VERIFIED | `gemini.rs:79` initializes `TokenUsage::default()`, lines 96-98 extract `promptTokenCount`/`candidatesTokenCount` from `usageMetadata`, returns `Ok(token_usage)` at line 114. `ai.rs:342-344` accumulates. |
| 4 | Token counts accumulate across multiple queries grouped by provider+model | VERIFIED | `state.rs:112-136` `UsageAccumulator` uses `HashMap<(String, String), UsageEntry>` keyed by `(provider, model)`. `record()` method adds non-None values to running totals and increments `query_count`. `ai.rs:343` calls `acc.record(provider.display_name(), &model, &token_usage)`. |
| 5 | All curated models have accurate pricing data in $/1M tokens format | VERIFIED | `models.rs:22-48` `curated_models()` returns 16 models across 4 providers (OpenAI: 5, Anthropic: 3, Gemini: 3, xAI: 3), all with `Some(...)` pricing values. OpenRouter returns empty vec (correct -- uses dynamic pricing). |
| 6 | IPC command get_usage_stats returns accumulated usage with estimated costs per model | VERIFIED | `usage.rs:33-77` `get_usage_stats` is a `#[tauri::command]` that reads `state.usage`, looks up pricing from curated then OpenRouter cache, computes per-entry cost and `session_total_cost`. Returns `UsageStatsResponse` with `Vec<UsageStatEntry>`. |
| 7 | IPC command reset_usage clears all session usage stats | VERIFIED | `usage.rs:81-83` `reset_usage` is a `#[tauri::command]` that calls `state.usage.lock().unwrap().reset()`. `reset()` at `state.rs:129` calls `self.entries.clear()`. |
| 8 | Models without pricing data return pricing_available: false | VERIFIED | `usage.rs:51-60` match on pricing returns `(None, false)` when no pricing found. `UsageStatEntry` has `pricing_available: bool` and `estimated_cost: Option<f64>`. |
| 9 | OpenRouter model pricing is fetched from their API and cached in AppState | VERIFIED | `models.rs:215-228` `OpenRouterPricing` struct with `prompt`/`completion` string fields. Lines 393-428 parse pricing, convert per-token to per-million (`* 1_000_000.0`), insert into `pricing_cache`, then store via `*state.openrouter_pricing.lock().unwrap() = pricing_cache`. |
| 10 | Cost calculation uses curated pricing for known models and OpenRouter pricing for OR models | VERIFIED | `usage.rs:46-49` pricing lookup: `curated_pricing.get(model).copied().or_else(|| or_pricing.get(model).copied())`. Two-tier: curated first, OpenRouter fallback. |

**Score:** 10/10 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/state.rs` | TokenUsage, UsageEntry, UsageAccumulator structs + usage/openrouter_pricing on AppState | VERIFIED | All types defined (lines 89-136). AppState has both fields (lines 178-180). Default impl initializes them (lines 196-197). |
| `src-tauri/src/commands/models.rs` | Pricing fields on ModelWithMeta + curated pricing + curated_models_pricing() helper + OpenRouterPricing | VERIFIED | `input_price_per_m`/`output_price_per_m` on ModelWithMeta (lines 14-18) with `skip_serializing_if`. `curated_models_pricing()` helper (lines 53-69). OpenRouterPricing struct (lines 223-228). All API-fetched models include `None` pricing. |
| `src-tauri/src/commands/providers/openai_compat.rs` | stream_options.include_usage + TokenUsage extraction | VERIFIED | `stream_options` in request body (line 25). TokenUsage import (line 5). Returns `Result<TokenUsage, String>` (line 20). Usage extraction at lines 75-78. |
| `src-tauri/src/commands/providers/anthropic.rs` | message_start + message_delta token extraction | VERIFIED | TokenUsage import (line 5). Returns `Result<TokenUsage, String>` (line 25). message_start parsing (lines 73-76), message_delta parsing (lines 78-81). |
| `src-tauri/src/commands/providers/gemini.rs` | usageMetadata parsing | VERIFIED | TokenUsage import (line 5). Returns `Result<TokenUsage, String>` (line 24). usageMetadata parsing (lines 96-99). |
| `src-tauri/src/commands/ai.rs` | State parameter + accumulation after adapter call | VERIFIED | `state: tauri::State<'_, crate::state::AppState>` parameter (line 188). `let token_usage = match provider.adapter_kind()` captures result (line 298). Accumulation at lines 342-344. |
| `src-tauri/src/commands/usage.rs` | get_usage_stats and reset_usage IPC commands | VERIFIED | Both commands exist with `#[tauri::command]` attribute. Full cost calculation logic. Correct response types. |
| `src-tauri/src/lib.rs` | IPC commands registered | VERIFIED | `get_usage_stats` and `reset_usage` imported (line 18) and in `generate_handler!` (lines 265-266). |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| openai_compat.rs | state.rs | Returns TokenUsage | WIRED | `use crate::state::TokenUsage` (line 5), return type `Result<TokenUsage, String>` (line 20) |
| anthropic.rs | state.rs | Returns TokenUsage | WIRED | `use crate::state::TokenUsage` (line 5), return type `Result<TokenUsage, String>` (line 25) |
| gemini.rs | state.rs | Returns TokenUsage | WIRED | `use crate::state::TokenUsage` (line 5), return type `Result<TokenUsage, String>` (line 24) |
| ai.rs | state.rs | Accumulates via Mutex | WIRED | `state.usage.lock()` at line 342, calls `acc.record()` at line 343 |
| usage.rs | state.rs | Reads AppState.usage and openrouter_pricing | WIRED | `state.usage.lock().unwrap()` (line 36), `state.openrouter_pricing.lock().unwrap()` (line 38) |
| usage.rs | models.rs | Calls curated_models_pricing() | WIRED | `super::models::curated_models_pricing()` at line 37 |
| models.rs (fetch) | state.rs | Stores OpenRouter pricing | WIRED | `*state.openrouter_pricing.lock().unwrap() = pricing_cache` at line 426 |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| TRAK-01 | 25-01 | Extract tokens from OpenAI-compat streaming | SATISFIED | `stream_options.include_usage` in request, usage chunk parsing in openai_compat.rs |
| TRAK-02 | 25-01 | Extract tokens from Anthropic streaming | SATISFIED | message_start/message_delta parsing in anthropic.rs |
| TRAK-03 | 25-01 | Extract tokens from Gemini streaming | SATISFIED | usageMetadata parsing in gemini.rs |
| TRAK-04 | 25-01 | Accumulate per provider+model in session state | SATISFIED | UsageAccumulator in state.rs, accumulation in ai.rs |
| PRIC-01 | 25-01 | Curated models have bundled pricing | SATISFIED | 16 models across 4 providers with $/1M prices in curated_models() |
| PRIC-02 | 25-02 | OpenRouter dynamic pricing from API | SATISFIED | OpenRouterPricing struct, pricing cache in fetch_models, stored in AppState |
| PRIC-03 | 25-02 | Models without pricing show unavailable | SATISFIED | pricing_available: false and estimated_cost: None for unpriced models |

No orphaned requirements found. All 7 requirements mapped to Phase 25 in REQUIREMENTS.md are claimed by plans and verified.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| - | - | None found | - | - |

No TODOs, FIXMEs, placeholders, empty implementations, or stub patterns detected in any modified files.

### Human Verification Required

### 1. OpenAI-compat usage chunk reception

**Test:** Send a query to OpenAI/xAI/OpenRouter and check logs for token counts
**Expected:** `[DONE]` preceded by a chunk with `usage.prompt_tokens` and `usage.completion_tokens` values
**Why human:** Requires live API call to verify provider actually returns usage data when `stream_options.include_usage` is set

### 2. Anthropic token extraction

**Test:** Send a query to Anthropic and verify token counts appear in get_usage_stats
**Expected:** Non-zero input_tokens and output_tokens in the usage stats response
**Why human:** Requires live Anthropic API call with streaming

### 3. Gemini usageMetadata extraction

**Test:** Send a query to Gemini and verify token counts appear in get_usage_stats
**Expected:** Non-zero input_tokens and output_tokens from usageMetadata parsing
**Why human:** Requires live Gemini API call with streaming

### 4. OpenRouter pricing cache population

**Test:** Call fetch_models with OpenRouter provider, then call get_usage_stats after a query
**Expected:** OpenRouter models have pricing_available: true and estimated_cost with a dollar amount
**Why human:** Requires OpenRouter API key and live API response

### 5. Cost calculation accuracy

**Test:** Send a known model query, note token counts, manually verify cost = (input * price / 1M) + (output * price / 1M)
**Expected:** Calculated cost matches manual arithmetic
**Why human:** Requires end-to-end verification with real token counts

### Gaps Summary

No gaps found. All 10 observable truths verified. All 7 artifacts pass three-level verification (exists, substantive, wired). All 7 key links confirmed wired. All 7 requirements satisfied. No anti-patterns detected. Implementation matches plan specifications exactly.

---

_Verified: 2026-03-10T09:15:00Z_
_Verifier: Claude (gsd-verifier)_
