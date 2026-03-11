# Technology Stack: Windows Terminal Detection Fix (v0.2.8)

**Project:** CMD+K -- Windows IDE Terminal Detection Fix
**Researched:** 2026-03-11
**Overall confidence:** HIGH

> This document supersedes the v0.2.6 STACK.md for the v0.2.8 milestone. The validated stack (Tauri v2, windows-sys 0.59, uiautomation 0.24, CreateToolhelp32Snapshot, NtQueryInformationProcess, UIA for text reading) is not re-researched. Focus is strictly on what API additions, crate changes, and new techniques are needed to fix IDE terminal detection on Windows.

---

## Problem Analysis

Three problems exist in the current Windows terminal detection code:

### Problem 1: Cannot identify active terminal tab in VS Code/Cursor
The current `find_shell_by_ancestry()` finds ALL shell processes descending from Code.exe, then picks the highest PID. This is wrong when multiple terminal tabs exist -- the user may have the first tab focused while the code picks the newest-spawned shell.

### Problem 2: UIA tree walk for Electron apps is noisy
`try_walk_children()` in `uia_reader.rs` walks ALL descendants with `TreeScope::Descendants` and concatenates every element's Name property. For VS Code, this captures the entire UI -- sidebar, editor, menus -- not just terminal content. The `is_window_chrome()` filter is too simple.

### Problem 3: cmd.exe false positives from internal IDE processes
VS Code spawns many internal cmd.exe processes for git operations, task runners, and extensions. The current heuristic filters cmd.exe only when `descendant_shells.len() > 1`, but this is fragile. A single cmd.exe terminal tab with internal cmd.exe processes creates ambiguity.

---

## Recommended Stack Additions

### Core: Process Command Line Reading via PEB

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| windows-sys (new feature) | 0.59 (existing) | Add `Win32_System_Console` feature | Enables `GetConsoleProcessList` and `AttachConsole` for console-based process identification |

**What this enables:**

Reading process command line arguments from the PEB is the key to distinguishing internal cmd.exe from user-interactive shells. The codebase already has `read_cwd_from_peb()` that reads `RTL_USER_PROCESS_PARAMETERS.CurrentDirectory` from a remote process via `NtQueryInformationProcess` + `ReadProcessMemory`. The **exact same technique** reads `RTL_USER_PROCESS_PARAMETERS.CommandLine` from the same struct -- it is at a different offset in the same memory region already being read.

**Why this matters:** VS Code internal cmd.exe processes have command lines like:
- `cmd.exe /D /C "...git..."` (git operations)
- `cmd.exe /C "...node..."` (extension host, tasks)
- `cmd.exe /c echo %PROMPT%` (shell detection probes)

User-interactive shells have command lines like:
- `cmd.exe` (no args, or just `/K`)
- `powershell.exe -NoLogo`
- `pwsh.exe`
- `bash.exe --login`
- `wsl.exe` or `wsl.exe -d Ubuntu`

**Confidence:** HIGH -- the PEB reading code already exists in `process.rs` (lines 273-362). Adding CommandLine reading is a 20-line change to the existing `read_cwd_from_peb()` function. The `RTL_USER_PROCESS_PARAMETERS` struct layout is documented by Microsoft and stable across Windows versions.

### Core: Process Creation Time via GetProcessTimes

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| windows-sys (existing feature) | 0.59 | `GetProcessTimes` from `Win32_System_Threading` | Already available, just not used. Enables sorting shell candidates by creation time instead of PID. |

**What this enables:**

The current code uses `max_by_key(|(pid, _)| *pid)` to pick the "most recently spawned" shell. PID is a poor proxy for recency -- Windows PIDs recycle. `GetProcessTimes` returns `lpCreationTime` as a `FILETIME`, which is the actual creation timestamp. This is already available through the existing `Win32_System_Threading` feature.

**Confidence:** HIGH -- `GetProcessTimes` is in the already-enabled `Win32_System_Threading` feature set. No new features needed.

### Supporting: sysinfo crate (DO NOT ADD)

Considered `sysinfo` (v0.38.4) which exposes `cmd()`, `start_time()`, and `parent()` per process. However, it pulls in a large dependency for something achievable with 40 lines of direct Win32 calls that the codebase already makes. The codebase's pattern is raw Win32 FFI via `windows-sys`, and sysinfo would be an architectural mismatch.

---

## No New Crates Needed

The entire fix uses APIs already available through the existing `windows-sys 0.59` dependency, with one minor Cargo.toml feature addition.

### Cargo.toml Change

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_System_Diagnostics_Debug",
    "Win32_Security",
    "Win32_UI_Input_KeyboardAndMouse",
    "Wdk_System_Threading",
    "Win32_System_Console",           # NEW: for GetConsoleProcessList
] }
```

**Note on windows-sys version:** The latest is 0.61.2 (October 2025). Upgrading from 0.59 is possible but unnecessary for this milestone -- 0.59 has all needed APIs. Upgrading windows-sys can cause cascading dependency updates and should be a separate chore.

---

## Three Fix Strategies

### Strategy 1: Command Line Filtering for Shell Disambiguation

**API:** `NtQueryInformationProcess` + `ReadProcessMemory` (existing code path)

Read the `CommandLine` field from `RTL_USER_PROCESS_PARAMETERS` for each shell candidate process. Filter out cmd.exe processes whose command line contains `/D /C`, `/c "`, or other batch-execution flags. These are non-interactive background processes.

**Implementation approach:**

Extend the existing `read_cwd_from_peb()` function (or create a parallel `read_cmdline_from_peb()`) that reads the `CommandLine` UNICODE_STRING from the same `RTL_USER_PROCESS_PARAMETERS` struct the CWD reader already accesses.

In `RTL_USER_PROCESS_PARAMETERS`:
- `CurrentDirectory` is at offset 0x38 (the DosPath within CURDIR)
- `CommandLine` is at offset 0x70 (a UNICODE_STRING: Length u16, MaxLength u16, Buffer ptr)

The code already reads `CurrentDirectory` by navigating PEB -> ProcessParameters -> reading memory at calculated offsets. `CommandLine` is at a known fixed offset in the same struct.

**Classification rules for cmd.exe:**
- `/C` or `/c` flag = non-interactive (batch execution, exits after command)
- `/K` or `/k` flag = interactive (keeps running after command)
- No flags or just `/Q`, `/A`, `/U` = interactive
- `/D /C` specifically = VS Code internal (git ops, tasks)

**Confidence:** HIGH -- this is the approach VS Code itself uses internally (via `@vscode/windows-process-tree` which reads command lines). The PEB layout is documented and the code pattern already exists in the codebase.

### Strategy 2: UIA Scoping for Terminal Panel Text

**API:** `uiautomation` 0.24 (existing crate)

The current `try_walk_children()` walks ALL descendants of the VS Code window, capturing sidebar, editor, and menu text alongside terminal text. This needs scoping.

**Approach: Walk the UIA tree with name/class filtering**

Instead of `find_all(TreeScope::Descendants, &true_condition)`, build a condition that targets terminal-related UIA elements:

1. **ControlType filtering:** Terminal panes in VS Code expose as `ControlType::Pane` or `ControlType::Custom` elements. Filter out `ControlType::MenuItem`, `ControlType::TreeItem`, `ControlType::Tab`, `ControlType::Button`.

2. **ClassName filtering:** VS Code's terminal area uses Chromium-rendered elements. The xterm.js terminal canvas elements have specific class names that can be targeted.

3. **AutomationId filtering:** VS Code terminal panels sometimes expose automation IDs containing "terminal" or "xterm".

4. **Fallback: TextPattern priority.** If any descendant element supports `UITextPattern`, prefer that over name-concatenation. Terminal controls in Windows Terminal expose TextPattern; VS Code's xterm.js may not, but this is a good first filter.

**What NOT to do:** Do not rely on VS Code's screen reader mode (`"terminal.integrated.accessibleViewPreserveFocus"`). This requires the user to enable accessibility settings in VS Code, violating the "zero setup" constraint. The UIA approach must work with VS Code's default settings.

**Confidence:** MEDIUM -- UIA element structure in Electron apps (VS Code, Cursor) varies across versions. The filtering approach is sound but specific class names/automation IDs need empirical verification on a real Windows machine. Flag for deeper research during implementation.

### Strategy 3: Focused Tab Detection via Window Title Parsing

**API:** `GetWindowTextW` (existing, in `detect_windows.rs`)

VS Code and Cursor window titles follow a predictable format:
```
filename.ext - workspace [optional decorators] - Visual Studio Code
filename.ext - workspace [WSL: Ubuntu] - Visual Studio Code
```

When a terminal tab is focused, the title changes to include the terminal shell info:
```
powershell - workspace - Visual Studio Code
bash - workspace - Visual Studio Code
cmd - workspace - Visual Studio Code
```

This is controlled by VS Code's `terminal.integrated.tabs.title` setting which defaults to `${process}` -- the shell process name.

**Approach:** Parse the window title to extract the terminal name when a terminal tab is active. If the title's first segment (before the first ` - `) matches a known shell name, the terminal is focused and the title tells us which shell type.

**Limitations:**
- Only works when a terminal tab is focused (not when the editor is focused)
- User can customize `terminal.integrated.tabs.title`, changing the format
- Does not identify the specific PID of the focused terminal

**Use as supplementary signal, not primary detection.** Combine with process tree walking: if the window title suggests "powershell", prefer powershell.exe candidates from the process tree.

**Confidence:** HIGH for the default title format. LOW for handling all custom title formats. Recommended as a heuristic boost, not sole detection method.

---

## Combined Detection Algorithm

The fix combines all three strategies in priority order:

```
1. Get HWND and exe name (existing: GetWindowThreadProcessId + QueryFullProcessImageNameW)
2. Get window title (existing: GetWindowTextW)
3. Parse title for terminal hint (Strategy 3)
4. Snapshot process tree (existing: CreateToolhelp32Snapshot)
5. Find shell candidates descending from app PID (existing)
6. For each candidate:
   a. Read command line from PEB (Strategy 1) -- filter out non-interactive
   b. Read creation time via GetProcessTimes -- for recency sorting
7. If title hint matches a candidate shell type, prefer that candidate
8. Otherwise pick most recently created interactive shell
9. For WSL: check process ancestry for wsl.exe (existing)
10. Read terminal text via UIA with scoped filtering (Strategy 2)
```

---

## windows-sys Feature: Win32_System_Console

### What it provides

| Function | Purpose | Use Case |
|----------|---------|----------|
| `GetConsoleProcessList` | List PIDs attached to a console | Could identify which shell owns a specific console session |
| `AttachConsole` | Attach to another process's console | Potential for reading console state (use cautiously) |
| `FreeConsole` | Detach from current console | Cleanup after AttachConsole |

### Why add it

`GetConsoleProcessList` can be called after `AttachConsole(pid)` to get the list of processes sharing a console session. For standalone terminals (cmd.exe, PowerShell), this directly identifies which shell owns the console. For ConPTY-based terminals (Windows Terminal, VS Code), each tab's shell gets its own pseudo-console, and the process list helps disambiguate.

**Caveat:** `AttachConsole` detaches from the current console first. The calling process (CMD+K) doesn't have a console (it's a GUI app), so this is safe. But it must be called carefully -- only one console attachment at a time.

**Confidence:** MEDIUM -- this is a viable supplementary signal but the process tree + command line approach (Strategy 1) may be sufficient alone. Add the feature flag now; use if needed during implementation.

---

## UIA Crate: No Version Change Needed

The `uiautomation` crate at version 0.24 provides everything needed:

| Capability | Available | API |
|------------|-----------|-----|
| Element from HWND | Yes | `automation.element_from_handle()` |
| ControlType filtering | Yes | `element.get_control_type()`, condition builders |
| ClassName reading | Yes | `element.get_classname()` |
| AutomationId reading | Yes | `element.get_automation_id()` |
| TextPattern | Yes | `element.get_pattern::<UITextPattern>()` |
| Tree walker | Yes | `automation.create_tree_walker()` |
| Condition AND/OR/NOT | Yes | `automation.create_and_condition()`, etc. |

All the filtering capabilities needed for Strategy 2 are already in the current crate version. The fix is in how we use the API, not what version we use.

**Confidence:** HIGH -- verified against current codebase usage in `uia_reader.rs`.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Shell disambiguation | PEB command line reading | `sysinfo` crate `cmd()` method | Adds 5MB+ dependency for 40 lines of Win32 code. Architectural mismatch with raw FFI pattern. |
| Shell disambiguation | PEB command line reading | WMI queries (`Win32_Process.CommandLine`) | WMI is slow (100ms+), requires COM initialization, and adds dependency complexity. PEB reading is <1ms. |
| Shell disambiguation | PEB command line reading | `wmic process get CommandLine` subprocess | Spawning a process is slow and unreliable. `wmic` is deprecated on modern Windows. |
| Process creation time | `GetProcessTimes` via windows-sys | `sysinfo` crate `start_time()` | Same reason -- unnecessary dependency for one API call. |
| Active tab detection | Window title parsing | VS Code extension API | Violates "no extension" architectural decision. Would require separate VS Code extension. |
| Active tab detection | Window title parsing | Named pipe / IPC with terminal | Would require terminal-side setup, violating "zero setup" constraint. |
| Terminal text reading | Scoped UIA tree walk | VS Code screen reader mode | Requires user to enable `terminal.integrated.accessibleViewPreserveFocus`. Violates zero-setup. |
| Terminal text reading | Scoped UIA tree walk | Electron `--force-renderer-accessibility` | Would require modifying VS Code's launch flags. Not our process to control. |
| Terminal text reading | Scoped UIA tree walk | Clipboard-based text capture | Destructive (overwrites clipboard), unreliable, poor UX. |
| Console identification | `Win32_System_Console` feature | No console APIs | Limits detection options; the feature flag is cheap to add. |
| windows-sys version | Stay on 0.59 | Upgrade to 0.61.2 | Risk of cascading dependency changes. 0.59 has all needed APIs. Defer upgrade to separate chore. |

---

## What NOT to Add

| Temptation | Why Not |
|------------|---------|
| `sysinfo` crate | Massive dependency (pulls in `libc`, `once_cell`, `rayon` on some configs) for 2-3 Win32 calls the codebase already makes directly. |
| `windows` crate (high-level) | The codebase uses `windows-sys` (zero-overhead FFI bindings). Mixing both creates confusion and larger binaries. |
| VS Code extension for terminal detection | Architectural decision already made: no extensions. The overlay is standalone. |
| WMI/COM queries for process info | Slow (100ms+), complex COM initialization, unreliable compared to direct NT API calls. |
| `ntapi` crate | Provides Rust bindings for NT native API, but the codebase already calls `NtQueryInformationProcess` directly via `windows-sys::Wdk`. Adding another crate for the same function is redundant. |
| Screen reader mode dependency | Requiring users to enable accessibility in VS Code violates the zero-setup constraint. Detection must work with default VS Code settings. |
| `AttachConsole` for text reading | `AttachConsole` + `ReadConsoleOutput` could theoretically read terminal buffer, but it's fragile, interferes with the target process, and UIA is already the established approach. |
| Upgrading `uiautomation` crate | 0.24 has all needed APIs. No newer version exists that would change the approach. |

---

## Implementation Priority

| Priority | Change | Effort | Impact |
|----------|--------|--------|--------|
| P0 | Add `read_cmdline_from_peb()` for command line reading | Low (20 lines, extends existing code) | HIGH -- fixes cmd.exe false positives |
| P0 | Filter non-interactive cmd.exe by `/C` and `/D /C` flags | Low (10 lines, classification logic) | HIGH -- eliminates VS Code internal processes |
| P1 | Replace PID-based shell selection with `GetProcessTimes` creation time | Low (15 lines) | MEDIUM -- more reliable "most recent" detection |
| P1 | Add window title parsing for terminal tab hint | Low (20 lines, regex on title) | MEDIUM -- helps identify focused terminal type |
| P2 | Scope UIA tree walk with ControlType/ClassName filtering | Medium (30-50 lines, needs empirical testing) | MEDIUM -- reduces noise in terminal text capture |
| P3 | Add `Win32_System_Console` feature and `GetConsoleProcessList` | Low (Cargo.toml + 20 lines) | LOW -- supplementary signal, may not be needed |

---

## Installation / Changes

### Cargo.toml

```toml
# ONE feature addition to existing windows-sys entry:
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_System_Diagnostics_Debug",
    "Win32_Security",
    "Win32_UI_Input_KeyboardAndMouse",
    "Wdk_System_Threading",
    "Win32_System_Console",           # NEW
] }
```

### No new crates. No npm changes. No frontend changes.

The entire fix is Rust-side process detection logic improvements using existing APIs.

---

## Sources

- Current codebase analysis: `detect_windows.rs`, `process.rs`, `uia_reader.rs`, `Cargo.toml` -- HIGH confidence
- [RTL_USER_PROCESS_PARAMETERS (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-rtl_user_process_parameters) -- HIGH confidence, stable struct layout
- [GetProcessTimes (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes) -- HIGH confidence
- [GetConsoleProcessList (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist) -- HIGH confidence
- [NtQueryInformationProcess (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess) -- HIGH confidence
- [Windows Console APIs in windows-sys](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Console/index.html) -- HIGH confidence
- [VS Code Terminal Advanced docs](https://code.visualstudio.com/docs/terminal/advanced) -- HIGH confidence for ConPTY architecture
- [VS Code Terminal Appearance docs](https://code.visualstudio.com/docs/terminal/appearance) -- HIGH confidence for title format
- [@vscode/windows-process-tree (GitHub)](https://github.com/microsoft/vscode-windows-process-tree) -- MEDIUM confidence for understanding VS Code's own process tree approach
- [Windows Terminal ConPTY issue #5694](https://github.com/microsoft/terminal/issues/5694) -- MEDIUM confidence, confirms no tab-level PID API from Windows Terminal
- [windows-sys crate (crates.io)](https://crates.io/crates/windows-sys) -- latest is 0.61.2, staying on 0.59
- [sysinfo crate (docs.rs)](https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html) -- v0.38.4, evaluated and rejected
- [uiautomation crate (crates.io)](https://crates.io/crates/uiautomation) -- v0.24.1, current version sufficient

---
*Stack research for: CMD+K v0.2.8 Windows Terminal Detection Fix*
*Researched: 2026-03-11*
