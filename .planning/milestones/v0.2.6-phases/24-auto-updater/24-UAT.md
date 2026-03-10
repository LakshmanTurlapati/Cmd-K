---
status: complete
phase: 24-auto-updater
source: [24-01-SUMMARY.md, 24-02-SUMMARY.md]
started: 2026-03-09T17:30:00Z
updated: 2026-03-09T17:35:00Z
---

## Current Test
<!-- OVERWRITE each test - shows where we are -->

[testing complete]

## Tests

### 1. Tray Menu Shows Update Item
expected: Open the app. Right-click the system tray icon. The menu includes a "Check for Updates..." item positioned above "Settings".
result: pass

### 2. Manual Update Check via Tray
expected: Click "Check for Updates..." in the tray menu. The menu item text briefly changes to "Checking..." then returns to "Check for Updates..." (or shows "Update Available (vX.Y.Z)" if an update exists). No UI freeze or blocking.
result: pass

### 3. Silent Launch Check (No UI Block)
expected: Launch the app fresh. The app opens and the overlay responds immediately — no visible delay or spinner from update checking. The check happens silently in the background.
result: pass

### 4. Auto-Update Disable Respects Settings
expected: Open Settings, set autoUpdate to false (or add "autoUpdate": false to settings.json). Relaunch the app. The tray menu still shows "Check for Updates..." but no automatic background check fires on launch. Clicking the menu item still works for manual checks.
result: skipped
reason: No UI toggle exists for this yet

### 5. Install-on-Quit Behavior
expected: When an update has been downloaded (pending), quitting the app via the tray "Quit" option triggers the update installer. On next launch, the app runs the new version. (Requires a real GitHub release with a higher version number to fully test.)
result: skipped
reason: Needs a real release to test

### 6. CI Workflow Produces Updater Artifacts
expected: Review .github/workflows/release.yml. The macOS build step includes TAURI_SIGNING_PRIVATE_KEY env vars and copies .app.tar.gz + .sig files. The Windows build includes signing env vars and renames .exe.sig. The release job assembles latest.json with darwin-aarch64, darwin-x86_64, and windows-x86_64 platform entries, then uploads all artifacts to the GitHub Release.
result: pass

### 7. App Compiles Successfully
expected: Run `cargo check -p cmd-k` from the project root. The build completes without errors, confirming the updater plugin, state machine, and tray integration all compile cleanly.
result: pass

## Summary

total: 7
passed: 5
issues: 0
pending: 0
skipped: 2
## Gaps

[none yet]
