---
phase: 40-local-provider-frontend
verified: 2026-03-17T23:00:00Z
status: human_needed
score: 5/5 must-haves verified
re_verification: false
human_verification:
  - test: "Start onboarding with Ollama selected, click Next from Provider step"
    expected: "API Key step is skipped entirely; you land directly on Model Select (step 3). The stepper shows step 2 (API Key) with a checkmark, not removed."
    why_human: "Cannot programmatically test the rendered step sequence and stepper visual state at runtime"
  - test: "On Model Select step with Ollama selected and no server running"
    expected: "Shows 'No models found' in the model box and 'Is your server running?' as subtitle text"
    why_human: "Cannot invoke Tauri backend in static analysis; needs runtime verification with server offline"
  - test: "On Model Select step with Ollama selected and server running"
    expected: "Model list populates with models from the running server"
    why_human: "Requires a live Ollama instance; cannot verify dynamically fetched model list statically"
  - test: "Run a query with Ollama or LM Studio, then open Settings > Model tab"
    expected: "Usage stats show 'Free (local) — X in / Y out' with no dollar sign"
    why_human: "Requires a real query session to populate usage stats"
  - test: "Run onboarding with a cloud provider (e.g. OpenAI)"
    expected: "API Key step is NOT skipped; onboarding proceeds normally through the API Key step"
    why_human: "Regression check on cloud provider path requires runtime flow"
---

# Phase 40: Local Provider Frontend Verification Report

**Phase Goal:** The settings and onboarding UI adapts for local providers -- URL configuration instead of API keys, connection health instead of key validation, and provider branding
**Verified:** 2026-03-17T23:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from PLAN must_haves)

| #  | Truth | Status | Evidence |
|----|-------|--------|---------|
| 1  | Onboarding with a local provider selected skips the API Key step and lands on Model Select | VERIFIED | `OnboardingWizard.tsx:33` — `if (isLocalProvider && nextStep === 1) { nextStep = 2; }` — correctly jumps from step 0 to step 2 |
| 2  | Model Select step fetches and displays models from a running local server during onboarding | VERIFIED | `StepModelSelect.tsx:36-52` — `useEffect` with `invoke("validate_api_key")` + `invoke<ModelWithMeta[]>("fetch_models")` fires on mount when `isLocal` |
| 3  | Model Select shows 'No models found' / 'Is your server running?' when local server is not running | VERIFIED | `StepModelSelect.tsx:144,147-149` — `isLocal ? "No models found"` and `isLocal ? "Is your server running?"` both present |
| 4  | Usage stats display 'Free (local)' instead of '$0.00' for all-local sessions | VERIFIED | `ModelTab.tsx:251-254` — `allUnpricedAreLocal` branch renders `Free (local)` as visible text; no `$0.00` string exists in the file |
| 5  | Skipped API Key step appears as completed (checkmark) in stepper, not removed | VERIFIED | `OnboardingWizard.tsx:10` — `TOTAL_STEPS = 5` unchanged; stepper uses `index < onboardingStep` (line 83) for checkmark, so skipped step 1 renders as completed when step is 2 |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/components/Onboarding/OnboardingWizard.tsx` | Local provider step-skip logic in handleNext | VERIFIED | Contains `isLocalProvider && nextStep === 1` at line 33; PROVIDERS import at line 2; `useOverlayStore.getState().selectedProvider` at line 31; local check precedes Windows check |
| `src/components/Onboarding/StepModelSelect.tsx` | Local provider model fetch on mount, local-specific empty state | VERIFIED | Contains `useEffect` with both `invoke` calls; `isLocal ? "No models found"` at line 144; `isLocal ? "Is your server running?"` at line 147 |
| `src/components/Settings/ModelTab.tsx` | Free (local) cost label | VERIFIED | `Free (local)` at line 252 as visible text; no `title=` attribute; no `$0.00` string present |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `OnboardingWizard.tsx` | `src/store/index.ts` | `PROVIDERS.find(...).local` in handleNext | WIRED | `import { useOverlayStore, PROVIDERS }` at line 2; `PROVIDERS.find((p) => p.id === provider)?.local` at line 32 |
| `StepModelSelect.tsx` | Tauri backend | `invoke("validate_api_key")` + `invoke("fetch_models")` in useEffect | WIRED | Both invocations present at lines 40 and 42-45; `setModels(models)` wires result to store at line 46 |
| `AccountTab.tsx` | Tauri backend | URL input + `invoke("validate_api_key")` health check | WIRED | `baseUrlInput` state at line 18; debounced URL save + health check effect at lines 181-216; `isLocal` branch renders Server URL input instead of API key field at lines 320-374 |
| `StepProviderSelect.tsx` | `ProviderIcon.tsx` | `ProviderIcon provider={provider.id}` | WIRED | `ProviderIcon` imported at line 4; rendered at line 51 for every provider including Ollama and LM Studio |
| `AccountTab.tsx` | `ProviderIcon.tsx` | `ProviderIcon provider={selectedProvider}` and per-provider dropdown | WIRED | Used at lines 287 and 307 in provider selector and dropdown |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| LFUI-01 | 40-01-PLAN.md (via Phase 37) | Connection health checkmark when server reachable | SATISFIED | `AccountTab.tsx:347-348` — `apiKeyStatus === "valid"` renders `<Check>` icon for local providers; health check via `invoke("validate_api_key")` at line 55 |
| LFUI-02 | 40-01-PLAN.md (via Phase 37) | Settings shows base URL input instead of API key field for local providers | SATISFIED | `AccountTab.tsx:320-374` — `isLocal` branch renders "Server URL" label + text input with `baseUrlInput` value; cloud providers continue showing API key field |
| LFUI-03 | 40-01-PLAN.md | Onboarding wizard skips API key step for local providers | SATISFIED | `OnboardingWizard.tsx:30-35` — `isLocalProvider && nextStep === 1` skips to step 2; verified at code level |
| LFUI-04 | 40-01-PLAN.md (via Phase 37) | Provider SVG icons for Ollama and LM Studio | SATISFIED | `ProviderIcon.tsx:42-54` — distinct `ollama` and `lmstudio` entries with SVG path data; used in both `StepProviderSelect.tsx` (onboarding) and `AccountTab.tsx` (settings) |

**Note on LFUI-01, LFUI-02, LFUI-04:** The 40-01-PLAN.md claims all four requirements but its tasks only implemented LFUI-03 and the "Free (local)" label (part of the Phase 40 success criteria). LFUI-01, LFUI-02, and LFUI-04 were implemented in Phase 37 and confirmed present in the codebase as of this verification. The requirement IDs are correctly attributed to Phase 40 as the phase responsible for their final delivery.

### ROADMAP Success Criteria vs. Plan Must-Haves

The ROADMAP defines four success criteria for Phase 40. Three are directly in the plan's must_haves; one (SC1) is not explicitly in the must_haves but is present in the codebase:

| SC# | Success Criterion | Status | Evidence |
|-----|------------------|--------|---------|
| SC1 | Settings shows base URL input (not API key) with checkmark when server reachable | VERIFIED | `AccountTab.tsx:320-374` — Server URL input rendered when `isLocal`; Check icon at line 347 |
| SC2 | Onboarding with local provider skips API key step | VERIFIED | `OnboardingWizard.tsx:33-35` |
| SC3 | Ollama and LM Studio have distinct SVG icons in both onboarding and settings | VERIFIED | `ProviderIcon.tsx:42-54`; used in `StepProviderSelect.tsx:51` and `AccountTab.tsx:287,307` |
| SC4 | Usage stats display "Free (local)" instead of dollar amount | VERIFIED | `ModelTab.tsx:252` |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `StepModelSelect.tsx` | 188 | `// eslint-disable-line react-hooks/exhaustive-deps` on useEffect for local fetch | Info | Intentional — `selectedProvider` is the only meaningful dep; missing `isLocal`, `setApiKeyStatus`, `setModels` are stable store selectors |
| `ModelTab.tsx` | 73 | `// eslint-disable-line react-hooks/exhaustive-deps` on model refresh effect | Info | Intentional — effect runs once on mount to refresh models |

No blocker or warning anti-patterns found. No stub implementations, no TODO/FIXME comments, no empty handlers, no static/hardcoded return values in modified files.

### Commit Verification

| Commit | Message | Files |
|--------|---------|-------|
| `aeb4017` | feat(40-01): add local provider onboarding step-skip and model fetch | `OnboardingWizard.tsx`, `StepModelSelect.tsx` |
| `001a41d` | feat(40-01): replace $0.00 with Free (local) in usage cost display | `ModelTab.tsx` |

Both commits exist and match the declared files in the SUMMARY.

### Human Verification Required

#### 1. Onboarding Step Skip — Visual Stepper State

**Test:** Start onboarding with Ollama selected, click Next from Provider step.
**Expected:** API Key step is bypassed entirely; you land on Model Select (step 3 in UI). The stepper shows the API Key node (second dot) with a checkmark icon, not missing or empty.
**Why human:** Cannot programmatically execute the rendered step-skip flow or verify the checkmark renders on the visually skipped step.

#### 2. Model Select Empty State — Server Offline

**Test:** Select Ollama as provider during onboarding with no Ollama server running.
**Expected:** Model Select step shows "No models found" in the model input box and "Is your server running?" as subtitle text.
**Why human:** Requires runtime Tauri backend invocation to fail; static analysis confirms the strings are present and branched correctly but cannot verify the runtime exception path.

#### 3. Model Select Populated State — Server Online

**Test:** Select Ollama as provider during onboarding with Ollama server running and at least one model loaded.
**Expected:** Model list populates with models from the server; one is auto-selected.
**Why human:** Requires a live Ollama instance.

#### 4. Usage Display "Free (local)"

**Test:** Complete a query with a local provider (Ollama or LM Studio), then open Settings > Model tab.
**Expected:** Usage stats section shows "Free (local) — X in / Y out" with no dollar sign.
**Why human:** Requires a real query to populate `usageStats` with entries that have `pricing_available: false` and a local provider name.

#### 5. Cloud Provider Regression

**Test:** Start onboarding with OpenAI or Anthropic selected, click Next.
**Expected:** The API Key step (step 2) is NOT skipped; onboarding proceeds through the API Key step normally.
**Why human:** Validates the negative path of the `isLocalProvider` check at runtime.

### Gaps Summary

No gaps found. All five must-have truths are verified at the code level. All four requirement IDs (LFUI-01 through LFUI-04) have implementation evidence. All four ROADMAP success criteria are satisfied in the codebase. The five human verification items listed are runtime behavior checks that cannot be automated via static analysis — they require the Tauri application running with or without a local server.

---

_Verified: 2026-03-17T23:00:00Z_
_Verifier: Claude (gsd-verifier)_
