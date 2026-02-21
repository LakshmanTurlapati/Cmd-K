# Phase 3: Terminal Context Reading - Research

**Researched:** 2026-02-21
**Domain:** macOS Accessibility API (AXUIElement), process inspection (libproc/darwin-libproc), terminal detection
**Confidence:** MEDIUM (AX tree traversal for Terminal.app/iTerm2 HIGH; GPU-rendered terminals MEDIUM; CWD via darwin-libproc HIGH)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- Support all 5 terminals equally: Terminal.app, iTerm2, Alacritty, kitty, WezTerm
- Detect through tmux and screen sessions running within those terminals
- Detect the frontmost terminal only (whichever was last active before overlay opened)
- For iTerm2 with multiple tabs/panes, detect the active tab/pane specifically
- If the frontmost app is not a terminal, provide no context (overlay opens with empty context, AI still works)
- Detection is on-demand only -- triggered when Cmd+K is pressed, no background polling
- Detect shell type (bash, zsh, fish, etc.) in addition to CWD and output
- Detect running process (e.g., 'node server.js', 'python script.py')
- Silent fallback for unsupported/unknown terminals -- no user notification
- Capture visible screen content only (no scrollback)
- Include command prompts and typed commands along with output
- Structured parsing: identify individual commands and their outputs as separate entries
- Filter sensitive data (API keys, passwords, tokens) before sending to AI
- Use Accessibility tree (AXUIElement) as primary read method, with per-terminal fallbacks
- Hard timeout on detection -- overlay opens with whatever was captured within the time limit
- Do NOT read directory listings or git context -- just CWD path, visible output, shell type, and running process
- Show only the detected shell type (e.g., 'zsh') as subtle text below the input field
- Shell type label appears to the left: `zsh` (no CWD path shown to user)
- CWD, terminal output, running process are captured internally for AI but NOT displayed in the overlay
- Subtle spinner during detection; if detection times out or fails, hide the shell area entirely
- When no context is available (non-terminal app), hide the context area completely -- no placeholder
- Overlay height adjusts: slightly taller when shell type is shown, shorter without it
- No manual override of detected context
- AI works regardless of whether context was captured
- Partial detection: use whatever was captured (partial context better than none)
- Accessibility permission denied: persistent banner in overlay saying 'Enable Accessibility for terminal context' with click action that opens macOS System Settings directly
- Banner is always visible until Accessibility permission is granted (not dismissable)
- Permission re-checked each time overlay opens (banner disappears once granted)
- Debug logging for detection failures (internal, not user-facing)
- AI still works without any terminal context for general command questions

### Claude's Discretion

- Exact hard timeout duration for detection
- Accessibility tree traversal strategy per terminal
- Structured parsing heuristics for identifying commands vs output
- Sensitive data pattern matching implementation
- Spinner design and animation details

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TERM-02 | App detects the current working directory of the active terminal without shell plugins | darwin-libproc crate `pid_cwd()` using `PROC_PIDVNODEPATHINFO` via macOS libproc; process PID obtained from AXUIElement or TTY inspection |
| TERM-03 | App reads recent terminal output for context without shell plugins | AX tree traversal using `AXUIElementCopyAttributeValue` with `kAXValueAttribute` on `AXTextArea`/`AXScrollArea` children of frontmost terminal window |
| TERM-04 | Works with Terminal.app, iTerm2, Alacritty, kitty, WezTerm | Bundle ID detection for terminal identification; AX tree works for Terminal.app/iTerm2; GPU-rendered terminals (Alacritty, kitty, WezTerm) have limited/no AX text exposure, requiring fallback strategy |
</phase_requirements>

## Summary

The core challenge of this phase is that macOS offers no single universal API for reading terminal state. The implementation requires combining two distinct systems: the macOS Accessibility API (AXUIElement) for reading visible window text content, and process inspection via `darwin-libproc` for reading the current working directory and foreground process information.

The Accessibility API approach works well for Terminal.app and iTerm2, which expose their text area content through standard AX attributes (`kAXValueAttribute` on `AXTextArea` elements reachable by walking the window's `AXScrollArea` children). However, the three GPU-rendered terminals (Alacritty, kitty, WezTerm) render directly to OpenGL/Metal and intentionally provide minimal accessibility tree exposure for text content. This is a confirmed architectural limitation; the WezTerm issue tracker explicitly documents that accessibility support is unimplemented as of late 2024. For these terminals, the fallback must rely entirely on process inspection (CWD, foreground process name, shell type derived from process argv) -- visible output will not be available.

The CWD path is obtained reliably across all five terminals via `darwin-libproc 0.2.0` which wraps macOS libproc's `proc_pidinfo(PROC_PIDVNODEPATHINFO)` call. The flow is: (1) identify the frontmost app's bundle ID, (2) confirm it is a known terminal, (3) get the terminal app's PID from the AXUIElement, (4) walk child processes (the foreground process group) to find the shell/running process PID via `sysctl`/`ioctl(TIOCGPGRP)`, (5) call `darwin_libproc::pid_cwd(shell_pid)` to get the working directory path.

**Primary recommendation:** Implement a two-tier detection strategy: AX text read (Tier 1, for Terminal.app and iTerm2) and process-only read (Tier 2, fallback for GPU-rendered terminals). Both tiers share the same CWD + process-name extraction logic using darwin-libproc.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| darwin-libproc | 0.2.0 | Get process CWD and path from PID via macOS libproc | Only crate that wraps `PROC_PIDVNODEPATHINFO` in safe Rust; `libproc-rs` explicitly marks `pidcwd` as unimplemented on macOS |
| accessibility-sys | 0.2.0 | Raw FFI bindings for all AXUIElement functions and constants | Complete FFI coverage (`AXUIElementCreateApplication`, `AXUIElementCopyAttributeValue`, all kAX* constants); used directly via `extern "C"` as already done in `permissions.rs` |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde / serde_json | 1 (already present) | Serialize TerminalContext struct to pass to frontend via Tauri IPC | Already in Cargo.toml; used for all IPC payloads |
| regex | 1.x | Compile sensitive-data filter patterns once at startup | For filtering API keys, passwords, tokens from captured output before sending to AI |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| darwin-libproc | libproc (andrewdavidmackenzie) | libproc marks `pidcwd` as unimplemented on macOS; darwin-libproc is macOS-only and provides it |
| darwin-libproc | Custom FFI to libproc.h | More control but darwin-libproc already provides safe wrappers with PROC_PIDVNODEPATHINFO |
| accessibility-sys direct FFI | accessibility crate (eiz) | eiz/accessibility "safe bindings are pretty spotty"; accessibility-sys is complete |
| accessibility-sys direct FFI | objc2-application-services | objc2 does not yet expose AXUIElement (open issue #624 on madsmtm/objc2); accessibility-sys is correct choice |
| regex | Custom string scan | Secrets-Patterns-DB documents 1600+ patterns; regex crate handles entropy-style detection with standard patterns |

**Installation:**
```bash
# In src-tauri/Cargo.toml [dependencies]:
darwin-libproc = "0.2"
accessibility-sys = "0.2"
regex = "1"
```

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/
├── commands/
│   ├── terminal.rs        # New: Tauri IPC command get_terminal_context
│   └── mod.rs             # Add pub mod terminal
├── terminal/
│   ├── mod.rs             # TerminalContext struct + pub fn detect()
│   ├── detect.rs          # Frontmost app detection + bundle ID → terminal type
│   ├── ax_reader.rs       # AX tree walk for Terminal.app + iTerm2 text
│   ├── process.rs         # Process inspection: CWD, foreground PID, shell type
│   └── filter.rs          # Sensitive data regex filtering
└── state.rs               # (existing -- no change needed)
```

### Pattern 1: Frontmost Terminal Detection
**What:** Before any AX or process work, confirm the previously-frontmost app is a known terminal.
**When to use:** Called once per Cmd+K press (on-demand, no polling).

The key insight: when our NSPanel gains focus via `show_and_make_key()`, `NSWorkspace.sharedWorkspace().frontmostApplication` will return our app. We must capture the previous frontmost application BEFORE showing the panel. The correct approach is to capture the pre-panel app in the hotkey handler.

```rust
// Source: NSWorkspace.frontmostApplication -- Apple Developer Documentation
// Pattern from existing AXIsProcessTrusted FFI in permissions.rs

use std::ffi::CStr;

const TERMINAL_BUNDLE_IDS: &[&str] = &[
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "io.alacritty",
    "net.kovidgoyal.kitty",
    "com.github.wez.wezterm",
];

// Called via extern "C" block (same pattern as AXIsProcessTrusted)
extern "C" {
    fn AXUIElementCreateApplication(pid: i32) -> *mut std::ffi::c_void;
    fn AXUIElementCopyAttributeValue(
        element: *const std::ffi::c_void,
        attribute: *const std::ffi::c_void,
        value: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn AXUIElementGetPid(element: *const std::ffi::c_void, pid: *mut i32) -> i32;
    fn AXUIElementSetMessagingTimeout(element: *const std::ffi::c_void, timeout_in_seconds: f32) -> i32;
}
```

### Pattern 2: CWD via darwin-libproc
**What:** Given a shell process PID, call darwin-libproc to get the CWD.
**When to use:** After identifying the foreground process PID from the terminal window.

```rust
// Source: darwin-libproc 0.2.0 docs.rs
// https://docs.rs/darwin-libproc/0.2.0/x86_64-apple-darwin/darwin_libproc/

use darwin_libproc;

pub fn get_process_cwd(pid: i32) -> Option<std::path::PathBuf> {
    darwin_libproc::pid_cwd(pid).ok()
}

pub fn get_process_name(pid: i32) -> Option<String> {
    darwin_libproc::pid_path(pid)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}
```

### Pattern 3: AX Tree Walk for Terminal Text
**What:** Walk the AX tree of a terminal window to find and read the text content.
**When to use:** For Terminal.app and iTerm2 only (GPU-rendered terminals do not expose text).

The AX tree structure for Terminal.app:
```
AXApplication
  AXWindow (frontmost window)
    AXScrollArea
      AXTextArea  <-- kAXValueAttribute contains visible terminal text
```

For iTerm2 with tabs/panes, the active split pane is in `kAXFocusedUIElementAttribute` of the application.

```rust
// Source: accessibility-sys 0.2.0, Apple AX documentation
// kAXFocusedWindowAttribute -> window -> walk AXScrollArea -> AXTextArea -> kAXValueAttribute

use accessibility_sys::{
    AXUIElementCreateApplication, AXUIElementCopyAttributeValue,
    AXUIElementSetMessagingTimeout,
    kAXFocusedWindowAttribute, kAXChildrenAttribute, kAXValueAttribute,
    kAXRoleAttribute,
};
use core_foundation::{
    base::{CFRelease, TCFType},
    string::CFString,
    array::CFArray,
};

// Set timeout per element to avoid blocking indefinitely
// AXUIElementSetMessagingTimeout(app_element, 1.0);  // 1 second per AX call
```

### Pattern 4: Foreground Process PID from TTY
**What:** After getting the terminal app PID, find the foreground shell/process running inside.
**When to use:** For all 5 terminals (CWD and shell type depend on this).

Strategy: use `sysctl` with `CTL_KERN` / `KERN_PROC` / `KERN_PROC_ALL` to enumerate child processes, OR use `ioctl(fd, TIOCGPGRP)` on the terminal's TTY device. The simpler and more reliable approach is to use `darwin_libproc::all_pids()` and then check `ppid_only_pids(terminal_pid)` to find the shell (the direct child process of the terminal app process).

```rust
// Source: darwin-libproc 0.2.0 -- ppid_only_pids function
// Gets all PIDs whose parent matches terminal_pid (gives the shell process)

pub fn find_shell_pid(terminal_pid: i32) -> Option<i32> {
    darwin_libproc::ppid_only_pids(terminal_pid)
        .ok()?
        .into_iter()
        .next()  // The first child is the shell
}

// Shell type from process path
pub fn get_shell_type(shell_pid: i32) -> Option<String> {
    darwin_libproc::pid_path(shell_pid)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    // Returns "zsh", "bash", "fish", etc.
}
```

### Pattern 5: Sensitive Data Filtering
**What:** Remove API keys, passwords, tokens from captured output before sending to AI.
**When to use:** Applied to all terminal text before inclusion in TerminalContext.

```rust
// Source: secrets-patterns-db / gitleaks pattern library
use regex::Regex;
use once_cell::sync::Lazy;

static SENSITIVE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // AWS Access Key
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        // Generic high-entropy token patterns (16+ alphanum)
        Regex::new(r#"(?i)(api[_-]?key|token|password|secret|bearer)\s*[=:]\s*['"]?[a-zA-Z0-9+/]{16,}['"]?"#).unwrap(),
        // xAI / OpenAI style tokens
        Regex::new(r"xai-[a-zA-Z0-9]{32,}").unwrap(),
        Regex::new(r"sk-[a-zA-Z0-9]{32,}").unwrap(),
        // Private keys
        Regex::new(r"-----BEGIN [A-Z]+ PRIVATE KEY-----").unwrap(),
    ]
});

pub fn filter_sensitive(text: &str) -> String {
    let mut result = text.to_string();
    for pattern in SENSITIVE_PATTERNS.iter() {
        result = pattern.replace_all(&result, "[REDACTED]").into_owned();
    }
    result
}
```

### Pattern 6: Hard Timeout with tokio::time or std::thread
**What:** Enforce detection deadline so overlay never stalls.
**When to use:** Wrapping the entire `detect()` call before showing the overlay.

The recommended approach is a `std::thread::spawn` with a `channel` and `recv_timeout`, since the AX API calls are synchronous C FFI and cannot be cancelled mid-call. Using `tokio` would require the async runtime.

```rust
use std::sync::mpsc;
use std::time::Duration;

pub fn detect_with_timeout(timeout_ms: u64) -> Option<TerminalContext> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = detect(); // synchronous detection
        let _ = tx.send(result);
    });
    rx.recv_timeout(Duration::from_millis(timeout_ms)).ok().flatten()
}
```

### Anti-Patterns to Avoid
- **Calling NSWorkspace.frontmostApplication after show_and_make_key():** After the panel gains key status, frontmostApplication returns our app. Capture the previous app PID BEFORE calling show_and_make_key.
- **Polling for frontmost app changes:** The CONTEXT.md explicitly locks detection to on-demand only. No background polling.
- **Using libproc-rs (andrewdavidmackenzie) for CWD:** Its `pidcwd()` is explicitly unimplemented on macOS and returns an error. Use darwin-libproc instead.
- **Relying on AX text for Alacritty/kitty/WezTerm:** These GPU-rendered terminals do not expose terminal text via AXUIElement. WezTerm accessibility is documented as unimplemented. Plan for process-only fallback for these three.
- **Calling AXUIElementCopyAttributeValue without setting timeout:** Can deadlock or hang if the terminal is busy or unresponsive. Always set `AXUIElementSetMessagingTimeout` to ~1 second per element.
- **Blocking the main thread during detection:** Detection must run on a background thread; Tauri commands that block the main thread cause UI hangs.
- **Using `serde` to serialize raw C pointers or AXUIElementRef:** These are not Send + Sync; serialize to String/struct before returning to Tauri.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Get process CWD by PID on macOS | Custom `proc_pidinfo` FFI struct | `darwin-libproc::pid_cwd()` | darwin-libproc wraps `PROC_PIDVNODEPATHINFO` with safe Rust and correct struct alignment |
| List child PIDs of a process | Custom sysctl loop | `darwin_libproc::ppid_only_pids()` | macOS sysctl struct layout is tricky; darwin-libproc handles it |
| AXUIElement function signatures | Custom FFI declarations | `accessibility-sys` crate | Complete, versioned, correct CFTypeRef handling |
| Secret pattern detection | Custom regex list | Use patterns from secrets-patterns-db (GitHub: mazen160/secrets-patterns-db) | 1600+ verified patterns; rolling your own misses provider-specific formats |

**Key insight:** macOS process introspection requires C-level struct access (`proc_vnodepathinfo`, `PROC_PIDVNODEPATHINFO`) that is tricky to get right via raw FFI. darwin-libproc provides the only maintained, correct Rust wrapper for this specific call.

## Common Pitfalls

### Pitfall 1: Frontmost App Captured Too Late
**What goes wrong:** `NSWorkspace.sharedWorkspace().frontmostApplication` returns CMD+K itself after the panel gains focus.
**Why it happens:** `panel.show_and_make_key()` activates the panel; after this point the frontmost app is our overlay.
**How to avoid:** Capture `frontmostApplication.processIdentifier` in the hotkey handler BEFORE calling `show_overlay`. Store it in AppState so the detection command can read it.
**Warning signs:** The bundleIdentifier always equals `com.yourapp.cmdkapp`; CWD always points to your app's working directory.

### Pitfall 2: GPU-Rendered Terminals Have No AX Text
**What goes wrong:** Walking the AX tree of Alacritty, kitty, or WezTerm returns no text content, only structural nodes.
**Why it happens:** These terminals render screen content via OpenGL/Metal as pixel operations, not as platform text views. They intentionally have no NSTextView/AXTextArea exposing content. Confirmed: WezTerm GitHub issue #913 (Aug 2024) documents that accessibility is unimplemented.
**How to avoid:** Detect terminal type from bundle ID. For `io.alacritty`, `net.kovidgoyal.kitty`, `com.github.wez.wezterm` -- skip AX text read entirely; fall back to process-only context (CWD + shell type + running process name).
**Warning signs:** `AXUIElementCopyAttributeValue` for `kAXChildrenAttribute` returns `kAXErrorNoValue` or an empty array for the main window.

### Pitfall 3: darwin-libproc vs libproc-rs Confusion
**What goes wrong:** Adding `libproc` crate and calling `pidcwd()` which silently returns `Err` on macOS.
**Why it happens:** libproc-rs (andrewdavidmackenzie/libproc-rs) has `pidcwd()` that returns `unimplemented` error on macOS -- Linux-only. This is documented in the source at `libproc/proc_pid.rs`.
**How to avoid:** Use `darwin-libproc = "0.2"` (heim-rs/darwin-libproc) which wraps `vnode_path_info()` correctly.
**Warning signs:** `pidcwd` always returns `Err("not implemented for macos")`.

### Pitfall 4: AXUIElement Messaging Timeout Not Set
**What goes wrong:** Detection hangs for 30+ seconds on a busy or unresponsive terminal.
**Why it happens:** Default AXUIElement messaging timeout is 6 seconds per call; walking a large AX tree multiplies this.
**How to avoid:** Call `AXUIElementSetMessagingTimeout(element, 1.0)` on the application element immediately after `AXUIElementCreateApplication`. The hard detection timeout (from Pattern 6) provides an outer bound, but per-element timeouts prevent accumulation.
**Warning signs:** Detection sporadically takes 5-10 seconds; users see spinner for a long time.

### Pitfall 5: Alacritty Accessibility Permission Behavior Changed in macOS Sequoia
**What goes wrong:** `AXIsProcessTrusted()` returns true but AX calls to Alacritty fail.
**Why it happens:** Alacritty has known issues with Accessibility permissions in macOS Sequoia (15.x) per GitHub issue #8493 -- the permission reporting diverges from actual access grants.
**How to avoid:** Treat `kAXErrorCannotComplete` and `kAXErrorNotImplemented` return codes from AX calls as silent fallback triggers, not as errors that surface to the user.
**Warning signs:** AX calls to Alacritty fail with non-zero AXError codes despite permission being granted.

### Pitfall 6: tmux/screen Process Hierarchy Depth
**What goes wrong:** `ppid_only_pids(terminal_pid)` returns the tmux client process, not the shell running inside the pane.
**Why it happens:** tmux inserts an extra layer: `terminal_app (pid A) -> tmux client (pid B) -> tmux server (pid C) -> shell (pid D)`. `ppid_only_pids` only walks one level.
**How to avoid:** After finding the immediate child, check if it is `tmux` or `screen` by name. If so, walk deeper: use `darwin_libproc::pid_path()` and inspect process name. For the shell CWD inside tmux, walk the full child tree to find the actual shell PID. Alternatively, filter process names: if process name is `tmux`, recurse one more level.
**Warning signs:** CWD shows `/tmp/tmux-<uid>/<session>` instead of the actual project directory.

### Pitfall 7: Shell Type Derived From $SHELL vs Actual Running Process
**What goes wrong:** Using `$SHELL` environment variable returns the user's default shell even if they are running a different shell in the current terminal.
**Why it happens:** `$SHELL` is set at login, not updated when user runs `fish` or `zsh` interactively over `bash`.
**How to avoid:** Use `darwin_libproc::pid_path(foreground_pid)` to get the actual binary path, then extract the filename. This reflects the actual running shell binary, not the login default.
**Warning signs:** Shell label shows `bash` in overlay but user is running `fish`.

## Code Examples

Verified patterns from official sources:

### Full Detection Pipeline (pseudocode, verified APIs)
```rust
// Source: darwin-libproc 0.2.0 + accessibility-sys 0.2.0 + NSWorkspace docs

pub struct TerminalContext {
    pub shell_type: Option<String>,     // "zsh", "bash", "fish"
    pub cwd: Option<String>,            // "/Users/foo/projects/bar"
    pub visible_output: Option<String>, // filtered terminal text
    pub running_process: Option<String>,// "node server.js" if not just the shell
}

pub fn detect(previous_app_pid: i32) -> Option<TerminalContext> {
    // 1. Get bundle ID of previous frontmost app
    let bundle_id = get_bundle_id(previous_app_pid)?;

    // 2. Confirm it is a known terminal
    if !TERMINAL_BUNDLE_IDS.contains(&bundle_id.as_str()) {
        return None;  // Not a terminal, overlay works without context
    }

    // 3. Determine terminal type for per-terminal strategy
    let is_gpu_terminal = matches!(
        bundle_id.as_str(),
        "io.alacritty" | "net.kovidgoyal.kitty" | "com.github.wez.wezterm"
    );

    // 4. Find foreground process (shell or running command)
    // Walk from terminal app PID -> child -> (skip tmux if present) -> shell
    let (shell_pid, running_process_pid) = find_foreground_pid(previous_app_pid)?;

    // 5. Get CWD (works for all terminals via darwin-libproc)
    let cwd = darwin_libproc::pid_cwd(shell_pid)
        .ok()
        .map(|p| p.to_string_lossy().into_owned());

    // 6. Get shell type from process binary name
    let shell_type = darwin_libproc::pid_path(shell_pid)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));

    // 7. Get running process name (if different from shell)
    let running_process = if running_process_pid != shell_pid {
        darwin_libproc::name(running_process_pid).ok()
    } else {
        None
    };

    // 8. Read visible terminal text (AX only for Terminal.app and iTerm2)
    let visible_output = if !is_gpu_terminal {
        read_ax_terminal_text(previous_app_pid)
            .map(|text| filter_sensitive(&text))
    } else {
        None  // Silent fallback -- no text for GPU terminals
    };

    Some(TerminalContext { shell_type, cwd, visible_output, running_process })
}
```

### AXUIElement Text Read Pattern
```rust
// Source: accessibility-sys 0.2.0, AX API documented in AXAttributeConstants.h
// Walks: AXApplication -> AXFocusedWindow -> AXScrollArea -> AXTextArea -> kAXValueAttribute

use accessibility_sys::*;
use core_foundation::string::CFString;

unsafe fn read_ax_terminal_text(app_pid: i32) -> Option<String> {
    // Create application element
    let app_el = AXUIElementCreateApplication(app_pid);
    if app_el.is_null() { return None; }

    // Set per-element timeout to prevent hangs
    AXUIElementSetMessagingTimeout(app_el as _, 1.0);

    // Get focused window
    let mut window: *mut std::ffi::c_void = std::ptr::null_mut();
    let attr = CFString::new("AXFocusedWindow");
    let err = AXUIElementCopyAttributeValue(app_el as _, attr.as_concrete_TypeRef() as _, &mut window);
    if err != 0 || window.is_null() { return None; }

    // Get children of window (looking for AXScrollArea)
    // ... recursive walk for AXTextArea, read kAXValueAttribute ...
    // Return the CFString value as a Rust String
    todo!() // detailed impl in task
}
```

### darwin-libproc CWD Read
```rust
// Source: darwin-libproc 0.2.0
// https://docs.rs/darwin-libproc/0.2.0/x86_64-apple-darwin/darwin_libproc/fn.pid_cwd.html

use darwin_libproc;

pub fn get_cwd_for_pid(pid: i32) -> Option<String> {
    darwin_libproc::pid_cwd(pid)
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

// Shell type (binary name from full path)
pub fn get_shell_type_for_pid(pid: i32) -> Option<String> {
    darwin_libproc::pid_path(pid)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}
```

### Pre-Panel Frontmost App Capture (AppState addition)
```rust
// In state.rs, add field to capture the pre-panel app PID:
pub struct AppState {
    // ... existing fields ...
    /// PID of the frontmost app captured BEFORE showing overlay.
    /// Populated in hotkey handler before show_and_make_key().
    pub previous_app_pid: Mutex<Option<i32>>,
}

// In hotkey.rs, before toggle_overlay:
// Capture frontmost app before showing panel
// Uses NSWorkspace via extern "C" (same pattern as AXIsProcessTrusted)
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Shell plugin (.zshrc precmd hook) | AX API + process inspection | Required by zero-shell-setup constraint | No user configuration needed |
| `libproc-rs pidcwd()` on macOS | `darwin-libproc::pid_cwd()` | libproc-rs never implemented for macOS | darwin-libproc is the correct crate |
| objc2 AXUIElement bindings | accessibility-sys raw FFI | objc2 issue #624 (open as of 2025) | accessibility-sys is complete and stable |
| Reading scrollback buffer | Visible screen only | User decision in CONTEXT.md | Simpler, lower latency, no scrollback API needed |

**Deprecated/outdated:**
- `libproc` crate (andrewdavidmackenzie/libproc-rs) for macOS CWD: `pidcwd()` returns error; use darwin-libproc instead
- `tauri-plugin-macos-permissions`: already decided against in Phase 2 (using `extern "C"` block directly for AXIsProcessTrusted)
- macos-accessibility-client crate: archived Jan 2026 on GitHub, migrated to Codeberg; not recommended for new work

## Open Questions

1. **Exact tree depth for iTerm2 active pane**
   - What we know: iTerm2 exposes a `kAXFocusedUIElementAttribute` at the app level that should point to the active text area directly
   - What's unclear: Does it return the AXTextArea directly or an intermediate split-pane container? May require empirical testing with Accessibility Inspector.
   - Recommendation: Plan for a task that validates AX tree structure with Accessibility Inspector (Xcode instrument) before writing the traversal code.

2. **darwin-libproc `ppid_only_pids` for tmux hierarchy**
   - What we know: tmux creates a multi-level process tree; `ppid_only_pids(terminal_pid)` gives direct children only
   - What's unclear: The exact number of levels to walk for tmux server -> shell. May be 2 or 3 levels depending on tmux client/server architecture.
   - Recommendation: Add a `max_depth` loop (3 iterations) in the process walk; stop when process name is a known shell.

3. **Hard timeout duration**
   - What we know: Per-element AX timeout is configurable via `AXUIElementSetMessagingTimeout`; detection spawns a background thread
   - What's unclear: Ideal outer timeout (user decision deferred to Claude). Too short misses data; too long stalls overlay open.
   - Recommendation: Use 500ms outer timeout with per-element 1s AX timeout. The outer thread kill ensures the overlay appears quickly; per-element timeout prevents any single AX call from blocking the thread for the full 500ms.

4. **WezTerm bundle ID confirmation**
   - What we know: WezTerm's bundle ID is commonly listed as `com.github.wez.wezterm`
   - What's unclear: Whether this is consistent across all install methods (Homebrew, direct download, nightly builds)
   - Recommendation: Verify empirically; may need to check `wezterm.app/Contents/Info.plist` on test machine.

## Sources

### Primary (HIGH confidence)
- darwin-libproc 0.2.0 `docs.rs` - `pid_cwd`, `pid_path`, `ppid_only_pids` functions confirmed present
- accessibility-sys 0.2.0 `docs.rs` - `AXUIElementCreateApplication`, `AXUIElementCopyAttributeValue`, `AXUIElementSetMessagingTimeout`, `kAXValueAttribute`, `kAXFocusedWindowAttribute`, `kAXChildrenAttribute` all confirmed present
- `AXUIElementSetMessagingTimeout` - Apple Developer Documentation (confirmed function exists)
- `AXUIElementGetPid` - Apple Developer Documentation (confirmed function exists)
- libproc-rs source `proc_pid.rs` on docs.rs - `pidcwd` for macOS explicitly returns error, confirmed unimplemented
- darwin-libproc `pid_cwd.rs` on GitHub - uses `vnode_path_info(pid)?.pvi_cdir.vip_path`, confirmed implementation

### Secondary (MEDIUM confidence)
- WezTerm GitHub issue #913 (verified via WebFetch) - accessibility is unimplemented as of Aug 2024
- Bundle ID list (NSWorkspace frontmostApplication pattern) - `com.apple.Terminal`, `com.googlecode.iterm2`, `io.alacritty`, `net.kovidgoyal.kitty`, `com.github.wez.wezterm` -- sourced from community; `com.github.wez.wezterm` unverified
- secrets-patterns-db (mazen160) - 1600+ regex patterns for secret detection; patterns referenced are standard (AWS AKIA, sk- prefix, generic api_key pattern)
- Alacritty GitHub issue #8493 - Accessibility permissions broken in macOS Sequoia

### Tertiary (LOW confidence)
- iTerm2 AX tree structure (AXScrollArea -> AXTextArea) - inferred from general macOS AX patterns + Hammerspoon docs; needs empirical validation with Accessibility Inspector
- tmux process depth (2-3 levels) - inferred from TTY/process group documentation; exact depth needs testing
- WezTerm bundle ID `com.github.wez.wezterm` - from community sources only; needs verification

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - darwin-libproc and accessibility-sys are the correct, documented Rust crates for these exact APIs; confirmed via docs.rs
- Architecture: MEDIUM - AX tree walk patterns are well-documented for Cocoa apps; terminal-specific tree structure needs empirical validation for iTerm2 panes; GPU terminal limitation confirmed
- Pitfalls: HIGH - frontmostApplication timing pitfall is architecture-level (confirmed); GPU terminal AX limitation confirmed via WezTerm issue tracker; darwin-libproc vs libproc-rs confusion confirmed via source code

**Research date:** 2026-02-21
**Valid until:** 2026-04-21 (stable macOS APIs; darwin-libproc at 0.2.0 is small/stable; terminal AX behavior unlikely to change)
