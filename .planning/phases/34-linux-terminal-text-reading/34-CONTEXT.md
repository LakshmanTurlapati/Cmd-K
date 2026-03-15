# Phase 34: Linux Terminal Text Reading - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Read recent terminal output on Linux across multiple terminal emulators, populating the existing `visible_output` field in `TerminalContext`. Three strategies: AT-SPI2 D-Bus for VTE-based terminals, kitty remote control subprocess, WezTerm CLI subprocess. Graceful `None` for unsupported terminals. No UI changes — pure Rust infrastructure that feeds into the Phase 33 smart context pipeline.

</domain>

<decisions>
## Implementation Decisions

### AT-SPI2 integration
- Use `zbus` crate for pure Rust async D-Bus client (no C deps, no libdbus/libatspi linking)
- Sync D-Bus calls with 500ms timeout — matches macOS AX reader pattern
- Request visible screen text only (not scrollback) — consistent with macOS/Windows behavior
- Find terminal widget by AT-SPI2 role-based match (VTE widgets report role "terminal") — targeted, fast
- If AT-SPI2 is unavailable or times out, return None silently

### Subprocess APIs (kitty & WezTerm)
- kitty: shell out to `kitty @ get-text --extent screen` — uses kitty's built-in IPC socket
- WezTerm: shell out to `wezterm cli get-text` — reads visible pane text
- Screen-only text for both (no scrollback) — consistent with AT-SPI2 decision
- 500ms timeout for both subprocess calls — keeps hotkey response snappy
- If kitty remote control is disabled (`allow_remote_control` not set), return None silently — zero setup principle, no nag or setup prompt

### Terminal detection routing
- Process name match from /proc/PID/exe (already detected in Phase 30) determines which strategy to use
- Routing map: gnome-terminal/tilix/terminator/mate-terminal/xfce4-terminal/konsole → AT-SPI2, kitty → kitty @, wezterm-gui → wezterm cli, everything else → None
- No fallback chain — if the matched strategy fails, return None immediately. No cascading attempts. Simple, predictable, fast.
- Reader function signature: `read_terminal_text_linux(pid, exe_name)` — accepts terminal process name from caller, no redundant /proc lookups

### Module structure
- Single `terminal/linux_reader.rs` module paralleling `ax_reader.rs` (macOS) and `uia_reader.rs` (Windows)
- One pub fn `read_terminal_text_linux()` dispatches to internal AT-SPI2/kitty/WezTerm functions
- Wire into `detect_inner_linux()` in `mod.rs` to populate `visible_output` (currently `None` at line 323)

### Scope & coverage
- Must-haves: GNOME Terminal (+ all VTE terminals automatically), kitty, WezTerm — per LTXT-01/02/03
- Konsole (KDE) added to AT-SPI2 process name map cheaply — zero extra code, Qt accessibility uses AT-SPI2
- Alacritty, st, and other terminals without text APIs → return None gracefully (LTXT-04)
- zbus as `cfg(target_os = "linux")` dependency — only compiles on Linux, macOS/Windows builds unaffected

### Claude's Discretion
- Exact zbus API calls for AT-SPI2 text extraction
- Process name matching list (additional VTE-based terminals beyond the ones listed)
- Error handling granularity (which errors to log vs silently ignore)
- Test strategy and mocking approach for D-Bus and subprocess calls
- Internal data structures for routing

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ax_reader.rs` (macOS): Pattern for platform-specific terminal text reader — pub fn with internal dispatch by terminal type
- `uia_reader.rs` (Windows): Role-based accessible element matching (ControlType::List) — similar to AT-SPI2 role matching
- `detect_linux::get_exe_name_for_pid()`: Already reads /proc/PID/exe — provides process name for routing
- `detect_linux::is_known_terminal_exe()`: Terminal classification list — extend for reader routing

### Established Patterns
- Platform-specific code uses `#[cfg(target_os = "...")]` at function level
- Terminal readers return `Option<String>` — None on any failure
- `visible_output` field in `TerminalContext` struct already exists, set to None on Linux
- Phase 33's `context.rs` handles ANSI stripping and smart truncation downstream — reader provides raw text

### Integration Points
- `detect_inner_linux()` in mod.rs line 320-326: Currently returns `visible_output: None` — wire in `linux_reader::read_terminal_text_linux()`
- `detect_app_context_linux()` in mod.rs line 441-454: Same pattern — wire in for `visible_text`
- `terminal/mod.rs`: Add `pub mod linux_reader;` with `#[cfg(target_os = "linux")]`
- Cargo.toml: Add `zbus` dependency with `target.'cfg(target_os = "linux")'`

</code_context>

<specifics>
## Specific Ideas

- Processing pipeline: linux_reader returns raw text → Phase 33 context.rs strips ANSI + truncates → filter.rs redacts sensitive data → AI prompt
- Follow the "zero setup" principle throughout — if a terminal's API isn't available (kitty remote disabled, AT-SPI2 not running), silently degrade. Never prompt users to configure anything.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 34-linux-terminal-text-reading*
*Context gathered: 2026-03-15*
