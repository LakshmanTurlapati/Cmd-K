# Phase 39: Streaming Integration - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Ensure end-to-end streaming from local providers works with the same UX as cloud providers. Phase 37 already built the core plumbing (dynamic URL, auth bypass, 120s timeout, token tracking). This phase verifies it works, handles edge cases (cost display, error handling), and closes any gaps.

</domain>

<decisions>
## Implementation Decisions

### Cost display for local models
- Show $0.00 as the cost with token counts for local provider queries
- Local models have no pricing data — the existing `pricing_available: false` code path handles this, showing "$---" currently. Override to show "$0.00" for local providers instead of "$---"

### Streaming error handling
- On mid-stream disconnect (server crash, model unload): keep partial tokens already streamed, append error message
- Same behavior as cloud provider stream errors — don't discard partial output
- The existing `openai_compat::stream()` error path returns an Err after partial tokens have already been sent via `on_token` channel — this behavior is correct for the "partial + error" UX

### Already implemented (Phase 37)
- Dynamic URL resolution in `stream_ai_response` (ai.rs lines 207-213)
- Conditional auth header omission in `openai_compat::stream()` (via empty api_key check)
- 120s timeout for local providers (`default_timeout_secs()` returns 120)
- Token tracking via `stream_options.include_usage` in request body
- Usage accumulation via `state.usage.lock().record()` (ai.rs line 356-358)

### Claude's Discretion
- Whether additional verification tests are needed beyond cargo test/tsc
- Whether Ollama's `/v1/chat/completions` has any format differences from standard OpenAI that need handling
- Whether the `pricing_available: false` → "$0.00" change belongs in the backend (usage.rs) or frontend (ModelTab.tsx)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Streaming path
- `src-tauri/src/commands/ai.rs` — `stream_ai_response` with local provider URL resolution (lines 207-224) and adapter dispatch (lines 312-353)
- `src-tauri/src/commands/providers/openai_compat.rs` — `stream()` with dynamic `api_url` param and conditional auth header
- `src-tauri/src/commands/providers/mod.rs` — `default_timeout_secs()` returns 120 for local providers

### Cost and usage display
- `src-tauri/src/commands/usage.rs` — Usage accumulation and `get_usage_stats` IPC command
- `src/components/Settings/ModelTab.tsx` — Cost display section (lines 184+), `pricing_available` checks, "$---" rendering for unpriced models

### Research findings
- `.planning/research/STACK.md` — Confirms both providers support `stream_options.include_usage`
- `.planning/research/PITFALLS.md` — Cold-start timeout concerns (addressed by 120s timeout)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `openai_compat::stream()` — Already parameterized for dynamic URL and optional auth. Both local providers route through this.
- `UsageAccumulator.record()` — Records usage by provider display name + model ID. Works for local providers unchanged.
- `pricing_available` field in `UsageStatEntry` — Already false when model has no pricing. Just need to change display from "$---" to "$0.00".

### Established Patterns
- Stream errors return `Err(String)` after partial tokens sent — overlay shows partial result + error toast
- `on_token` channel sends tokens as they arrive — partial output is already visible before any error

### Integration Points
- Cost display in `ModelTab.tsx` — check for `pricing_available === false` and render "$0.00" instead of "$---"
- No new IPC commands needed — everything uses existing `stream_ai_response` and `get_usage_stats`

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

*Phase: 39-streaming-integration*
*Context gathered: 2026-03-17*
