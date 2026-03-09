# Phase 23: WSL Terminal Context - Context

**Gathered:** 2026-03-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect WSL sessions across all 4 host types (Windows Terminal, VS Code Remote-WSL, Cursor Remote-WSL, standalone wsl.exe), read Linux CWD/shell/output, and generate Linux-appropriate commands with correct safety patterns. Users in WSL get the same context-aware experience as native terminal users.

</domain>

<decisions>
## Implementation Decisions

### WSL Detection Strategy
- **Process tree walk** to identify WSL sessions — walk from focused terminal PID, detect wsl.exe or WSL init process in ancestry
- **Same detection path for all 4 host types** (Windows Terminal, VS Code, Cursor, standalone wsl.exe) — no host-specific code paths
- **No distro detection** — just identify "WSL" vs "not WSL", don't parse Ubuntu/Debian/Arch name
- Graceful fallback: if wsl.exe found but no running distro, fall back to normal Windows terminal context silently

### Context Badge Display
- Badge shows just **"WSL"** as the context label — no distro name, no shell prefix
- CWD displayed as **Linux paths as-is** — /home/user/project, /mnt/c/Users/... (no translation to Windows paths)
- **Show running process** inside WSL shell (node, python, etc.) — same behavior as native terminals

### Linux Command Generation
- **Full Linux mode** — system prompt tells AI "You are in a Linux terminal"
- OS context sent to AI says **"WSL on Windows"** — AI knows it's WSL and can mention Windows-WSL interop when relevant (e.g. `code .` opens VS Code)
- **Linux destructive patterns** applied when in WSL session (rm -rf, dd, chmod 777, mkfs, etc.) — not Windows patterns
- **WSL-aware confirmation text** — "This Linux command may be destructive" instead of generic text

### WSL Output Reading
- **Same UIA approach** as Windows terminals — WSL runs inside host windows (WT, VS Code), UIA tree has the text
- **Add Linux-specific secret filtering** — SSH keys, /etc/shadow lines, sudo password prompts, .env file contents
- **Same ~4KB output limit** as native terminals — consistent behavior
- Graceful fallback if WSL not properly configured

### Claude's Discretion
- Whether to cache WSL detection per terminal window or re-detect each invocation (based on perf characteristics)
- Process tree walk implementation details (Windows API calls, depth limits)
- Exact Linux secret filter regex patterns
- How to extend TerminalContext struct (new field vs enum)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TerminalContext` struct (terminal/mod.rs): Has shell_type, cwd, visible_output, running_process — needs WSL flag
- `detect_windows.rs`: KNOWN_TERMINAL_EXES, KNOWN_IDE_EXES, KNOWN_SHELL_EXES lists, process tree helpers
- `process.rs`: Process inspection patterns (macOS libproc) — Windows equivalent needed
- `safety.rs`: Already has WSL pass-through destructive patterns (wsl.exe rm -rf, dd, mkfs, --unregister)
- `uia_reader.rs`: Windows UI Automation for reading terminal text — reuse for WSL output
- `filter.rs`: Secret filtering for terminal output — extend with Linux patterns

### Established Patterns
- Process tree walking via platform-specific FFI (macOS libproc, Windows will need similar)
- Shell detection from binary name (KNOWN_SHELLS list)
- TerminalContext → AppContext wrapping for frontend
- Safety regex patterns organized by platform section

### Integration Points
- `terminal/mod.rs`: Add WSL-related field(s) to TerminalContext
- `detect_windows.rs`: Add WSL detection to Windows process tree walk
- `commands/ai.rs`: System prompt template needs WSL/Linux branch
- `commands/safety.rs`: Linux destructive patterns need conditional application based on WSL flag
- Frontend context badge: Render "WSL" when WSL detected
- `filter.rs`: Add Linux secret patterns

</code_context>

<specifics>
## Specific Ideas

- The user develops on WSL daily (this repo is at /mnt/c/...) — this is a real daily workflow, not theoretical
- Cross-platform: this phase is Windows-only (WSL doesn't exist on macOS), but code should compile on both platforms with cfg(target_os) guards
- Existing WSL pass-through patterns in safety.rs handle `wsl.exe rm -rf` from Windows side — new patterns handle `rm -rf` from inside WSL

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 23-wsl-terminal-context*
*Context gathered: 2026-03-09*
