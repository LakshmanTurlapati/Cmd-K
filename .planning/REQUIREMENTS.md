# Requirements: CMD+K

**Defined:** 2026-03-17
**Core Value:** The overlay must appear on top of the active application and feel instant

## v1 Requirements

Requirements for v0.3.11 Local LLM Providers. Each maps to roadmap phases.

### Local Provider Backend

- [x] **LPROV-01**: User can select Ollama as an AI provider in settings and onboarding
- [x] **LPROV-02**: User can select LM Studio as an AI provider in settings and onboarding
- [x] **LPROV-03**: Ollama and LM Studio require no API key — keyless auth bypass in backend and frontend
- [x] **LPROV-04**: User can configure base URL for each local provider (defaults: localhost:11434 for Ollama, localhost:1234 for LM Studio)
- [x] **LPROV-05**: App checks local provider health (Ollama GET /, LM Studio GET /v1/models) and surfaces connection status
- [x] **LPROV-06**: Provider-specific error messages differentiate "server not running" from "model not loaded" from network errors

### Model Discovery

- [x] **LMOD-01**: App auto-discovers available models from Ollama via /api/tags endpoint
- [x] **LMOD-02**: App auto-discovers available models from LM Studio via /v1/models endpoint
- [x] **LMOD-03**: Model list displays metadata where available (parameter size, quantization level for Ollama models)

### Streaming & Generation

- [x] **LSTR-01**: AI command generation streams from local providers using existing OpenAI-compat SSE adapter with dynamic URL
- [x] **LSTR-02**: Local provider timeout is 120s (vs cloud default) to handle cold-start model loading
- [x] **LSTR-03**: Token tracking works for local providers via stream_options.include_usage

### Frontend UX

- [x] **LFUI-01**: Connection health shown via same checkmark indicator used for API key validation — checkmark when server reachable, no checkmark when not
- [x] **LFUI-02**: Settings shows base URL input field instead of API key field for local providers
- [x] **LFUI-03**: Onboarding wizard skips API key step for local providers
- [x] **LFUI-04**: Provider SVG icons for Ollama and LM Studio in onboarding and settings

## Future Requirements

### Local Provider Enhancements

- **LFUT-01**: Model pre-warming on provider selection to avoid cold-start latency
- **LFUT-02**: Context window entries in CONTEXT_WINDOWS for common local model families
- **LFUT-03**: Per-chunk idle timeout instead of total-stream timeout for very large local models

## Out of Scope

| Feature | Reason |
|---------|--------|
| Ollama native /api/chat endpoint | OpenAI-compat endpoint covers all needs, avoids maintaining separate ndjson parser |
| Local model downloading/management | Ollama and LM Studio have their own UIs for this |
| GPU/VRAM monitoring | Out of scope for a command overlay app |
| Model quantization selection | Users manage this in Ollama/LM Studio directly |
| Local provider auto-detection | User selects provider explicitly; scanning localhost ports is fragile |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| LPROV-01 | Phase 37 | Complete |
| LPROV-02 | Phase 37 | Complete |
| LPROV-03 | Phase 37 | Complete |
| LPROV-04 | Phase 37 | Complete |
| LPROV-05 | Phase 37 | Complete |
| LPROV-06 | Phase 37 | Complete |
| LMOD-01 | Phase 38 | Complete |
| LMOD-02 | Phase 38 | Complete |
| LMOD-03 | Phase 38 | Complete |
| LSTR-01 | Phase 39 | Complete |
| LSTR-02 | Phase 39 | Complete |
| LSTR-03 | Phase 39 | Complete |
| LFUI-01 | Phase 40 | Complete |
| LFUI-02 | Phase 40 | Complete |
| LFUI-03 | Phase 40 | Complete |
| LFUI-04 | Phase 40 | Complete |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0

---
*Requirements defined: 2026-03-17*
*Last updated: 2026-03-17 after roadmap creation*
