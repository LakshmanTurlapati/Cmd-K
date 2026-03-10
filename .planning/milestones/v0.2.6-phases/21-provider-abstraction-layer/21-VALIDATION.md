---
phase: 21
slug: provider-abstraction-layer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 21 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | None — no test infrastructure exists for this phase |
| **Config file** | none — Wave 0 installs |
| **Quick run command** | `cargo build` (compile check) |
| **Full suite command** | `cargo build && cd src-tauri && cargo check && cd ../src && npm run build` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo build`
- **After every plan wave:** Run `cargo build && cd src-tauri && cargo check && cd ../src && npm run build`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 21-01-01 | 01 | 1 | PROV-01, PROV-06, PROV-07 | build | `cargo build` | ✅ | ⬜ pending |
| 21-01-02 | 01 | 1 | PROV-02, PROV-03 | build | `cargo build` | ✅ | ⬜ pending |
| 21-02-01 | 02 | 2 | PROV-04, PROV-05 | build | `cargo build` | ✅ | ⬜ pending |
| 21-02-02 | 02 | 2 | PROV-01, PROV-06 | build | `npm run build` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

*Existing infrastructure covers all phase requirements — compilation-based verification only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Provider selection persists across restart | PROV-01 | Requires app restart cycle | Select provider, quit app, reopen, verify selection preserved |
| Per-provider keychain storage | PROV-02 | Platform keychain access | Save key for each of 5 providers, retrieve each, verify correct |
| v0.2.4 xAI key migration | PROV-03 | Requires v0.2.4 state | Install v0.2.4, add xAI key, upgrade to v0.2.6, verify key preserved + xAI default |
| API key validation per provider | PROV-04 | External API calls | Enter valid/invalid keys for each provider, verify success/error messages |
| Available models per provider | PROV-05 | External API calls | Select each provider, verify model list appears |
| Real-time streaming all providers | PROV-06 | External API calls + visual | Generate command with each of 5 providers, verify tokens stream in real-time |
| Provider-specific error messages | PROV-07 | Error conditions | Trigger errors (invalid key, rate limit), verify provider name + actionable hint in message |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
