# Feature Landscape: Windows Terminal Detection Fixes (v0.2.8)

**Domain:** Windows terminal/shell detection and WSL identification
**Researched:** 2026-03-11
**Focus:** Fixes for unreliable WSL detection and shell type differentiation in IDE terminals

## Table Stakes

Features that are broken or unreliable today. Fixing these is the entire point of v0.2.8.

| Feature | Why Expected | Complexity | Dependencies | Notes |
|---------|--------------|------------|--------------|-------|
| **ConPTY-aware shell discovery** | Windows Terminal shells are NOT descendants of WindowsTerminal.exe -- they're children of OpenConsole.exe. Current code has a fallback for this but it's fragile (picks highest PID among all ConPTY shells, not the focused tab's shell) | Med | `process.rs` find_shell_by_ancestry ConPTY fallback path | OpenConsole.exe is a locally-built conhost.exe used by WT to communicate with shells via ConPTY. Each tab spawns a separate OpenConsole.exe instance. |
| **cmd.exe false positive filtering in IDEs** | VS Code/Cursor spawn many internal cmd.exe processes for git, tasks, extensions. Current code deprioritizes cmd.exe but only when `descendant_shells.len() > 1` -- if all descendants are cmd.exe, it picks one anyway, which may be an internal process | Med | `process.rs` IDE cmd.exe filtering logic | Need to distinguish user-interactive cmd.exe (has a ConPTY/console allocated) from background cmd.exe (spawned for `git status`, extension host, etc.) |
| **WSL detection without UIA text** | UIA text reading was removed for VS Code because xterm.js doesn't expose text without screen reader mode. This eliminates the primary WSL detection mechanism (`detect_wsl_from_text`). Remaining signals: window title `[WSL:]` (only works in Remote-WSL mode), process ancestry (fails for WSL 2 Hyper-V), CWD path style | High | `mod.rs` detect_full_with_hwnd, `process.rs` detect_wsl_in_ancestry | WSL 2 runs Linux processes in a Hyper-V VM invisible to Windows process APIs. The wsl.exe process IS visible but it's not always in the shell's ancestry chain in IDE terminals |
| **Correct shell type for active terminal tab** | When VS Code has multiple terminal tabs (one PowerShell, one bash, one WSL), current code picks highest PID which may not be the focused tab | High | `process.rs` find_shell_by_ancestry, no CWD-based disambiguation on Windows (focused_cwd unused) | macOS solves this with AX-based focused tab CWD matching. Windows has no equivalent without UIA focused element support |
| **PowerShell vs pwsh differentiation** | Windows PowerShell (powershell.exe, v5.1) and PowerShell Core (pwsh.exe, v7+) have different capabilities and syntax. Both must be correctly identified | Low | `detect_windows.rs` KNOWN_SHELL_EXES already lists both | Already handled in exe lists. Just verify process tree correctly returns the right one |

## Differentiators

Features that go beyond fixing bugs to make detection notably better than current state.

| Feature | Value Proposition | Complexity | Dependencies | Notes |
|---------|-------------------|------------|--------------|-------|
| **Console API-based shell identification** | Use `GetConsoleProcessList` on a console handle to enumerate all processes attached to a specific console session. This directly answers "which shell owns this console?" without ancestry guessing | Med | New Win32 API calls, requires console handle from HWND | Windows Terminal attaches each tab to a separate console session. `GetConsoleProcessList` returns PIDs sharing that console, which includes the shell. More reliable than process tree walking for standalone terminals |
| **wsl.exe sibling detection** | Instead of checking ancestry (fails for WSL 2), check if wsl.exe is a sibling process of the detected shell under the same parent (OpenConsole.exe or conhost.exe). In ConPTY, wsl.exe is spawned by OpenConsole and the Linux init process runs inside the VM | Med | `process.rs` process snapshot, parent_map | Pattern: OpenConsole.exe -> wsl.exe -> (VM boundary) -> bash. The wsl.exe is visible even though bash inside the VM is not |
| **Environment block reading for shell detection** | Read the target process's environment block via PEB to check for WSL_DISTRO_NAME, WSLENV, or TERM_PROGRAM variables. These are definitive signals | Med-High | `process.rs` PEB reading (already have CWD reading via PEB), need to extend to environment block | RTL_USER_PROCESS_PARAMETERS.Environment is at offset 0x80 (64-bit). Same ReadProcessMemory pattern as CWD. HIGH confidence signal when available |
| **Focused pane detection via UIA tree position** | For Windows Terminal, walk the UIA tree to find the focused terminal control, then correlate its process. WT exposes each pane as a separate UIA element with TextPattern | Med-High | `uia_reader.rs`, Windows Terminal only (not IDEs) | Only works for Windows Terminal, not VS Code. WT's UIA tree structure: root -> TabView -> TabViewItem -> TermControl. The focused TermControl can be found via UIA focus tracking |
| **OSC sequence-based shell integration** | Modern shells emit OSC 133 (command start/end) and OSC 7 (CWD reporting) sequences. If the terminal emulator exposes these via its API or UIA properties, use them for shell identification and CWD | High | Would require terminal emulator cooperation or ConPTY output reading | Windows Terminal supports OSC 7/9;9/133. cmd.exe via Clink emits these. PowerShell 7+ emits them natively. Would be the gold standard but requires reading the ConPTY output stream, which is not accessible from an external process |

## Anti-Features

Features to explicitly NOT build for v0.2.8.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Screen reader mode injection for VS Code UIA text** | Was previously attempted (UIA focused element strategy). Requires enabling screen reader mode in VS Code which degrades performance and changes the UI. Removed in prior work | Use process tree + window title + wsl.exe sibling detection instead of UIA text for IDE terminals |
| **VS Code extension for terminal detection** | Dropped in project scope. Would give perfect shell/CWD info but adds installation friction and maintenance burden | Keep zero-setup approach. Use Win32 APIs and process inspection only |
| **Polling/watching terminal state** | Continuously monitoring process trees or UIA state would consume CPU and create race conditions | Snapshot-based detection at hotkey time (current approach). Single CreateToolhelp32Snapshot per detection cycle |
| **WMI/CIM queries for process information** | WMI is notoriously slow (100-500ms per query). wmic.exe is deprecated | Continue using CreateToolhelp32Snapshot (< 5ms) and direct Win32 API calls |
| **Windows Terminal settings.json parsing** | Could read default profile and tab configs but: file location varies, JSON schema changes between versions, doesn't tell you which tab is focused | Process tree + UIA is more reliable than config file parsing |
| **Named pipe or IPC to terminal emulators** | Would require each terminal emulator to implement a custom protocol | Not feasible for a standalone tool. Use publicly available Win32 APIs only |

## Feature Dependencies

```
ConPTY-aware shell discovery
  -> cmd.exe false positive filtering (needs accurate shell list per-tab)
  -> Correct shell type for active terminal tab (needs per-tab isolation)

WSL detection without UIA text
  -> wsl.exe sibling detection (new detection signal)
  -> Environment block reading (definitive WSL signal)
  -> Window title [WSL:] detection (existing, Remote-WSL only)
  -> CWD path style detection (existing fallback)

Focused pane detection via UIA
  -> Console API-based shell identification (need to know which console)
  -> Only applicable to Windows Terminal, not IDEs
```

## MVP Recommendation

### Priority 1: Fix broken detection (Table Stakes)

1. **wsl.exe sibling detection** - New WSL detection signal that works without UIA text. Check if wsl.exe is a child of the same parent as the detected shell process or any OpenConsole.exe in the tree. This is the single highest-impact fix because it restores WSL detection capability lost when UIA focused-element reading was removed.

2. **cmd.exe false positive filtering improvement** - Distinguish interactive cmd.exe (has console allocation, is child of ConPTY/conhost) from background cmd.exe (spawned transiently for git/tasks). Check if the cmd.exe process has a console window (GetConsoleWindow from the process, or check if it's a child of OpenConsole.exe/conhost.exe vs. a child of node.exe/Code Helper).

3. **ConPTY shell-to-tab correlation** - When multiple OpenConsole.exe instances exist (multi-tab Windows Terminal), the current code picks highest PID. Instead, try to correlate the focused terminal tab's OpenConsole.exe with its child shell. For Windows Terminal, UIA can identify the focused pane. For IDEs, the focused terminal tab identification remains unsolved without shell integration or UIA text.

### Priority 2: Improve accuracy (Differentiators)

4. **Environment block reading** - Extend existing PEB reading to extract environment variables. Check for WSL_DISTRO_NAME (definitive WSL), TERM_PROGRAM (shell context), SHELL (default shell). This is the most reliable programmatic signal available.

5. **Console API shell identification** - Use GetConsoleProcessList to enumerate processes on a console session. More direct than tree walking for standalone terminals.

### Defer

- **OSC sequence reading**: Requires ConPTY output stream access, which is not available to external processes. Would need a fundamentally different architecture.
- **Focused pane detection via UIA**: Complex, Windows Terminal only, diminishing returns given that WT's ConPTY fallback already works reasonably well for single-tab usage.

## Complexity Assessment

| Feature | LOC Estimate | Risk | Notes |
|---------|-------------|------|-------|
| wsl.exe sibling detection | 40-60 | Low | Uses existing snapshot infrastructure, just different traversal logic |
| cmd.exe filtering improvement | 30-50 | Low | Extend existing IDE filtering with parent-process checks |
| ConPTY tab correlation | 60-100 | Med | UIA tree walking for WT, still no solution for IDE multi-tab |
| Environment block reading | 80-120 | Med | PEB reading infrastructure exists for CWD, extend to env block parsing |
| Console API shell ID | 50-70 | Med | New API calls but well-documented Win32 functions |

## Sources

- [ConPTY Architecture - Microsoft DevBlogs](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/)
- [Console Host Architecture - DeepWiki](https://deepwiki.com/microsoft/terminal/2.5-console-host-architecture)
- [OpenConsole.exe Role - WindowsForum](https://windowsforum.com/threads/what-is-openconsole-exe-understanding-its-role-in-windows-terminal.358722/)
- [VS Code Terminal Shell Integration](https://code.visualstudio.com/docs/terminal/shell-integration)
- [VS Code windows-process-tree](https://github.com/microsoft/vscode-windows-process-tree)
- [Windows Terminal Process Detection Issue #14902](https://github.com/microsoft/terminal/issues/14902)
- [WSL Activity Tracking via API](https://www.hackthebox.com/blog/tracking-wsl-activity-with-api-hooking)
- [Creating a Pseudoconsole Session - Microsoft Learn](https://learn.microsoft.com/en-us/windows/console/creating-a-pseudoconsole-session)
- [ConEmu Terminal vs Shell](https://conemu.github.io/en/TerminalVsShell.html)
- [Windows Terminal Tab Title Setup](https://learn.microsoft.com/en-us/windows/terminal/tutorials/tab-title)
