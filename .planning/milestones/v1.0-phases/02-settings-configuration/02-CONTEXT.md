# Phase 2: Settings & Configuration - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

User can configure xAI API credentials and model preferences securely. Includes first-run onboarding wizard (Accessibility permissions + API key setup), settings UI accessible via menu bar and /settings command, API key storage in macOS Keychain with validation, and Grok model selection. Creating commands, terminal context, and safety features are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Onboarding flow
- Step-by-step wizard, not single page
- Step order: Accessibility permission -> API key entry -> Model selection -> Done
- Onboarding appears inside the overlay itself (no separate window)
- If user closes mid-setup, resume where they left off next time (track progress)

### Settings UI access
- Two entry points: menu bar tray icon click AND typing /settings in the overlay
- Both routes open the overlay in "settings mode" (reuses the overlay UI)
- Settings organized with tabbed sections (e.g., Account, Model, Preferences)
- Changes auto-save immediately -- no save button needed

### API key experience
- Masked input field by default (dots/asterisks) with eye icon toggle to reveal
- Validate immediately on paste/entry (no separate validate button)
- Inline status indicator: green checkmark for valid, red X for invalid
- No helper text or "Get API key" links -- keep it clean
- Single API key only (no multi-key support in v1)
- When stored in Keychain, show last 4 characters by default with reveal toggle for full key
- If key becomes invalid later, show error inline in the overlay with link to settings (don't auto-open settings)

### Model selection
- Dropdown list for model selection
- Fetch models dynamically from xAI API (requires valid key first)
- Smart default: pre-select the best general-purpose Grok model
- Dropdown shows model name + short tag (e.g., "Fast", "Most capable")
- Model dropdown disabled/greyed out until API key is validated
- User's model choice persists across app restarts
- Mini usage dashboard in settings panel showing estimated cost

### Claude's Discretion
- Exact onboarding step animations/transitions
- Tab naming and icon choices for settings sections
- Loading states during API key validation
- How to calculate/display estimated cost in usage dashboard
- Error state styling and copy

</decisions>

<specifics>
## Specific Ideas

- Onboarding lives inside the overlay itself -- the same UI the user will use for commands, just in "setup mode"
- Settings reuses the overlay too -- menu bar click and /settings both open overlay in settings mode
- Usage dashboard should show cost only (no request counts) -- keep it minimal

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 02-settings-configuration*
*Context gathered: 2026-02-21*
