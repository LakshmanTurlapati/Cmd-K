---
phase: 38
slug: model-discovery
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 38 — Validation Strategy

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
| TBD | 01 | 1 | LMOD-01 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LMOD-02 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LMOD-03 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Ollama model picker shows installed models | LMOD-01 | Requires running Ollama server | Start Ollama with models, open Settings Model tab, verify list populates |
| LM Studio model picker shows loaded models | LMOD-02 | Requires running LM Studio | Start LM Studio with loaded model, open Settings Model tab, verify list |
| Auto-tiering by param size | LMOD-03 | Requires visual inspection of tier grouping | Verify models appear under correct tier headers |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
