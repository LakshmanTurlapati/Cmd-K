---
phase: 37-provider-foundation
plan: 02
subsystem: ui
tags: [ollama, lmstudio, local-llm, provider-dropdown, url-config, health-check, provider-icons]

# Dependency graph
requires:
  - phase: 37-provider-foundation
    provides: Provider enum variants with is_local(), health-check validation, keychain bypass backend
provides:
  - PROVIDERS array with local flag for 7 providers (5 cloud + 2 local)
  - AccountTab conditional UI -- Server URL input for local providers, API Key for cloud
  - Overlay-open silent health check for local providers updating apiKeyStatus
  - submitQuery bypass allowing local providers through without valid API key status
  - SVG icon data for Ollama and LM Studio in ProviderIcon component
  - Debounced base URL save and health check re-trigger
affects: [38-model-discovery, 39-streaming-integration, 40-local-provider-frontend]

# Tech tracking
tech-stack:
  added: []
  patterns: [conditional UI rendering based on provider.local flag, silent fire-and-forget health check on overlay open, debounced URL save with health re-check]

key-files:
  created: []
  modified:
    - src/store/index.ts
    - src/components/Settings/AccountTab.tsx
    - src/components/icons/ProviderIcon.tsx
    - src/components/Settings/ModelTab.tsx

key-decisions:
  - "Mixed alphabetical provider ordering (local and cloud interleaved, not grouped)"
  - "Silent overlay-open health check (no validating spinner, just updates status indicator)"
  - "Debounced URL save with 800ms delay re-triggers health check on each URL change"
  - "Scrollable provider dropdown with max-h-60 overflow-y-auto for 7-provider list"

patterns-established:
  - "provider.local flag: PROVIDERS array entries carry local boolean for conditional rendering"
  - "Silent health check: overlay-open health probe updates status without spinner or blocking input"
  - "URL persistence: tauri-plugin-store with provider-specific keys (ollama_base_url, lmstudio_base_url)"

requirements-completed: [LPROV-01, LPROV-02, LPROV-03, LPROV-04, LPROV-05, LPROV-06]

# Metrics
duration: 12min
completed: 2026-03-17
---

# Phase 37 Plan 02: Frontend Wiring Summary

**PROVIDERS array extended to 7 entries with local flag, conditional Server URL / API Key UI in AccountTab, overlay-open silent health check, submitQuery keyless bypass, and Ollama/LM Studio SVG icons**

## Performance

- **Duration:** 12 min
- **Started:** 2026-03-17T18:35:00Z
- **Completed:** 2026-03-17T18:47:00Z
- **Tasks:** 3 (2 auto + 1 human-verify checkpoint)
- **Files modified:** 4

## Accomplishments
- Extended PROVIDERS array from 5 to 7 entries with local boolean flag, alphabetically sorted
- Built conditional AccountTab UI: Server URL input with per-provider placeholder for local providers, API Key input for cloud providers
- Added silent fire-and-forget health check on overlay open for local providers (updates status indicator without blocking input)
- Modified submitQuery to bypass API key gate for local providers, allowing keyless query submission
- Added SVG icon paths for Ollama (llama silhouette) and LM Studio (chip icon) in ProviderIcon component
- Added scrollable dropdown (max-h-60 overflow-y-auto) to handle expanded 7-provider list
- Added local provider messaging in ModelTab ("models auto-discovered when server running")

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend PROVIDERS array, icons, submitQuery bypass, overlay-open health check** - `c6e667e` (feat)
2. **Task 2: AccountTab conditional URL input and health-check-as-validation** - `83c843f` (feat)

**Post-task fix:** `a537435` (fix) - Scrollable provider dropdown, local provider model tab messages

## Files Created/Modified
- `src/store/index.ts` - PROVIDERS array with local flag, submitQuery bypass for local providers, overlay-open silent health check
- `src/components/Settings/AccountTab.tsx` - Conditional Server URL / API Key rendering, debounced URL save, health check on provider selection
- `src/components/icons/ProviderIcon.tsx` - SVG icon data for ollama and lmstudio entries
- `src/components/Settings/ModelTab.tsx` - Local provider model tab messaging

## Decisions Made
- Mixed alphabetical provider ordering (Ollama between OpenAI and OpenRouter) per CONTEXT.md user decision
- Silent overlay-open health check avoids "validating" spinner flash -- just updates checkmark/X status
- 800ms debounce on URL input change before saving to store and re-running health check
- Placeholder icons (llama silhouette, chip icon) used -- final branding deferred to Phase 40 (LFUI-04)
- Added scrollable dropdown as a deviation fix when 7 providers caused overflow

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Scrollable provider dropdown for 7-provider list**
- **Found during:** Post-task 2 verification
- **Issue:** Provider dropdown with 7 entries exceeded visible area without scroll
- **Fix:** Added max-h-60 overflow-y-auto to dropdown container
- **Files modified:** src/components/Settings/AccountTab.tsx
- **Committed in:** a537435

**2. [Rule 2 - Missing Critical] Local provider model tab messaging**
- **Found during:** Post-task 2 verification
- **Issue:** ModelTab showed no helpful message for local providers (models empty until Phase 38 adds discovery)
- **Fix:** Added informational text for local providers in ModelTab
- **Files modified:** src/components/Settings/ModelTab.tsx
- **Committed in:** a537435

---

**Total deviations:** 2 auto-fixed (1 bug, 1 missing critical)
**Impact on plan:** Both fixes improve UX for the expanded provider list. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 37 complete: all backend (Plan 01) and frontend (Plan 02) wiring done for Ollama and LM Studio
- Phase 38 can implement model discovery using existing fetch_models IPC command and ModelTab UI
- Phase 39 can implement streaming using existing parameterized openai_compat::stream
- Phase 40 can refine provider icons and onboarding flow using established PROVIDERS.local pattern

## Self-Check: PASSED

All 4 modified files verified present. All 3 commit hashes (c6e667e, 83c843f, a537435) verified in git log.

---
*Phase: 37-provider-foundation*
*Completed: 2026-03-17*
