---
phase: 02-settings-configuration
plan: 03
subsystem: ui
tags: [react, onboarding, wizard, tauri, accessibility, api-key, model-selection, stepper]

# Dependency graph
requires:
  - phase: 02-settings-configuration/02-01
    provides: check_accessibility_permission, open_accessibility_settings, validate_and_fetch_models, save_api_key, get_api_key Tauri IPC commands
  - phase: 02-settings-configuration/02-02
    provides: Zustand store with OverlayMode, apiKeyStatus, availableModels, selectedModel, openOnboarding action; Settings components for pattern reuse

provides:
  - 4-step onboarding wizard (Accessibility, API Key, Model Select, Done) in overlay
  - Startup onboarding check with progress persistence across restarts
  - macOS-style stepper progress bar with glassmorphism active node
  - Slash command autocomplete with ghost suggestion and Tab completion
  - grok-code-fast-1 as recommended default model
  - List-style model picker with greyed labels

affects:
  - 03-xx (AI command integration will use selectedModel and apiKeyStatus from store)
  - 04-xx (future slash commands extend COMMANDS array in CommandInput)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Onboarding persistence: step progress stored in settings.json via tauri-plugin-store, resumed on restart
    - NSPanel Space key workaround: clickable Space button in HotkeyRecorder since macOS filters Space keydown in nonactivating panels
    - Ghost autocomplete: invisible suggestion layer behind textarea with Tab to accept
    - show/hide mode preservation: Zustand show/hide check onboardingComplete before resetting mode

key-files:
  created:
    - src/components/Onboarding/OnboardingWizard.tsx
    - src/components/Onboarding/StepAccessibility.tsx
    - src/components/Onboarding/StepApiKey.tsx
    - src/components/Onboarding/StepModelSelect.tsx
    - src/components/Onboarding/StepDone.tsx
  modified:
    - src/components/Overlay.tsx
    - src/App.tsx
    - src/store/index.ts
    - src/components/CommandInput.tsx
    - src/components/HotkeyRecorder.tsx
    - src/components/HotkeyConfig.tsx
    - src/components/Settings/AccountTab.tsx
    - src/components/Settings/ModelTab.tsx
    - src-tauri/src/commands/xai.rs
    - src-tauri/tauri.conf.json

key-decisions:
  - "macOS-style stepper progress bar instead of dot indicators: track bar with glassmorphism active node and checkmark completed nodes"
  - "grok-code-fast-1 as Recommended default instead of grok-3 Balanced: faster and more appropriate for command generation"
  - "List-style model picker replacing native select: enables mixed styling with greyed-out labels"
  - "Space key clickable button workaround: macOS NSPanel filters Space keydown events at OS level"
  - "Ghost autocomplete for slash commands: type / to see suggestion, Tab to accept"
  - "show/hide preserve onboarding mode: prevents Cmd+K hotkey from resetting to command mode during onboarding"
  - "invoke show_overlay after openOnboarding: native NSPanel must be shown explicitly, React state alone insufficient"
  - "Removed redundant valid key text: green check icon is sufficient feedback"

patterns-established:
  - "Pattern: Onboarding persistence via tauri-plugin-store -- step and completion state survive app restarts"
  - "Pattern: Ghost autocomplete -- invisible text layer behind input with Tab completion"
  - "Pattern: NSPanel keyboard workarounds -- clickable UI fallbacks for keys filtered by macOS"

requirements-completed: [SETT-04]

# Metrics
duration: 45min
completed: 2026-02-21
---

# Phase 2 Plan 3: Onboarding Wizard with Stepper, Autocomplete, and Verification Fixes Summary

**4-step onboarding wizard with macOS-style stepper, slash command autocomplete, grok-code-fast default model, and critical fixes for onboarding visibility, API key status, and tray icon bundling**

## Performance

- **Duration:** 45 min (including human verification and refinements)
- **Tasks:** 2 (1 auto + 1 human-verify checkpoint)
- **Files modified:** 15

## Accomplishments

- Onboarding wizard with 4 steps (Accessibility, API Key, Model Select, Done) rendered inside overlay NSPanel
- macOS-style stepper progress bar with track fill, glassmorphism active node, checkmark completed nodes
- First-launch startup check: opens onboarding if not complete, resumes at persisted step
- Fixed onboarding not appearing: show/hide preserve mode, invoke show_overlay for native window
- Fixed "API not configured" always showing: submit() now checks apiKeyStatus from store
- Fixed tray icon missing in production builds: K.png added to bundle resources
- grok-code-fast-1 set as Recommended default model
- List-style model picker with grey labels replacing native select dropdown
- Space key clickable button in HotkeyRecorder (macOS NSPanel filters Space keydown)
- Custom hotkey display with readable labels (Super+KeyJ -> Cmd + J)
- Slash command autocomplete: ghost /settings suggestion on /, Tab to accept
- Removed redundant "Key ending in XXXX is valid" text

## Task Commits

1. **Task 1: Build onboarding wizard and wire into Overlay + App startup** - `cc47d00` (feat)
2. **Checkpoint: Human verification** - approved with refinements:
   - `c69bf01` (fix) - onboarding flow, API key status, tray icon bundling
   - `de91db2` (feat) - macOS-style stepper progress bar
   - `ff86969` (feat) - grok-code-fast recommended default, list-style model picker
   - `e0635ff` (fix) - Space key capture via clickable button
   - `6b359a8` (feat) - slash command autocomplete

## Files Created/Modified

- `src/components/Onboarding/OnboardingWizard.tsx` - Step orchestrator with macOS stepper, progress persistence
- `src/components/Onboarding/StepAccessibility.tsx` - Permission check with System Settings launcher
- `src/components/Onboarding/StepApiKey.tsx` - Masked input, debounced validation, auto-save
- `src/components/Onboarding/StepModelSelect.tsx` - List-style model picker with recommended default
- `src/components/Onboarding/StepDone.tsx` - Summary and completion transition
- `src/components/Overlay.tsx` - OnboardingWizard replaces placeholder, onboarding width
- `src/App.tsx` - Startup onboarding check, invoke show_overlay for native window
- `src/store/index.ts` - show/hide preserve onboarding mode, submit checks apiKeyStatus
- `src/components/CommandInput.tsx` - Ghost autocomplete with Tab completion
- `src/components/HotkeyRecorder.tsx` - e.code detection, clickable Space button
- `src/components/HotkeyConfig.tsx` - tauriToDisplay helper, custom hotkey display
- `src/components/Settings/AccountTab.tsx` - Removed "Key ending in XXXX" text
- `src/components/Settings/ModelTab.tsx` - List-style picker, xAI Model label
- `src-tauri/src/commands/xai.rs` - grok-code-fast Recommended label
- `src-tauri/tauri.conf.json` - K.png in bundle resources

## Decisions Made

- macOS-style stepper inspired by user-provided HTML snippet, adapted to 4 steps with Tailwind
- grok-code-fast-1 as default per user request (better fit for command generation use case)
- List-style model picker to support mixed text styling (model name white, label grey)
- Space key button workaround after confirming NSPanel limitation is OS-level (not fixable in JS)
- Ghost autocomplete pattern chosen over dropdown suggestions (cleaner, terminal-style)

## Deviations from Plan

### Refinements During Verification

Multiple refinements made based on user testing feedback:
1. Removed redundant valid key confirmation text
2. Replaced dot progress indicator with macOS stepper
3. Changed default model from grok-3 to grok-code-fast-1
4. Replaced native select with styled list picker
5. Fixed three bugs found during testing (onboarding visibility, API status, tray icon)
6. Added Space key workaround for NSPanel limitation
7. Added slash command autocomplete

**Impact on plan:** All refinements improve UX quality. Core functionality unchanged.

## Issues Encountered

- macOS NSPanel with nonactivating_panel style mask filters Space key events at OS level -- resolved with clickable Space button
- show/hide Zustand actions reset mode to "command" which broke onboarding -- resolved with conditional mode preservation
- openOnboarding only set React state but didn't show the native NSPanel window -- resolved by adding invoke("show_overlay")

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Complete settings and onboarding system verified by human testing
- API key securely stored in macOS Keychain, validated against xAI API
- Model selection persisted, ready for AI command integration
- Slash command autocomplete extensible for future commands
- All SETT-01 through SETT-04 requirements satisfied

---
*Phase: 02-settings-configuration*
*Completed: 2026-02-21*
