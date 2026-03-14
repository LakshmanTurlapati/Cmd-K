---
phase: 27-conpty-discovery-process-snapshot
verified: 2026-03-11T16:00:00Z
status: passed
score: 5/5 must-haves verified
re_verification:
  previous_status: passed
  previous_score: 5/5
  gaps_closed: []
  gaps_remaining: []
  regressions: []
---

# Phase 27: ConPTY Discovery & Process Snapshot Verification Report

**Phase Goal:** User's active shell correctly identified in multi-tab IDE terminals without false positives from internal IDE processes
**Verified:** 2026-03-11T16:00:00Z
**Status:** passed
**Re-verification:** Yes -- confirming previous passed status

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | In VS Code with multiple terminal tabs, the correct interactive shell is selected via ConPTY parentage -- not highest PID | VERIFIED | `find_shell_by_ancestry` (line 908) implements 3-tier priority: ConPTY descendant shells (line 962), direct descendant fallback (line 979), all ConPTY shells fallback (line 996). `pick_most_recent` (line 876) uses `GetProcessTimes` for recency sorting with PID fallback. |
| 2 | cmd.exe users in IDE terminals are supported -- only cmd.exe with /C flags filtered out | VERIFIED | `filter_cmd` closure (line 926) calls `is_interactive_cmd()` per candidate. `is_interactive_cmd` (line 1453) uses two-signal approach: fast ConPTY parent check + PEB command line analysis via `read_command_line_from_peb` (line 1371, offset 0x70). Conservative fallback to interactive on PEB failure. |
| 3 | A single ProcessSnapshot is created per hotkey press and shared across detection functions | VERIFIED | `detect_app_context_windows` (mod.rs line 478) calls `ProcessSnapshot::capture()` once, passes `snapshot.as_ref()` to `get_foreground_info` (line 482). Only two `CreateToolhelp32Snapshot` calls: `ProcessSnapshot::capture()` (process.rs line 1306) and `get_child_pids_windows` (line 561, retained for cross-platform `find_shell_recursive` fast path). |
| 4 | For standalone terminals without ConPTY, detection falls back to direct descendant walk | VERIFIED | Priority 2 block (lines 979-991) walks `shell_candidates` checking `is_descendant()` against `app_pid`. Fires when ConPTY set is empty or no ConPTY descendants found. |
| 5 | macOS code paths are completely untouched -- all changes are cfg(target_os = "windows") gated | VERIFIED | macOS `find_shell_by_ancestry` (line 640) uses pgrep-based approach. Non-Windows `find_shell_pid` (line 622) takes `_snapshot: Option<&()>` as unused parameter. All new code (ProcessSnapshot, is_interactive_cmd, read_command_line_from_peb, Windows find_shell_by_ancestry) is `#[cfg(target_os = "windows")]` gated. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/terminal/process.rs` | ProcessSnapshot struct, capture(), read_command_line_from_peb(), extract_cmd_args(), has_batch_flag_in_cmdline(), is_interactive_cmd(), find_shell_by_ancestry with snapshot, detect_wsl_in_ancestry with snapshot, scan_wsl_processes_diagnostic with snapshot, get_process_creation_time, pick_most_recent | VERIFIED | All functions present. ProcessSnapshot struct at line 1288 with 4 fields. capture() at line 1298. read_command_line_from_peb at line 1371 reading PEB offset 0x70. 14 unit tests at lines 1505-1597. |
| `src-tauri/src/terminal/mod.rs` | ProcessSnapshot threading through detect_app_context_windows | VERIFIED | Line 478: `let snapshot = process::ProcessSnapshot::capture();` Line 482: passed to `get_foreground_info`. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| detect_app_context_windows (mod.rs) | ProcessSnapshot::capture() | creates snapshot once, passes to get_foreground_info | WIRED | mod.rs line 478 creates, line 482 passes `snapshot.as_ref()` |
| find_shell_by_ancestry | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 908: `fn find_shell_by_ancestry(app_pid: i32, _focused_cwd: Option<&str>, snapshot: &ProcessSnapshot, shell_type_hint: Option<&str>)` |
| find_shell_by_ancestry | is_interactive_cmd | filters cmd.exe candidates | WIRED | Line 930: `let interactive = is_interactive_cmd(*pid, snapshot);` inside `filter_cmd` closure |
| detect_wsl_in_ancestry | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 1122: `fn detect_wsl_in_ancestry(pid: i32, snapshot: &ProcessSnapshot) -> bool` |
| scan_wsl_processes_diagnostic | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 1094: `fn scan_wsl_processes_diagnostic(_shell_pid: i32, snapshot: &ProcessSnapshot)` |
| ConPTY shell selection | GetProcessTimes | sorts by creation time for recency | WIRED | `pick_most_recent` (line 876) calls `get_process_creation_time` (line 847) which uses `GetProcessTimes` |
| read_command_line_from_peb | PEB offset 0x70 | reads CommandLine UNICODE_STRING | WIRED | Line 1413: `let cmdline_unicode_string_addr = params_ptr + 0x70;` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| PROC-01 | 27-02 | User's active shell correctly identified via ConPTY parentage instead of highest-PID heuristic | SATISFIED | 3-tier ConPTY-first priority in find_shell_by_ancestry (lines 962-1011), GetProcessTimes-based recency sorting in pick_most_recent (line 876) |
| PROC-02 | 27-01, 27-02 | Internal IDE cmd.exe filtered out -- only interactive console-attached cmd.exe selected | SATISFIED | is_interactive_cmd (line 1453) with two-signal approach, integrated into find_shell_by_ancestry via filter_cmd closure (line 926) |
| PROC-03 | 27-01, 27-02 | Process snapshot consolidated into single CreateToolhelp32Snapshot call shared across detection | SATISFIED | ProcessSnapshot::capture() (line 1298) builds all maps from one call. detect_app_context_windows creates it once (mod.rs line 478), threads through get_foreground_info to all detection functions. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | - | - | - | No TODOs, FIXMEs, placeholders, or empty implementations in modified files |

### Human Verification Required

### 1. Multi-tab IDE shell selection accuracy

**Test:** Open VS Code/Cursor with 3+ terminal tabs (PowerShell, cmd.exe, bash). Activate each tab and press CMD+K. Check detected shell_type matches the active tab.
**Expected:** Each tab's shell is correctly identified. Internal IDE git/extension cmd.exe processes are not selected.
**Why human:** Requires real Windows IDE environment with ConPTY architecture to verify end-to-end selection.

### 2. Standalone terminal fallback

**Test:** Open Windows Terminal or plain cmd.exe window. Press CMD+K.
**Expected:** Shell detected via direct descendant walk (Priority 2 path).
**Why human:** Need real standalone terminal to verify ConPTY fallback path works correctly.

### 3. cmd.exe as interactive shell

**Test:** Open a terminal tab running cmd.exe (not launched with /C flag). Press CMD+K.
**Expected:** shell_type = "cmd", not filtered out.
**Why human:** Requires real cmd.exe session to verify PEB command line reading returns no batch flags.

### Gaps Summary

No gaps found. All 5 observable truths verified, all artifacts substantive and wired at all 3 levels, all 7 key links connected, all 3 requirements (PROC-01, PROC-02, PROC-03) satisfied. No regressions from previous verification. The implementation follows the planned architecture: single ProcessSnapshot per hotkey press, ConPTY-first shell selection with GetProcessTimes recency, two-signal cmd.exe filtering, and macOS code completely untouched.

The `get_child_pids_windows` retaining its own `CreateToolhelp32Snapshot` call (line 561) is an acceptable documented decision -- that function serves the cross-platform `find_shell_recursive` fast path and cannot accept a Windows-only snapshot parameter.

---

_Verified: 2026-03-11T16:00:00Z_
_Verifier: Claude (gsd-verifier)_
