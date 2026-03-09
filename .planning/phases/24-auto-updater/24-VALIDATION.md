---
phase: 24
slug: auto-updater
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-09
---

# Phase 24 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual testing (no automated test framework detected) |
| **Config file** | none |
| **Quick run command** | `cargo check -p cmd-k` |
| **Full suite command** | Manual smoke test with test GitHub release |
| **Estimated runtime** | ~30 seconds (cargo check); manual tests ~10 min |

---

## Sampling Rate

- **After every task commit:** Run `cargo check -p cmd-k`
- **After every plan wave:** Manual smoke test with a local/test GitHub release
- **Before `/gsd:verify-work`:** Full manual test on both macOS and Windows with a real release
- **Max feedback latency:** 30 seconds (compile check)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 24-01-01 | 01 | 1 | UPDT-05 | unit | Verify tauri.conf.json has pubkey and endpoints | N/A | ⬜ pending |
| 24-01-02 | 01 | 1 | UPDT-01 | manual | Launch app, verify no UI blocking | N/A | ⬜ pending |
| 24-01-03 | 01 | 1 | UPDT-02 | manual | Publish test release, verify tray menu text changes | N/A | ⬜ pending |
| 24-01-04 | 01 | 1 | UPDT-03 | manual | Click tray menu item, verify download starts | N/A | ⬜ pending |
| 24-01-05 | 01 | 1 | UPDT-04 | manual | Quit app, relaunch, verify new version | N/A | ⬜ pending |
| 24-01-06 | 01 | 1 | UPDT-07 | manual | Set interval to 10s for dev, verify re-check fires | N/A | ⬜ pending |
| 24-01-07 | 01 | 1 | UPDT-08 | manual | Per CONTEXT: no dismiss, menu stays visible | N/A | ⬜ pending |
| 24-02-01 | 02 | 2 | UPDT-06 | integration | Run release workflow on test tag, verify artifacts | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Ed25519 keypair generated and `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` added as GitHub secrets
- [ ] A test/pre-release tag created to validate CI pipeline before shipping v0.2.6

*These are prerequisites, not code stubs — existing infrastructure otherwise covers phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Non-blocking update check on launch | UPDT-01 | Requires observing app startup UX | Launch app with no network; verify no delay or freeze |
| Tray menu shows update indicator | UPDT-02 | Requires real GitHub release with higher version | Publish test release, verify tray menu text changes to "Update Available (vX.Y.Z)" |
| One-click download and install | UPDT-03 | Requires real update artifact download | Click tray menu item, verify download progress and completion |
| Update applied on next launch | UPDT-04 | Requires full app lifecycle | Install update, quit, relaunch, verify version number changed |
| 24h background re-check | UPDT-07 | Timing-dependent behavior | Set interval to 10s for dev testing, verify periodic re-check fires |
| Dismiss suppresses until next launch | UPDT-08 | Per CONTEXT: no dismiss behavior; menu stays visible always | Verify menu item remains after viewing |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
