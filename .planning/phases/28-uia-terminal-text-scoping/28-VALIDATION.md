---
phase: 28
slug: uia-terminal-text-scoping
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 28 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (#[test] + #[cfg(test)]) |
| **Config file** | None (Cargo.toml test section) |
| **Quick run command** | `cargo test --lib -p cmd-k -- --nocapture` |
| **Full suite command** | `cargo test --lib -p cmd-k` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib -p cmd-k -- --nocapture`
- **After every plan wave:** Run `cargo test --lib -p cmd-k`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 28-01-01 | 01 | 1 | UIAS-01 | unit | `cargo test --lib -p cmd-k -- uia_reader --nocapture` | ❌ W0 | ⬜ pending |
| 28-01-02 | 01 | 1 | UIAS-01 | manual-only | Manual UAT with VS Code | N/A | ⬜ pending |
| 28-02-01 | 02 | 1 | UIAS-02 | unit | `cargo test --lib -p cmd-k -- detect_wsl --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src-tauri/src/terminal/mod.rs` tests — add `detect_wsl_from_text` unit tests (currently NO tests for this function)
- [ ] `src-tauri/src/terminal/uia_reader.rs` — add unit tests for `is_window_chrome` and fallback logic

*Existing infrastructure covers framework needs — no new framework install required.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Scoped UIA walk returns only terminal text | UIAS-01 | UIA requires live VS Code window with terminal | Open VS Code, open editor + terminal, invoke CMD+K, verify captured text has no editor content |
| Inactive tab text excluded | UIAS-01 | UIA tree structure varies at runtime | Open VS Code with 2 terminals, verify only active tab text captured |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
