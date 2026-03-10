# Phase 26: Cost Display Frontend - Context

**Gathered:** 2026-03-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the static placeholder in the Settings Model tab with a live session cost display. Users see estimated cost, input/output token counts, a per-query cost sparkline, and can reset stats. Persistent cost history, cost alerts, and per-provider breakdowns are out of scope.

</domain>

<decisions>
## Implementation Decisions

### Cost display layout
- Summary only — no per-model breakdown table
- Single line: cost + input/output token counts (e.g., "$0.0042 — 1,234 in / 567 out")
- Cost shown in exact format with enough decimals to be meaningful (e.g., $0.0042, not rounded to $0.00)
- Small greyscale sparkline bar chart below the cost line showing per-query cost over time
- Sparkline bars are straight/angular, not curved — all session queries shown (not capped)
- Requires backend addition: track per-query cost history in UsageAccumulator (Vec of recent costs)

### Live update mechanism
- Fetch usage stats on tab open (call `get_usage_stats` each time Model tab becomes visible)
- No manual refresh button — fetch-on-open is sufficient
- No loading state — IPC call to Rust is near-instant, show data immediately

### Reset button UX
- Inline "Reset" text button next to the session total cost
- Small text-xs in white/40, subtle and not attention-grabbing
- Instant reset — no confirmation dialog (session data is low-stakes)
- No visual feedback animation — cost goes to $0.0000 and sparkline clears, the change itself is the feedback

### No-pricing state
- Show tokens with dash for cost: "1,234 in / 567 out — $—" with tooltip "Pricing unavailable for this model"
- Session total shows partial total from priced queries with asterisk note "*excludes queries without pricing"
- Sparkline: unpriced queries show as zero-height bars (no cost = no bar)

### Claude's Discretion
- Exact sparkline implementation (canvas, SVG, or div-based bars)
- How to store per-query cost history in the backend accumulator
- Responsive behavior if token counts get very large
- Exact tooltip implementation for pricing-unavailable indicator

</decisions>

<specifics>
## Specific Ideas

- User wants the sparkline to feel like a "cool greyscale line graph that isn't curved" — think straight bar chart, not a smooth line
- The sparkline visualizes per-query cost over time across the entire session
- Keep the overall display compact — this lives inside the Model tab which is already dense

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ModelTab.tsx:145-154`: Existing placeholder section to replace — 9 lines of static text in a `flex flex-col gap-1.5` container
- `lucide-react` icons: RotateCcw available if needed, but going with text button instead
- Tailwind opacity classes: `text-white/40`, `text-white/30`, `text-white/20` for label hierarchy
- Button pattern from AdvancedTab: `px-3 py-2 text-xs bg-white/8 border border-white/10 rounded-lg`

### Established Patterns
- IPC invoke: `import { invoke } from "@tauri-apps/api/core"` then `await invoke<Type>("command_name", { params })`
- State: Zustand store (`useOverlayStore`) for app state
- Settings persistence: `Store.load("settings.json")` from `@tauri-apps/plugin-store`
- Color system: white-based with opacity (text-white/70 primary, text-white/40 labels, text-white/20 dim)
- Text sizes: `text-xs` for settings content, `text-sm` sparingly

### Integration Points
- Backend IPC commands ready: `get_usage_stats` returns `UsageStatsResponse { entries, session_total_cost }`
- Backend IPC commands ready: `reset_usage` clears all stats
- `UsageStatEntry`: provider, model, input_tokens, output_tokens, query_count, estimated_cost, pricing_available
- Need backend addition: per-query cost history for sparkline (currently only accumulated totals)

</code_context>

<deferred>
## Deferred Ideas

- Per-model breakdown view — future requirement
- Persistent cost tracking across sessions — listed in Future Requirements
- Daily/weekly/monthly cost summaries — listed in Future Requirements
- Cost alerts when spending exceeds threshold — listed in Future Requirements

</deferred>

---

*Phase: 26-cost-display-frontend*
*Context gathered: 2026-03-10*
