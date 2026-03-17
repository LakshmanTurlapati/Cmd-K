---
phase: 37
slug: provider-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-17
---

# Phase 37 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test --lib` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test --lib`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | 01 | 1 | LPROV-01 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LPROV-02 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LPROV-03 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LPROV-04 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LPROV-05 | integration | `cargo test --lib` | ❌ W0 | ⬜ pending |
| TBD | 01 | 1 | LPROV-06 | unit | `cargo test --lib` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements. No new test framework setup needed.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Health check connects to running Ollama server | LPROV-05 | Requires running local server | Start Ollama, select provider in settings, verify checkmark |
| Health check connects to running LM Studio server | LPROV-05 | Requires running local server | Start LM Studio, select provider in settings, verify checkmark |
| Base URL persists across app restart | LPROV-04 | Requires full app lifecycle | Set custom URL, quit app, relaunch, verify URL restored |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
