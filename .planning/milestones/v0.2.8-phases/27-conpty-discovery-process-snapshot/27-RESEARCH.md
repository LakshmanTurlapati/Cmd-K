# Phase 27: ConPTY Discovery & Process Snapshot - Research

**Researched:** 2026-03-11
**Domain:** Windows process tree inspection, ConPTY architecture, PEB command line reading
**Confidence:** HIGH

## Summary

This phase replaces the highest-PID shell selection heuristic with ConPTY-aware shell discovery, consolidates three redundant `CreateToolhelp32Snapshot` calls into a single shared `ProcessSnapshot` struct, and filters out internal IDE cmd.exe processes using PEB command line reading combined with parent-process checks.

The existing codebase already has all the building blocks: `find_shell_by_ancestry()` already detects OpenConsole.exe parents (lines 838-862 of process.rs), `read_cwd_from_peb()` already reads PEB via NtQueryInformationProcess + ReadProcessMemory (lines 274-348), and the three snapshot-taking functions (`find_shell_by_ancestry`, `detect_wsl_in_ancestry`, `scan_wsl_processes_diagnostic`) share identical PROCESSENTRY32W iteration patterns. The work is restructuring, not greenfield.

**Primary recommendation:** Build a `ProcessSnapshot` struct that captures parent_map, exe_map, shell_candidates, and openconsole_pids from a single CreateToolhelp32Snapshot call. Pass it as a parameter to all detection functions. Extend the existing PEB reading pattern to read CommandLine at offset 0x70 (vs CurrentDirectory at 0x38) for cmd.exe filtering.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **cmd.exe Filtering**: Use PEB command line reading to check for /C or /D /C flags -- background cmd.exe spawned for git/tasks will have these, interactive cmd.exe won't. Check parent process: interactive cmd.exe is child of conhost.exe/OpenConsole.exe, internal IDE cmd.exe is child of node.exe/Code Helper. Both signals combined: parent-based check is fast, command line is definitive fallback.
- **Snapshot Sharing**: Consolidate the 3 separate CreateToolhelp32Snapshot calls (find_shell_by_ancestry, detect_wsl_in_ancestry, scan_wsl_processes_diagnostic) into a single ProcessSnapshot struct. Build once at hotkey press, pass as parameter to all detection functions. Struct holds parent_map, exe_map, and pre-computed lists (shell_candidates, openconsole_pids).
- **Fallback Behavior**: ConPTY-aware detection is primary for Windows Terminal and IDE terminals. For standalone terminals (powershell.exe, cmd.exe launched directly), ConPTY detection won't apply -- fall back to direct descendant walk (current behavior). For older Windows without OpenConsole.exe, gracefully degrade to highest-PID among descendants. Never break the macOS path -- all changes are #[cfg(target_os = "windows")] gated.

### Claude's Discretion
- ProcessSnapshot struct design and field layout
- Exact PEB command line reading implementation (extend existing CWD reading pattern)
- Error handling for access-denied scenarios on elevated processes
- Logging verbosity and diagnostic output format
- Whether to use GetProcessTimes for recency sorting instead of PID comparison

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| PROC-01 | User's active shell correctly identified via ConPTY parentage (OpenConsole.exe/conhost.exe) instead of highest-PID heuristic | ConPTY architecture research confirms OpenConsole.exe (Win11/WT) and conhost.exe (Win10) as ConPTY host processes; existing code at line 838-862 already has partial detection |
| PROC-02 | Internal IDE cmd.exe processes (git, extensions, tasks) filtered out -- only interactive console-attached cmd.exe selected | PEB CommandLine at offset 0x70 confirmed via Vergilius Project; parent-process check (conhost/OpenConsole parent = interactive) is fast primary filter |
| PROC-03 | Process snapshot consolidated into single CreateToolhelp32Snapshot call shared across shell discovery, WSL detection, and diagnostics | Three identical snapshot+iterate patterns identified at lines 774-808, 1031-1056, 971-997; all build parent_map + exe_map |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| windows-sys | 0.59 | Win32 API bindings (CreateToolhelp32Snapshot, NtQueryInformationProcess, ReadProcessMemory, OpenProcess, GetProcessTimes) | Already in project Cargo.toml with all required features enabled |

### Required Feature Flags (already present)
| Feature | APIs Used |
|---------|-----------|
| `Win32_System_Diagnostics_ToolHelp` | CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W |
| `Win32_System_Diagnostics_Debug` | ReadProcessMemory |
| `Win32_System_Threading` | OpenProcess, NtQueryInformationProcess, GetProcessTimes, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_QUERY_LIMITED_INFORMATION |
| `Win32_Foundation` | CloseHandle, INVALID_HANDLE_VALUE, FILETIME |
| `Wdk_System_Threading` | NtQueryInformationProcess |

No new crates or feature flags are needed.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| PEB command line reading | Win32 GetCommandLineW (own process only) | Cannot read another process's command line -- PEB reading is the only option |
| GetProcessTimes for recency | PID comparison | PIDs can be reused by Windows; GetProcessTimes gives absolute creation timestamp. Recommended for correctness. |

## Architecture Patterns

### Recommended Changes to Existing Structure
```
src-tauri/src/terminal/
  process.rs         # Modified: ProcessSnapshot struct, refactored detection functions
  detect_windows.rs  # No changes needed
  mod.rs             # Modified: thread ProcessSnapshot through detect_app_context_windows
```

No new files needed. All changes are within existing files.

### Pattern 1: ProcessSnapshot Struct
**What:** A struct built once per hotkey press that caches the full process table
**When to use:** At the entry point of Windows detection (detect_app_context_windows or get_foreground_info)

```rust
// Source: derived from existing patterns in process.rs lines 780-807
#[cfg(target_os = "windows")]
pub(crate) struct ProcessSnapshot {
    /// PID -> parent PID mapping
    pub parent_map: std::collections::HashMap<u32, u32>,
    /// PID -> exe name mapping
    pub exe_map: std::collections::HashMap<u32, String>,
    /// Shell process candidates: (pid, exe_name)
    pub shell_candidates: Vec<(u32, String)>,
    /// PIDs of OpenConsole.exe / conhost.exe processes (ConPTY hosts)
    pub conpty_host_pids: Vec<u32>,
}
```

### Pattern 2: PEB Command Line Reading (extend existing CWD pattern)
**What:** Read CommandLine UNICODE_STRING from RTL_USER_PROCESS_PARAMETERS at offset 0x70
**When to use:** To filter cmd.exe processes -- check if command line contains /C or /D /C flags

The existing `read_cwd_from_peb` reads CurrentDirectory at offset 0x38. CommandLine is at offset 0x70 in the same struct. The reading pattern is identical:
1. NtQueryInformationProcess to get PEB base address
2. Read ProcessParameters pointer from PEB+0x20
3. Read UNICODE_STRING (Length u16, MaxLength u16, 4-byte padding, Buffer pointer u64) at params+0x70
4. Read the UTF-16 string from the Buffer pointer

```rust
// Source: Vergilius Project Windows 11 22H2 _RTL_USER_PROCESS_PARAMETERS
// Offsets confirmed:
//   CurrentDirectory: 0x38 (already used in read_cwd_from_peb)
//   ImagePathName:    0x60
//   CommandLine:      0x70
#[cfg(target_os = "windows")]
fn read_command_line_from_peb(handle: windows_sys::Win32::Foundation::HANDLE) -> Option<String> {
    // Same PBI + NtQueryInformationProcess pattern as read_cwd_from_peb
    // Read UNICODE_STRING at params_ptr + 0x70 instead of 0x38
    // Return the full command line string
}
```

### Pattern 3: ConPTY-Aware Shell Selection
**What:** Prioritize shells parented by OpenConsole.exe/conhost.exe over direct descendant walk
**When to use:** When selecting the active shell from multiple candidates

Detection priority:
1. Shells whose parent is a ConPTY host (OpenConsole.exe or conhost.exe) -- these are interactive terminal shells
2. Direct descendant shells of the app PID -- standalone terminal scenario
3. Highest-PID fallback -- degraded mode for older systems

### Pattern 4: cmd.exe Filtering (Two-Signal Approach)
**What:** Combine fast parent check with definitive command line check
**When to use:** For every cmd.exe candidate found in the process tree

```
For each cmd.exe process:
  1. FAST CHECK: Is parent conhost.exe or OpenConsole.exe?
     YES -> likely interactive, keep as candidate
     NO  -> likely internal (parent is node.exe, Code Helper, etc.)
  2. DEFINITIVE CHECK (on remaining cmd.exe candidates, or when fast check is ambiguous):
     Read PEB command line
     Contains "/C " or "/D /C " -> background task, EXCLUDE
     No /C flag -> interactive shell, INCLUDE
```

### Anti-Patterns to Avoid
- **Taking multiple snapshots:** Never call CreateToolhelp32Snapshot more than once per hotkey press. Build ProcessSnapshot once and pass it through.
- **Modifying macOS code paths:** All changes must be `#[cfg(target_os = "windows")]` gated. Never change the macOS detection logic.
- **Adding wsl.exe to KNOWN_SHELL_EXES:** This was explicitly reverted in prior work (see STATE.md). WSL detection is separate from shell detection.
- **Relying solely on PID ordering:** Windows reuses PIDs. Use GetProcessTimes creation time for reliable recency sorting when multiple ConPTY shells match.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process enumeration | Custom WMI/CIM queries | CreateToolhelp32Snapshot | WMI is 100-500ms, snapshot is ~1ms |
| Command line reading | Spawn `wmic process get` | PEB reading via NtQueryInformationProcess | wmic.exe is deprecated, subprocess is slow |
| Process creation time | Track process start order manually | GetProcessTimes | Win32 API gives exact FILETIME creation stamp |

**Key insight:** The existing PEB reading pattern for CWD is battle-tested in this codebase. Extending it to read CommandLine is a mechanical change (different offset, same read pattern), not a new capability.

## Common Pitfalls

### Pitfall 1: conhost.exe vs OpenConsole.exe on Different Windows Versions
**What goes wrong:** Code only checks for OpenConsole.exe, misses conhost.exe on Windows 10
**Why it happens:** Windows Terminal ships OpenConsole.exe; Windows 10 without WT uses conhost.exe. They are the same binary compiled differently.
**How to avoid:** Check for BOTH `OpenConsole.exe` and `conhost.exe` (case-insensitive) as ConPTY host processes. The existing code at line 796 only checks OpenConsole.exe -- this must be extended.
**Warning signs:** Shell detection works in Windows Terminal but fails in standalone cmd.exe/powershell.exe windows.

### Pitfall 2: Access Denied on Elevated Processes
**What goes wrong:** PEB command line reading fails for elevated (admin) processes when CMD+K runs as standard user
**Why it happens:** OpenProcess with PROCESS_QUERY_INFORMATION | PROCESS_VM_READ requires matching or higher privilege level
**How to avoid:** Treat PEB read failure as "unable to determine" -- fall back to parent-process check only. Log the access-denied gracefully, don't crash.
**Warning signs:** cmd.exe filtering breaks when user has an elevated terminal alongside normal ones.

### Pitfall 3: Stale Parent PIDs in Snapshot
**What goes wrong:** PROCESSENTRY32W.th32ParentProcessID may reference a PID that has been reused
**Why it happens:** Windows aggressively reuses PIDs. Parent process may have exited and its PID reassigned.
**How to avoid:** When walking ancestry, validate that the parent chain makes sense (exe names are expected). Don't walk more than 20 levels. If a parent's exe_map entry doesn't exist in the snapshot, stop walking.
**Warning signs:** Shell incorrectly identified as descendant of wrong process tree.

### Pitfall 4: Cursor IDE exe name differs from VS Code
**What goes wrong:** IDE-specific filtering doesn't trigger for Cursor
**Why it happens:** Cursor.exe is in KNOWN_IDE_EXES but might use different internal process names
**How to avoid:** The existing KNOWN_IDE_EXES already includes "Cursor.exe". Test both VS Code and Cursor. Log parent process names during development.
**Warning signs:** cmd.exe filtering works in VS Code but not Cursor.

### Pitfall 5: cmd.exe /C with Spaces in Arguments
**What goes wrong:** Naive substring search for "/C " misses cmd.exe launched with `/C"command"` (no space)
**Why it happens:** Windows command line parsing is notoriously inconsistent
**How to avoid:** Check for `/C` at the start of arguments (after the exe path) or after a space. Also check for `/D /C` and `/S /D /C` patterns. Case-insensitive matching.
**Warning signs:** Some internal IDE cmd.exe processes slip through the filter.

## Code Examples

### Building ProcessSnapshot (single snapshot)
```rust
// Source: refactoring of existing pattern at process.rs:774-808
#[cfg(target_os = "windows")]
impl ProcessSnapshot {
    fn capture() -> Option<Self> {
        use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
        use windows_sys::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
            PROCESSENTRY32W, TH32CS_SNAPPROCESS,
        };

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return None;
            }

            let mut parent_map = std::collections::HashMap::new();
            let mut exe_map = std::collections::HashMap::new();
            let mut shell_candidates = Vec::new();
            let mut conpty_host_pids = Vec::new();

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) != 0 {
                loop {
                    let pid = entry.th32ProcessID;
                    let ppid = entry.th32ParentProcessID;
                    parent_map.insert(pid, ppid);

                    let name_len = entry.szExeFile.iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                    if name.eq_ignore_ascii_case("OpenConsole.exe")
                        || name.eq_ignore_ascii_case("conhost.exe")
                    {
                        conpty_host_pids.push(pid);
                    }
                    if super::detect_windows::is_known_shell_exe(&name) {
                        shell_candidates.push((pid, name.clone()));
                    }

                    exe_map.insert(pid, name);

                    if Process32NextW(snapshot, &mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);

            Some(ProcessSnapshot {
                parent_map,
                exe_map,
                shell_candidates,
                conpty_host_pids,
            })
        }
    }
}
```

### Reading Command Line from PEB (extending existing CWD pattern)
```rust
// Source: extending process.rs:274-348 (read_cwd_from_peb)
// Offsets from Vergilius Project _RTL_USER_PROCESS_PARAMETERS
#[cfg(target_os = "windows")]
unsafe fn read_command_line_from_peb(
    handle: windows_sys::Win32::Foundation::HANDLE
) -> Option<String> {
    use std::mem::size_of;
    use windows_sys::Wdk::System::Threading::NtQueryInformationProcess;

    #[repr(C)]
    struct ProcessBasicInformation {
        _reserved1: usize,
        peb_base_address: usize,
        _reserved2: [usize; 2],
        unique_process_id: usize,
        _reserved3: usize,
    }

    let mut pbi = std::mem::zeroed::<ProcessBasicInformation>();
    let mut return_length: u32 = 0;
    let status = NtQueryInformationProcess(
        handle, 0,
        &mut pbi as *mut _ as *mut _,
        size_of::<ProcessBasicInformation>() as u32,
        &mut return_length,
    );
    if status != 0 || pbi.peb_base_address == 0 {
        return None;
    }

    // Read ProcessParameters pointer from PEB+0x20
    let params_ptr_addr = pbi.peb_base_address + 0x20;
    let mut params_ptr: usize = 0;
    if !read_process_memory(handle, params_ptr_addr,
        &mut params_ptr as *mut _ as *mut u8, size_of::<usize>()) || params_ptr == 0 {
        return None;
    }

    // CommandLine UNICODE_STRING at offset 0x70
    let cmd_unicode_string_addr = params_ptr + 0x70;
    let mut length: u16 = 0;
    if !read_process_memory(handle, cmd_unicode_string_addr,
        &mut length as *mut _ as *mut u8, size_of::<u16>()) || length == 0 {
        return None;
    }

    let mut buffer_ptr: usize = 0;
    let buffer_ptr_addr = cmd_unicode_string_addr + 8;
    if !read_process_memory(handle, buffer_ptr_addr,
        &mut buffer_ptr as *mut _ as *mut u8, size_of::<usize>()) || buffer_ptr == 0 {
        return None;
    }

    let char_count = (length as usize) / 2;
    let mut buf = vec![0u16; char_count];
    if !read_process_memory(handle, buffer_ptr,
        buf.as_mut_ptr() as *mut u8, length as usize) {
        return None;
    }

    Some(String::from_utf16_lossy(&buf))
}
```

### Checking if cmd.exe is Interactive
```rust
// Source: project-specific logic based on CONTEXT.md decisions
#[cfg(target_os = "windows")]
fn is_interactive_cmd(
    pid: u32,
    snapshot: &ProcessSnapshot,
) -> bool {
    // Fast check: parent is ConPTY host
    if let Some(&ppid) = snapshot.parent_map.get(&pid) {
        if snapshot.conpty_host_pids.contains(&ppid) {
            return true; // parented by conhost/OpenConsole = interactive
        }
        // Parent is node.exe, Code Helper, etc. = likely internal
        if let Some(parent_exe) = snapshot.exe_map.get(&ppid) {
            let lower = parent_exe.to_lowercase();
            if lower.contains("node") || lower.contains("code") || lower.contains("cursor") {
                // Definitive check: read command line
                return !has_batch_flag(pid);
            }
        }
    }
    // Fallback: read command line
    !has_batch_flag(pid)
}

fn has_batch_flag(pid: u32) -> bool {
    // Read PEB command line, check for /C or /D /C
    // Return true if cmd.exe was launched with batch execution flags
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Highest-PID shell selection | ConPTY parentage detection | This phase | Correct shell in multi-tab IDEs |
| Blanket cmd.exe exclusion in IDEs | PEB command line filtering | This phase | cmd.exe users in IDEs are supported |
| 3 separate CreateToolhelp32Snapshot calls | Single ProcessSnapshot struct | This phase | ~2ms saved per hotkey press |

**Key architecture insight:** On Windows 10, ConPTY uses `conhost.exe`. On Windows 11 with Windows Terminal, it uses `OpenConsole.exe`. VS Code/Cursor use ConPTY internally and spawn `conhost.exe` as the host process for each terminal tab, regardless of Windows version.

## Open Questions

1. **Cursor.exe internal process names**
   - What we know: Cursor is a VS Code fork, KNOWN_IDE_EXES includes "Cursor.exe"
   - What's unclear: Whether Cursor uses identical internal process hierarchy (Code Helper, etc.)
   - Recommendation: Log parent chain during testing; treat Cursor same as VS Code unless evidence contradicts

2. **GetProcessTimes vs PID for recency**
   - What we know: PIDs can be reused; GetProcessTimes gives absolute timestamps
   - What's unclear: Whether PID reuse actually causes problems in practice for short-lived detection
   - Recommendation: Use GetProcessTimes -- it's already available via Win32_System_Threading feature, adds minimal complexity, and eliminates a theoretical failure mode

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test (#[test] / #[cfg(test)]) |
| Config file | None (Cargo-native) |
| Quick run command | `cargo test --manifest-path src-tauri/Cargo.toml` |
| Full suite command | `cargo test --manifest-path src-tauri/Cargo.toml` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PROC-01 | ConPTY shell discovery via OpenConsole/conhost parentage | manual-only | Manual: open VS Code with multiple terminal tabs, run CMD+K, verify correct shell detected | N/A -- requires live Windows terminal environment |
| PROC-02 | cmd.exe filtering (PEB command line /C flag check) | unit + manual | Unit: test command line parsing logic. Manual: open cmd.exe tab in VS Code alongside git operations | Wave 0: unit test for `has_batch_flag` parsing |
| PROC-03 | Single ProcessSnapshot shared across detection | unit | Unit: test that ProcessSnapshot::capture() builds correct maps. Integration requires Windows. | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path src-tauri/Cargo.toml`
- **Per wave merge:** `cargo test --manifest-path src-tauri/Cargo.toml` + manual Windows testing
- **Phase gate:** Full suite green + manual verification on Windows with VS Code/Cursor

### Wave 0 Gaps
- [ ] Unit tests for command line flag parsing (`has_batch_flag` / `/C` detection logic) -- can test on any platform with string parsing
- [ ] Unit tests for `is_interactive_cmd` logic (mock snapshot data)
- [ ] Note: ProcessSnapshot::capture() and PEB reading cannot be unit tested cross-platform -- require manual Windows testing

## Sources

### Primary (HIGH confidence)
- [Vergilius Project _RTL_USER_PROCESS_PARAMETERS](https://www.vergiliusproject.com/kernels/x64/windows-11/22h2/_RTL_USER_PROCESS_PARAMETERS) - confirmed offsets: CurrentDirectory=0x38, ImagePathName=0x60, CommandLine=0x70
- [Microsoft RTL_USER_PROCESS_PARAMETERS docs](https://learn.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-rtl_user_process_parameters) - official (partial) structure documentation
- [Windows Terminal Discussion #12115](https://github.com/microsoft/terminal/discussions/12115) - OpenConsole.exe vs conhost.exe relationship confirmed
- [GetProcessTimes (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes) - process creation time API
- Existing codebase: `src-tauri/src/terminal/process.rs` lines 274-363 (PEB reading pattern), 767-896 (find_shell_by_ancestry with OpenConsole detection)

### Secondary (MEDIUM confidence)
- [Windows Console Ecosystem Roadmap](https://learn.microsoft.com/en-us/windows/console/ecosystem-roadmap) - ConPTY architecture overview
- [ConPTY Blog Post](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/) - ConPTY introduction and architecture

### Tertiary (LOW confidence)
- [Process ID reuse on Windows](http://m-a-tech.blogspot.com/2010/05/process-id-reuse-on-windows.html) - PID reuse behavior (blog, but aligns with known Windows behavior)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all APIs already used in codebase
- Architecture: HIGH - refactoring existing patterns, not introducing new concepts
- Pitfalls: HIGH - derived from concrete codebase analysis and documented Windows behavior
- PEB offsets: HIGH - verified via Vergilius Project (authoritative kernel struct source)

**Research date:** 2026-03-11
**Valid until:** 2026-04-11 (stable domain -- Windows process APIs don't change frequently)
