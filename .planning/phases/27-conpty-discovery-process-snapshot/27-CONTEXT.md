# Phase 27: ConPTY Discovery & Process Snapshot - Context

**Gathered:** 2026-03-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Replace the highest-PID shell selection heuristic with ConPTY-aware shell discovery. Consolidate redundant process snapshots into a single shared snapshot. Filter internal IDE cmd.exe processes so only interactive user shells are selected.

</domain>

<decisions>
## Implementation Decisions

### cmd.exe Filtering
- Use PEB command line reading to check for /C or /D /C flags — background cmd.exe spawned for git/tasks will have these, interactive cmd.exe won't
- Check parent process: interactive cmd.exe is child of conhost.exe/OpenConsole.exe, internal IDE cmd.exe is child of node.exe/Code Helper
- Both signals combined: parent-based check is fast, command line is definitive fallback

### Snapshot Sharing
- Consolidate the 3 separate CreateToolhelp32Snapshot calls (find_shell_by_ancestry, detect_wsl_in_ancestry, scan_wsl_processes_diagnostic) into a single ProcessSnapshot struct
- Build once at hotkey press, pass as parameter to all detection functions
- Struct holds parent_map, exe_map, and pre-computed lists (shell_candidates, openconsole_pids)

### Fallback Behavior
- ConPTY-aware detection is primary for Windows Terminal and IDE terminals
- For standalone terminals (powershell.exe, cmd.exe launched directly), ConPTY detection won't apply — fall back to direct descendant walk (current behavior)
- For older Windows without OpenConsole.exe, gracefully degrade to highest-PID among descendants
- Never break the macOS path — all changes are #[cfg(target_os = "windows")] gated

### Claude's Discretion
- ProcessSnapshot struct design and field layout
- Exact PEB command line reading implementation (extend existing CWD reading pattern)
- Error handling for access-denied scenarios on elevated processes
- Logging verbosity and diagnostic output format
- Whether to use GetProcessTimes for recency sorting instead of PID comparison

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The key constraint is that all 3 requirements (PROC-01, PROC-02, PROC-03) must be addressed as a cohesive change since the snapshot consolidation affects both shell discovery and cmd.exe filtering.

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `find_shell_by_ancestry()` (process.rs:767-896): Already has ConPTY fallback path with OpenConsole.exe detection — needs restructuring, not rewriting
- `detect_wsl_in_ancestry()` (process.rs:1024-1075): Builds identical parent_map/exe_map — prime candidate for snapshot sharing
- PEB CWD reading (process.rs): Existing NtQueryInformationProcess + ReadProcessMemory pattern can be extended to read command line from RTL_USER_PROCESS_PARAMETERS
- `get_exe_name_for_pid()` (detect_windows.rs:92-111): Already does OpenProcess + QueryFullProcessImageNameW

### Established Patterns
- Windows process inspection uses `unsafe` blocks with `windows_sys` crate — no abstraction layer
- All detection functions are `#[cfg(target_os = "windows")]` with stubs for other platforms
- Diagnostic output via `eprintln!("[process] ...")` — keep this pattern
- PROCESSENTRY32W iteration pattern repeated 3 times — exactly what snapshot consolidation removes

### Integration Points
- `find_shell_by_ancestry()` called from `detect_app_context_windows()` in mod.rs
- `detect_wsl_in_ancestry()` called from `get_foreground_info()` in process.rs
- `scan_wsl_processes_diagnostic()` called from `get_foreground_info()` for debug logging
- The ProcessSnapshot struct needs to be created in the detection entry point and threaded through

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 27-conpty-discovery-process-snapshot*
*Context gathered: 2026-03-11*
