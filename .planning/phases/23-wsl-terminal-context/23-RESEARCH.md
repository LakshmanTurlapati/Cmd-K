# Phase 23: WSL Terminal Context - Research

**Researched:** 2026-03-09
**Domain:** Windows process tree walking, WSL session detection, Linux command generation
**Confidence:** HIGH

## Summary

Phase 23 adds WSL session detection to the existing Windows terminal context pipeline. The core challenge is detecting that a terminal session is running inside WSL (rather than native Windows) by finding `wsl.exe` in the process ancestry, then switching the entire context pipeline to Linux mode: reading the Linux CWD/shell from inside WSL via `wsl.exe -e` subprocess calls, applying Linux destructive patterns, and sending a Linux system prompt to the AI.

The existing codebase already has all the infrastructure needed -- Windows process tree walking via `CreateToolhelp32Snapshot`, UIA text reading, destructive pattern matching, and context badge rendering. The implementation is primarily about adding WSL-awareness as a branch in the existing Windows detection flow, not building new systems.

**Primary recommendation:** Add an `is_wsl: bool` field to `TerminalContext`, detect WSL by finding `wsl.exe` in the process tree ancestry during the existing `find_shell_by_ancestry` walk, read Linux CWD/shell via `wsl.exe -e` subprocess commands, and branch on `is_wsl` in the system prompt and safety modules.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Process tree walk** to identify WSL sessions -- walk from focused terminal PID, detect wsl.exe or WSL init process in ancestry
- **Same detection path for all 4 host types** (Windows Terminal, VS Code, Cursor, standalone wsl.exe) -- no host-specific code paths
- **No distro detection** -- just identify "WSL" vs "not WSL", don't parse Ubuntu/Debian/Arch name
- Graceful fallback: if wsl.exe found but no running distro, fall back to normal Windows terminal context silently
- Badge shows just **"WSL"** as the context label -- no distro name, no shell prefix
- CWD displayed as **Linux paths as-is** -- /home/user/project, /mnt/c/Users/... (no translation to Windows paths)
- **Show running process** inside WSL shell (node, python, etc.) -- same behavior as native terminals
- **Full Linux mode** -- system prompt tells AI "You are in a Linux terminal"
- OS context sent to AI says **"WSL on Windows"** -- AI knows it's WSL and can mention Windows-WSL interop when relevant
- **Linux destructive patterns** applied when in WSL session (rm -rf, dd, chmod 777, mkfs, etc.) -- not Windows patterns
- **WSL-aware confirmation text** -- "This Linux command may be destructive" instead of generic text
- **Same UIA approach** as Windows terminals -- WSL runs inside host windows (WT, VS Code), UIA tree has the text
- **Add Linux-specific secret filtering** -- SSH keys, /etc/shadow lines, sudo password prompts, .env file contents
- **Same ~4KB output limit** as native terminals -- consistent behavior
- Cross-platform: this phase is Windows-only but code must compile on both platforms with `cfg(target_os)` guards

### Claude's Discretion
- Whether to cache WSL detection per terminal window or re-detect each invocation (based on perf characteristics)
- Process tree walk implementation details (Windows API calls, depth limits)
- Exact Linux secret filter regex patterns
- How to extend TerminalContext struct (new field vs enum)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WSLT-01 | CMD+K detects WSL sessions in Windows Terminal | Process tree walk finds wsl.exe; ConPTY fallback already exists in codebase |
| WSLT-02 | CMD+K detects WSL sessions in VS Code Remote-WSL terminals | Same process tree walk; VS Code already in KNOWN_IDE_EXES |
| WSLT-03 | CMD+K detects WSL sessions in Cursor Remote-WSL terminals | Same process tree walk; Cursor already in KNOWN_IDE_EXES |
| WSLT-04 | CMD+K detects standalone wsl.exe console sessions | wsl.exe already in KNOWN_TERMINAL_EXES (as bash.exe); add wsl.exe to list |
| WSLT-05 | CMD+K reads the current working directory from WSL sessions | Use `wsl.exe -e pwd` subprocess to get Linux CWD |
| WSLT-06 | CMD+K detects the shell type (bash, zsh, fish) in WSL sessions | Use `wsl.exe -e sh -c "basename $SHELL"` or parse from process name |
| WSLT-07 | CMD+K reads visible terminal output from WSL sessions | Existing UIA reader works; WSL text appears in host window UIA tree |
| WSLT-08 | AI generates Linux commands when user is in a WSL session | New WSL system prompt template with Linux focus |
| WSLT-09 | Linux destructive command patterns are applied in WSL sessions | Existing Linux patterns in safety.rs; branch on is_wsl flag |
| WSLT-10 | WSL distro name is shown in the context badge | **CONTEXT.md overrides**: badge shows "WSL" only, no distro name |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| windows-sys | 0.59 | Win32 process APIs (CreateToolhelp32Snapshot, QueryFullProcessImageNameW) | Already in Cargo.toml, used for process tree walking |
| uiautomation | 0.24 | UIA TextPattern for terminal text reading | Already in Cargo.toml, handles WSL text in host windows |
| regex | 1 | Destructive pattern matching and secret filtering | Already in Cargo.toml |
| std::process::Command | stdlib | Execute `wsl.exe -e` subprocess calls for CWD/shell detection | No external dependency needed |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| once_cell | 1 | Lazy static for compiled regex patterns | Already used in safety.rs and filter.rs |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `wsl.exe -e` subprocess | Read WSL init process CWD from Windows PEB | PEB gives Windows-side path (e.g., `\\wsl.localhost\Ubuntu\...`), not the Linux-native path; subprocess is simpler and gives exact Linux CWD |
| `wsl` crate from crates.io | Custom detection | Crate adds dependency for functionality we can implement in ~20 lines; avoid extra deps |

## Architecture Patterns

### Recommended Changes to Existing Structure
```
src-tauri/src/
├── terminal/
│   ├── mod.rs             # Add is_wsl field to TerminalContext
│   ├── detect_windows.rs  # Add wsl.exe to KNOWN_TERMINAL_EXES, add WSL detection helpers
│   ├── process.rs         # WSL-aware CWD/shell reading via wsl.exe -e subprocess
│   ├── filter.rs          # Add Linux-specific secret filtering patterns
│   └── uia_reader.rs      # No changes needed (WSL text already visible via UIA)
├── commands/
│   ├── ai.rs              # Add WSL system prompt template (Linux mode)
│   ├── safety.rs          # Branch on is_wsl for Linux vs Windows destructive patterns
│   └── terminal.rs        # No changes needed (passes through TerminalContext)
└── src/
    └── store/index.ts     # Update resolveBadge() to show "WSL" when is_wsl is true
```

### Pattern 1: WSL Detection via Process Tree Walk
**What:** During the existing `find_shell_by_ancestry()` walk in `process.rs`, check if any ancestor in the process chain is `wsl.exe`. If found, set `is_wsl = true` on the returned context.
**When to use:** Every invocation of terminal context detection on Windows.
**Example:**
```rust
// In the existing find_shell_by_ancestry Windows implementation:
// While walking the parent_map to find shell candidates,
// also track if wsl.exe appears in the ancestry chain.

// After building parent_map from CreateToolhelp32Snapshot:
fn is_wsl_session(pid: u32, parent_map: &HashMap<u32, u32>, exe_map: &HashMap<u32, String>) -> bool {
    let mut current = pid;
    for _ in 0..20 {
        if let Some(exe) = exe_map.get(&current) {
            if exe.eq_ignore_ascii_case("wsl.exe") {
                return true;
            }
        }
        match parent_map.get(&current) {
            Some(&ppid) if ppid <= 1 || ppid == current => break,
            Some(&ppid) => current = ppid,
            None => break,
        }
    }
    false
}
```

### Pattern 2: Linux CWD/Shell via wsl.exe Subprocess
**What:** When WSL is detected, use `wsl.exe -e` to run Linux commands that return the CWD and shell type, rather than reading from the Windows PEB (which would give a Windows-style path).
**When to use:** Only when `is_wsl == true`.
**Example:**
```rust
// Get Linux CWD from inside WSL
fn get_wsl_cwd() -> Option<String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-e", "sh", "-c", "readlink /proc/$$/cwd 2>/dev/null || pwd"])
        .output()
        .ok()?;
    if output.status.success() {
        let cwd = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cwd.is_empty() { Some(cwd) } else { None }
    } else {
        None
    }
}

// Get shell type from WSL default shell
fn get_wsl_shell() -> Option<String> {
    let output = std::process::Command::new("wsl.exe")
        .args(["-e", "sh", "-c", "basename \"$SHELL\""])
        .output()
        .ok()?;
    if output.status.success() {
        let shell = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !shell.is_empty() { Some(shell) } else { None }
    } else {
        None
    }
}
```

### Pattern 3: WSL System Prompt Branch
**What:** When `is_wsl` is true, use a Linux-focused system prompt even though the compile target is Windows.
**When to use:** In `ai.rs` when building the system prompt.
**Example:**
```rust
// New constant in ai.rs (alongside existing Windows template):
const WSL_TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for Linux (WSL on Windows). Given the user's task \
     description and terminal context, output ONLY the exact command(s) to run. No explanations, \
     no markdown, no code fences. Just the raw command(s). If multiple commands are needed, \
     separate them with && or use pipes. Prefer common POSIX tools (grep, find, sed, awk). \
     The user is in a WSL Linux terminal with {shell_type} shell. You may reference WSL-Windows \
     interop features (e.g., `code .` to open VS Code) when relevant.";
```

### Pattern 4: TerminalContext Extension
**What:** Add `is_wsl: bool` field to `TerminalContext` struct (and corresponding frontend interface).
**When to use:** Preferred over an enum because it's additive and doesn't break existing code.
**Example:**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct TerminalContext {
    pub shell_type: Option<String>,
    pub cwd: Option<String>,
    pub visible_output: Option<String>,
    pub running_process: Option<String>,
    pub is_wsl: bool,  // New field -- false on macOS, detected on Windows
}
```
```typescript
// Frontend store/index.ts
export interface TerminalContext {
  shell_type: string | null;
  cwd: string | null;
  visible_output: string | null;
  running_process: string | null;
  is_wsl: boolean;  // New field
}
```

### Anti-Patterns to Avoid
- **Host-specific detection paths:** Do NOT write separate WSL detection for Windows Terminal vs VS Code vs Cursor. The process tree walk with wsl.exe ancestor check works identically for all hosts.
- **Reading Linux CWD from Windows PEB:** The PEB gives `\\wsl.localhost\Ubuntu\home\user\...` which is a Windows UNC path, not the Linux path the user sees. Always use `wsl.exe -e` to get the native Linux path.
- **Parsing distro name:** Per CONTEXT.md locked decision, do NOT parse distro names. Badge shows "WSL" only.
- **Running `wsl.exe --list --running`:** This is slow (~200ms) and gives distro names we don't need. Avoid it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Process tree walking | Custom WMI queries | Existing `CreateToolhelp32Snapshot` in process.rs | Already implemented and proven |
| UIA text reading for WSL | New text capture mechanism | Existing `uia_reader.rs` | WSL text appears in host window UIA tree, same as native shells |
| Secret filtering | Custom string scanning | Existing `filter.rs` pattern infrastructure | Just add new regex patterns to the existing Vec |
| Destructive detection | Separate WSL safety module | Existing `safety.rs` DESTRUCTIVE_PATTERNS | Linux patterns already exist in the set; just need frontend branching |

**Key insight:** Almost everything needed for WSL support already exists in the codebase. The phase is primarily about detecting the WSL condition and branching existing logic, not building new systems.

## Common Pitfalls

### Pitfall 1: wsl.exe Subprocess Timeout
**What goes wrong:** `wsl.exe -e` commands can hang if WSL is not properly initialized or the distro is stopped.
**Why it happens:** WSL boot/startup can take seconds on first invocation. If the distro was stopped, `wsl.exe -e pwd` will boot it first.
**How to avoid:** Set a hard timeout (500ms) on all `wsl.exe -e` subprocess calls. If it doesn't respond in time, fall back to Windows-native context with `is_wsl = true` but `cwd = None`.
**Warning signs:** Context detection takes >1 second; UX feels laggy.

### Pitfall 2: ConPTY Process Tree Disconnect (Windows Terminal)
**What goes wrong:** In Windows Terminal, `wsl.exe` is NOT a direct descendant of `WindowsTerminal.exe` due to ConPTY architecture. The shell runs under `OpenConsole.exe`, not under the terminal app.
**Why it happens:** ConPTY creates a pseudoconsole where the shell process has `OpenConsole.exe` as its parent, not the terminal app.
**How to avoid:** The existing ConPTY fallback in `find_shell_by_ancestry()` already handles this by searching for shells parented by `OpenConsole.exe`. Extend this to also check for `wsl.exe` parented by `OpenConsole.exe`.
**Warning signs:** WSL detection works in standalone wsl.exe but fails in Windows Terminal.

### Pitfall 3: Multiple WSL Instances
**What goes wrong:** User has multiple WSL distros or multiple WSL terminal tabs, and the wrong one's CWD is returned.
**Why it happens:** `wsl.exe -e pwd` runs in the default distro, not necessarily the one the user has focused.
**How to avoid:** The process tree walk should find the specific `wsl.exe` process that is in the ancestry of the focused terminal. Use `wsl.exe -d <distro> -e pwd` with the specific PID's associated distro if needed. But per the CONTEXT.md decision (no distro detection), the simple `wsl.exe -e` approach targeting the default distro is acceptable.
**Warning signs:** CWD shows wrong directory when multiple WSL tabs are open.

### Pitfall 4: cfg(target_os) Compilation Guards
**What goes wrong:** Code fails to compile on macOS because WSL-specific types or functions are referenced without cfg guards.
**Why it happens:** New WSL code uses Windows-only APIs but is referenced from shared code paths.
**How to avoid:** All WSL detection code must be behind `#[cfg(target_os = "windows")]`. The `is_wsl` field on `TerminalContext` is always present (defaults to `false` on non-Windows), but detection logic is Windows-only.
**Warning signs:** `cargo build --target aarch64-apple-darwin` fails.

### Pitfall 5: UIA Text from WSL Contains ANSI Escape Codes
**What goes wrong:** WSL terminal output captured via UIA may contain raw ANSI escape sequences (colors, cursor movements) that pollute the AI context.
**Why it happens:** Some terminal emulators pass ANSI codes through UIA text; others strip them.
**How to avoid:** Add ANSI escape code stripping in the filter pipeline before sending to AI. A simple regex `\x1b\[[0-9;]*[a-zA-Z]` handles standard SGR sequences.
**Warning signs:** AI responses contain garbage characters or respond to color codes.

### Pitfall 6: `wsl.exe -e` vs User's Active Shell CWD
**What goes wrong:** `wsl.exe -e pwd` returns the home directory, not the CWD of the user's active WSL shell session.
**Why it happens:** `wsl.exe -e` spawns a new process; it doesn't connect to the existing shell session.
**How to avoid:** Instead of `wsl.exe -e pwd`, use the UIA-captured terminal text to infer the CWD from the shell prompt. Alternatively, read the CWD from `/proc/<pid>/cwd` of the shell PID visible from the WSL side. The most reliable approach: use the `infer_shell_from_text` approach already in `mod.rs` and extend it to infer Linux CWD from prompt patterns like `user@host:/path/to/dir$`.
**Warning signs:** CWD always shows `/home/user` regardless of where the user actually is.

## Code Examples

### Example 1: Extended TerminalContext with is_wsl
```rust
// terminal/mod.rs
#[derive(Debug, Clone, Serialize)]
pub struct TerminalContext {
    pub shell_type: Option<String>,
    pub cwd: Option<String>,
    pub visible_output: Option<String>,
    pub running_process: Option<String>,
    /// True when the session is running inside Windows Subsystem for Linux.
    /// Always false on non-Windows platforms.
    pub is_wsl: bool,
}
```

### Example 2: WSL Detection in Process Tree Walk
```rust
// In process.rs, extend the existing Windows find_shell_by_ancestry
// to also build an exe_name map and check for wsl.exe ancestry.

// During the CreateToolhelp32Snapshot loop that already builds parent_map:
let mut exe_map: HashMap<u32, String> = HashMap::new();
// ... in the loop body, after getting `name`:
exe_map.insert(entry.th32ProcessID, name.clone());

// After finding the shell PID, check if wsl.exe is in its ancestry:
let is_wsl = is_ancestor_wsl(shell_pid, &parent_map, &exe_map);
```

### Example 3: Badge Resolution with WSL
```typescript
// store/index.ts
export function resolveBadge(ctx: AppContext | null): string | null {
  if (!ctx) return null;

  // Priority 0: WSL indicator
  if (ctx.terminal?.is_wsl) {
    return "WSL";
  }

  // Priority 1: Shell type (from terminal or editor integrated terminal)
  if (ctx.terminal?.shell_type) {
    return ctx.terminal.shell_type;
  }
  // ... rest unchanged
}
```

### Example 4: Linux CWD Inference from Terminal Text
```rust
/// Attempt to infer the Linux CWD from visible terminal text.
/// Looks for common prompt patterns: user@host:/path$, [user@host path]$
fn infer_linux_cwd_from_text(text: &str) -> Option<String> {
    // Check last few lines for prompt patterns
    for line in text.lines().rev().take(10) {
        let trimmed = line.trim();
        // Pattern: user@host:/absolute/path$ or user@host:/absolute/path #
        if let Some(colon_pos) = trimmed.find(':') {
            if trimmed[..colon_pos].contains('@') {
                let after_colon = &trimmed[colon_pos + 1..];
                // Extract path before $ or # prompt character
                let path = after_colon
                    .trim_end_matches(|c: char| c == '$' || c == '#' || c == ' ')
                    .trim();
                if path.starts_with('/') {
                    return Some(path.to_string());
                }
            }
        }
    }
    None
}
```

### Example 5: WSL-Aware System Prompt Selection
```rust
// ai.rs - in the system prompt selection logic
let system_prompt = if is_terminal_mode {
    let shell_type = ctx.terminal.as_ref()
        .and_then(|t| t.shell_type.as_deref())
        .unwrap_or("bash");

    let is_wsl = ctx.terminal.as_ref()
        .map(|t| t.is_wsl)
        .unwrap_or(false);

    if is_wsl {
        WSL_TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type)
    } else {
        TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type)
    }
} else {
    ASSISTANT_SYSTEM_PROMPT.to_string()
};
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| WSL 1 (translation layer) | WSL 2 (lightweight VM) | Windows 10 2004 (May 2020) | Process tree structure changed; wsl.exe is the bridge process |
| Shells visible in Windows process tree | Linux processes NOT visible in Windows Task Manager | WSL 2 | Cannot use Windows APIs to see bash/zsh inside WSL directly |
| No wslhost.exe | wslhost.exe manages WSL session lifecycle | WSL 2 | Additional process in the tree; wslhost.exe takes over when wsl.exe exits |

**Key architectural fact:** With WSL 2, Linux processes run inside a lightweight VM. The Windows process tree sees `wsl.exe` and `wslhost.exe` but NOT the Linux shell processes (bash, zsh). This is why we must use `wsl.exe -e` subprocess calls or UIA text inference to get Linux-side context, rather than trying to read the Linux shell's CWD via Windows PEB.

## Open Questions

1. **CWD detection reliability**
   - What we know: `wsl.exe -e pwd` spawns a new process and returns its CWD (home dir), not the active shell's CWD. UIA text inference can parse the prompt but is fragile.
   - What's unclear: Whether there's a reliable way to get the active WSL shell's CWD without relying on prompt parsing.
   - Recommendation: Try `wsl.exe -e pwd` first (fast, reliable for the default case). Fall back to UIA prompt parsing. Accept that CWD may be `None` in some edge cases. **Claude's discretion** per CONTEXT.md.

2. **Performance of wsl.exe subprocess calls**
   - What we know: `wsl.exe -e` can be slow (100-500ms) if WSL is cold-starting. Once running, it's ~50ms.
   - What's unclear: Whether cold-start latency will cause the 750ms detection timeout to expire.
   - Recommendation: Cache WSL running state; if subprocess times out, return `is_wsl = true` with `cwd = None` (graceful degradation). **Claude's discretion** per CONTEXT.md.

3. **Shell type detection accuracy**
   - What we know: `wsl.exe -e sh -c "basename $SHELL"` returns the configured default shell. The user might be running a different shell (e.g., configured for bash but running zsh).
   - What's unclear: Whether we can detect the actual running shell vs the configured default.
   - Recommendation: Use the `infer_shell_from_text` approach on UIA text (already exists for Windows in mod.rs) as primary, fall back to `$SHELL` query. **Claude's discretion** per CONTEXT.md.

## Sources

### Primary (HIGH confidence)
- Existing codebase: `terminal/mod.rs`, `terminal/process.rs`, `terminal/detect_windows.rs`, `commands/ai.rs`, `commands/safety.rs`, `terminal/filter.rs`, `terminal/uia_reader.rs`, `store/index.ts` -- direct code review
- [Microsoft WSL Basic Commands](https://learn.microsoft.com/en-us/windows/wsl/basic-commands) -- wsl.exe CLI reference
- [ConPTY Architecture Blog](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/) -- ConPTY process tree architecture

### Secondary (MEDIUM confidence)
- [WSL Technical Documentation - wslhost.exe](https://wsl.dev/technical-documentation/wslhost.exe/) -- process lifecycle management
- [WSL Process Tree Detection Rules](https://detection.fyi/elastic/detection-rules/windows/defense_evasion_wsl_child_process/) -- security detection for wsl.exe child processes
- [WSL GitHub Issue #6881](https://github.com/microsoft/WSL/issues/6881) -- Linux processes not visible in Windows Task Manager (confirms WSL 2 architecture)

### Tertiary (LOW confidence)
- [wsl-dirutils crate](https://crates.io/crates/wsl-dirutils) -- Rust crate for WSL directory translation (not using, but confirms the CWD challenge)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- using existing codebase dependencies, no new crates needed
- Architecture: HIGH -- extending well-understood existing patterns (process tree walk, context struct, system prompts)
- Pitfalls: MEDIUM -- CWD detection from WSL is inherently imperfect; subprocess timeout behavior needs runtime validation
- WSL detection: HIGH -- finding wsl.exe in process ancestry is straightforward with existing CreateToolhelp32Snapshot infrastructure

**Research date:** 2026-03-09
**Valid until:** 2026-04-09 (WSL architecture is stable)
