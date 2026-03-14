# Phase 30: Linux Process Detection - Context

**Gathered:** 2026-03-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect terminal CWD and shell type on Linux via /proc filesystem. Process tree walking finds the shell process from any terminal emulator PID. The project compiles on Linux with real /proc code paths replacing stubs. No UI work — pure Rust infrastructure.

</domain>

<decisions>
## Implementation Decisions

### Terminal emulator scope
- Generic /proc-based approach that works with ANY terminal emulator — not terminal-specific code paths
- Explicitly test with: GNOME Terminal, kitty, Alacritty, Konsole, WezTerm, xfce4-terminal
- Process tree shape is the same regardless of emulator: emulator → shell → children
- No terminal-specific APIs in this phase (kitty remote control, AT-SPI2 are Phase 34)

### IDE terminal handling
- Phase 30 SHOULD handle IDE terminals (VS Code, Cursor, JetBrains) on Linux
- IDE process trees on Linux are simpler than Windows — no ConPTY layer, shells are direct children of the IDE process
- Reuse the existing `find_shell_pid` recursive walk + ancestry search pattern
- If multi-tab disambiguation is needed (multiple shells under one IDE), use the same focused-CWD matching pattern from macOS

### Fallback behavior
- Permission denied on /proc/PID/cwd or /proc/PID/exe → return None, log debug warning (not user-visible)
- Containerized environments (Docker, Flatpak) may restrict /proc access → graceful None, app still functions without context
- Race conditions (/proc entry disappears mid-read) → catch errors, return None
- No alternative detection methods (no subprocess calls like lsof) — /proc is sufficient on Linux

### cfg gate architecture
- Split from current two-way (`macos` / `not(macos)`) to three-way: `macos` / `linux` / `windows`
- Functions that were `#[cfg(not(target_os = "macos"))]` stubs get real Linux implementations
- `find_shell_pid` maintains 3-arg arity: `(terminal_pid, focused_cwd, snapshot)` where snapshot is `Option<&()>` on Linux (same as current non-Windows)

### Claude's Discretion
- Exact /proc parsing approach (read /proc/PID/stat vs /proc/PID/status for ppid)
- Whether to use /proc/PID/task/*/children or scan /proc/*/stat for child discovery
- Error handling granularity (which errors to log vs silently ignore)
- Test structure and mocking strategy for /proc filesystem

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `find_shell_pid()` in process.rs: Recursive walk + broad ancestry search — logic is platform-agnostic, only leaf functions need Linux impl
- `KNOWN_SHELLS` constant: Already lists bash, zsh, fish, etc. — reuse directly
- `infer_shell_from_text()` in mod.rs: Already detects Linux prompt patterns (user@host:$)
- `infer_linux_cwd_from_text()` in mod.rs: Parses user@host:/path$ patterns — fallback for text-based CWD

### Established Patterns
- Platform-specific code uses `#[cfg(target_os = "...")]` at function level, not module level
- Stubs return `None` or empty `Vec` — Linux implementations follow same return types
- macOS uses raw FFI (proc_pidinfo), Windows uses Win32 APIs — Linux uses /proc filesystem (zero deps)
- ProcessSnapshot is Windows-only — Linux doesn't need equivalent (no ConPTY concept)

### Integration Points
- `detect_inner()` in mod.rs: Orchestrates CWD + shell detection — needs Linux branch
- `detect_app_context()` in mod.rs: Returns app name for terminal classification — needs Linux impl
- `compute_window_key()`: Calls `find_shell_pid` — already cross-platform compatible
- `get_terminal_context` Tauri command: Frontend-facing API — no changes needed (calls detect_inner)

</code_context>

<specifics>
## Specific Ideas

No specific requirements — /proc filesystem approach is well-defined by the requirements (LPROC-01, LPROC-02, LPROC-03). Follow the same patterns established by macOS (raw syscall/filesystem) rather than Windows (snapshot-based).

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 30-linux-process-detection*
*Context gathered: 2026-03-13*
