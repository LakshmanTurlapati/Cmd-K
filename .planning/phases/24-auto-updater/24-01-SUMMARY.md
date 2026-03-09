---
phase: 24-auto-updater
plan: 01
subsystem: infra
tags: [tauri, updater, auto-update, tray-menu, ed25519, tokio]

# Dependency graph
requires: []
provides:
  - "Update state machine (UpdateStatus enum + UpdateState managed state)"
  - "Background update checker (launch + 24h interval via tokio)"
  - "Tray menu update item with dynamic text transitions"
  - "Install-on-quit handler for seamless update application"
  - "Updater plugin configuration with GitHub Releases endpoint"
affects: [24-02-ci-updater-pipeline]

# Tech tracking
tech-stack:
  added: [tauri-plugin-updater]
  patterns: [separate-managed-state, background-tokio-spawn, tray-menu-state-machine]

key-files:
  created: [src-tauri/src/commands/updater.rs]
  modified: [src-tauri/Cargo.toml, src-tauri/tauri.conf.json, src-tauri/capabilities/default.json, src-tauri/src/state.rs, src-tauri/src/commands/mod.rs, src-tauri/src/commands/tray.rs, src-tauri/src/lib.rs]

key-decisions:
  - "UpdateState as separate managed state (not inside AppState) because tauri_plugin_updater::Update is not Default"
  - "MenuItem stored in UpdateState for dynamic tray text updates without global lookups"
  - "Pubkey placeholder in tauri.conf.json -- user generates real Ed25519 keypair before release"

patterns-established:
  - "Separate managed state: non-Default types get their own .manage() call instead of embedding in AppState"
  - "Background lifecycle tasks: spawn in setup() after all state is managed, using tauri::async_runtime::spawn"

requirements-completed: [UPDT-01, UPDT-02, UPDT-03, UPDT-04, UPDT-05, UPDT-07, UPDT-08]

# Metrics
duration: 4min
completed: 2026-03-09
---

# Phase 24 Plan 01: Auto-Updater Backend Summary

**Updater plugin with background check loop, tray menu state machine, auto-download, and install-on-quit via tauri-plugin-updater**

## Performance

- **Duration:** 4 min
- **Started:** 2026-03-09T17:20:58Z
- **Completed:** 2026-03-09T17:25:15Z
- **Tasks:** 2
- **Files modified:** 8

## Accomplishments
- Full auto-updater lifecycle: check on launch, 24h re-check, auto-download, install on quit
- Tray menu "Check for Updates..." item with state-driven text transitions (Idle/Checking/Available/Downloading/Ready/Disabled)
- Install-on-quit handler seamlessly applies pending updates when user quits via tray
- Auto-update disable toggle reads from settings store (autoUpdate key, defaults to enabled)

## Task Commits

Each task was committed atomically:

1. **Task 1: Configure updater plugin and create update state + background checker** - `9c5965f` (feat)
2. **Task 2: Integrate tray menu update items and install-on-quit handler** - `7150400` (feat)

## Files Created/Modified
- `src-tauri/src/commands/updater.rs` - Update state machine, background checker, download/install logic (~160 lines)
- `src-tauri/src/state.rs` - UpdateStatus enum and UpdateState struct with pending_update, pending_bytes, menu_item
- `src-tauri/src/commands/tray.rs` - Added "Check for Updates..." menu item, check_for_updates event, install-on-quit in quit handler
- `src-tauri/src/lib.rs` - Registered updater plugin, managed UpdateState, spawns background checker in setup()
- `src-tauri/Cargo.toml` - Added tauri-plugin-updater = "2" dependency
- `src-tauri/tauri.conf.json` - Added updater config block with GitHub endpoint, pubkey placeholder, createUpdaterArtifacts
- `src-tauri/capabilities/default.json` - Added updater:default permission
- `src-tauri/src/commands/mod.rs` - Added pub mod updater

## Decisions Made
- UpdateState as separate managed state (not inside AppState) -- tauri_plugin_updater::Update does not implement Default
- MenuItem reference stored in UpdateState's menu_item field for efficient tray text updates without tray/menu lookups
- Pubkey set to placeholder string -- user must generate Ed25519 keypair and replace before first updater-enabled release
- Silent failure on all update errors (network, download, check) -- retries on next 24h cycle or manual click

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow lifetime error in setup_tray**
- **Found during:** Task 2
- **Issue:** Block-scoped `app.state::<UpdateState>()` temporary dropped before mutex guard, causing E0597
- **Fix:** Moved state binding outside the block to extend its lifetime
- **Files modified:** src-tauri/src/commands/tray.rs
- **Verification:** cargo check passes
- **Committed in:** 7150400 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor borrow lifetime fix, no scope change.

## Issues Encountered
None beyond the auto-fixed borrow issue above.

## User Setup Required

Before the first updater-enabled release (v0.2.6+):
1. Generate Ed25519 keypair: `pnpm tauri signer generate -w ~/.tauri/cmd-k-updater.key`
2. Replace `UPDATER_PUBKEY_PLACEHOLDER` in `src-tauri/tauri.conf.json` with the contents of the .pub file
3. Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as GitHub Actions secrets
4. Back up the private key in a secure location (loss is irrecoverable)

## Next Phase Readiness
- Backend updater lifecycle complete -- ready for CI/CD pipeline integration (Plan 02)
- Frontend settings toggle for auto-update disable can be wired to `autoUpdate` key in settings.json
- Ed25519 keypair generation is a pre-release blocker (documented in STATE.md)

---
*Phase: 24-auto-updater*
*Completed: 2026-03-09*
