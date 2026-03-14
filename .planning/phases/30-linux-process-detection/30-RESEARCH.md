# Phase 30: Linux Process Detection - Research

**Researched:** 2026-03-13
**Domain:** Linux /proc filesystem process inspection (Rust)
**Confidence:** HIGH

## Summary

Linux process detection via `/proc` is the simplest of all three platforms. The `/proc` pseudo-filesystem exposes process information as regular files readable with standard filesystem operations -- no FFI, no external crates, no subprocess calls needed. The existing macOS implementation in `process.rs` provides the exact template: each platform-specific leaf function (`get_process_cwd`, `get_process_name`, `get_child_pids`, `get_parent_pid`, `find_shell_by_ancestry`, `is_descendant_of`, `build_parent_map`) currently has a `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` stub returning `None` or empty `Vec`. These stubs get replaced with real `/proc`-based implementations.

The process tree shape on Linux is straightforward: terminal emulator (gnome-terminal-server, kitty, alacritty, etc.) spawns shell (bash, zsh, fish) as a direct child, sometimes through a wrapper (`login`). IDE terminals (VS Code, Cursor) on Linux are simpler than Windows -- no ConPTY layer, shells are direct children of the IDE's pty-helper process. The existing `find_shell_recursive` + `find_shell_by_ancestry` pattern handles both cases without modification.

**Primary recommendation:** Replace the 7 stub functions with `/proc`-based implementations using `std::fs::read_link` and `std::fs::read_to_string`. Zero external dependencies needed. The `detect_inner` and `detect_app_context` functions need Linux-specific branches that identify terminal/IDE apps by executable name (same pattern as Windows `detect_windows.rs`).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Generic /proc-based approach that works with ANY terminal emulator -- not terminal-specific code paths
- Explicitly test with: GNOME Terminal, kitty, Alacritty, Konsole, WezTerm, xfce4-terminal
- Phase 30 SHOULD handle IDE terminals (VS Code, Cursor, JetBrains) on Linux
- IDE process trees on Linux are simpler than Windows -- no ConPTY layer, shells are direct children of the IDE process
- Reuse the existing `find_shell_pid` recursive walk + ancestry search pattern
- Permission denied on /proc/PID/cwd or /proc/PID/exe -> return None, log debug warning
- Race conditions (/proc entry disappears mid-read) -> catch errors, return None
- No alternative detection methods (no subprocess calls like lsof) -- /proc is sufficient on Linux
- Split from two-way (macos / not(macos)) to three-way: macos / linux / windows cfg gates
- `find_shell_pid` maintains 3-arg arity: (terminal_pid, focused_cwd, snapshot) where snapshot is Option<&()> on Linux

### Claude's Discretion
- Exact /proc parsing approach (read /proc/PID/stat vs /proc/PID/status for ppid)
- Whether to use /proc/PID/task/*/children or scan /proc/*/stat for child discovery
- Error handling granularity (which errors to log vs silently ignore)
- Test structure and mocking strategy for /proc filesystem

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LPROC-01 | CWD detected via `/proc/PID/cwd` readlink for the active terminal's shell process | `get_process_cwd()` implementation using `std::fs::read_link("/proc/{pid}/cwd")` -- verified working on this system |
| LPROC-02 | Shell type detected via `/proc/PID/exe` readlink (bash, zsh, fish, etc.) | `get_process_name()` implementation using `std::fs::read_link("/proc/{pid}/exe")` then extracting filename -- verified working |
| LPROC-03 | Process tree walking via `/proc/PID/children` to find shell process from terminal emulator PID | Two approaches researched: `/proc/PID/task/TID/children` for direct children, or `/proc/PID/stat` field 4 for parent PID. Both verified on this system. Recommendation: use `/proc/PID/stat` field 4 for ppid (simpler, more reliable) and scan `/proc/*/stat` for child discovery |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::fs` | Rust stdlib | Read /proc files (read_link, read_to_string) | Zero deps, /proc is a filesystem -- use filesystem APIs |
| `std::path::Path` | Rust stdlib | Path manipulation for /proc paths and exe name extraction | Already used throughout codebase |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None | - | - | No external crates needed for /proc access |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Raw /proc reading | `procfs` crate | Adds dependency for something achievable in ~50 lines of std::fs. Project prefers zero-dep platform code (macOS uses raw FFI, not a crate) |
| `/proc/PID/stat` parsing | `ps` subprocess | Decision explicitly forbids subprocess calls. /proc is faster and avoids fork overhead |
| `/proc/PID/task/TID/children` | `/proc/PID/stat` scan | `/proc/PID/task/TID/children` gives direct children but requires knowing the TID (usually == PID for main thread). Scanning `/proc/*/stat` for ppid is more reliable for ancestry building |

## Architecture Patterns

### Existing Stub Functions to Replace

All stubs are in `src-tauri/src/terminal/process.rs`. Each currently returns `None` or `Vec::new()`.

```
process.rs stubs needing Linux implementations:
├── get_process_cwd(_pid) -> Option<String>          # Line 432 — LPROC-01
├── get_process_name(_pid) -> Option<String>          # Line 466 — LPROC-02
├── get_child_pids(_pid) -> Vec<i32>                  # Line 587 — LPROC-03
├── find_shell_by_ancestry(_app_pid, _focused_cwd)    # Line 1017 — LPROC-03
├── is_descendant_of (currently macOS-only)            # Needs Linux impl
├── get_parent_pid (currently macOS-only)              # Needs Linux impl
└── build_parent_map (currently macOS-only)            # Needs Linux impl
```

### New Functions Needed in mod.rs

```
mod.rs changes:
├── detect_inner() Linux branch                       # Currently returns None
├── detect_app_context() Linux branch                 # Currently returns None
├── detect_app_context_linux()                        # New function (parallel to _macos/_windows)
├── Linux terminal/IDE classification                 # New constants + helpers
```

### Pattern 1: /proc CWD Detection (LPROC-01)
**What:** Read symlink at `/proc/PID/cwd` to get process working directory
**When to use:** `get_process_cwd()` on Linux
**Example:**
```rust
// Verified on this WSL2 system — std::fs::read_link works on /proc symlinks
#[cfg(target_os = "linux")]
fn get_process_cwd(pid: i32) -> Option<String> {
    let path = format!("/proc/{}/cwd", pid);
    match std::fs::read_link(&path) {
        Ok(p) => {
            let s = p.to_string_lossy().into_owned();
            if s.is_empty() { None } else { Some(s) }
        }
        Err(e) => {
            eprintln!("[process] /proc/{}/cwd read_link failed: {}", pid, e);
            None
        }
    }
}
```

### Pattern 2: /proc Exe Detection (LPROC-02)
**What:** Read symlink at `/proc/PID/exe` to get process binary path, extract filename
**When to use:** `get_process_name()` on Linux
**Example:**
```rust
// Verified: readlink /proc/self/exe returns e.g. "/usr/bin/bash"
#[cfg(target_os = "linux")]
fn get_process_name(pid: i32) -> Option<String> {
    let path = format!("/proc/{}/exe", pid);
    let target = std::fs::read_link(&path).ok()?;
    target.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}
```

### Pattern 3: Child PID Discovery (LPROC-03)
**What:** Find child processes of a given PID
**When to use:** `get_child_pids()` on Linux

Two approaches available:

**Approach A: /proc/PID/task/TID/children (recommended for get_child_pids)**
```rust
// /proc/PID/task/PID/children contains space-separated child PIDs
// Verified: cat /proc/$$/task/$$/children returns child PIDs
#[cfg(target_os = "linux")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    let path = format!("/proc/{}/task/{}/children", pid, pid);
    match std::fs::read_to_string(&path) {
        Ok(content) => content.split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .filter(|&p| p > 0)
            .collect(),
        Err(_) => Vec::new(),
    }
}
```

**Approach B: /proc/PID/stat field 4 for parent PID (recommended for ancestry)**
```rust
// /proc/PID/stat: "PID (comm) S PPID ..." — field 4 is parent PID
// Robust parsing: find closing ')' then split remaining fields
#[cfg(target_os = "linux")]
fn get_parent_pid(pid: i32) -> Option<i32> {
    let content = std::fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    // comm field can contain spaces and parens, so find last ')' first
    let after_comm = content.rfind(')')? + 1;
    let fields: Vec<&str> = content[after_comm..].split_whitespace().collect();
    // fields[0] = state, fields[1] = ppid
    fields.get(1)?.parse::<i32>().ok()
}
```

### Pattern 4: Ancestry Search Without Subprocess
**What:** `find_shell_by_ancestry` and `build_parent_map` without `pgrep`/`ps` subprocesses
**When to use:** Linux broad shell search (IDE terminals with deep process trees)
**Example:**
```rust
// Scan /proc for all PIDs, build parent map from /proc/PID/stat
// Replaces macOS `ps -eo pid=,ppid=` subprocess
#[cfg(target_os = "linux")]
fn build_parent_map() -> std::collections::HashMap<i32, i32> {
    let mut map = std::collections::HashMap::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(pid) = name.parse::<i32>() {
                    if let Some(ppid) = get_parent_pid(pid) {
                        map.insert(pid, ppid);
                    }
                }
            }
        }
    }
    map
}
```

### Pattern 5: Linux Terminal/IDE Classification
**What:** Identify terminal emulators and IDEs by executable name on Linux
**When to use:** `detect_inner()` and `detect_app_context()` Linux branches
**Example:**
```rust
// Linux terminal emulators — match on executable name (no bundle IDs like macOS)
const KNOWN_TERMINAL_EXES_LINUX: &[&str] = &[
    "gnome-terminal-server",  // GNOME Terminal (actual process name)
    "kitty",
    "alacritty",
    "konsole",
    "wezterm-gui",
    "xfce4-terminal",
    "tilix",
    "terminator",
    "xterm",
    "urxvt",
    "st",                     // suckless terminal
    "foot",                   // Wayland terminal
    "sakura",
    "terminology",
];

const KNOWN_IDE_EXES_LINUX: &[&str] = &[
    "code",           // VS Code
    "cursor",         // Cursor
    "idea",           // IntelliJ IDEA (via wrapper)
    "pycharm",        // PyCharm
    "webstorm",       // WebStorm
    "clion",          // CLion
    "goland",         // GoLand
];
```

**Note on GNOME Terminal:** The actual process name is `gnome-terminal-server`, not `gnome-terminal`. The `gnome-terminal` command is a client that talks to the server process. The PID captured by the overlay system will be the server PID.

### Pattern 6: detect_app_context_linux
**What:** Linux equivalent of `detect_app_context_macos` and `detect_app_context_windows`
**When to use:** `detect_app_context()` Linux branch
```rust
#[cfg(target_os = "linux")]
fn detect_app_context_linux(previous_app_pid: i32, _pre_captured_text: Option<String>) -> Option<AppContext> {
    // 1. Get exe name from /proc/PID/exe
    let exe_name = get_process_name(previous_app_pid);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let app_name = Some(clean_linux_app_name(exe_str));

    // 2. Walk process tree for shell detection (reuses existing find_shell_pid)
    let proc_info = get_foreground_info(previous_app_pid);
    let has_shell = proc_info.shell_type.is_some() || proc_info.cwd.is_some();

    let terminal = if has_shell {
        Some(TerminalContext {
            shell_type: proc_info.shell_type,
            cwd: proc_info.cwd,
            visible_output: None,  // Phase 34 handles terminal text reading
            running_process: proc_info.running_process,
            is_wsl: false,
        })
    } else {
        None
    };

    Some(AppContext {
        app_name,
        terminal,
        console_detected: false,
        console_last_line: None,
        visible_text: None,  // No AX API on Linux; Phase 34 handles text
    })
}
```

### Anti-Patterns to Avoid
- **Spawning subprocesses for /proc data:** Do NOT use `pgrep`, `ps`, `lsof`, or any subprocess call. All data is available directly from /proc. The macOS code uses `pgrep` and `ps` as fallbacks because `proc_listchildpids` has permission issues -- Linux /proc has no such limitation for user-owned processes.
- **Parsing /proc/PID/stat naively by splitting on spaces:** The `comm` field (field 2) is in parentheses and can contain spaces and parentheses itself (e.g., `(Web Content)`). Always find the LAST `)` before parsing remaining fields.
- **Using /proc/PID/status when /proc/PID/stat suffices:** `/proc/PID/status` is human-readable multi-line format. `/proc/PID/stat` is a single line with fixed field positions -- faster to parse and sufficient for ppid.
- **Checking process existence before reading /proc:** TOCTOU race. Just read and handle the error.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process name from PID | String parsing of /proc/PID/cmdline | `std::fs::read_link("/proc/PID/exe")` + `Path::file_name()` | /proc/PID/exe is a symlink to the actual binary; cmdline can be modified by the process and contains arguments |
| Child process listing | Scanning all of /proc for every get_child_pids call | `/proc/PID/task/PID/children` | Direct file read vs scanning thousands of /proc entries. Only use full scan for build_parent_map |
| Parent PID | Subprocess call to `ps` | `/proc/PID/stat` field 4 | Direct file read, no fork overhead |
| Process tree ancestry | Manual walk with get_parent_pid per level | `build_parent_map()` then HashMap lookups | Single /proc scan vs N file reads |

**Key insight:** Every piece of data needed (CWD, exe path, parent PID, child PIDs) is directly readable from /proc as a file or symlink. No FFI, no external crates, no subprocesses. This makes Linux the simplest platform implementation.

## Common Pitfalls

### Pitfall 1: /proc/PID/stat Comm Field Parsing
**What goes wrong:** Splitting `/proc/PID/stat` on spaces breaks when the process name contains spaces or parentheses (e.g., `1234 (Web Content) S 1200 ...`)
**Why it happens:** The comm field is `(name)` but name can contain any characters
**How to avoid:** Find the LAST `)` in the line, then split everything after it on whitespace. Field indices are relative to after the closing paren.
**Warning signs:** Parent PID returned as "S" or other non-numeric values

### Pitfall 2: /proc/PID/exe "deleted" Suffix
**What goes wrong:** After an upgrade, `/proc/PID/exe` symlink target becomes `/usr/bin/bash (deleted)` -- the process is still running but the binary was replaced
**Why it happens:** Linux kernel appends ` (deleted)` to the symlink target when the original file is unlinked
**How to avoid:** Strip ` (deleted)` suffix from the readlink result before extracting the filename
**Warning signs:** Shell type returned as "bash (deleted)" or file_name() returning None

### Pitfall 3: Permission Denied on Other Users' Processes
**What goes wrong:** Reading `/proc/PID/cwd` or `/proc/PID/exe` for root-owned or other-user processes returns EACCES
**Why it happens:** /proc enforces per-process permissions. `cwd` and `exe` require the reader to have the same UID or CAP_SYS_PTRACE
**How to avoid:** Handle `PermissionError` gracefully (return None). Log at debug level, not user-visible. Most terminals are user-owned so this is rare.
**Warning signs:** Works for terminal apps but fails for root-spawned login sessions

### Pitfall 4: GNOME Terminal Process Architecture
**What goes wrong:** Looking for a process named `gnome-terminal` finds nothing
**Why it happens:** GNOME Terminal uses a client-server model. `gnome-terminal` (the command) sends a D-Bus message to `gnome-terminal-server` (the long-running process). The server PID is what the window manager reports.
**How to avoid:** Use `gnome-terminal-server` in the known terminals list. The shell processes are children of gnome-terminal-server.
**Warning signs:** Terminal classification fails for GNOME Terminal while working for kitty, Alacritty

### Pitfall 5: Race Conditions in /proc
**What goes wrong:** Process exits between reading /proc/PID/stat and /proc/PID/cwd, causing ENOENT
**Why it happens:** /proc entries are ephemeral -- they disappear when the process exits
**How to avoid:** Every /proc read must handle ENOENT (or any IO error). Never assume a PID is valid because an earlier read succeeded. The existing code pattern of returning `None` on error handles this correctly.
**Warning signs:** Sporadic panics or unwrap failures during rapid terminal tab switching

### Pitfall 6: Zombie Processes in Process Tree
**What goes wrong:** Process tree walk finds a zombie process that has a valid /proc entry but no useful data
**Why it happens:** Zombie processes (state 'Z' in /proc/PID/stat) have been terminated but not yet reaped by their parent. Their /proc entry exists but /proc/PID/cwd and /proc/PID/exe may return errors.
**How to avoid:** This is naturally handled by the None-returning pattern. No special zombie handling needed.
**Warning signs:** Shell detection succeeds (finds PID) but CWD/exe reads fail

### Pitfall 7: JetBrains IDE Process Names
**What goes wrong:** JetBrains IDEs on Linux may run as `java` process with no way to distinguish from other Java apps
**Why it happens:** JetBrains IDEs are Java applications. The actual process might be `java`, `idea.sh`, or a platform-specific launcher
**How to avoid:** Check `/proc/PID/cmdline` as fallback for Java processes to look for JetBrains-specific class names. However, for Phase 30, it is acceptable to handle this as "generic app with shell child" -- if a shell is found in the process tree, it works regardless of whether we classify the parent as an IDE.
**Warning signs:** JetBrains terminal not detected as IDE terminal (but still works because shell is found in process tree)

## Code Examples

### Complete get_child_pids with Fallback
```rust
// Source: Verified on WSL2 Linux 6.6.87.2
#[cfg(target_os = "linux")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    // Fast path: /proc/PID/task/PID/children
    let path = format!("/proc/{}/task/{}/children", pid, pid);
    if let Ok(content) = std::fs::read_to_string(&path) {
        let pids: Vec<i32> = content.split_whitespace()
            .filter_map(|s| s.parse::<i32>().ok())
            .filter(|&p| p > 0)
            .collect();
        if !pids.is_empty() {
            return pids;
        }
    }

    // Fallback: scan /proc for processes with ppid == pid
    // This handles multi-threaded processes where TID != PID
    let mut children = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(child_pid) = name.parse::<i32>() {
                    if child_pid > 0 {
                        if let Some(ppid) = get_parent_pid(child_pid) {
                            if ppid == pid {
                                children.push(child_pid);
                            }
                        }
                    }
                }
            }
        }
    }
    children
}
```

### Complete get_process_name with (deleted) Handling
```rust
// Source: Verified on WSL2 Linux 6.6.87.2
#[cfg(target_os = "linux")]
fn get_process_name(pid: i32) -> Option<String> {
    let path = format!("/proc/{}/exe", pid);
    let target = std::fs::read_link(&path).ok()?;
    let target_str = target.to_string_lossy();
    // Handle " (deleted)" suffix from upgraded binaries
    let clean = target_str.trim_end_matches(" (deleted)");
    std::path::Path::new(clean)
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
}
```

### Complete find_shell_by_ancestry for Linux
```rust
// Source: Adapted from macOS pattern, using /proc instead of pgrep/ps
#[cfg(target_os = "linux")]
fn find_shell_by_ancestry(app_pid: i32, focused_cwd: Option<&str>) -> Option<i32> {
    // Build parent map from single /proc scan (replaces `ps -eo pid=,ppid=`)
    let parent_map = build_parent_map();

    // Find all shell processes by scanning /proc/*/exe (replaces `pgrep -x`)
    let mut shell_pids: Vec<i32> = Vec::new();
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(pid) = name.parse::<i32>() {
                    if let Some(proc_name) = get_process_name(pid) {
                        if KNOWN_SHELLS.contains(&proc_name.as_str()) {
                            shell_pids.push(pid);
                        }
                    }
                }
            }
        }
    }

    // Filter to shells that are descendants of app_pid
    let descendant_shells: Vec<(i32, Option<String>)> = shell_pids.iter()
        .filter(|&&pid| is_descendant_of_map(pid, app_pid, &parent_map))
        .map(|&pid| (pid, get_process_name(pid)))
        .collect();

    if descendant_shells.is_empty() {
        return None;
    }

    // Reuse existing sub-shell filtering + CWD matching + highest PID logic
    // (same as macOS find_shell_by_ancestry)
    // ... (sub-shell filtering, CWD matching, highest PID selection)
}

fn is_descendant_of_map(pid: i32, ancestor: i32, parent_map: &std::collections::HashMap<i32, i32>) -> bool {
    let mut current = pid;
    for _ in 0..15 {
        match parent_map.get(&current) {
            Some(&ppid) if ppid == ancestor => return true,
            Some(&ppid) if ppid <= 1 => return false,
            Some(&ppid) => current = ppid,
            None => return false,
        }
    }
    false
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `#[cfg(not(target_os = "macos"))]` two-way split | Three-way `macos` / `linux` / `windows` | Phase 30 | Existing non-macOS stubs become Linux-specific; Windows already has its own cfg blocks |
| `pgrep`/`ps` subprocess for ancestry (macOS) | Direct /proc scan (Linux) | Phase 30 | Faster, no fork overhead, no PATH dependency |
| ProcessSnapshot (Windows) | Not needed (Linux) | Phase 30 | Linux /proc is live and consistent; no snapshot needed |

**Key cfg gate changes:**
- Functions currently gated `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` become `#[cfg(target_os = "linux")]`
- Functions currently gated `#[cfg(not(target_os = "macos"))]` (like `get_foreground_info`) keep the broad gate since they handle both Windows and Linux, but internal logic may need Linux-specific branches
- Functions currently gated `#[cfg(target_os = "macos")]` that have no non-macOS equivalent (like `is_descendant_of`, `get_parent_pid`, `build_parent_map`) need new `#[cfg(target_os = "linux")]` implementations

## Open Questions

1. **JetBrains IDE detection reliability**
   - What we know: JetBrains IDEs may run as `java` on Linux, making exe-name classification unreliable
   - What's unclear: Whether the actual launcher process (idea, pycharm, etc.) is the one whose PID gets reported by the window manager
   - Recommendation: Test empirically. Even without IDE classification, shell detection works via the generic process tree walk. Classification only affects the `app_name` field in AppContext.

2. **GNOME Terminal multi-tab process tree**
   - What we know: gnome-terminal-server is a single process managing multiple tabs. Each tab's shell is a direct child.
   - What's unclear: When overlay is triggered, which tab's shell should be selected? The PID reported is the server PID, not a per-tab PID.
   - Recommendation: Use focused_cwd matching (same as IDE multi-tab handling on macOS) or fall back to highest PID. This is the existing pattern and should work identically.

3. **Flatpak/Snap terminal emulators**
   - What we know: Flatpak terminals may have restricted /proc access or run in a separate PID namespace
   - What's unclear: Whether /proc/PID/cwd readlink works across PID namespaces
   - Recommendation: Handle gracefully (return None). Decision already made to gracefully degrade for containerized environments.

## Sources

### Primary (HIGH confidence)
- **Linux /proc filesystem** - Verified directly on WSL2 Linux 6.6.87.2-microsoft-standard-WSL2
  - `/proc/PID/cwd` symlink to CWD -- tested with `readlink`
  - `/proc/PID/exe` symlink to executable -- tested with `readlink`
  - `/proc/PID/stat` field 4 = ppid -- verified format and parsing
  - `/proc/PID/task/TID/children` space-separated child PIDs -- verified exists and works
- **Existing codebase** - `src-tauri/src/terminal/process.rs` (1500+ lines, all platform implementations reviewed)
- **Existing codebase** - `src-tauri/src/terminal/mod.rs` (800 lines, orchestration logic reviewed)
- **Existing codebase** - `src-tauri/src/terminal/detect.rs` (macOS terminal/IDE classification reviewed)
- **Existing codebase** - `src-tauri/src/terminal/detect_windows.rs` (Windows terminal/IDE classification reviewed)

### Secondary (MEDIUM confidence)
- **proc(5) man page** - Standard Linux /proc filesystem documentation. `/proc/PID/stat` field layout, `/proc/PID/task` structure
- **GNOME Terminal architecture** - gnome-terminal uses client-server model; server process name is `gnome-terminal-server`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using only std::fs, no external deps. Verified on live system.
- Architecture: HIGH - Direct mapping from existing macOS/Windows patterns to /proc equivalents. Every stub function has a clear implementation path.
- Pitfalls: HIGH - /proc filesystem is well-documented and stable. Pitfalls (comm parsing, deleted suffix, permissions) are well-known.
- cfg gate changes: HIGH - Clear three-way split pattern visible in existing code.

**Research date:** 2026-03-13
**Valid until:** 2026-04-13 (stable -- /proc filesystem ABI is frozen by Linux kernel policy)
