# Project Research Summary

**Project:** CMD+K v0.2.8 -- Windows Terminal Detection Fix
**Domain:** Windows Win32/ConPTY process detection, UIA accessibility, IDE terminal integration
**Researched:** 2026-03-11
**Confidence:** HIGH

## Executive Summary

CMD+K v0.2.8 is a targeted bug-fix milestone addressing three interrelated failures in Windows terminal detection: (1) the wrong shell process is selected in multi-tab IDE terminals because the "highest PID" heuristic picks internal IDE processes over user shells, (2) WSL detection is structurally broken for WSL 2 because Linux processes are invisible to Windows process APIs, and (3) UIA text reading captures the entire VS Code window (editor, sidebar, menus) creating false positives for WSL and shell inference. These are not independent bugs -- they compound. Fixing one without the others yields partial results at best.

The recommended approach is a three-layer fix using exclusively existing dependencies (no new crates). Layer 1 replaces the "pick highest PID" heuristic with ConPTY-aware shell discovery: user terminal shells are always parented by conhost.exe/OpenConsole.exe, while internal IDE processes are not. This single insight eliminates cmd.exe false positives and wsl.exe pollution without fragile exe-name filtering. Layer 2 scopes UIA text reading to the terminal panel element rather than walking the entire VS Code window tree. Layer 3 adds active-tab identification via window title parsing and CWD-based disambiguation (already proven on macOS, currently disabled on Windows).

The primary risk is regression in working detection paths. Four terminal host types (Windows Terminal, VS Code, Cursor, standalone shells) each have distinct process architectures. IDE-specific fixes must be gated behind `is_ide_with_terminal_exe` checks to avoid breaking standalone and Windows Terminal detection. The project's own history shows this risk is real: a previous WSL fix (adding wsl.exe to shell lists) was reverted because it broke all VS Code detection. Every change must be validated against a 5x4 test matrix (5 hosts x 4 shell types).

## Key Findings

### Recommended Stack

No new crates are needed. The entire fix uses APIs already available through `windows-sys 0.59` with one minor Cargo.toml feature addition (`Win32_System_Console`). The codebase's PEB reading infrastructure (`read_cwd_from_peb()`) already navigates `RTL_USER_PROCESS_PARAMETERS` via `NtQueryInformationProcess` + `ReadProcessMemory`. Adding command-line reading is a ~20-line extension to the same struct at a different offset. `GetProcessTimes` is already available through the existing `Win32_System_Threading` feature. The `uiautomation` crate at v0.24 has all filtering capabilities (ControlType, ClassName, AutomationId, TextPattern) needed for scoped tree walking.

**Core technologies:**
- `windows-sys 0.59` + `Win32_System_Console` feature: PEB command-line reading and console process enumeration -- already in the codebase, minimal addition
- `GetProcessTimes` (existing): Replace PID-based shell selection with creation-time ordering -- more reliable than PID recycling
- `uiautomation 0.24` (existing): Scoped UIA tree walking with condition filtering -- no version change needed

Explicitly rejected: `sysinfo` crate (5MB+ dependency for 40 lines of Win32 code), WMI queries (100ms+ latency), `windows` high-level crate (binary size, architectural mismatch), VS Code extension approach (violates zero-setup constraint).

### Expected Features

**Must have (table stakes -- currently broken):**
- ConPTY-aware shell discovery: distinguish user terminal shells from internal IDE processes by conhost.exe parentage
- cmd.exe false positive filtering: use PEB command-line reading to filter `/C` and `/D /C` batch-execution flags
- WSL detection without UIA text: wsl.exe sibling detection as new signal that works without screen reader mode
- Correct shell type for active terminal tab: replace highest-PID heuristic with CWD-matching or creation-time ordering

**Should have (differentiators):**
- Console API-based shell identification via `GetConsoleProcessList` -- supplementary disambiguation signal
- wsl.exe sibling detection under shared ConPTY parent -- works for both WSL 1 and WSL 2
- Environment block reading from PEB for definitive WSL_DISTRO_NAME detection

**Defer:**
- OSC sequence reading (requires ConPTY output stream access, fundamentally different architecture)
- Focused pane detection via UIA for Windows Terminal (complex, diminishing returns)
- VS Code extension for terminal detection (architectural decision: zero-setup)

### Architecture Approach

The fix introduces a `ProcessSnapshot` struct that captures a single `CreateToolhelp32Snapshot` and serves all queries (ConPTY shell finding, WSL ancestry, sub-shell filtering), replacing the current 3+ snapshots per detection cycle. A new `process_windows.rs` file implements ConPTY-aware shell discovery. The detection pipeline follows a signal hierarchy pattern: ConPTY shell exe name (most reliable) -> window title patterns -> scoped UIA text -> unscoped UIA text (least reliable). Each signal can SET `is_wsl=true` but never CLEAR it, preventing the circular dependency that broke WSL detection previously.

**Major components:**
1. `ProcessSnapshot` (new struct in `process_windows.rs`) -- single snapshot, multiple queries; ConPTY shell discovery via conhost.exe parentage
2. `read_ide_terminal_text()` (new function in `uia_reader.rs`) -- scoped UIA reading targeting terminal panel only, not entire VS Code window
3. Active tab matching (extension to `detect_windows.rs`) -- window title parsing + CWD-based disambiguation enabling the unused `focused_cwd` parameter on Windows
4. WSL signal waterfall (modifications to `mod.rs`) -- unconditional multi-signal detection: title -> ConPTY exe -> UIA text -> CWD path

### Critical Pitfalls

1. **wsl.exe as a shell process** -- Never add wsl.exe to KNOWN_SHELL_EXES. VS Code spawns 10-16+ internal wsl.exe processes. Already caused a revert cycle (commits e0ef4be -> bcea9cd). WSL detection must use output-based signals only.
2. **Circular dependency in WSL detection** -- UIA text WSL detection must run UNCONDITIONALLY, not gated behind `if is_wsl`. The prior code structure made text detection depend on process detection which structurally fails for WSL 2. Already diagnosed and partially fixed; must not be reintroduced during refactoring.
3. **Highest PID picks internal IDE processes** -- PID order does not equal spawn recency (PIDs recycle), and internal processes spawn continuously. Filter by ConPTY parentage BEFORE any PID-based selection.
4. **WSL 2 Hyper-V invisibility** -- Linux processes in WSL 2 are invisible to `CreateToolhelp32Snapshot`. Process-tree-based WSL detection is architecturally impossible for WSL 2. Only output-based signals work.
5. **UIA focused element fails for Electron** -- Focus shifts to CMD+K overlay before UIA runs. Use `element_from_handle(hwnd)` with pre-captured HWND, not `focused_element()`. xterm.js accessibility tree may be empty without screen reader mode.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Process Snapshot Consolidation and ConPTY Shell Discovery

**Rationale:** This is the foundation. Every subsequent phase depends on having a reliable list of user-interactive shells separated from internal IDE processes. The ConPTY parentage insight is the single highest-leverage change.
**Delivers:** `ProcessSnapshot` struct with single-snapshot infrastructure; `find_conpty_shells()` returning only conhost.exe-parented shells; PEB command-line reading for cmd.exe `/C` flag filtering; `GetProcessTimes` for creation-time ordering.
**Addresses:** ConPTY-aware shell discovery (table stakes), cmd.exe false positive filtering (table stakes).
**Avoids:** Pitfall 3 (highest PID picks internal process), Pitfall 6 (cmd.exe pollution), Pitfall 12 (multiple snapshots).
**Effort:** ~150 LOC new file + ~50 LOC refactoring existing `find_shell_by_ancestry()`.

### Phase 2: UIA Terminal Text Scoping

**Rationale:** Independent of Phase 1 (can run in parallel). Reduces false positives from IDE chrome in UIA text, which directly improves WSL detection accuracy and shell inference in subsequent phases.
**Delivers:** `read_ide_terminal_text()` function using focused-element ancestry and ControlType filtering to scope UIA reads to terminal panel only.
**Addresses:** Scoped UIA tree walk (differentiator), reduces false WSL detection from editor content.
**Avoids:** Pitfall 8 (noisy UIA text), Pitfall 11 (false positives from editor content), Pitfall 5 (focused element timing -- use HWND-based rooting instead).
**Effort:** ~50-80 LOC, needs empirical testing of VS Code UIA tree structure.

### Phase 3: Active Tab Matching

**Rationale:** Depends on Phase 1 (needs ConPTY shell list to match against) and benefits from Phase 2 (scoped UIA for CWD extraction). Solves the multi-tab problem by enabling CWD-based disambiguation that already works on macOS.
**Delivers:** Window title parsing for terminal tab hints; CWD-based shell matching (wiring up the unused `focused_cwd` parameter on Windows); CWD extraction from scoped UIA terminal text.
**Addresses:** Correct shell type for active terminal tab (table stakes).
**Avoids:** Pitfall 3 (wrong tab selected), Pitfall 14 (timeout budget -- title parsing is < 1ms).
**Effort:** ~60-100 LOC across `detect_windows.rs` and `process.rs`.

### Phase 4: WSL Detection Hardening

**Rationale:** Benefits from all prior phases. ConPTY discovery (Phase 1) provides `is_wsl` flag when shell exe is wsl.exe. Scoped UIA (Phase 2) eliminates false positives from editor content. Active tab matching (Phase 3) ensures the correct shell is being evaluated.
**Delivers:** Reliable WSL detection across all scenarios: Remote-WSL, local WSL terminal profiles, Windows Terminal WSL tabs. wsl.exe sibling detection under shared ConPTY parent. Fixed `get_wsl_cwd()` preferring UIA-inferred CWD over subprocess home directory.
**Addresses:** WSL detection without UIA text (table stakes), wsl.exe sibling detection (differentiator).
**Avoids:** Pitfall 1 (wsl.exe as shell), Pitfall 2 (circular dependency), Pitfall 4 (WSL 2 invisibility), Pitfall 9 (title-only detection), Pitfall 10 (subprocess returns home dir).
**Effort:** ~80-120 LOC modifications to WSL detection pipeline in `mod.rs`.

### Phase Ordering Rationale

- **Phase 1 first** because it produces the `ProcessSnapshot` infrastructure and ConPTY shell list that Phases 3 and 4 consume. Without accurate shell candidates, tab matching and WSL detection operate on garbage data.
- **Phase 2 parallel with Phase 1** because it is architecturally independent (modifies UIA reader, not process detection). Both can be developed and tested separately.
- **Phase 3 after Phase 1** because it needs the ConPTY shell list to match CWD/title against. Phase 2 improves its CWD extraction quality but is not a hard dependency.
- **Phase 4 last** because it is the beneficiary of all prior improvements. With clean shell candidates (Phase 1), scoped text (Phase 2), and correct tab identification (Phase 3), WSL detection becomes primarily a matter of wiring signals correctly.
- **Each phase delivers standalone value** -- after Phase 1 alone, internal IDE processes no longer pollute results, which is the most impactful single fix.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2 (UIA Scoping):** VS Code's UIA tree structure (ControlType, ClassName, AutomationId of terminal panel elements) must be empirically verified on a real Windows machine. Confidence is MEDIUM for specific element targeting.
- **Phase 3 (Active Tab Matching):** CWD extraction from scoped UIA text depends on prompt format parsing, which varies across shell configurations. The macOS CWD-matching code provides a reference but Windows implementation needs validation.

Phases with standard patterns (skip research-phase):
- **Phase 1 (ConPTY Discovery):** ConPTY architecture is well-documented by Microsoft. PEB reading code already exists. `GetProcessTimes` is a trivial Win32 call. HIGH confidence.
- **Phase 4 (WSL Hardening):** All signals are documented from prior debugging. The project's own failure history (commits, debug docs) provides definitive guidance. HIGH confidence on what NOT to do.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | No new dependencies. All APIs verified against existing codebase and Microsoft docs. PEB struct layout is stable. |
| Features | HIGH | Features derived from diagnosed bugs with clear root causes. Table stakes are broken-today items with known fixes. |
| Architecture | HIGH (process), MEDIUM (UIA) | ConPTY parentage model is well-documented. UIA tree structure for Electron apps needs empirical verification. |
| Pitfalls | HIGH | 5 critical pitfalls drawn from project's own revert history and debug documentation. Battle-tested knowledge. |

**Overall confidence:** HIGH

### Gaps to Address

- **VS Code UIA tree structure for terminal panel:** Specific ControlType, ClassName, and AutomationId values for xterm.js terminal elements need empirical testing. No public documentation exists for Electron app UIA trees at this granularity. Mitigation: Phase 2 should start with UIA tree dumping/exploration before implementing filters.
- **Multi-distro WSL CWD:** When multiple WSL distros are installed, `get_wsl_cwd()` queries the default distro. Extracting the correct distro name from terminal context (to call `wsl.exe -d <distro> -e pwd`) is possible via window title or UIA text but not yet validated. Mitigation: defer to Phase 4, use UIA-inferred CWD as primary source.
- **Cursor IDE process tree:** Research focused on VS Code. Cursor (a VS Code fork) likely has the same ConPTY/pty-host architecture but this is assumed, not verified. Mitigation: test Cursor during Phase 1 validation alongside VS Code.
- **Windows 10 vs Windows 11 ConPTY differences:** ConPTY behavior may differ slightly between Windows 10 (older conhost.exe) and Windows 11 (newer OpenConsole.exe). Both should be tested. Mitigation: include OS version in test matrix.

## Sources

### Primary (HIGH confidence)
- Project codebase: `process.rs`, `detect_windows.rs`, `uia_reader.rs`, `mod.rs` -- current implementation analysis
- Project debug docs: `.planning/debug/wsl-detection-failure.md` -- root cause analysis
- Project commit history: bcea9cd, e0ef4be, 1e4cd7a, d63f909 -- revert/fix documentation
- [RTL_USER_PROCESS_PARAMETERS (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-rtl_user_process_parameters) -- PEB struct layout
- [GetProcessTimes (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)
- [GetConsoleProcessList (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/console/getconsoleprocesslist)
- [ConPTY Architecture (Microsoft DevBlogs)](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/)
- [Windows Console Ecosystem Roadmap (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/console/ecosystem-roadmap)

### Secondary (MEDIUM confidence)
- [VS Code Terminal Advanced docs](https://code.visualstudio.com/docs/terminal/advanced) -- ConPTY integration
- [VS Code windows-process-tree (GitHub)](https://github.com/microsoft/vscode-windows-process-tree) -- VS Code's own process tree approach
- [microsoft/node-pty (GitHub)](https://github.com/microsoft/node-pty) -- VS Code pty backend
- [OpenConsole.exe discussion (GitHub)](https://github.com/microsoft/terminal/discussions/12115) -- process tree structure
- [Windows Terminal Tab Title (Microsoft Learn)](https://learn.microsoft.com/en-us/windows/terminal/tutorials/tab-title)

### Tertiary (LOW confidence)
- VS Code Electron UIA tree structure -- inferred from Chromium accessibility bridge behavior, needs empirical validation
- Cursor IDE process architecture -- assumed identical to VS Code fork, not independently verified

---
*Research completed: 2026-03-11*
*Ready for roadmap: yes*
