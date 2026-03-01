---
phase: 03-terminal-context-reading
plan: 03
subsystem: ui
tags: [zustand, react, tauri, rust, accessibility, terminal]

# Dependency graph
requires:
  - phase: 03-02-terminal-context-reading
    provides: AX text reader, sensitive data filter, 500ms timeout wrapper, full Rust detection pipeline
  - phase: 03-01-terminal-context-reading
    provides: get_terminal_context IPC command, terminal detection module, AppState.previous_app_pid
provides:
  - Zustand store extended with terminalContext, isDetectingContext, accessibilityGranted state
  - Non-blocking async context detection fired on every overlay show
  - Shell type label (zsh/bash/fish) displayed below overlay input in command mode
  - Accessibility permission banner (amber, persistent, clickable) shown when permission denied
  - Subtle spinner during detection, replaced by shell label or nothing after
  - Rust backend enhanced with lsof/pgrep fallbacks for robust shell detection across all process topologies
affects:
  - 04-ai-integration (reads terminalContext from Zustand store in CommandInput)
  - 03-04 (app badges), 03-05 (browser console)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Fire-and-forget async IIFE in Zustand show() action for non-blocking side effects
    - Selector-per-field pattern for Zustand subscriptions (each UI element subscribes to one field)
    - lsof fallback for proc_pidinfo CWD when root-owned login process blocks direct access
    - pgrep ancestry search for shell detection in deep process trees (VS Code, Cursor)

key-files:
  created: []
  modified:
    - src/store/index.ts
    - src/components/Overlay.tsx
    - src-tauri/src/terminal/mod.rs
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/terminal/detect.rs
    - src-tauri/src/commands/terminal.rs
    - src-tauri/src/commands/hotkey.rs

key-decisions:
  - "Fire-and-forget async IIFE in show() keeps overlay appearance instant while detection happens in background"
  - "terminalContext reset to null on each show() so stale context never leaks between overlay opens"
  - "Accessibility banner is non-dismissable and re-checked on each overlay open (not cached)"
  - "lsof fallback for CWD: proc_pidinfo fails for processes spawned under root-owned login wrappers"
  - "pgrep ancestry search: enables shell detection in deep process trees beyond 3-level recursive walk"
  - "Shell label shows actual process name (zsh/bash/fish) from libproc, not $SHELL env variable"

patterns-established:
  - "Non-blocking detection: show() sets isDetectingContext=true synchronously, async fills in afterward"
  - "Conditional min-h-[20px] div: collapses when no context, preserves height during loading"
  - "eprintln! debug logging in all Rust detection stages for diagnosability in tauri dev"

requirements-completed: [TERM-02, TERM-03, TERM-04]

# Metrics
duration: 15min
completed: 2026-02-22
---

# Phase 3 Plan 03: Frontend Overlay Integration Summary

**Zustand store wired to Rust terminal detection with non-blocking async detection, shell type label, accessibility banner, and robust pgrep/lsof fallbacks for all process topologies**

## Performance

- **Duration:** ~15 min
- **Started:** 2026-02-22T23:32:09Z
- **Completed:** 2026-02-22T23:47:00Z
- **Tasks:** 1 of 2 automated (Task 2 is human verification checkpoint)
- **Files modified:** 7

## Accomplishments

- Zustand store extended with `terminalContext`, `isDetectingContext`, `accessibilityGranted` fields and actions
- `show()` action fires non-blocking async detection: checks accessibility permission + invokes `get_terminal_context` each open
- Overlay.tsx displays: (1) persistent amber banner when accessibility denied, (2) spinner during detection, (3) shell type label when detected, (4) nothing for non-terminal apps
- Rust backend enhanced: lsof fallback for CWD, pgrep fallback for child PIDs, ancestry search for deep process trees
- All TypeScript and Rust compile with zero errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend Zustand store with terminal context, wire detection on overlay show, update Overlay UI**
   - `baeb646` (feat) - Initial store + Overlay.tsx integration
   - `620abb0` (fix) - ARM64 objc_msgSend calling convention fix
   - `9e7b029` (fix) - Walk through login wrapper to find actual shell
   - `ba710aa` (fix) - lsof/pgrep fallbacks for shell detection and debug logging

2. **Task 2: Verify complete Phase 3 terminal context reading experience** - CHECKPOINT (human-verify)

## Files Created/Modified

- `src/store/index.ts` - Added TerminalContext interface, terminalContext/isDetectingContext/accessibilityGranted state fields, async detection in show() action
- `src/components/Overlay.tsx` - Added accessibility banner, spinner, shell type label in command mode
- `src-tauri/src/terminal/mod.rs` - Loosened terminal gate to use process tree shell detection; added lsof/pgrep for non-terminal app support
- `src-tauri/src/terminal/process.rs` - Added lsof CWD fallback, pgrep child PID fallback, ancestry search for deep trees
- `src-tauri/src/terminal/detect.rs` - Added eprintln debug logging to bundle ID lookup
- `src-tauri/src/commands/terminal.rs` - Added verbose eprintln logging for PID capture diagnostics
- `src-tauri/src/commands/hotkey.rs` - Added eprintln logging for PID capture timing

## Decisions Made

- Fire-and-forget async IIFE in `show()` keeps overlay appearance instant while detection runs in background
- `terminalContext` reset to null on each `show()` so stale context never leaks between opens
- Accessibility banner is non-dismissable and re-checked on every overlay open (not cached between opens)
- lsof used as CWD fallback because proc_pidinfo fails for processes spawned under root-owned `login` wrappers (Terminal.app pattern)
- pgrep ancestry search enables shell detection in deep process trees beyond the 3-level recursive walk (VS Code, Cursor, apps with integrated terminals)
- Shell label shows actual process binary name from libproc, not `$SHELL` env variable (avoids stale env pitfall)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ARM64 objc_msgSend calling convention for i32 argument**
- **Found during:** Task 1 (testing with Terminal.app)
- **Issue:** objc_msgSend on ARM64 requires the pid_t (i32) argument in a specific register; incorrect transmute caused nil returns for bundle ID lookup
- **Fix:** Corrected MsgSendI32 function pointer type for ARM64 ABI compatibility
- **Files modified:** src-tauri/src/terminal/detect.rs
- **Committed in:** 620abb0

**2. [Rule 1 - Bug] Shell detection failed when Terminal.app uses login wrapper**
- **Found during:** Task 1 (zsh not detected via recursive walk)
- **Issue:** Terminal.app spawns `login` as child of terminal PID; `login` then spawns `zsh`. The recursive walk (3 levels) was not walking through login to find the shell
- **Fix:** Updated find_shell_recursive to walk through known wrapper processes (login, sshd, su, sudo)
- **Files modified:** src-tauri/src/terminal/process.rs
- **Committed in:** 9e7b029

**3. [Rule 2 - Missing Critical] Added lsof fallback for CWD when proc_pidinfo fails**
- **Found during:** Task 1 (CWD returning None for login-wrapped shells)
- **Issue:** proc_pidinfo PROC_PIDVNODEPATHINFO fails for processes whose parent is a root-owned process (login), returning -1
- **Fix:** Added get_process_cwd_lsof() as fallback; called after proc_pidinfo failure
- **Files modified:** src-tauri/src/terminal/process.rs
- **Committed in:** ba710aa

**4. [Rule 2 - Missing Critical] Added pgrep child PID fallback and ancestry search**
- **Found during:** Task 1 (child PID enumeration failing for some process trees)
- **Issue:** proc_listchildpids returns empty for root-owned processes; get_child_pids returned nothing causing false negatives
- **Fix:** Added get_child_pids_sysctl() via pgrep -P and find_shell_by_ancestry() for deep process trees
- **Files modified:** src-tauri/src/terminal/process.rs, src-tauri/src/terminal/mod.rs
- **Committed in:** ba710aa

---

**Total deviations:** 4 auto-fixed (2 bugs, 2 missing critical)
**Impact on plan:** All auto-fixes required for shell detection to work reliably across macOS Terminal.app's login-wrapper process topology. No scope creep.

## Issues Encountered

- Terminal.app on macOS wraps shells in a `login` process: Terminal.app PID -> login -> zsh. The 3-level recursive walk missed this until wrappers were added to the passthrough list.
- proc_pidinfo for CWD fails when the shell's ancestor chain includes a root-owned process (login); lsof -Fn fallback reliably returns the CWD in this case.
- ARM64 ABI requires careful function pointer typing for Objective-C message sends with i32 arguments.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Terminal context detection pipeline is complete end-to-end
- Shell type label, accessibility banner, and spinner all wired in Overlay.tsx
- CWD, visible output, and running process stored in Zustand for Phase 4 AI use
- Phase 4 AI integration can read `terminalContext` from `useOverlayStore` in CommandInput.tsx
- Human verification of the complete Phase 3 experience still pending (Task 2 checkpoint)

---
*Phase: 03-terminal-context-reading*
*Completed: 2026-02-22*
