---
phase: 03-terminal-context-reading
plan: 02
subsystem: api
tags: [rust, tauri, accessibility-api, macos, terminal-detection, security, filtering]

# Dependency graph
requires:
  - phase: 03-terminal-context-reading
    plan: 01
    provides: terminal/mod.rs TerminalContext struct, detect() function, detect.rs is_gpu_terminal(), process.rs ProcessInfo

provides:
  - ax_reader::read_terminal_text(): AX tree walker for Terminal.app (window->scroll->textarea) and iTerm2 (focused element)
  - filter::filter_sensitive(): regex-based credential redaction with 7 patterns
  - detect() wraps detect_inner() in 500ms background thread timeout
  - visible_output field populated for AX-capable terminals, None for GPU terminals

affects: [03-terminal-context-reading/03-03, 04-ai-integration]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Raw CF FFI inline declarations (CFRelease, CFRetain, CFStringCreateWithBytes, CFArrayGetCount, etc.) -- avoids direct core-foundation-sys dep
    - accessibility_sys crate used for kAXErrorSuccess constant (AX_SUCCESS alias)
    - once_cell::Lazy for zero-cost static regex compilation
    - mpsc channel + recv_timeout(500ms) for hard pipeline timeout
    - CFRetain before CFRelease of parent array -- correct ownership for CFArrayGetValueAtIndex items

key-files:
  created:
    - src-tauri/src/terminal/ax_reader.rs
    - src-tauri/src/terminal/filter.rs
  modified:
    - src-tauri/src/terminal/mod.rs

key-decisions:
  - "Raw CF FFI declarations inline in ax_reader.rs rather than importing core-foundation-sys directly: keeps Cargo.toml clean, no new dependency needed since types are simple *const c_void aliases"
  - "accessibility_sys imported only for kAXErrorSuccess constant: makes the crate dependency explicit and verifiable while avoiding complex type import conflicts with inline CF types"
  - "detect() renamed to wrap detect_inner() transparently: commands/terminal.rs already calls terminal::detect(pid) from Plan 01, no change needed in that file"
  - "CFRetain called before CFRelease(children_val) in find_text_area_in_children: CFArrayGetValueAtIndex does not retain items, so the item must be retained before the array is released"

patterns-established:
  - "Inline raw CF FFI pattern: declare extern C block with CF functions instead of importing core-foundation-sys"
  - "AX text read only for non-GPU terminals: is_gpu_terminal() guards the ax_reader call, GPU terminals silently return None"
  - "Sensitive data filtering as pipeline step: filter_sensitive() applied via .map() on Option<String> before struct construction"

requirements-completed: [TERM-03, TERM-04]

# Metrics
duration: 4min
completed: 2026-02-21
---

# Phase 3 Plan 02: AX Text Reading, Sensitive Data Filter, Timeout Wrapper Summary

**AX tree text reader for Terminal.app and iTerm2 with raw CF FFI, 7-pattern credential filter via once_cell regex, and 500ms hard timeout wrapper that keeps existing detect() API unchanged for commands/terminal.rs**

## Performance

- **Duration:** 4 min
- **Started:** 2026-02-21T13:38:01Z
- **Completed:** 2026-02-21T13:42:00Z
- **Tasks:** 2
- **Files modified:** 3 (2 created, 1 updated)

## Accomplishments

- `ax_reader.rs` implements AX tree walking: Terminal.app path (AXApplication -> AXFocusedWindow -> AXScrollArea -> AXTextArea -> AXValue) and iTerm2 shortcut path (AXApplication -> AXFocusedUIElement -> AXValue)
- Per-element AX messaging timeout set to 1.0 seconds via `AXUIElementSetMessagingTimeout` to prevent hangs on unresponsive terminals
- All AX errors (kAXErrorCannotComplete, kAXErrorNotImplemented, etc.) treated as silent `None` returns -- GPU terminals trigger these and get a graceful fallback
- `filter.rs` defines 7 `SENSITIVE_PATTERNS` via `once_cell::Lazy<Vec<Regex>>`: AWS access keys, generic api_key/token/password assignments, xAI tokens, OpenAI sk- keys, GitHub tokens, PEM private key headers, shell export secrets
- `mod.rs` updated: `detect()` now wraps `detect_inner()` in a background thread with `mpsc::channel` + `recv_timeout(500ms)` -- the overlay never stalls
- `detect_inner()` calls `ax_reader::read_terminal_text()` only for non-GPU terminals (guarded by `is_gpu_terminal()`), then applies `filter::filter_sensitive()` before setting `visible_output`

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement AX tree text reader for Terminal.app and iTerm2** - `94e6419` (feat)
2. **Task 2: Add sensitive data filter, timeout wrapper, and wire full detection pipeline** - `e2cdd88` (feat)

## Files Created/Modified

- `src-tauri/src/terminal/ax_reader.rs` - AX tree walker: raw CF FFI + accessibility_sys kAXErrorSuccess, Terminal.app and iTerm2 traversal paths, CFRetain/CFRelease correctness, 5-level max recursion for find_text_area_in_children
- `src-tauri/src/terminal/filter.rs` - 7 SENSITIVE_PATTERNS compiled via once_cell::Lazy, filter_sensitive() pure function, unit tests for AWS/OpenAI/safe-text cases
- `src-tauri/src/terminal/mod.rs` - Added pub mod filter; detect() now wraps detect_inner() in 500ms thread timeout; detect_inner() wires AX read + filtering + GPU terminal guard

## Decisions Made

- Used raw CF FFI declarations inline in `ax_reader.rs` rather than importing `core-foundation-sys` directly. The types are simple `*const c_void` aliases and the functions have stable ABI. Avoids adding a new Cargo.toml entry for what would be a thin type-alias layer. The `accessibility_sys` crate is imported for the `kAXErrorSuccess` constant to make the dependency explicit.
- `detect()` function kept as the public API name (wrapping `detect_inner()`). `commands/terminal.rs` already calls `terminal::detect(pid)` from Plan 01 -- by keeping the function name, no changes were needed in that file. The timeout wrapping is transparent to callers.
- `CFRetain(child)` called inside `find_text_area_in_children` before releasing the children CFArray. `CFArrayGetValueAtIndex` does not retain returned items; the array's retain callback manages item lifetimes. When `CFRelease(children_val)` is called, the item's ref count would drop to zero without the explicit CFRetain. This is the correct macOS CF memory management pattern for borrowing items from a collection.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Avoided core-foundation-sys import conflict with inline FFI declarations**
- **Found during:** Task 1 cargo check
- **Issue:** `core_foundation_sys` is a transitive dependency (via accessibility-sys) but not a direct one. Importing it via `use core_foundation_sys::...` resulted in E0433 "failed to resolve: use of undeclared crate or module".
- **Fix:** Declared all CF types and functions inline as a raw `extern "C"` block with type aliases (`CFTypeRef = *const c_void`, etc.). This is equivalent functionality with zero new dependencies. Used `accessibility_sys::kAXErrorSuccess` (which is available because accessibility-sys IS a direct dep) for the success constant.
- **Files modified:** src-tauri/src/terminal/ax_reader.rs (rewrote imports section)
- **Verification:** cargo check passes with zero errors after the fix.
- **Committed in:** 94e6419 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 import resolution error)
**Impact on plan:** The raw FFI approach provides identical functionality. No scope change, plan objectives fully met.

## Issues Encountered

- `core_foundation_sys` not importable as a direct module despite being a transitive dep. Resolved by inline extern "C" declarations (documented above as deviation).
- clippy `manual_c_str_literals` warnings in pre-existing ObjC FFI code (`hotkey.rs`, `detect.rs`) -- these are out-of-scope pre-existing warnings from Plan 01 and were NOT fixed (scope boundary rule).

## Next Phase Readiness

- Full TerminalContext pipeline complete: CWD (libproc), shell type (proc_pidpath), running process (child PID walk), visible output (AX tree for Terminal.app/iTerm2, None for GPU terminals), credential filtering
- Plan 03 (frontend integration) ready: frontend can call `invoke("get_terminal_context")` and receive TerminalContext with all fields populated
- The 500ms hard timeout ensures the overlay shows instantly even if AX messaging is slow

## Self-Check: PASSED

- FOUND: src-tauri/src/terminal/ax_reader.rs
- FOUND: src-tauri/src/terminal/filter.rs
- FOUND: .planning/phases/03-terminal-context-reading/03-02-SUMMARY.md
- FOUND commit 94e6419 (feat: AX tree text reader)
- FOUND commit e2cdd88 (feat: filter, timeout, pipeline wiring)
- cargo check: Finished dev profile with zero errors
- cargo clippy: Zero errors (8 pre-existing warnings in Plan 01 code, out of scope)

---
*Phase: 03-terminal-context-reading*
*Completed: 2026-02-21*
