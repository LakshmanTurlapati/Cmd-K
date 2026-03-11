---
phase: 27-conpty-discovery-process-snapshot
verified: 2026-03-11T14:30:00Z
status: passed
score: 5/5 must-haves verified
re_verification: false
---

# Phase 27: ConPTY Discovery & Process Snapshot Verification Report

**Phase Goal:** User's active shell correctly identified in multi-tab IDE terminals without false positives from internal IDE processes
**Verified:** 2026-03-11T14:30:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | In VS Code with multiple terminal tabs, the correct interactive shell is selected via ConPTY parentage -- not highest PID | VERIFIED | `find_shell_by_ancestry` (line 908) implements 3-tier ConPTY-first priority. `pick_most_recent` (line 876) uses `GetProcessTimes` for recency sorting instead of highest-PID. ConPTY descendant shells checked first (line 944), then direct descendants (line 962), then all ConPTY shells fallback (line 978). |
| 2 | cmd.exe users in IDE terminals are supported -- only cmd.exe with /C flags filtered out | VERIFIED | `filter_cmd` closure (line 926) calls `is_interactive_cmd()` per candidate instead of blanket exclusion. `is_interactive_cmd` (line 1433) uses two-signal approach: fast ConPTY parent check + PEB command line analysis. Conservative fallback to interactive on PEB failure. |
| 3 | A single ProcessSnapshot is created per hotkey press and shared across detection functions | VERIFIED | `detect_app_context_windows` in mod.rs (line 462) calls `ProcessSnapshot::capture()` once and passes `snapshot.as_ref()` to `get_foreground_info` (line 466). `get_foreground_info` threads it to `find_shell_pid`, `detect_wsl_in_ancestry`, and `scan_wsl_processes_diagnostic`. Only two `CreateToolhelp32Snapshot` calls in codebase: `ProcessSnapshot::capture()` (line 1286) and `get_child_pids_windows` (line 561, retained for cross-platform `find_shell_recursive` fast path -- documented decision). |
| 4 | For standalone terminals without ConPTY, detection falls back to direct descendant walk | VERIFIED | Priority 2 block (line 961-973) walks all `shell_candidates` checking `is_descendant()` against `app_pid`. If ConPTY set is empty or no ConPTY descendants found, this path is taken. |
| 5 | macOS code paths are completely untouched -- all changes are cfg(target_os = "windows") gated | VERIFIED | macOS `find_shell_by_ancestry` (line 640) still uses pgrep-based approach. macOS `get_foreground_info` (line 184+) has no snapshot parameter. macOS `detect_app_context_macos` (line 365) unchanged. All new code (`ProcessSnapshot`, `is_interactive_cmd`, `read_command_line_from_peb`, Windows `find_shell_by_ancestry`) is `#[cfg(target_os = "windows")]` gated. |

**Score:** 5/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/terminal/process.rs` | ProcessSnapshot struct, capture(), read_command_line_from_peb(), extract_cmd_args(), has_batch_flag_in_cmdline(), check_batch_flags(), is_interactive_cmd(), find_shell_by_ancestry with snapshot, detect_wsl_in_ancestry with snapshot, scan_wsl_processes_diagnostic with snapshot, get_process_creation_time, pick_most_recent | VERIFIED | All functions present at expected lines. ProcessSnapshot struct at line 1268 with 4 fields. capture() at line 1278 with single CreateToolhelp32Snapshot. read_command_line_from_peb at line 1351 reading offset 0x70. 14 unit tests at line 1485-1562. |
| `src-tauri/src/terminal/mod.rs` | ProcessSnapshot threading through detect_app_context_windows | VERIFIED | Line 462: `let snapshot = process::ProcessSnapshot::capture();` Line 466: passed to `get_foreground_info`. `detect_inner_windows` passes `None` (line 289) as expected for the non-full-context path. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| detect_app_context_windows (mod.rs) | ProcessSnapshot::capture() | creates snapshot once, passes to get_foreground_info | WIRED | mod.rs line 462 creates snapshot, line 466 passes `snapshot.as_ref()` |
| find_shell_by_ancestry | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 908: `fn find_shell_by_ancestry(app_pid: i32, _focused_cwd: Option<&str>, snapshot: &ProcessSnapshot)` |
| find_shell_by_ancestry | is_interactive_cmd | filters cmd.exe candidates | WIRED | Line 930: `let interactive = is_interactive_cmd(*pid, snapshot);` inside `filter_cmd` closure |
| detect_wsl_in_ancestry | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 1104: `fn detect_wsl_in_ancestry(pid: i32, snapshot: &ProcessSnapshot)` |
| scan_wsl_processes_diagnostic | ProcessSnapshot | receives snapshot as parameter | WIRED | Line 1076: `fn scan_wsl_processes_diagnostic(_shell_pid: i32, snapshot: &ProcessSnapshot)` |
| ConPTY shell selection | GetProcessTimes | sorts by creation time for recency | WIRED | `pick_most_recent` (line 876) calls `get_process_creation_time` (line 847) which uses `GetProcessTimes`. Falls back to PID comparison. |
| read_command_line_from_peb | PEB offset 0x70 | reads CommandLine UNICODE_STRING | WIRED | Line 1393: `let cmdline_unicode_string_addr = params_ptr + 0x70;` |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| PROC-01 | 27-02 | User's active shell correctly identified via ConPTY parentage instead of highest-PID heuristic | SATISFIED | 3-tier ConPTY-first priority in find_shell_by_ancestry (lines 944-993), GetProcessTimes-based recency sorting in pick_most_recent (line 876) |
| PROC-02 | 27-01, 27-02 | Internal IDE cmd.exe filtered out -- only interactive console-attached cmd.exe selected | SATISFIED | is_interactive_cmd (line 1433) with two-signal approach, integrated into find_shell_by_ancestry via filter_cmd closure (line 926) |
| PROC-03 | 27-01, 27-02 | Process snapshot consolidated into single CreateToolhelp32Snapshot call shared across detection | SATISFIED | ProcessSnapshot::capture() (line 1278) builds all maps from one call. detect_app_context_windows creates it once (mod.rs line 462), threads through get_foreground_info to all detection functions. Only exception: get_child_pids_windows retains own snapshot for find_shell_recursive fast path (documented decision). |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | - | - | - | No TODOs, FIXMEs, placeholders, or empty implementations detected in modified files |

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

No gaps found. All observable truths verified, all artifacts substantive and wired, all key links connected, all three requirements (PROC-01, PROC-02, PROC-03) satisfied. The implementation follows the planned architecture: single ProcessSnapshot created per hotkey press, ConPTY-first shell selection with GetProcessTimes recency, two-signal cmd.exe filtering, and macOS code completely untouched.

The one noted architectural decision is that `get_child_pids_windows` retains its own `CreateToolhelp32Snapshot` call for the `find_shell_recursive` fast path, which is acceptable since that function serves the cross-platform recursive child walk and cannot accept a Windows-only snapshot parameter.

---

_Verified: 2026-03-11T14:30:00Z_
_Verifier: Claude (gsd-verifier)_
