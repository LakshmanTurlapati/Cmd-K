---
phase: 27
slug: conpty-discovery-process-snapshot
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-11
---

# Phase 27 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test (#[test] / #[cfg(test)]) |
| **Config file** | None (Cargo-native) |
| **Quick run command** | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Full suite command** | `cargo test --manifest-path src-tauri/Cargo.toml` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --manifest-path src-tauri/Cargo.toml`
- **After every plan wave:** Run `cargo test --manifest-path src-tauri/Cargo.toml` + manual Windows testing
- **Before `/gsd:verify-work`:** Full suite must be green + manual verification on Windows
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 27-01-01 | 01 | 1 | PROC-03 | unit | `cargo test process_snapshot` | ❌ W0 | ⬜ pending |
| 27-01-02 | 01 | 1 | PROC-02 | unit | `cargo test has_batch_flag` | ❌ W0 | ⬜ pending |
| 27-01-03 | 01 | 1 | PROC-01 | manual-only | Manual: VS Code multi-tab shell detection | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Unit tests for `has_batch_flag` / command line flag parsing (PROC-02) — string parsing, cross-platform testable
- [ ] Unit tests for `ProcessSnapshot::capture()` map building (PROC-03) — mock data, cross-platform testable
- [ ] Unit tests for `is_interactive_cmd` logic — mock snapshot with conhost/OpenConsole parents

*Note: ConPTY discovery (PROC-01) and PEB reading require live Windows environment — manual testing only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| ConPTY shell discovery selects correct shell in IDE | PROC-01 | Requires live Windows with VS Code + multiple terminal tabs | Open VS Code with PowerShell, cmd, bash tabs. Activate each tab, press Cmd+K, verify correct shell detected |
| PEB command line reading filters background cmd.exe | PROC-02 | Requires Windows process APIs + running IDE | Open VS Code with cmd.exe tab. Run git operations. Verify CMD+K shows "cmd" not internal git cmd.exe |
| Single snapshot per hotkey (no redundant calls) | PROC-03 | Requires Windows debug logging inspection | Check eprintln output — should see single "[process] ProcessSnapshot captured" per hotkey press |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
