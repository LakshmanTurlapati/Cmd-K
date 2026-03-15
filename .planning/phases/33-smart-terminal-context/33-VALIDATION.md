---
phase: 33
slug: smart-terminal-context
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-14
---

# Phase 33 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `#[cfg(test)] mod tests` |
| **Config file** | None (standard cargo test) |
| **Quick run command** | `cargo test --lib -p cmd-k -- terminal::context` |
| **Full suite command** | `cargo test --lib -p cmd-k` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -p cmd-k -- terminal::context`
- **After every plan wave:** Run `cargo test --lib -p cmd-k`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 33-01-01 | 01 | 1 | SCTX-01 | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_strip` | ❌ W0 | ⬜ pending |
| 33-01-02 | 01 | 1 | SCTX-02 | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_budget` | ❌ W0 | ⬜ pending |
| 33-01-03 | 01 | 1 | SCTX-03 | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_segment` | ❌ W0 | ⬜ pending |
| 33-01-04 | 01 | 1 | SCTX-04 | manual | Code review: `grep cfg(target_os) context.rs` | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/terminal/context.rs` — new file with inline `#[cfg(test)] mod tests` containing stubs for SCTX-01, SCTX-02, SCTX-03
- No framework install needed — Rust test framework is built in

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| No `cfg(target_os)` in context.rs | SCTX-04 | Static code property, not runtime behavior | `grep -c 'cfg(target_os)' src-tauri/src/terminal/context.rs` should return 0 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
