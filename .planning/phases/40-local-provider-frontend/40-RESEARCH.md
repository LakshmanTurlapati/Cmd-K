# Phase 40: Local Provider Frontend - Research

**Researched:** 2026-03-17
**Domain:** React/TypeScript frontend -- onboarding wizard step-skip logic, usage cost display
**Confidence:** HIGH

## Summary

Phase 40 is a small, well-scoped frontend-only phase. Most of the LFUI requirements (LFUI-01, LFUI-02, LFUI-04) were already completed in Phase 37 -- the remaining work is two focused changes: (1) onboarding wizard skips the API key step for local providers, and (2) the usage cost display shows "Free (local)" instead of "$0.00" for all-local sessions.

The codebase already has all the necessary infrastructure: the `PROVIDERS` array has the `local` boolean field, the `OnboardingWizard.tsx` already has a step-skip pattern for Windows/Accessibility, and `ModelTab.tsx` already computes `allUnpricedAreLocal`. Both changes are straightforward conditional logic adjustments.

**Primary recommendation:** Two small, targeted edits -- one in OnboardingWizard.tsx (step skip) and one in ModelTab.tsx (cost label). No new files, libraries, or architectural changes needed.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Auto-skip the API key step (step index 1) when a local provider is selected -- go straight from Provider Select to Model Select
- Same pattern as the existing Accessibility step skip on Windows (`isWindows() && nextStep === 3`)
- Keep the 5-step stepper layout unchanged -- the skipped API key dot shows as completed (checkmark), not removed
- No back navigation -- current onboarding has no back button, keep it that way
- If local provider server isn't running during Model Select step, show "No models found -- is your server running?" but allow proceeding via existing "Skip this step" link
- Replace `$0.00` with visible text "Free (local)" for all-local-unpriced sessions
- Token counts still shown after the dash: `Free (local) -- 1,234 in / 567 out`
- Mixed sessions (local + cloud queries): show cloud cost total normally, no special annotation for local queries
- Remove the tooltip-only approach -- "Free (local)" should be the visible label
- Keep current generic SVG icons for Ollama and LM Studio
- Already implemented (Phase 37): LFUI-01, LFUI-02, LFUI-04

### Claude's Discretion
- Exact conditional logic placement in OnboardingWizard.tsx for the step skip
- Whether to extract the local provider check into a shared utility or inline it

### Deferred Ideas (OUT OF SCOPE)
None.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LFUI-01 | Connection health checkmark indicator | Already done in Phase 37 (AccountTab lines 320-374). No work needed. |
| LFUI-02 | Base URL input instead of API key field | Already done in Phase 37 (AccountTab lines 320-374). No work needed. |
| LFUI-03 | Onboarding wizard skips API key step for local providers | OnboardingWizard.tsx `handleNext` needs local provider check at step 0->1 transition. StepModelSelect.tsx needs empty-state text update for local providers. |
| LFUI-04 | Provider SVG icons for Ollama and LM Studio | Already done in Phase 37 (ProviderIcon.tsx). No work needed. |
</phase_requirements>

## Standard Stack

No new libraries needed. This phase modifies existing React components only.

### Core (already in project)
| Library | Purpose | Relevant to Phase |
|---------|---------|-------------------|
| React 18 | UI framework | Component logic changes |
| Zustand | State management (`useOverlayStore`) | Reading `selectedProvider` state |
| TailwindCSS | Styling | No style changes needed |

## Architecture Patterns

### Pattern 1: Step Skip in OnboardingWizard

**What:** The existing `handleNext` function in OnboardingWizard.tsx already skips steps based on conditions. The Windows/Accessibility skip at line 31 is the exact pattern to replicate.

**Current code (line 28-36):**
```typescript
const handleNext = async () => {
  let nextStep = onboardingStep + 1;
  // Skip Accessibility step (index 3) on Windows -- not required
  if (isWindows() && nextStep === 3) {
    nextStep = 4;
  }
  setOnboardingStep(nextStep);
  await persistStep(nextStep);
};
```

**Required change:** Add a check before the Windows check: if the selected provider is local and `nextStep === 1`, skip to step 2 (Model Select). The provider's `local` field is accessible via `PROVIDERS.find(p => p.id === selectedProvider)?.local`.

**Key detail:** The `selectedProvider` is already in the Zustand store and set by StepProviderSelect before `onNext` (handleNext) is called. The wizard already reads from the store at the component level -- `handleNext` needs access to `selectedProvider` which must be read inside the handler (or via store selector).

**Implementation approach:** Read `selectedProvider` from the store inside `handleNext`, look it up in PROVIDERS, check `.local`, and skip step 1 if true. This mirrors how `isWindows()` is used -- a simple boolean check.

```typescript
const handleNext = async () => {
  let nextStep = onboardingStep + 1;
  const provider = useOverlayStore.getState().selectedProvider;
  const isLocalProvider = PROVIDERS.find((p) => p.id === provider)?.local ?? false;
  // Skip API Key step (index 1) for local providers -- no key needed
  if (isLocalProvider && nextStep === 1) {
    nextStep = 2;
  }
  // Skip Accessibility step (index 3) on Windows -- not required
  if (isWindows() && nextStep === 3) {
    nextStep = 4;
  }
  setOnboardingStep(nextStep);
  await persistStep(nextStep);
};
```

**Why `useOverlayStore.getState()` inside the handler:** This pattern is already used in StepModelSelect.tsx (line 53) and StepProviderSelect.tsx for reading current state imperatively. It avoids stale closure issues since `selectedProvider` may change between render and click.

### Pattern 2: Local Provider Health Trigger for Onboarding

**What:** When a local provider is selected and the API key step is skipped, the health check and model fetch must still happen. Currently, StepApiKey triggers validation and model fetching. With the step skipped, this must happen elsewhere.

**Key insight:** Looking at the flow:
1. StepProviderSelect sets `selectedProvider` in store
2. StepApiKey reads `selectedProvider` and triggers `validate_api_key` + `fetch_models`
3. StepModelSelect reads `availableModels` to show model list

When step 1 (API Key) is skipped, StepApiKey never mounts, so validation/fetch never triggers. The health check for local providers is already handled by the overlay-open event (from Phase 37's silent health check), but model fetching needs to happen.

**Solution:** StepModelSelect already has a model refresh effect for local providers in ModelTab.tsx. Check if StepModelSelect handles the case where it mounts without models -- it shows "No models available" with text "Configure API key first to select a model". This text needs to change for local providers to "No models found -- is your server running?" per the locked decision.

Additionally, StepModelSelect should trigger a `fetch_models` call on mount for local providers (similar to ModelTab.tsx lines 59-73). The existing "Skip this step" link already allows proceeding.

### Pattern 3: Cost Display Text Change

**What:** ModelTab.tsx lines 250-254 currently show `$0.00` with a tooltip for all-local-unpriced sessions. Change to visible "Free (local)" text.

**Current code (lines 250-254):**
```typescript
{allUnpricedAreLocal ? (
  <span title="Free (local model)">
    $0.00
    <span className="text-white/40"> &mdash; {tokenStr}</span>
  </span>
)
```

**Required change:**
```typescript
{allUnpricedAreLocal ? (
  <span>
    Free (local)
    <span className="text-white/40"> &mdash; {tokenStr}</span>
  </span>
)
```

Remove the `title` attribute (no longer tooltip-only) and replace `$0.00` with `Free (local)`.

### Anti-Patterns to Avoid
- **Modifying the stepper UI:** The 5-step layout must stay unchanged. The skipped step shows a checkmark (completed), not removed.
- **Adding a local-specific onboarding step:** No new steps -- just skip the API key step.
- **Triggering health check from OnboardingWizard:** Let StepModelSelect handle model fetching for local providers, not the wizard itself.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Provider local detection | Custom provider type system | `PROVIDERS.find(p => p.id === id)?.local` | Already established pattern used everywhere |
| Model fetching for local | New fetch trigger system | `invoke("fetch_models", ...)` in StepModelSelect useEffect | Same pattern as ModelTab.tsx |

## Common Pitfalls

### Pitfall 1: Stale selectedProvider in handleNext closure
**What goes wrong:** Using the `selectedProvider` from the component's render-time selector inside `handleNext` may be stale if the user rapidly clicks.
**Why it happens:** React closures capture values at render time, not call time.
**How to avoid:** Use `useOverlayStore.getState().selectedProvider` inside the handler (imperative read), exactly as done in StepModelSelect.tsx line 53.
**Warning signs:** Step skip works on second onboarding attempt but not first.

### Pitfall 2: StepModelSelect mounts without models for local providers
**What goes wrong:** When API key step is skipped, StepApiKey never triggers model fetch. StepModelSelect shows empty state.
**Why it happens:** Model fetching was coupled to the API key validation flow.
**How to avoid:** Add a useEffect in StepModelSelect that triggers `fetch_models` for local providers on mount (same pattern as ModelTab.tsx lines 59-73).
**Warning signs:** "No models available" shown even when Ollama is running.

### Pitfall 3: Health check not triggered before model fetch
**What goes wrong:** `fetch_models` fails because the backend health check hasn't run yet for the local provider.
**Why it happens:** The health check normally runs during API key validation or overlay-open event.
**How to avoid:** Call `validate_api_key` (which is the health check for local providers) before `fetch_models` in the StepModelSelect mount effect. Or trigger health check + model fetch together.
**Warning signs:** "Connect to server first" message in model select even when server is running.

### Pitfall 4: Mixed session asterisk still showing for local queries
**What goes wrong:** The asterisk footnote appears for sessions with both local and cloud queries.
**Why it happens:** The `someUnpriced && !unpricedAreLocal` check (line 262) should correctly suppress it already, but verify it handles the mixed case.
**How to avoid:** The existing `unpricedAreLocal` computed value (lines 223-228) already handles this -- unpriced entries that are all local don't trigger the asterisk.
**Warning signs:** Asterisk appears when switching between local and cloud providers in same session.

## Code Examples

### OnboardingWizard.tsx -- Complete handleNext with local skip

```typescript
// In OnboardingWizard component, add PROVIDERS import and modify handleNext:
import { PROVIDERS } from "@/store";

const handleNext = async () => {
  let nextStep = onboardingStep + 1;
  const provider = useOverlayStore.getState().selectedProvider;
  const isLocalProvider = PROVIDERS.find((p) => p.id === provider)?.local ?? false;
  // Skip API Key step (index 1) for local providers -- no key needed
  if (isLocalProvider && nextStep === 1) {
    nextStep = 2;
  }
  // Skip Accessibility step (index 3) on Windows -- not required
  if (isWindows() && nextStep === 3) {
    nextStep = 4;
  }
  setOnboardingStep(nextStep);
  await persistStep(nextStep);
};
```

### StepModelSelect.tsx -- Local provider model fetch on mount

```typescript
// Add to StepModelSelect, after existing hooks:
const currentProv = PROVIDERS.find((p) => p.id === selectedProvider);
const isLocal = currentProv?.local ?? false;

useEffect(() => {
  if (!isLocal) return;
  const fetchLocal = async () => {
    try {
      // Health check first (same as validate_api_key for local providers)
      await invoke("validate_api_key", { provider: selectedProvider, apiKey: "" });
      setApiKeyStatus("valid");
      const models = await invoke<ModelWithMeta[]>(
        "fetch_models",
        { provider: selectedProvider, apiKey: "" }
      );
      setModels(models);
    } catch {
      // Server not running -- show empty state message
      setApiKeyStatus("invalid");
    }
  };
  fetchLocal();
}, [selectedProvider]); // eslint-disable-line react-hooks/exhaustive-deps
```

### StepModelSelect.tsx -- Updated empty state for local providers

```typescript
// Replace the current empty state text:
{!hasModels ? (
  <div className="flex flex-col gap-2">
    <div className="w-full bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white/30">
      {isLocal ? "No models found" : "No models available"}
    </div>
    <p className="text-white/30 text-xs">
      {isLocal
        ? "Is your server running?"
        : "Configure API key first to select a model"}
    </p>
  </div>
)
```

### ModelTab.tsx -- "Free (local)" label

```typescript
// Replace lines 250-254:
{allUnpricedAreLocal ? (
  <span>
    Free (local)
    <span className="text-white/40"> &mdash; {tokenStr}</span>
  </span>
)
```

## State of the Art

No technology changes -- all patterns are established in the existing codebase.

| Old Approach (Phase 39) | New Approach (Phase 40) | Impact |
|--------------------------|-------------------------|--------|
| `$0.00` with tooltip for local providers | "Free (local)" visible label | Clearer UX for local users |
| API key step always shown in onboarding | Skipped for local providers | Smoother local provider onboarding |

## Open Questions

1. **Store access for setApiKeyStatus/setModels in StepModelSelect**
   - What we know: StepModelSelect currently reads `apiKeyStatus` and `availableModels` from store but does not import `setApiKeyStatus` or `setModels` setters
   - What's unclear: Whether to add these selectors to StepModelSelect or trigger the health check differently
   - Recommendation: Add `setApiKeyStatus` and `setModels` selectors -- same pattern used in StepApiKey.tsx. Minimal additional coupling since these are already Zustand store actions.

## Sources

### Primary (HIGH confidence)
- Direct code inspection of OnboardingWizard.tsx, StepApiKey.tsx, StepModelSelect.tsx, StepProviderSelect.tsx, ModelTab.tsx, AccountTab.tsx, store/index.ts
- Existing step-skip pattern (OnboardingWizard.tsx line 31)
- Existing `allUnpricedAreLocal` computation (ModelTab.tsx line 219)
- PROVIDERS array with `local` field (store/index.ts line 4-12)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new libraries, pure code changes
- Architecture: HIGH - all patterns already exist in codebase, just extending them
- Pitfalls: HIGH - identified from direct code reading, especially the model fetch gap

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable -- internal codebase patterns)
