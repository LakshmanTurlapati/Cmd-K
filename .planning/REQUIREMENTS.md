# Requirements: CMD+K

**Defined:** 2026-03-10
**Core Value:** The overlay must appear on top of the currently active application and feel instant

## v1 Requirements

Requirements for v0.2.7 Cost Estimation. Each maps to roadmap phases.

### Token Tracking

- [ ] **TRAK-01**: App extracts input/output token counts from OpenAI-compatible streaming responses via `stream_options.include_usage`
- [ ] **TRAK-02**: App extracts input/output token counts from Anthropic streaming responses (`message_start` + `message_delta` usage events)
- [ ] **TRAK-03**: App extracts input/output token counts from Gemini streaming responses (`usageMetadata`)
- [ ] **TRAK-04**: Token counts accumulate per provider+model in session-scoped Rust state

### Pricing

- [ ] **PRIC-01**: Curated models (OpenAI, Anthropic, Gemini, xAI) have bundled pricing data ($/1M input, $/1M output) updated with each app release
- [ ] **PRIC-02**: OpenRouter model pricing is fetched dynamically from their `/api/v1/models` endpoint (prompt/completion fields)
- [ ] **PRIC-03**: Models without pricing data show token counts but display "pricing unavailable" instead of a dollar amount

### Display

- [ ] **DISP-01**: Settings Model tab shows session cost estimate replacing the current placeholder
- [ ] **DISP-02**: User can see token breakdown (input/output tokens) and total estimated cost
- [ ] **DISP-03**: Cost display updates live after each AI query completes
- [ ] **DISP-04**: User can reset session usage stats

## Future Requirements

- Persistent cost tracking across sessions (disk-backed)
- Daily/weekly/monthly cost summaries
- Cost alerts when spending exceeds a threshold
- Per-provider cost breakdown view

## Out of Scope

| Feature | Reason |
|---------|--------|
| Persistent cost history | Session-scoped is v1; disk persistence deferred |
| Real-time pricing API for all providers | Only OpenRouter exposes pricing API; others hardcode |
| Cost limits / spending caps | Future feature; v1 is observability only |
| Billing integration | Out of scope; this is estimation, not billing |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| TRAK-01 | 25 | Pending |
| TRAK-02 | 25 | Pending |
| TRAK-03 | 25 | Pending |
| TRAK-04 | 25 | Pending |
| PRIC-01 | 25 | Pending |
| PRIC-02 | 25 | Pending |
| PRIC-03 | 25 | Pending |
| DISP-01 | 26 | Pending |
| DISP-02 | 26 | Pending |
| DISP-03 | 26 | Pending |
| DISP-04 | 26 | Pending |

**Coverage:**
- v1 requirements: 11 total
- Mapped to phases: 11 (100%)
- Phase 25: 7 requirements (TRAK-01..04, PRIC-01..03)
- Phase 26: 4 requirements (DISP-01..04)

---
*Requirements defined: 2026-03-10*
*Last updated: 2026-03-10 after roadmap creation*
