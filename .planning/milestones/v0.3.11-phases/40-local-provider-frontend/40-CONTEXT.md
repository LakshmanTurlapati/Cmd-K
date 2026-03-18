# Phase 40: Local Provider Frontend - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Adapt the settings and onboarding UI for local providers — URL configuration instead of API keys, connection health instead of key validation, and provider branding. Most frontend work was already completed in Phase 37 (AccountTab URL input, health checkmark, provider icons). Remaining work: onboarding wizard skip for local providers and "Free (local)" usage label.

</domain>

<decisions>
## Implementation Decisions

### Onboarding flow for local providers
- Auto-skip the API key step (step index 1) when a local provider is selected — go straight from Provider Select to Model Select
- Same pattern as the existing Accessibility step skip on Windows (`isWindows() && nextStep === 3`)
- Keep the 5-step stepper layout unchanged — the skipped API key dot shows as completed (checkmark), not removed
- No back navigation — current onboarding has no back button, keep it that way
- If local provider server isn't running during Model Select step, show "No models found — is your server running?" but allow proceeding via existing "Skip this step" link

### Usage stats display for local queries
- Replace `$0.00` with visible text "Free (local)" for all-local-unpriced sessions
- Token counts still shown after the dash: `Free (local) — 1,234 in / 567 out`
- Mixed sessions (local + cloud queries): show cloud cost total normally, no special annotation for local queries — they're simply free and don't affect the dollar amount
- Remove the tooltip-only approach — "Free (local)" should be the visible label

### Provider icons
- Keep current generic SVG icons for Ollama and LM Studio — they're recognizable enough
- No need to source official brand logos

### Already implemented (Phase 37)
- LFUI-01: Connection health checkmark in AccountTab — done
- LFUI-02: Base URL input instead of API key field — done
- LFUI-04: Ollama and LM Studio SVG icons in ProviderIcon.tsx — done

### Claude's Discretion
- Exact conditional logic placement in OnboardingWizard.tsx for the step skip
- Whether to extract the local provider check into a shared utility or inline it

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Onboarding wizard
- `src/components/Onboarding/OnboardingWizard.tsx` — Step flow with `handleNext`, step skip pattern for Windows/Accessibility (line 31), stepper rendering
- `src/components/Onboarding/StepApiKey.tsx` — API key entry step that must be skipped for local providers
- `src/components/Onboarding/StepModelSelect.tsx` — Model selection step, needs graceful empty state for local providers with server not running
- `src/components/Onboarding/StepProviderSelect.tsx` — Provider selection step, sets `selectedProvider` in store

### Settings and usage display
- `src/components/Settings/ModelTab.tsx` — Cost display section (lines 202-306), `allUnpricedAreLocal` check (line 219), current `$0.00` rendering (lines 250-254)
- `src/components/Settings/AccountTab.tsx` — Already has local provider URL input and health check (lines 320-374)

### Store
- `src/store/index.ts` — `PROVIDERS` array with `local: boolean` field (line 4+), `selectedProvider` state, `apiKeyStatus`

### Provider icons
- `src/components/icons/ProviderIcon.tsx` — SVG icon data for all 7 providers including Ollama and LM Studio

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `isWindows()` platform check in `OnboardingWizard.tsx` — same pattern for skipping steps based on context
- `PROVIDERS` array with `local` field — already used in AccountTab and store for local provider branching
- `allUnpricedAreLocal` computed value in ModelTab.tsx — already detects all-local sessions, just needs display change

### Established Patterns
- Step skip pattern: `if (isWindows() && nextStep === 3) { nextStep = 4; }` — extend with local provider check
- `apiKeyStatus` reuse: local providers map health check success to "valid" status — onboarding can use this same mapping
- Provider `local` field on PROVIDERS entries drives all cloud/local branching in the frontend

### Integration Points
- `OnboardingWizard.handleNext()` — add local provider check alongside Windows check to skip step 1
- `ModelTab.tsx` lines 250-254 — change `$0.00` to "Free (local)" text
- `ModelTab.tsx` lines 262-263 — mixed session handling (remove `someUnpriced && !unpricedAreLocal` asterisk for local queries)

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

*Phase: 40-local-provider-frontend*
*Context gathered: 2026-03-17*
