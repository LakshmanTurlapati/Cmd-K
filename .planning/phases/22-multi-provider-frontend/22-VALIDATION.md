---
phase: 22
slug: multi-provider-frontend
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 22 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None (no test framework configured — consistent with project pattern) |
| **Config file** | none |
| **Quick run command** | `npm run build` |
| **Full suite command** | `npm run build` (TypeScript compile check) |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `npm run build`
- **After every plan wave:** Run `npm run build` + manual smoke test
- **Before `/gsd:verify-work`:** Full build must be green + manual verification of all 7 requirements
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 22-01-01 | 01 | 1 | PFUI-01 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-01-02 | 01 | 1 | PFUI-02 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-01-03 | 01 | 1 | PFUI-05 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-02-01 | 02 | 1 | PFUI-03 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-02-02 | 02 | 1 | PFUI-04 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-02-03 | 02 | 1 | ORTR-01 | manual + build | `npm run build` | N/A | ⬜ pending |
| 22-02-04 | 02 | 1 | ORTR-02 | manual + build | `npm run build` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. No test framework installation needed — project uses TypeScript compilation as its automated verification layer (consistent across 22 prior phases).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Provider selection in onboarding | PFUI-01 | Visual UI interaction | Launch app with fresh settings.json, verify provider picker appears as step 1, select provider, verify API key step follows |
| Provider switching in settings | PFUI-02 | Visual UI interaction | Open settings Account tab, verify provider dropdown above key input, switch providers, verify key status updates |
| Model dropdown filtered to provider | PFUI-03 | Visual UI interaction | Validate API key, open model tab, verify only selected provider's models appear |
| Tier-grouped model display | PFUI-04 | Visual UI interaction | Check model list shows "Fast", "Balanced", "Most Capable" section headers with correct models under each |
| Provider switch preserves history | PFUI-05 | State persistence check | Send a query, switch provider in settings, verify conversation history is intact |
| OpenRouter single key access | ORTR-01 | Requires valid API key | Enter OpenRouter API key, verify models from multiple providers appear in model list |
| OpenRouter chat-model filtering | ORTR-02 | Visual verification | With OpenRouter selected, verify model list excludes embedding/image models, shows only chat-capable models |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
