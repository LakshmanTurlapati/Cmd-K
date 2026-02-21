---
phase: 03-terminal-context-reading
plan: 01
subsystem: api
tags: [rust, tauri, libproc, objc-ffi, terminal-detection, macos]

# Dependency graph
requires:
  - phase: 01-foundation-overlay
    provides: AppState struct, hotkey handler, toggle_overlay function
  - phase: 02-settings-configuration
    provides: Tauri IPC command patterns, ObjC FFI extern C pattern from permissions.rs

provides:
  - TerminalContext struct (shell_type, cwd, visible_output, running_process)
  - terminal::detect() orchestrator function
  - terminal/detect.rs: bundle ID detection for 5 terminals via NSRunningApplication ObjC FFI
  - terminal/process.rs: libproc raw FFI for CWD, binary name, child PID enumeration, tmux/screen tree walk
  - get_terminal_context Tauri IPC command
  - AppState.previous_app_pid field capturing frontmost app before overlay shows

affects: [03-terminal-context-reading/03-02, 03-terminal-context-reading/03-03, 04-ai-integration]

# Tech tracking
tech-stack:
  added:
    - accessibility-sys = "0.2" (for Plan 02 AX text reading)
    - regex = "1" (for Plan 02 output filtering)
    - once_cell = "1" (for Plan 02 lazy static patterns)
    - libproc (macOS system library, accessed via raw FFI -- no crate needed)
  patterns:
    - Raw libproc FFI using extern "C" blocks (proc_pidinfo, proc_pidpath, proc_listchildpids)
    - NSRunningApplication ObjC FFI for bundle ID lookup by PID
    - NSWorkspace ObjC FFI for frontmost app PID capture
    - Capture-before-show pattern: PID captured BEFORE toggle_overlay() to avoid focus change

key-files:
  created:
    - src-tauri/src/terminal/mod.rs
    - src-tauri/src/terminal/detect.rs
    - src-tauri/src/terminal/process.rs
    - src-tauri/src/commands/terminal.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/state.rs
    - src-tauri/src/commands/hotkey.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs

key-decisions:
  - "Raw libproc FFI instead of darwin-libproc crate: darwin-libproc 0.2 pins memchr ~2.3 which conflicts with the rest of the dependency tree (memchr ^2.7 required by tauri chain)"
  - "proc_listchildpids for child PID enumeration: stable macOS API, two-pass pattern (NULL for count, then allocate)"
  - "PROC_PIDVNODEPATHINFO flavor for CWD: reads proc_vnodepathinfo.pvi_cdir.pvip.vip_path via proc_pidinfo"
  - "is_gpu_terminal() left unused intentionally: Plan 02 (AX text reading) will use it to select text extraction strategy"
  - "Capture-before-show PID pattern: get_frontmost_pid() called in hotkey handler BEFORE toggle_overlay() to avoid race with NSPanel focus acquisition"

patterns-established:
  - "ObjC FFI pattern: extern C block with objc_getClass/sel_registerName/objc_msgSend for macOS API calls (consistent with permissions.rs AXIsProcessTrusted)"
  - "libproc raw FFI: extern C block in platform-gated cfg(target_os = macos) mod ffi block"
  - "cfg-gated stubs: non-macOS stubs return None/empty for all libproc functions"
  - "Process tree walk: find_shell_pid -> walk_multiplexer_tree(max_depth=3) for tmux/screen"

requirements-completed: [TERM-02, TERM-04]

# Metrics
duration: 4min
completed: 2026-02-21
---

# Phase 3 Plan 01: Terminal Context Reading - Rust Backend Summary

**Rust terminal detection pipeline: libproc raw FFI for CWD + shell type, ObjC NSRunningApplication for bundle ID matching across 5 terminals, tmux/screen tree walk, and get_terminal_context IPC command with frontmost PID capture before NSPanel show**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-21T13:29:16Z
- **Completed:** 2026-02-21T13:33:45Z
- **Tasks:** 2
- **Files modified:** 9

## Accomplishments
- AppState extended with `previous_app_pid` field; hotkey handler captures frontmost app PID via NSWorkspace ObjC FFI before toggle_overlay() is called
- terminal/detect.rs: identifies all 5 terminal emulators (Terminal.app, iTerm2, Alacritty, Kitty, WezTerm) by bundle ID using NSRunningApplication ObjC FFI
- terminal/process.rs: reads CWD via raw libproc proc_pidinfo (PROC_PIDVNODEPATHINFO flavor), binary name via proc_pidpath, child PIDs via proc_listchildpids; walks tmux/screen multiplexer trees up to 3 levels
- get_terminal_context IPC command registered in Tauri invoke_handler; returns None for non-terminal apps, TerminalContext for recognized terminals

## Task Commits

Each task was committed atomically:

1. **Task 1: Add dependencies, extend AppState, capture frontmost PID in hotkey handler** - `52986e2` (feat)
2. **Task 2: Create terminal detection module with process inspection and IPC command** - `8d8a079` (feat)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified
- `src-tauri/src/terminal/mod.rs` - TerminalContext struct + detect() orchestrator
- `src-tauri/src/terminal/detect.rs` - Bundle ID lookup via NSRunningApplication, TERMINAL_BUNDLE_IDS for 5 terminals, is_gpu_terminal() for Plan 02
- `src-tauri/src/terminal/process.rs` - libproc raw FFI: CWD (PROC_PIDVNODEPATHINFO), binary name (proc_pidpath), child PIDs (proc_listchildpids), process tree walk for tmux/screen
- `src-tauri/src/commands/terminal.rs` - get_terminal_context Tauri IPC command
- `src-tauri/src/state.rs` - Added previous_app_pid: Mutex<Option<i32>> field
- `src-tauri/src/commands/hotkey.rs` - Added get_frontmost_pid() via NSWorkspace ObjC FFI; captures PID before toggle_overlay
- `src-tauri/src/commands/mod.rs` - Added pub mod terminal
- `src-tauri/src/lib.rs` - Added mod terminal; registered get_terminal_context in generate_handler
- `src-tauri/Cargo.toml` - Added accessibility-sys, regex, once_cell dependencies

## Decisions Made
- Used raw libproc FFI instead of darwin-libproc crate because darwin-libproc 0.2 pins memchr ~2.3, which conflicts with the transitive memchr ^2.7 requirement from the tauri dependency chain. The raw FFI approach calls the same underlying macOS C functions and is equally stable.
- proc_listchildpids two-pass pattern: first call with NULL/0 to get byte count, then allocate and call again with proper buffer. This is the documented macOS pattern for variable-length proc list APIs.
- PROC_PIDVNODEPATHINFO (flavor 9) for CWD: returns proc_vnodepathinfo struct; the CWD path is extracted from pvi_cdir.pvip.vip_path at byte offset 64 within the struct (after the 64-byte vnode_info header).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Replaced darwin-libproc crate with raw libproc FFI**
- **Found during:** Task 1 (cargo check after adding darwin-libproc to Cargo.toml)
- **Issue:** darwin-libproc 0.2.0 depends on memchr ~2.3 (pinned to 2.3.x). The existing dependency chain (tauri-nspanel -> tauri -> embed-resource) requires memchr ^2.7. Cargo cannot satisfy both constraints simultaneously.
- **Fix:** Removed darwin-libproc from Cargo.toml. Implemented equivalent functionality in terminal/process.rs using raw `extern "C"` FFI calling the macOS libproc C library directly (proc_pidinfo, proc_pidpath, proc_listchildpids). The libproc C library is part of macOS system libraries and requires no Cargo dependency.
- **Files modified:** src-tauri/Cargo.toml (removed darwin-libproc), src-tauri/src/terminal/process.rs (raw FFI implementation)
- **Verification:** cargo check passes with zero errors; all libproc functionality implemented
- **Committed in:** 52986e2 (Task 1 commit, dependency removal), 8d8a079 (Task 2 commit, raw FFI implementation)

---

**Total deviations:** 1 auto-fixed (1 blocking dependency conflict)
**Impact on plan:** The raw libproc FFI approach provides identical functionality to darwin-libproc with no additional crate dependency. No scope creep; plan objectives fully met.

## Issues Encountered
- darwin-libproc 0.2 memchr version conflict resolved by switching to raw FFI (documented above as deviation)
- clippy warns about `b"...\0".as_ptr()` pattern (manual_c_str_literals) in ObjC FFI code -- these are warnings only, not errors; the pattern is consistent with the existing codebase (permissions.rs) and pre-dates Rust 1.77 c"" literal syntax adoption in this project

## Next Phase Readiness
- Rust backend complete: get_terminal_context returns TerminalContext with cwd, shell_type, running_process when frontmost app is a known terminal
- Plan 02 ready: is_gpu_terminal() stub in detect.rs flags Alacritty/Kitty/WezTerm for different AX text handling strategy; visible_output field in TerminalContext awaiting Plan 02 population
- Plan 03 ready: Frontend can call `invoke("get_terminal_context")` on "overlay-shown" event; typed TerminalContext will arrive (or null for non-terminal)

---
*Phase: 03-terminal-context-reading*
*Completed: 2026-02-21*
