# Phase 22: Multi-Provider Frontend - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can discover, select, and switch AI providers through polished onboarding and settings UI. Includes provider selection in onboarding, provider switching in settings Account tab, tiered model picker, OpenRouter integration, and per-provider model memory. Backend IPC is already complete from Phase 21.

</domain>

<decisions>
## Implementation Decisions

### Provider Picker in Onboarding
- Add provider selection as the **first step** in onboarding: Provider → API Key → Model → Accessibility → Done (5 steps total)
- Providers presented as a vertical **icon + name list** (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter)
- **No default pre-selected** — user must explicitly choose a provider
- **Name only** — no taglines, model lists, or descriptions next to provider names
- After selecting provider, API Key step adapts placeholder and validation to that provider

### Provider Switching in Settings
- **Dropdown above API key input** in the Account tab — selecting a provider switches the key input below
- **Immediate switch** — selecting a new provider immediately makes it the active provider, no "Save" button needed
- When switching providers, **show stored key status** for the new provider (last4 + validation status if key exists, empty input if not)
- Provider dropdown options show a **green checkmark** next to providers that have a valid key saved
- Settings tabs navigable via **arrow keys** (left/right to switch between Account, Model, Preferences, Advanced)

### Model Picker with Tier Grouping
- Models grouped under **section headers**: "Fast", "Balanced", "Most Capable"
- **"All Models" section always visible** below the tier sections — not collapsed, shows every model the API returned
- Models **auto-update on provider switch** — model list refreshes immediately when provider changes (models already fetched during key validation)
- **Per-provider model selection remembered** — switching back to a provider restores the previously selected model for that provider (store `selectedModel` per provider in settings.json)

### OpenRouter Positioning
- OpenRouter shown as **equal** in the provider list — 5th option alongside the 4 direct providers, no special treatment
- Model list uses **same tier sections** (Fast, Balanced, Most Capable) as direct providers, just with more models in each tier
- **Same generic placeholder** for API key input — no provider-specific help text or links
- Model list **filtered to chat-capable models only** — removes image generation, embedding, and other non-applicable models

### Claude's Discretion
- Provider icon/logo assets and sizing
- Exact dropdown styling and animation
- How to handle the transition when provider changes and models are loading
- Section header styling for tier groups
- Arrow key navigation implementation details for settings tabs

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `useOverlayStore`: Already has `selectedProvider`, `setSelectedProvider`, `availableModels`, `setModels`, `selectedModel`, `setSelectedModel` — all wired up
- `ModelWithMeta { id, label, tier }`: Model type with tier field already present
- `AccountTab.tsx`: API key input with debounced validation, reveal/hide toggle, status indicators — extend with provider dropdown
- `StepApiKey.tsx`: Onboarding API key step — adapt to use selected provider instead of hardcoded "xAI"
- `StepModelSelect.tsx`: Onboarding model step — add tier grouping
- `ModelTab.tsx`: Settings model picker — add tier grouping and "All Models" section
- `OnboardingWizard.tsx`: 4-step wizard with stepper UI — extend to 5 steps with provider step first

### Established Patterns
- Tauri `invoke()` for IPC: `validate_api_key`, `fetch_models`, `get_api_key`, `save_api_key` all accept `provider` param already
- `Store.load("settings.json")` for persisting user preferences (model selection, onboarding progress)
- Zustand store for all UI state — single store pattern
- Lucide icons for UI elements (Check, X, Loader2, AlertCircle, Eye, EyeOff)
- Tailwind CSS with `white/opacity` color pattern for glassmorphic overlay

### Integration Points
- `OnboardingWizard.tsx`: Insert new `StepProviderSelect` component as step 0, shift other steps +1
- `AccountTab.tsx`: Add provider dropdown above existing API key input
- `ModelTab.tsx` + `StepModelSelect.tsx`: Add tier section headers and "All Models" section
- `store/index.ts`: Add per-provider model memory (e.g., `selectedModels: Record<string, string>`)
- `settings.json`: Persist `selectedProvider` and `selectedModels` map
- `SettingsPanel.tsx`: Add arrow key listener for tab navigation

</code_context>

<specifics>
## Specific Ideas

- Onboarding stepper needs to update from 4 nodes to 5 nodes with the new Provider step
- All hardcoded "xAI" text in StepApiKey and ModelTab needs to become provider-aware
- The provider dropdown in Account tab should feel native — same glassmorphic styling as the rest of the overlay

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 22-multi-provider-frontend*
*Context gathered: 2026-03-09*
