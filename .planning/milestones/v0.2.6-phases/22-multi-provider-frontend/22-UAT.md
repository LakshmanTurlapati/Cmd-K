---
status: complete
phase: 22-multi-provider-frontend
source: [22-01-SUMMARY.md, 22-02-SUMMARY.md]
started: 2026-03-09T09:10:00Z
updated: 2026-03-09T09:20:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Provider Selection During Onboarding
expected: On first launch (or after clearing settings), onboarding wizard starts at step 0: Provider Selection. Five providers are shown (OpenAI, Anthropic, Google, xAI, OpenRouter) with styled initials. No provider is pre-selected. Clicking one highlights it, then Next button becomes enabled.
result: pass

### 2. Provider-Aware API Key Step
expected: After selecting a provider and pressing Next, the API Key step shows provider-specific placeholder text (not hardcoded "xAI"). Entering/pasting a key validates against the correct provider.
result: pass

### 3. Onboarding Summary Shows Provider
expected: On the final Done step, the configuration summary includes a "Provider" row showing the name of the provider you selected.
result: pass

### 4. Provider Dropdown in Settings
expected: In Settings > Account tab, a provider dropdown appears above the API key input. Clicking it shows all 5 providers. Providers with a saved API key show a green checkmark. Selecting a different provider switches the active provider.
result: issue
reported: "provider picking could also be a drop down in onboarding as well"
severity: cosmetic

### 5. Tier-Grouped Model Lists
expected: In Settings > Model tab (and during onboarding model step), models are grouped under tier headers: Fast, Balanced, Most Capable. An "All Models" section shows every model. Tier headers only appear when models exist for that tier.
result: pass

### 6. Per-Provider Model Memory
expected: Select a model for Provider A. Switch to Provider B and select a different model. Switch back to Provider A — your previous model selection is remembered and restored.
result: pass

### 7. Arrow Key Tab Navigation in Settings
expected: When Settings panel is open, pressing Left/Right arrow keys navigates between settings tabs (Account, Model, etc.).
result: pass

### 8. Model List Scrollable (Max Height)
expected: For providers with many models (e.g. OpenRouter), the model list has a fixed max height with a scrollbar. The scrollbar matches the overlay's dark aesthetic (transparent track, subtle thumb) — no bright white scrollbar background.
result: pass

## Summary

total: 8
passed: 7
issues: 1
pending: 0
skipped: 0

## Gaps

- truth: "Provider selection in onboarding uses same dropdown pattern as settings"
  status: failed
  reason: "User reported: provider picking could also be a drop down in onboarding as well"
  severity: cosmetic
  test: 4
  artifacts: []
  missing: []
