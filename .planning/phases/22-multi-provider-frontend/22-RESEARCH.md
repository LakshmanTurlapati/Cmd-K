# Phase 22: Multi-Provider Frontend - Research

**Researched:** 2026-03-09
**Domain:** React UI components for multi-provider AI selection (onboarding, settings, model picker)
**Confidence:** HIGH

## Summary

This phase is a pure frontend task: extending the existing React + Zustand + Tailwind overlay app to support five AI providers (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) in onboarding and settings UI. All backend IPC is complete from Phase 21 -- `validate_api_key`, `fetch_models`, `get_api_key`, `save_api_key` all accept a `provider` parameter already, and the `ModelWithMeta` type includes a `tier` field ("fast", "balanced", "capable", or "" for uncategorized).

The work decomposes into four distinct areas: (1) new `StepProviderSelect` onboarding step, (2) provider dropdown in `AccountTab`, (3) tier-grouped model lists in `ModelTab` and `StepModelSelect`, and (4) store changes for per-provider model memory + settings persistence. There are no new dependencies needed -- everything uses existing Zustand state, Lucide icons, Tailwind styling, and Tauri `invoke()` IPC.

**Primary recommendation:** Build provider selection first (onboarding + settings), then tier grouping, then per-provider memory -- each layer builds on the previous.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Add provider selection as the **first step** in onboarding: Provider -> API Key -> Model -> Accessibility -> Done (5 steps total)
- Providers presented as a vertical **icon + name list** (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter)
- **No default pre-selected** -- user must explicitly choose a provider
- **Name only** -- no taglines, model lists, or descriptions next to provider names
- After selecting provider, API Key step adapts placeholder and validation to that provider
- **Dropdown above API key input** in the Account tab -- selecting a provider switches the key input below
- **Immediate switch** -- selecting a new provider immediately makes it the active provider, no "Save" button needed
- When switching providers, **show stored key status** for the new provider (last4 + validation status if key exists, empty input if not)
- Provider dropdown options show a **green checkmark** next to providers that have a valid key saved
- Settings tabs navigable via **arrow keys** (left/right to switch between Account, Model, Preferences, Advanced)
- Models grouped under **section headers**: "Fast", "Balanced", "Most Capable"
- **"All Models" section always visible** below the tier sections -- not collapsed, shows every model the API returned
- Models **auto-update on provider switch** -- model list refreshes immediately when provider changes (models already fetched during key validation)
- **Per-provider model selection remembered** -- switching back to a provider restores the previously selected model for that provider (store `selectedModel` per provider in settings.json)
- OpenRouter shown as **equal** in the provider list -- 5th option alongside the 4 direct providers, no special treatment
- Model list uses **same tier sections** (Fast, Balanced, Most Capable) as direct providers, just with more models in each tier
- **Same generic placeholder** for API key input -- no provider-specific help text or links
- Model list **filtered to chat-capable models only** -- removes image generation, embedding, and other non-applicable models

### Claude's Discretion
- Provider icon/logo assets and sizing
- Exact dropdown styling and animation
- How to handle the transition when provider changes and models are loading
- Section header styling for tier groups
- Arrow key navigation implementation details for settings tabs

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PFUI-01 | User can select a provider during first-run onboarding | New `StepProviderSelect` component as step 0; OnboardingWizard extended to 5 steps |
| PFUI-02 | User can switch providers in the settings Account tab | Provider dropdown added above API key input in `AccountTab.tsx`; `setSelectedProvider` triggers key status refresh |
| PFUI-03 | User can pick a model from a dropdown filtered to their selected provider | `ModelTab` and `StepModelSelect` already use `availableModels` from store, which is fetched per-provider via `fetch_models` IPC |
| PFUI-04 | Models grouped by capability tier (Fast, Balanced, Most Capable) across all providers | Backend already returns `tier` field on `ModelWithMeta`; frontend groups by tier value |
| PFUI-05 | User can switch providers without losing conversation history | Conversation history is tied to `windowKey` (per-window), not provider; `turnHistory` is unaffected by provider switch |
| ORTR-01 | User can use a single OpenRouter API key to access models from all supported providers | OpenRouter is equal in provider list; IPC already handles OpenRouter routing |
| ORTR-02 | OpenRouter model list is filtered to chat-capable models with sensible grouping | Backend already filters by `context_length > 0`; frontend needs additional filtering for chat-only models and tier grouping for OpenRouter models (no curated models exist for OpenRouter) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React | 19.1.0 | UI framework | Already in project |
| Zustand | 5.0.11 | State management | Single store pattern already established |
| Tailwind CSS | 4.2.0 | Styling | Glassmorphic overlay theme already defined |
| Lucide React | 0.575.0 | Icons | Check, ChevronDown, Loader2 already in use |
| @tauri-apps/api | 2.x | IPC with Rust backend | invoke() pattern established |
| @tauri-apps/plugin-store | 2.4.2 | Settings persistence | Store.load("settings.json") pattern established |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | -- | -- | All requirements met by existing stack |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Lucide icons for provider logos | SVG files or image assets | Lucide has no provider brand icons; use inline SVGs or simple text initials |

## Architecture Patterns

### Current Project Structure (relevant files)
```
src/
  components/
    Onboarding/
      OnboardingWizard.tsx    # Extend from 4 to 5 steps
      StepProviderSelect.tsx  # NEW: provider picker (step 0)
      StepApiKey.tsx          # Modify: provider-aware placeholder + validation
      StepModelSelect.tsx     # Modify: tier-grouped model list
      StepAccessibility.tsx   # Unchanged (now step 3 instead of 2)
      StepDone.tsx            # Modify: show provider in summary
    Settings/
      SettingsPanel.tsx       # Modify: add arrow key navigation
      AccountTab.tsx          # Modify: add provider dropdown above key input
      ModelTab.tsx            # Modify: tier-grouped model list
  store/
    index.ts                  # Modify: add selectedModels map for per-provider memory
```

### Pattern 1: Provider List as Constant Array
**What:** Define the provider list once, use everywhere (onboarding, settings, display names)
**When to use:** Any time provider options are rendered
**Example:**
```typescript
// Provider metadata for UI rendering
const PROVIDERS = [
  { id: "openai", name: "OpenAI" },
  { id: "anthropic", name: "Anthropic" },
  { id: "gemini", name: "Google Gemini" },
  { id: "xai", name: "xAI" },
  { id: "openrouter", name: "OpenRouter" },
] as const;
```
Note: Provider `id` values must match the Rust `Provider` enum's serde names: `"openai"`, `"anthropic"`, `"gemini"`, `"xai"`, `"openrouter"` (all lowercase).

### Pattern 2: Tier Grouping for Model Lists
**What:** Group `ModelWithMeta[]` by tier field, render under section headers
**When to use:** Both `ModelTab` and `StepModelSelect`
**Example:**
```typescript
const TIER_ORDER = [
  { key: "fast", label: "Fast" },
  { key: "balanced", label: "Balanced" },
  { key: "capable", label: "Most Capable" },
] as const;

function groupByTier(models: ModelWithMeta[]) {
  const groups: Record<string, ModelWithMeta[]> = {};
  for (const tier of TIER_ORDER) {
    groups[tier.key] = models.filter(m => m.tier === tier.key);
  }
  // "All Models" = the entire array (always visible below tier sections)
  return { tiers: groups, all: models };
}
```

### Pattern 3: Per-Provider State Reset on Switch
**What:** When provider changes, reset API key status, clear input, check stored key for new provider
**When to use:** `AccountTab` provider dropdown onChange, `StepProviderSelect` onSelect
**Example:**
```typescript
const handleProviderChange = async (provider: string) => {
  setSelectedProvider(provider);
  setApiKeyStatus("unknown");
  setApiKeyLast4("");
  setInputValue("");
  // Restore per-provider model selection
  const savedModels = /* from store */;
  if (savedModels[provider]) setSelectedModel(savedModels[provider]);
  // Check if key exists for this provider
  const key = await invoke<string | null>("get_api_key", { provider });
  if (key) {
    setApiKeyLast4(key.slice(-4));
    setApiKeyStatus("validating");
    // ... validate and fetch models
  }
};
```

### Pattern 4: Settings Persistence for Per-Provider Models
**What:** Store `selectedModels: Record<string, string>` in settings.json and Zustand
**When to use:** Whenever model selection changes or provider switches
**Example:**
```typescript
// In store
selectedModels: Record<string, string>;  // { "openai": "gpt-4o", "xai": "grok-3" }

// On model select
const handleModelSelect = async (modelId: string) => {
  setSelectedModel(modelId);
  const provider = useOverlayStore.getState().selectedProvider;
  const store = await Store.load("settings.json");
  const map = (await store.get<Record<string, string>>("selectedModels")) ?? {};
  map[provider] = modelId;
  await store.set("selectedModels", map);
  await store.save();
};
```

### Anti-Patterns to Avoid
- **Hardcoded "xAI" strings:** Both `StepApiKey.tsx` and `AccountTab.tsx` currently have hardcoded "Paste your xAI API key" placeholder and "xAI Model" headers. All must become provider-aware using `PROVIDERS.find(p => p.id === selectedProvider)?.name`.
- **Separate model state per provider in Zustand:** Do NOT create `openaiModels`, `anthropicModels`, etc. Use the existing single `availableModels` array -- it gets replaced on provider switch via `fetch_models` IPC.
- **Blocking UI on model fetch during provider switch:** Show a spinner/loading state while models load, do not freeze the dropdown.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Provider icon rendering | Custom SVG icon system | Simple Lucide fallback icons or styled text initials | Brand logos are trademark-sensitive; text initials ("O", "A", "G", "x", "OR") in styled circles are clean and legal |
| Dropdown component | Custom dropdown from scratch | Native `<select>` styled with Tailwind, or simple button+list pattern matching existing `ModelTab` button list | Existing codebase uses button lists, not dropdown libraries |
| Keyboard navigation for tabs | Complex focus management | Simple `onKeyDown` handler on the tab bar container | Only 4 tabs, left/right arrow is trivial |

## Common Pitfalls

### Pitfall 1: Onboarding Step Index Shift
**What goes wrong:** Inserting a new step 0 (Provider) shifts all subsequent step indices by +1 but persisted `onboardingStep` in settings.json from previous installations still uses old indices
**Why it happens:** Users upgrading from v0.2.4 may have partial onboarding progress saved
**How to avoid:** The onboarding step persisted in settings.json needs migration. If `onboardingComplete` is already true, this is moot. If onboarding is in-progress, treat any persisted step index as needing +1 offset, OR reset to step 0 (provider selection) for upgrading users.
**Warning signs:** Users see wrong step after upgrade

### Pitfall 2: Race Condition on Provider Switch
**What goes wrong:** User rapidly switches providers, old validation/model-fetch completes and overwrites new provider's state
**Why it happens:** Async `invoke` calls for the old provider complete after the new provider is selected
**How to avoid:** Track a "current provider" ref and discard results if provider changed during async operation. Use a simple counter or abortable pattern.
**Warning signs:** Wrong models appear for selected provider

### Pitfall 3: OpenRouter Model List is Huge
**What goes wrong:** OpenRouter returns hundreds of models, making the UI scroll extensively
**Why it happens:** OpenRouter aggregates all providers' models
**How to avoid:** The tier sections will be empty for OpenRouter (backend returns `tier: ""` for all OpenRouter models), so all models end up in "All Models". Consider frontend-side heuristics to assign tiers based on model name patterns (e.g., "mini" / "flash" = fast, "pro" / "opus" / "4" = capable), or accept that OpenRouter's "All Models" section will be long.
**Warning signs:** Empty tier sections, very long scroll for OpenRouter

### Pitfall 4: Missing Provider Persistence on Startup
**What goes wrong:** App.tsx `checkOnboarding` loads API key for `selectedProvider` which defaults to "xai" in store, but user may have selected a different provider
**Why it happens:** `selectedProvider` in settings.json isn't loaded before the API key check
**How to avoid:** Load `selectedProvider` from settings.json FIRST in App.tsx startup, then use it for API key validation
**Warning signs:** Wrong provider's API key validated on app startup

### Pitfall 5: Per-Provider Model Memory vs Global selectedModel
**What goes wrong:** Store has both `selectedModel` (global) and new `selectedModels` (per-provider map), causing confusion about source of truth
**Why it happens:** Migration from single-provider to multi-provider model selection
**How to avoid:** `selectedModel` remains the "currently active" model. `selectedModels` map is the persistence layer. On provider switch: `selectedModel = selectedModels[newProvider] ?? null`. On model select: `selectedModel = chosen`, `selectedModels[provider] = chosen`.
**Warning signs:** Model selection doesn't restore when switching back to a provider

## Code Examples

### New StepProviderSelect Component Structure
```typescript
// Source: project analysis - follows existing step component pattern
interface StepProviderSelectProps {
  onNext: () => void;
}

const PROVIDERS = [
  { id: "openai", name: "OpenAI" },
  { id: "anthropic", name: "Anthropic" },
  { id: "gemini", name: "Google Gemini" },
  { id: "xai", name: "xAI" },
  { id: "openrouter", name: "OpenRouter" },
] as const;

export function StepProviderSelect({ onNext }: StepProviderSelectProps) {
  const selectedProvider = useOverlayStore((s) => s.selectedProvider);
  const setSelectedProvider = useOverlayStore((s) => s.setSelectedProvider);
  const [chosen, setChosen] = useState<string | null>(null);

  const handleSelect = (providerId: string) => {
    setChosen(providerId);
    setSelectedProvider(providerId);
  };

  // No default pre-selected; user must click one
  return (
    <div className="flex flex-col gap-3">
      <p className="text-white/60 text-sm">Choose your AI provider</p>
      <div className="flex flex-col gap-1">
        {PROVIDERS.map((p) => (
          <button
            key={p.id}
            onClick={() => handleSelect(p.id)}
            className={/* selected vs unselected styling */}
          >
            {/* Icon/initial + name */}
            <span>{p.name}</span>
          </button>
        ))}
      </div>
      <button onClick={onNext} disabled={!chosen}>Next</button>
    </div>
  );
}
```

### Tier-Grouped Model List Rendering
```typescript
// Source: project analysis - matches existing ModelTab button list pattern
const TIER_DISPLAY: Record<string, string> = {
  fast: "Fast",
  balanced: "Balanced",
  capable: "Most Capable",
};

function TieredModelList({ models, selectedModel, onSelect }) {
  const tiered = ["fast", "balanced", "capable"].map(tier => ({
    tier,
    label: TIER_DISPLAY[tier],
    items: models.filter(m => m.tier === tier),
  })).filter(g => g.items.length > 0);

  return (
    <div className="flex flex-col gap-3">
      {tiered.map(group => (
        <div key={group.tier}>
          <p className="text-white/30 text-xs uppercase tracking-wider mb-1">
            {group.label}
          </p>
          {group.items.map(model => (
            <ModelButton key={model.id} model={model} selected={selectedModel === model.id} onSelect={onSelect} />
          ))}
        </div>
      ))}
      {/* All Models section - always visible */}
      <div>
        <p className="text-white/30 text-xs uppercase tracking-wider mb-1">All Models</p>
        {models.map(model => (
          <ModelButton key={model.id} model={model} selected={selectedModel === model.id} onSelect={onSelect} />
        ))}
      </div>
    </div>
  );
}
```

### Provider Dropdown in AccountTab
```typescript
// Source: project analysis - follows existing glassmorphic styling
function ProviderDropdown({ selected, onSelect, providerKeyStatus }) {
  const [open, setOpen] = useState(false);

  return (
    <div className="relative">
      <button
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between bg-white/8 border border-white/10 rounded-lg px-3 py-2 text-sm text-white"
      >
        <span>{PROVIDERS.find(p => p.id === selected)?.name}</span>
        <ChevronDown size={14} className="text-white/40" />
      </button>
      {open && (
        <div className="absolute top-full left-0 right-0 mt-1 bg-[#2a2a2c]/95 backdrop-blur-xl border border-white/10 rounded-lg overflow-hidden z-50">
          {PROVIDERS.map(p => (
            <button
              key={p.id}
              onClick={() => { onSelect(p.id); setOpen(false); }}
              className="w-full flex items-center justify-between px-3 py-2 text-sm text-white/70 hover:bg-white/8"
            >
              <span>{p.name}</span>
              {providerKeyStatus[p.id] === "valid" && (
                <Check size={14} className="text-green-400" />
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
```

### Arrow Key Tab Navigation
```typescript
// Source: project analysis - simple keydown handler
// Add to SettingsPanel.tsx
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    const tabIds = TABS.map(t => t.id);
    const currentIndex = tabIds.indexOf(settingsTab);
    if (e.key === "ArrowRight" && currentIndex < tabIds.length - 1) {
      setSettingsTab(tabIds[currentIndex + 1]);
    } else if (e.key === "ArrowLeft" && currentIndex > 0) {
      setSettingsTab(tabIds[currentIndex - 1]);
    }
  };
  window.addEventListener("keydown", handleKeyDown);
  return () => window.removeEventListener("keydown", handleKeyDown);
}, [settingsTab, setSettingsTab]);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Single hardcoded "xai" provider | Multi-provider selection | Phase 21 (backend) / Phase 22 (frontend) | All UI text referencing "xAI" must become dynamic |
| Flat model list | Tier-grouped model list | Phase 22 | `tier` field on ModelWithMeta enables grouping |
| Single `selectedModel` global | Per-provider `selectedModels` map | Phase 22 | settings.json schema changes |
| 4-step onboarding | 5-step onboarding (Provider first) | Phase 22 | Step indices shift +1 |

**Key migration concern:** Persisted `onboardingStep` in settings.json uses old indices. Must handle v0.2.4 -> v0.2.6 upgrade path.

## Open Questions

1. **OpenRouter tier assignment**
   - What we know: Backend returns `tier: ""` for all OpenRouter models (no curated list). Direct providers have curated models with tier tags.
   - What's unclear: Should the frontend assign tiers to OpenRouter models based on naming heuristics, or leave them all in "All Models"?
   - Recommendation: Leave all OpenRouter models in "All Models" section for now. Tier assignment heuristics would be fragile and model names vary across providers. The user can still browse and select any model.

2. **Provider icon assets**
   - What we know: Lucide has no brand-specific icons for AI providers.
   - What's unclear: Whether to use custom SVG icons, text initials, or a generic icon.
   - Recommendation: Use styled text initials in circles (matching the glassmorphic aesthetic). Clean, legally safe, no asset management. E.g., "O" for OpenAI, "A" for Anthropic, "G" for Gemini, "x" for xAI, "OR" for OpenRouter.

3. **Checking key validity status for all providers (green checkmark in dropdown)**
   - What we know: The dropdown should show a green checkmark next to providers with valid keys. But currently only the active provider's key is validated.
   - What's unclear: Should we validate all 5 provider keys when opening settings, or cache validity status?
   - Recommendation: On settings open / provider dropdown open, check `get_api_key` for each provider. If a key exists, mark that provider as "has key" (show checkmark). Full validation only happens for the active provider. This avoids 5 API calls on every settings open.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | None (no test framework configured in project) |
| Config file | none |
| Quick run command | N/A |
| Full suite command | `npm run build` (TypeScript compile check) |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PFUI-01 | Provider selection in onboarding | manual-only | Visual: launch app with fresh settings.json | N/A |
| PFUI-02 | Provider switching in settings | manual-only | Visual: open settings, switch providers | N/A |
| PFUI-03 | Model dropdown filtered to provider | manual-only | Visual: validate key, check model list | N/A |
| PFUI-04 | Tier-grouped model display | manual-only | Visual: check section headers in model list | N/A |
| PFUI-05 | Provider switch preserves history | manual-only | Visual: query, switch provider, check history | N/A |
| ORTR-01 | OpenRouter single key access | manual-only | Visual: enter OpenRouter key, check models | N/A |
| ORTR-02 | OpenRouter chat-model filtering | manual-only | Visual: check OpenRouter model list | N/A |

### Sampling Rate
- **Per task commit:** `npm run build` (TypeScript compilation verifies type safety)
- **Per wave merge:** `npm run build` + manual smoke test
- **Phase gate:** Full build + manual verification of all 7 requirements

### Wave 0 Gaps
- No test framework exists in the project -- all verification is manual + TypeScript compile
- This is consistent with the project's established pattern (22 prior phases, 0 test files)

## Sources

### Primary (HIGH confidence)
- **Project source code analysis** -- read all 12 files that will be created/modified
- **Backend provider enum** -- `src-tauri/src/commands/providers/mod.rs` defines exact provider IDs and display names
- **Backend model tiers** -- `src-tauri/src/commands/models.rs` defines curated models with tier tags per provider
- **Zustand store** -- `src/store/index.ts` defines all existing state and actions

### Secondary (MEDIUM confidence)
- **Tauri plugin-store API** -- `Store.load()`, `store.get()`, `store.set()`, `store.save()` pattern verified in existing code

### Tertiary (LOW confidence)
- None -- all findings based on direct source code analysis

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all libraries already in project, no new deps needed
- Architecture: HIGH -- all patterns derived from existing codebase analysis
- Pitfalls: HIGH -- identified from direct code review of race conditions, migration needs, and state management

**Research date:** 2026-03-09
**Valid until:** 2026-04-09 (stable -- pure frontend, no external API changes expected)
