# Architecture Patterns

**Domain:** Windows terminal detection fixes -- WSL, shell differentiation, IDE process filtering
**Researched:** 2026-03-11
**Confidence:** HIGH (process tree architecture, ConPTY model), MEDIUM (VS Code pty-host internals, UIA text filtering)

---

## Current Architecture Snapshot (Detection Path)

```
Hotkey fires (hotkey.rs)
  |-- Captures HWND + PID of foreground window BEFORE overlay steals focus
  |
  v
detect_full_with_hwnd(pid, pre_captured_text, hwnd)  [mod.rs:108]
  |-- Spawns background thread with 750ms timeout
  |
  v
detect_app_context_windows(pid, pre_captured_text)  [mod.rs:453]
  |-- get_exe_name_for_pid(pid) -> e.g. "Code.exe"
  |-- is_known_terminal_exe() / is_ide_with_terminal_exe()
  |-- process::get_foreground_info(pid)
  |     |-- find_shell_pid(pid, None)
  |     |     |-- find_shell_recursive(pid, 3)  -- fast path, depth-3 child walk
  |     |     |-- find_shell_by_ancestry(pid, None)  -- broad path
  |     |           |-- CreateToolhelp32Snapshot -> build parent_map + shell_candidates
  |     |           |-- Walk ancestry of each shell candidate to app_pid
  |     |           |-- IDE mode: filter out cmd.exe, prefer non-cmd shells
  |     |           |-- ConPTY fallback: find shells parented by OpenConsole.exe
  |     |           |-- Pick highest PID among candidates  <-- PROBLEM
  |     |-- get_process_cwd(shell_pid) via PEB ReadProcessMemory
  |     |-- detect_wsl_in_ancestry(shell_pid)
  |
  v  (back in detect_full_with_hwnd)
Window title WSL detection: "[WSL: Ubuntu]"  [mod.rs:126]
UIA text reading: read_terminal_text_windows(hwnd)  [mod.rs:143]
UIA-based WSL detection: detect_wsl_from_text()  [mod.rs:152]
UIA-based shell inference: infer_shell_from_text()  [mod.rs:165]
CWD-based WSL detection: detect_wsl_from_cwd()  [mod.rs:181]
```

---

## Problem Analysis: Three Distinct Failure Modes

### Problem 1: "Highest PID = Active" Heuristic Fails

**Current code (process.rs:886):**
```rust
// Pick the most recently spawned (highest PID) shell
descendant_shells.iter()
    .max_by_key(|(pid, _)| *pid)
    .map(|(pid, _)| *pid as i32)
```

**Why it breaks:** VS Code spawns shells for each terminal tab. User opens Tab A (powershell, PID 5000), then Tab B (powershell, PID 5500), then switches BACK to Tab A. Highest PID picks Tab B (PID 5500), but Tab A (PID 5000) is focused. On macOS, this is solved by AX-based focused CWD matching (process.rs:656-678). On Windows, `_focused_cwd` is always `None` -- the CWD matching path is never taken.

**Impact:** Wrong terminal context in multi-tab VS Code/Cursor. Wrong CWD, wrong history, wrong command suggestions.

### Problem 2: Internal IDE Processes Pollute Shell Candidates

**VS Code process tree on Windows (observed):**
```
Code.exe (main)
  +-- Code.exe (GPU process)
  +-- Code.exe (extension host)
  |     +-- cmd.exe (git operations)        <-- INTERNAL, not user shell
  |     +-- cmd.exe (task runner)            <-- INTERNAL, not user shell
  |     +-- wsl.exe (remote extension)      <-- INTERNAL, not user shell
  |     +-- node.exe (language server)
  +-- Code.exe (pty-host)
  |     +-- conhost.exe
  |     |     +-- powershell.exe (Tab 1)    <-- USER SHELL
  |     +-- conhost.exe
  |     |     +-- bash.exe (Tab 2, WSL)     <-- USER SHELL
  |     +-- conhost.exe
  |           +-- cmd.exe (Tab 3)           <-- USER SHELL
  +-- cmd.exe (file watcher)                <-- INTERNAL
  +-- wsl.exe (remote server bootstrap)     <-- INTERNAL
```

**Current mitigation (process.rs:871-889):** Only filters `cmd.exe` when other non-cmd shells exist. This is insufficient:
- If the user's only terminal tab IS cmd.exe, filtering removes the correct result
- Internal `wsl.exe` processes are not filtered at all (5-16+ of them)
- No distinction between `cmd.exe` spawned by pty-host vs extension host

### Problem 3: WSL Detection Unreliable in IDE Terminals

**Failure chain:**
1. `detect_wsl_in_ancestry()` walks the parent chain of the detected shell PID. But if the wrong shell is selected (Problem 1), ancestry check runs on wrong process.
2. WSL 2 shells run in Hyper-V VM -- `bash.exe` inside WSL is invisible to Win32 process tree. Only `wsl.exe` launcher is visible.
3. UIA text for VS Code captures entire window content (sidebar, editor, terminal panel) via `try_walk_children()`. Linux paths like `/home/` in editor text trigger false WSL positives.
4. `get_wsl_cwd()` spawns `wsl.exe -e sh -c "pwd"` -- returns the HOME directory of the default distro, not the CWD of the active terminal session.

---

## Recommended Architecture: Three-Layer Fix

### Layer 1: ConPTY-Aware Shell Discovery (New: `process_windows.rs`)

**Principle:** On Windows, user-interactive terminal shells are ALWAYS spawned through ConPTY. The ConPTY host process (conhost.exe or OpenConsole.exe) is the reliable marker. Shells spawned by IDE extensions, git operations, and task runners do NOT go through ConPTY -- they use direct `CreateProcess`.

**ConPTY process model (verified):**
```
Hosting App (WindowsTerminal.exe / Code.exe pty-host)
  calls CreatePseudoConsole() -> spawns conhost.exe
    conhost.exe manages the pseudoconsole session
      shell process (powershell.exe, bash.exe, etc.) is child of conhost.exe
```

**Key insight:** In VS Code, the `pty-host` helper process (a Code.exe child) is the one that creates ConPTY sessions. Its children are conhost.exe instances, each hosting one terminal tab's shell. Extension-spawned cmd.exe/wsl.exe are children of the extension host process, NOT children of pty-host.

**New detection strategy:**

```rust
// terminal/process_windows.rs (NEW FILE)

/// Identify user-interactive shells by finding those hosted by ConPTY.
///
/// Strategy:
/// 1. Take process snapshot
/// 2. Find all conhost.exe instances that are descendants of app_pid
/// 3. For each conhost.exe, find its shell child
/// 4. These are the user's terminal tab shells
/// 5. Non-conhost-parented shells are internal IDE processes
///
/// This replaces the "all descendant shells, pick highest PID" approach.
pub fn find_conpty_shells(app_pid: u32) -> Vec<ConPtyShell> {
    // Single CreateToolhelp32Snapshot call
    // Build: parent_map, exe_map
    // Find conhost.exe/OpenConsole.exe PIDs that are descendants of app_pid
    // For each conhost/openconsole:
    //   Find shell child (direct child matching KNOWN_SHELL_EXES or wsl.exe)
    //   Record: shell_pid, shell_exe, conhost_pid
    // Return list of ConPtyShell structs
}

pub struct ConPtyShell {
    pub shell_pid: u32,
    pub shell_exe: String,
    pub conhost_pid: u32,
    /// True if shell_exe is "wsl.exe" (WSL session)
    pub is_wsl: bool,
}
```

**Why this works:**
- IDE-internal cmd.exe (git, tasks): spawned by extension host, NOT through ConPTY, NOT under a conhost.exe child of pty-host. Automatically excluded.
- IDE-internal wsl.exe (remote server): spawned by extension host, NOT through ConPTY. Automatically excluded.
- User terminal tabs: ALL go through ConPTY -> conhost.exe -> shell. Correctly included.
- Windows Terminal: Same ConPTY architecture. Each tab has its own conhost.exe -> shell chain.

**Integration point -- `find_shell_by_ancestry()` refactor (process.rs:767):**

```rust
#[cfg(target_os = "windows")]
fn find_shell_by_ancestry(app_pid: i32, _focused_cwd: Option<&str>) -> Option<i32> {
    let conpty_shells = process_windows::find_conpty_shells(app_pid as u32);

    if conpty_shells.is_empty() {
        // Fallback: legacy approach for non-ConPTY terminals (mintty, etc.)
        return find_shell_by_ancestry_legacy(app_pid);
    }

    eprintln!("[process] found {} ConPTY shells", conpty_shells.len());
    for s in &conpty_shells {
        eprintln!("[process]   pid={} exe={} conhost={} is_wsl={}",
            s.shell_pid, s.shell_exe, s.conhost_pid, s.is_wsl);
    }

    // If only one ConPTY shell, return it (common case: single tab)
    if conpty_shells.len() == 1 {
        return Some(conpty_shells[0].shell_pid as i32);
    }

    // Multiple ConPTY shells: use Layer 2 (active tab identification)
    // For now, fall through to CWD matching or highest-PID
    // Layer 2 integration point goes here

    conpty_shells.iter()
        .max_by_key(|s| s.shell_pid)
        .map(|s| s.shell_pid as i32)
}
```

### Layer 2: Active Tab Identification via Window Title (New: `detect_windows.rs` extension)

**Principle:** VS Code/Cursor and Windows Terminal both encode the active terminal tab's information in the window title. This is the cheapest and most reliable signal for which tab is focused.

**Window title patterns:**

| Host | Window Title Format | Active Terminal Info |
|------|---------------------|---------------------|
| Windows Terminal | `PowerShell` or `Ubuntu` (tab name) | Profile name of active tab |
| VS Code | `file.rs - project - Visual Studio Code` | No terminal info by default |
| VS Code (terminal focused) | Changes to show terminal name when panel focused | Shell name visible |
| VS Code Remote-WSL | `file.rs - project [WSL: Ubuntu] - Visual Studio Code` | `[WSL: distro]` |

**Window title + UIA focused element approach:**

```rust
// In detect_windows.rs, add:

/// Extract active terminal profile name from Windows Terminal title.
/// Windows Terminal sets the window title to the active tab's profile name.
pub fn extract_wt_active_profile(title: &str) -> Option<String> {
    // Windows Terminal title format varies but often is just the profile name
    // e.g., "PowerShell", "Ubuntu", "Command Prompt"
    // When running a command: "command - ProfileName"
    Some(title.trim().to_string())
}

/// Match a ConPTY shell to the active tab using CWD comparison.
/// Reads CWD of each candidate shell via PEB and compares to
/// any CWD signal we can extract.
pub fn match_shell_to_active_tab(
    shells: &[ConPtyShell],
    window_title: &str,
    host_exe: &str,
) -> Option<u32> {
    // Strategy depends on host:
    // 1. Windows Terminal: title contains profile name -> match shell exe
    // 2. VS Code/Cursor: use CWD comparison (read each shell's CWD via PEB)
    // 3. Fallback: use UIA to find focused terminal element's text
    None // Placeholder for implementation
}
```

**CWD-based disambiguation for VS Code (enabling the unused macOS path):**

The macOS codebase has CWD-based disambiguation (process.rs:656-678) that is already proven to work. The Windows path never supplies `focused_cwd`. The fix:

```rust
// In detect_app_context_windows, BEFORE calling get_foreground_info:
// Read the window title to get CWD context

let focused_cwd = if is_ide {
    // For IDE terminals, try to extract focused tab CWD from UIA
    // The terminal panel's focused tab may expose CWD in its title/text
    previous_hwnd.and_then(|hwnd| {
        extract_focused_terminal_cwd_via_uia(hwnd)
    })
} else {
    None
};

let proc_info = process::get_foreground_info_with_cwd(previous_app_pid, focused_cwd.as_deref());
```

### Layer 3: UIA Text Scoping (Modified: `uia_reader.rs`)

**Principle:** Instead of walking ALL descendants of the VS Code window (which captures sidebar, editor, status bar, etc.), scope the UIA walk to the terminal panel element only.

**Current problem with `try_walk_children()`:**
```rust
// uia_reader.rs:132 -- walks ALL descendants
if let Ok(children) = element.find_all(TreeScope::Descendants, &true_condition) {
    for child in children {
        // Captures EVERYTHING: editor text, sidebar items, terminal text
        if let Ok(name) = child.get_name() {
            text_parts.push(name);
        }
    }
}
```

This causes:
- Linux paths in editor files trigger false WSL detection
- Shell prompts in editor text trigger wrong shell inference
- Huge text buffers (64KB of mixed UI chrome + terminal content)

**Fix -- scoped UIA terminal element search:**

```rust
// uia_reader.rs -- new function

/// For IDE windows (VS Code, Cursor), find and read only the terminal panel's text.
/// Uses UIA control type and automation ID to scope the search.
///
/// VS Code terminal panel UIA structure:
///   Window "VS Code"
///     +-- ... (many UI elements)
///     +-- Group "Terminal" or TabItem with terminal content
///           +-- Document or Text elements with actual terminal output
#[cfg(target_os = "windows")]
pub fn read_ide_terminal_text(hwnd: isize) -> Option<String> {
    let automation = UIAutomation::new().ok()?;
    let element = automation.element_from_handle(Handle::from(hwnd)).ok()?;

    // Strategy 1: Find element with ClassName containing "terminal"
    // Strategy 2: Find element with AutomationId containing "terminal"
    // Strategy 3: Find the focused element and walk up to find terminal container

    // Try finding the focused element first -- if user just pressed hotkey
    // from terminal panel, the focused element IS in the terminal
    if let Ok(focused) = automation.get_focused_element() {
        // Walk up from focused element to find terminal container
        // Then read text only from that subtree
        if let Some(terminal_text) = extract_terminal_subtree_text(&automation, &focused) {
            return Some(terminal_text);
        }
    }

    // Fallback: full tree walk (existing behavior)
    None
}
```

**Why focused element scoping works:** When the user presses the CMD+K hotkey, their cursor/focus is in the terminal panel. The UIA focused element will be within the terminal's element subtree. Walking up from there to find the terminal container, then reading only that subtree's text, eliminates editor/sidebar noise.

---

## Integration Map: New vs Modified Components

### New Files

| File | Purpose | LOC Estimate | Dependencies |
|------|---------|-------------|--------------|
| `terminal/process_windows.rs` | ConPTY-aware shell discovery | ~150 | windows-sys (existing) |

### Modified Files

| File | Change | Risk |
|------|--------|------|
| `terminal/mod.rs` | Wire ConPTY discovery into `detect_app_context_windows()`, pass focused_cwd | LOW -- additive, existing paths still work as fallback |
| `terminal/process.rs` | Refactor `find_shell_by_ancestry()` Windows impl to use ConPTY discovery first | MEDIUM -- core detection logic change, needs thorough testing |
| `terminal/detect_windows.rs` | Add window title parsing helpers, add `wsl.exe` handling | LOW -- additive functions |
| `terminal/uia_reader.rs` | Add `read_ide_terminal_text()` scoped reading, keep `read_terminal_text_windows()` as fallback | LOW -- new function, doesn't change existing |

### Unchanged Files

| File | Why No Change |
|------|---------------|
| `terminal/detect.rs` | macOS-only bundle ID logic |
| `terminal/ax_reader.rs` | macOS-only AX text reading |
| `terminal/filter.rs` | Filtering logic applies equally to scoped/unscoped text |
| `terminal/browser.rs` | Browser detection unrelated |
| `commands/ai.rs` | System prompt WSL handling already exists |
| `commands/paste.rs` | Paste logic unaffected by detection fixes |

---

## Data Flow: Fixed Detection Pipeline

```
Hotkey fires -> capture HWND + PID
  |
  v
detect_full_with_hwnd(pid, text, hwnd)
  |
  v
detect_app_context_windows(pid, text)
  |
  +-- get_exe_name_for_pid(pid) -> "Code.exe"
  |
  +-- is_ide_with_terminal_exe("Code.exe") -> true
  |
  +-- [NEW] find_conpty_shells(pid)
  |     Returns: [{pid:5000, exe:"powershell.exe", conhost:4998},
  |               {pid:5500, exe:"powershell.exe", conhost:5498},
  |               {pid:6000, exe:"wsl.exe",        conhost:5998}]
  |     NOTE: Internal cmd.exe/wsl.exe NOT in list (not ConPTY-hosted)
  |
  +-- [NEW] If multiple ConPTY shells:
  |     Try CWD-based matching (read each shell's PEB CWD)
  |     OR window title matching
  |     Fallback: highest PID among ConPTY shells only
  |
  +-- get_process_cwd(matched_shell_pid) -> "C:\Users\laksh\project"
  |
  +-- detect_wsl_in_ancestry(matched_shell_pid)
  |     OR: ConPtyShell.is_wsl flag already set
  |
  v
detect_full_with_hwnd continues:
  |
  +-- Window title: get_window_title(hwnd)
  |     "[WSL: Ubuntu]" -> terminal.is_wsl = true
  |
  +-- [MODIFIED] UIA text reading:
  |     if is_ide -> read_ide_terminal_text(hwnd)  [scoped to terminal panel]
  |     else -> read_terminal_text_windows(hwnd)   [existing full-window read]
  |
  +-- detect_wsl_from_text() -- now on scoped text, fewer false positives
  |
  +-- infer_shell_from_text() -- now on scoped text, more accurate
  |
  v
Return TerminalContext {
    shell_type: "powershell" (from ConPTY shell exe name)
    cwd: "C:\Users\laksh\project" (from PEB of correct shell)
    visible_output: "PS C:\Users\laksh\project> ..." (scoped terminal text)
    is_wsl: false
}
```

---

## Patterns to Follow

### Pattern 1: ConPTY Parentage as Shell Classification

**What:** Use process parentage through conhost.exe/OpenConsole.exe to distinguish user terminal shells from internal IDE processes.

**When:** Always on Windows when detecting shells in IDEs (VS Code, Cursor) or Windows Terminal.

**Example:**
```rust
fn is_conpty_hosted_shell(shell_pid: u32, parent_map: &HashMap<u32, u32>, exe_map: &HashMap<u32, String>) -> bool {
    if let Some(&parent_pid) = parent_map.get(&shell_pid) {
        if let Some(parent_exe) = exe_map.get(&parent_pid) {
            let lower = parent_exe.to_lowercase();
            return lower == "conhost.exe" || lower == "openconsole.exe";
        }
    }
    false
}
```

### Pattern 2: Signal Hierarchy with Fallback Chain

**What:** Order detection signals from most reliable to least reliable. Each signal validates or overrides the previous. Never rely on a single signal.

**When:** All detection decisions (shell type, WSL status, CWD).

**Signal hierarchy for shell type:**
1. ConPTY shell exe name (most reliable -- directly identifies the process)
2. Window title patterns (fast, < 1ms, good for Windows Terminal profile names)
3. Scoped UIA terminal text prompt patterns (good when ConPTY ambiguous)
4. Unscoped UIA text prompt patterns (least reliable -- noise from editor/sidebar)

**Signal hierarchy for WSL detection:**
1. Window title `[WSL: distro]` (definitive for Remote-WSL mode)
2. ConPTY shell exe is `wsl.exe` (definitive for WSL terminal tabs)
3. `detect_wsl_in_ancestry()` (reliable for WSL 1, fails for WSL 2 Hyper-V)
4. Scoped UIA text Linux path patterns (good but fragile)
5. CWD path style `\\wsl$\...` or starts with `/` (fallback)

### Pattern 3: Single Snapshot, Multiple Queries

**What:** Take one `CreateToolhelp32Snapshot` and build maps (parent_map, exe_map) that serve all queries: ConPTY shell finding, WSL ancestry detection, sub-shell filtering.

**When:** Any function that needs multiple process tree queries.

**Why:** Each snapshot is ~1ms. The old code takes 3+ snapshots per detection cycle (one in `find_shell_by_ancestry`, one in `detect_wsl_in_ancestry`, one in `scan_wsl_processes_diagnostic`). Sharing a single snapshot struct eliminates redundant work.

```rust
/// Shared process snapshot. Built once, queried many times.
pub struct ProcessSnapshot {
    parent_map: HashMap<u32, u32>,
    exe_map: HashMap<u32, String>,
    conhost_pids: Vec<u32>,
    shell_pids: Vec<(u32, String)>,  // (pid, exe_name)
    wsl_pids: Vec<u32>,
}

impl ProcessSnapshot {
    pub fn capture() -> Option<Self> { /* single CreateToolhelp32Snapshot call */ }
    pub fn find_conpty_shells(&self, app_pid: u32) -> Vec<ConPtyShell> { /* ... */ }
    pub fn is_descendant_of(&self, pid: u32, ancestor: u32) -> bool { /* ... */ }
    pub fn has_wsl_in_ancestry(&self, pid: u32) -> bool { /* ... */ }
}
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Filtering by Shell Exe Name

**What:** Hardcoding "exclude cmd.exe in IDEs" as the primary IDE process filter.
**Why bad:** Users legitimately run cmd.exe terminal tabs. The filter removes correct results. Also doesn't handle internal wsl.exe processes.
**Instead:** Filter by process parentage (ConPTY-hosted vs extension-hosted).

### Anti-Pattern 2: Unscoped UIA Tree Walk for IDEs

**What:** Using `find_all(TreeScope::Descendants, &true_condition)` on the entire VS Code window.
**Why bad:** Captures 50+ UI elements including editor text, sidebar labels, status bar. Creates false positives for WSL detection and shell inference. Produces huge text buffers.
**Instead:** Scope UIA to the terminal panel element via focused element ancestry or control type filtering.

### Anti-Pattern 3: Multiple Process Snapshots

**What:** Calling `CreateToolhelp32Snapshot` separately in `find_shell_by_ancestry()`, `detect_wsl_in_ancestry()`, and `scan_wsl_processes_diagnostic()`.
**Why bad:** Each snapshot is a full system process enumeration. Three snapshots = 3x the work. Process state can change between snapshots leading to inconsistencies.
**Instead:** Single `ProcessSnapshot::capture()` shared across all queries.

### Anti-Pattern 4: `wsl.exe -e pwd` for Active Session CWD

**What:** Spawning `wsl.exe -e sh -c "pwd"` to get the terminal's CWD.
**Why bad:** This spawns a NEW WSL session. It returns the HOME directory of the default user, not the CWD of the active terminal session. If user has multiple WSL distros, it queries the default one which may not be the one in the active tab.
**Instead:** Infer CWD from scoped UIA terminal text (prompt patterns like `user@host:/path$`). Fall back to `wsl.exe` subprocess only when UIA text has no CWD signal.

---

## Build Order (Dependency-Driven)

### Phase 1: Process Snapshot Consolidation

**Goal:** Single snapshot infrastructure, ConPTY shell discovery.

1. Create `terminal/process_windows.rs` with `ProcessSnapshot` struct
2. Implement `ProcessSnapshot::capture()` -- single `CreateToolhelp32Snapshot`
3. Implement `ProcessSnapshot::find_conpty_shells(app_pid)` -- conhost-parented shell discovery
4. Implement `ProcessSnapshot::is_descendant_of()` and `has_wsl_in_ancestry()`
5. Refactor `find_shell_by_ancestry()` in process.rs to use `ProcessSnapshot`
6. Remove redundant snapshots in `detect_wsl_in_ancestry()` and `scan_wsl_processes_diagnostic()`

**Validation:** Log output showing ConPTY shells found, internal processes excluded. Test with VS Code having 3+ terminal tabs and active extensions.

**Risk:** MEDIUM -- changes core detection logic. Mitigate by keeping legacy `find_shell_by_ancestry` as fallback when ConPTY discovery returns empty (handles mintty and other non-ConPTY terminals).

### Phase 2: UIA Terminal Scoping

**Goal:** Eliminate false positives from IDE chrome in UIA text.

1. Add `read_ide_terminal_text(hwnd)` to `uia_reader.rs` -- focused element scoping
2. Modify `detect_full_with_hwnd()` in mod.rs to use scoped reader for IDEs
3. Keep `read_terminal_text_windows()` as fallback for non-IDE terminals

**Validation:** UIA text from VS Code contains only terminal panel content, not editor/sidebar text. WSL detection false positives eliminated.

**Risk:** LOW -- additive function, existing path unchanged for non-IDE windows.

**Dependency:** Independent of Phase 1. Can be developed in parallel.

### Phase 3: Active Tab Matching

**Goal:** Correctly identify the focused terminal tab's shell PID.

1. Add window title parsing helpers to `detect_windows.rs`
2. Enable CWD-based disambiguation in `find_shell_by_ancestry()` -- wire up `focused_cwd` parameter that currently goes unused on Windows
3. Implement CWD extraction from scoped UIA terminal text (for IDE focused tab CWD)
4. Integrate with Phase 1's ConPTY shell list: match CWD or title against candidates

**Validation:** With 3 terminal tabs in VS Code, switching between tabs and pressing hotkey returns correct shell PID and CWD each time.

**Risk:** MEDIUM -- CWD-via-UIA may not always work (terminals with non-standard prompts). Fallback to highest PID among ConPTY shells is still better than current behavior.

**Dependency:** Depends on Phase 1 (ConPTY shell list to match against). Benefits from Phase 2 (scoped UIA for CWD extraction).

### Phase 4: WSL Detection Hardening

**Goal:** Reliable WSL detection without false positives.

1. Use ConPTY shell's exe name (`wsl.exe`) as primary WSL signal
2. Window title `[WSL: distro]` as secondary signal
3. Scoped UIA text as tertiary signal (with fewer false positives from Phase 2)
4. Fix `get_wsl_cwd()` to use UIA-inferred CWD when available, subprocess as last resort
5. Extract WSL distro name from window title for multi-distro environments

**Validation:** WSL correctly detected in: Windows Terminal WSL tab, VS Code WSL terminal tab, VS Code Remote-WSL mode, standalone wsl.exe. NOT falsely detected when editor has Linux paths.

**Risk:** LOW -- mostly benefits from Phases 1-3 improvements.

**Dependency:** Depends on Phase 1 (ConPTY shell list). Benefits from Phase 2 (scoped UIA).

### Dependency Graph

```
Phase 1 (ConPTY Discovery)
    |
    +---> Phase 3 (Active Tab Matching -- needs ConPTY shell list)
    |         |
    |         +---> Phase 4 (WSL Hardening -- benefits from both)
    |
Phase 2 (UIA Scoping -- independent)
    |
    +---> Phase 3 (benefits from scoped UIA for CWD)
    +---> Phase 4 (benefits from scoped text for WSL detection)
```

**Recommended sequence:** Phase 1 -> Phase 2 (parallel if possible) -> Phase 3 -> Phase 4.

Each phase delivers standalone value:
- After Phase 1: Internal IDE processes no longer pollute results
- After Phase 2: UIA text is clean terminal content, no IDE chrome noise
- After Phase 3: Multi-tab scenarios return correct tab's context
- After Phase 4: WSL detection is reliable without false positives

---

## Scalability Considerations

| Concern | Current State | After Fix |
|---------|--------------|-----------|
| Process snapshot cost | 3+ snapshots per detection (~3ms) | 1 snapshot (~1ms) |
| UIA tree walk for VS Code | Full window walk (~50-200ms, 50+ elements) | Scoped terminal walk (~20-50ms, 5-10 elements) |
| Detection budget (750ms timeout) | Tight with 3 snapshots + full UIA walk | Comfortable with consolidated approach |
| Multi-tab accuracy | Wrong for 2+ tabs (highest PID guess) | Correct via CWD matching |
| False positive rate (WSL) | High in IDEs (editor text triggers) | Low with scoped UIA + ConPTY signal |

---

## Sources

- Codebase: `terminal/process.rs:767-896` -- current Windows `find_shell_by_ancestry()` with ConPTY fallback and IDE cmd.exe filter
- Codebase: `terminal/process.rs:964-1017` -- `scan_wsl_processes_diagnostic()` showing 10-16+ wsl.exe processes in VS Code
- Codebase: `terminal/mod.rs:108-200` -- `detect_full_with_hwnd()` detection pipeline with UIA + WSL signals
- Codebase: `terminal/uia_reader.rs:120-153` -- `try_walk_children()` full descendant walk
- Codebase: `terminal/detect_windows.rs:187-204` -- window title WSL detection
- [ConPTY architecture blog post](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/) -- ConPTY process model (HIGH confidence)
- [OpenConsole.exe vs conhost.exe discussion](https://github.com/microsoft/terminal/discussions/12115) -- process tree structure (HIGH confidence)
- [Windows Console Ecosystem Roadmap](https://learn.microsoft.com/en-us/windows/console/ecosystem-roadmap) -- ConPTY as standard pty mechanism (HIGH confidence)
- [VS Code Terminal Advanced docs](https://code.visualstudio.com/docs/terminal/advanced) -- ConPTY integration, node-pty (MEDIUM confidence)
- [microsoft/node-pty](https://github.com/microsoft/node-pty) -- VS Code pty backend, ConPTY-only since 2026 (MEDIUM confidence)
- [VS Code Terminal Shell Integration](https://code.visualstudio.com/docs/terminal/shell-integration) -- terminal tab process identification (MEDIUM confidence)
