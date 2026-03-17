---
phase: 39
slug: streaming-integration
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 39 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) + tsc (TypeScript) |
| **Config file** | Cargo.toml, tsconfig.json |
| **Quick run command** | `cargo test --lib && npx tsc --noEmit` |
| **Full suite command** | `cargo test --lib && npx tsc --noEmit` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run full suite
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | LSTR-01 | manual | `cargo check --lib` | ✅ | ⬜ pending |
| TBD | 01 | 1 | LSTR-02 | unit | `cargo test --lib` | ✅ | ⬜ pending |
| TBD | 01 | 1 | LSTR-03 | manual | `npx tsc --noEmit` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Streamed response from local provider | LSTR-01 | Requires running local server | Select Ollama, type query, verify streamed output |
| Cold-start completes within 120s | LSTR-02 | Requires cold model load | Restart Ollama, immediately query, verify completion |
| Token counts in usage display | LSTR-03 | Requires actual query + settings check | Query local provider, open Settings, verify token counts |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
