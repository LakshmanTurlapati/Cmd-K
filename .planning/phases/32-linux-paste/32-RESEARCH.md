# Phase 32: Linux Paste - Research

**Researched:** 2026-03-14
**Domain:** Linux X11/Wayland clipboard and keystroke simulation via xdotool/xclip
**Confidence:** HIGH

## Summary

Phase 32 implements the final piece of the Linux Ctrl+K workflow: pasting the AI-generated command into the active terminal. The approach uses xdotool for keystroke simulation (Ctrl+U to clear, Ctrl+Shift+V to paste, Return to confirm) and xclip/wl-copy for clipboard writing. The existing `paste.rs` already has well-defined Linux stubs at three locations (`write_to_clipboard`, `paste_to_terminal`, `confirm_terminal_command`) that return errors -- these need to be replaced with real implementations.

The implementation follows the same pattern as macOS (osascript + CGEventPost) and Windows (arboard + SendInput): activate terminal window, clear line, write to clipboard, simulate paste keystroke. The key difference is that Linux uses external CLI tools (xdotool, xclip) rather than native APIs. A graceful fallback chain ensures the app always works: xdotool auto-paste > clipboard + hint > Tauri clipboard + hint.

**Primary recommendation:** Implement Linux paste as three subprocess-based helpers (xdotool for keystrokes, xclip/wl-copy for clipboard) with tool availability cached at startup and a fallback chain that degrades to clipboard-only with UI hint.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- X11 paste method: Clipboard + Ctrl+Shift+V approach (not xdotool type)
- Write to clipboard via xclip (`xclip -selection clipboard`), clipboard selection only
- xdotool sends `key ctrl+shift+v` to simulate terminal paste shortcut
- After paste, overlay stays visible and refocuses (same as macOS/Windows)
- Clear current shell line with `xdotool key ctrl+u` before pasting (matches macOS/Windows Ctrl+U pattern)
- 50ms delay between Ctrl+U and Ctrl+Shift+V for shell processing
- Confirm command uses `xdotool key Return`
- Detect Wayland via `WAYLAND_DISPLAY` env var + `XDG_SESSION_TYPE=wayland`
- If `GDK_BACKEND=x11` is set, treat as X11 (XWayland path) even on Wayland
- Wayland fallback: copy to clipboard, show inline hint in overlay "press Ctrl+Shift+V to paste"
- Wayland clipboard: try `wl-copy` first, fall back to xclip (via XWayland)
- Hint persists until user dismisses overlay (Escape or Ctrl+K)
- Wayland confirm fallback: show "press Enter" hint (same inline pattern)
- Check xdotool and xclip/wl-copy availability once at app startup, cache results
- If xdotool missing on X11: graceful fallback to clipboard + hint (same as Wayland path)
- If xclip/wl-copy both missing: use Tauri clipboard API as final fallback
- No hard errors -- app always has a working paste path

### Claude's Discretion
- Exact xdotool window activation approach (windowactivate vs windowfocus)
- Focus restoration timing and delays
- How to structure the Linux cfg block in paste.rs (single function vs helper modules)
- Error logging granularity

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LPST-01 | Auto-paste into active terminal via xdotool keystroke simulation on X11 | xdotool `key ctrl+shift+v` after writing to clipboard via xclip; windowactivate for focus; existing stubs in paste.rs at lines 217-221 and 626-629 |
| LPST-02 | Wayland graceful fallback -- copies to clipboard with "press Ctrl+Shift+V" hint | Env var detection (WAYLAND_DISPLAY + XDG_SESSION_TYPE), wl-copy/xclip clipboard write, new `pasteHint` field in return value or store state for frontend hint |
| LPST-03 | Destructive command detection works with Linux-specific patterns (already built) | Existing destructive detection already handles Linux commands; no new work needed, just verify the Linux paste path honors `isDestructive` flow |
</phase_requirements>

## Standard Stack

### Core
| Tool | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| xdotool | 3.x | X11 keystroke simulation | De facto standard for X11 automation; used by every Linux automation tool |
| xclip | 0.13+ | X11 clipboard write | Standard CLI clipboard tool, maps to X11 selections |
| wl-copy | 2.x | Wayland clipboard write | Part of wl-clipboard, standard Wayland clipboard utility |
| x11rb | 0.13 | X11 window queries | Already a dependency; used in Phase 31 for active window PID |

### Supporting
| Tool | Purpose | When to Use |
|------|---------|-------------|
| arboard | Rust clipboard crate | Final fallback when xclip and wl-copy are both missing |
| std::process::Command | Subprocess spawning | All xdotool/xclip/wl-copy invocations |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| xclip CLI | arboard crate directly | arboard uses X11 internally but xclip is more reliable for clipboard persistence after process exit |
| xdotool CLI | x11rb + XTEST extension | Native Rust but significantly more code; xdotool handles edge cases |
| wl-copy | wl-clipboard-rs crate | Adds compile dep; CLI tool is simpler and already validated |

## Architecture Patterns

### Recommended Structure in paste.rs

Keep all Linux code in `paste.rs` using `#[cfg(target_os = "linux")]` blocks at function level (matches existing macOS/Windows pattern). No separate module needed.

```
paste.rs additions:
  - write_to_clipboard (linux cfg) -- replaces stub at line 160-164
  - paste_to_terminal linux path -- replaces stub at lines 217-221
  - paste_to_terminal_linux() -- new helper function
  - confirm_terminal_command linux path -- replaces stub at lines 626-629
  - confirm_command_linux() -- new helper function
  - detect_display_server() -- returns X11 | Wayland | Unknown
  - LinuxToolAvailability struct -- cached tool checks
```

### Pattern 1: Display Server Detection
**What:** Detect whether running on X11 or Wayland to choose paste strategy
**When to use:** Called at paste time (not cached -- user could switch sessions)

```rust
#[cfg(target_os = "linux")]
enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

#[cfg(target_os = "linux")]
fn detect_display_server() -> DisplayServer {
    // GDK_BACKEND=x11 overrides everything (XWayland path)
    if std::env::var("GDK_BACKEND").map(|v| v == "x11").unwrap_or(false) {
        return DisplayServer::X11;
    }
    // Check for Wayland indicators
    let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
    let session_wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|v| v == "wayland")
        .unwrap_or(false);
    if wayland_display && session_wayland {
        return DisplayServer::Wayland;
    }
    // Default to X11 if DISPLAY is set
    if std::env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }
    DisplayServer::Unknown
}
```

### Pattern 2: Tool Availability Caching
**What:** Check once at startup whether xdotool, xclip, wl-copy are available
**When to use:** Cached in AppState, checked before paste operations

```rust
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Default)]
pub struct LinuxToolAvailability {
    pub has_xdotool: bool,
    pub has_xclip: bool,
    pub has_wl_copy: bool,
}

#[cfg(target_os = "linux")]
impl LinuxToolAvailability {
    pub fn detect() -> Self {
        Self {
            has_xdotool: Self::command_exists("xdotool"),
            has_xclip: Self::command_exists("xclip"),
            has_wl_copy: Self::command_exists("wl-copy"),
        }
    }

    fn command_exists(cmd: &str) -> bool {
        std::process::Command::new("which")
            .arg(cmd)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}
```

### Pattern 3: Subprocess Clipboard Write
**What:** Write text to clipboard via piping to xclip or wl-copy stdin
**When to use:** Before simulating Ctrl+Shift+V

```rust
#[cfg(target_os = "linux")]
fn write_to_clipboard_linux(command: &str, tools: &LinuxToolAvailability, display: &DisplayServer) {
    use std::io::Write;

    let (cmd, args): (&str, Vec<&str>) = match display {
        DisplayServer::Wayland if tools.has_wl_copy => ("wl-copy", vec![]),
        _ if tools.has_xclip => ("xclip", vec!["-selection", "clipboard"]),
        _ => {
            // Final fallback: arboard (already a dependency)
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                let _ = clipboard.set_text(command);
                eprintln!("[paste] clipboard written via arboard fallback");
            }
            return;
        }
    };

    match std::process::Command::new(cmd)
        .args(&args)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(command.as_bytes());
            }
            let _ = child.wait();
            eprintln!("[paste] clipboard written via {}", cmd);
        }
        Err(e) => eprintln!("[paste] {} failed: {}", cmd, e),
    }
}
```

### Pattern 4: Paste Return Value for Hint Communication
**What:** Signal to frontend whether paste was auto or needs manual action
**When to use:** Wayland or missing-tool fallback paths need to tell the frontend to show a hint

The current `paste_to_terminal` returns `Result<(), String>`. For the hint mechanism, two approaches:

**Option A (recommended): Return a string result indicating mode**
```rust
// Change return type from Result<(), String> to Result<String, String>
// "auto" = paste happened automatically
// "clipboard_hint" = copied to clipboard, user needs to paste manually
```

**Option B: Emit a Tauri event**
```rust
app.emit("paste-hint", "press Ctrl+Shift+V to paste").ok();
```

Option A is simpler and keeps the hint synchronous with the paste action. The frontend already handles `.catch()` on the invoke -- it just needs to also handle the success value.

### Anti-Patterns to Avoid
- **xdotool type instead of clipboard paste:** `xdotool type` has issues with special characters, Unicode, and speed. Clipboard + Ctrl+Shift+V is more reliable.
- **Not waiting between Ctrl+U and paste:** Shell needs time to process the line clear before receiving paste content.
- **Blocking on subprocess in async context:** Use `spawn()` + `wait()` pattern, not `output()` for clipboard writes (avoids holding stdin open).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| X11 keystroke simulation | XTEST extension via x11rb | xdotool CLI | xdotool handles key name resolution, modifier keys, timing |
| Clipboard write | Raw X11 selection protocol | xclip/wl-copy CLI | Selection ownership protocol is complex; CLI tools handle it |
| Wayland detection | Custom D-Bus queries | Environment variables | WAYLAND_DISPLAY + XDG_SESSION_TYPE is the standard approach |
| Tool availability check | PATH scanning | `which` command | Standard, handles aliases and symlinks |

**Key insight:** Linux paste tools are CLI-first by design. Shelling out to xdotool/xclip is the idiomatic approach -- even major applications (VS Code, Electron apps) use this pattern.

## Common Pitfalls

### Pitfall 1: xclip Exits Only After Clipboard Selection Changes
**What goes wrong:** xclip forks to background to maintain clipboard ownership. If you don't close stdin properly, the child process hangs.
**Why it happens:** X11 clipboard is a lazy protocol -- the owner must serve paste requests until another app claims the selection.
**How to avoid:** Drop/close stdin before calling `child.wait()`. The stdin drop happens automatically when `stdin.take()` scope ends, but be explicit. Also, don't wait indefinitely -- xclip's default behavior is fine.
**Warning signs:** Paste command hangs indefinitely.

### Pitfall 2: xdotool Key Names Are Case-Sensitive
**What goes wrong:** `xdotool key ctrl+shift+V` fails silently or sends wrong key.
**Why it happens:** xdotool uses X11 keysym names which are case-sensitive.
**How to avoid:** Use lowercase for modifiers, uppercase for letter keys in shifted combos: `ctrl+shift+v` (all lowercase is fine -- xdotool maps it correctly). The correct syntax is `key ctrl+shift+v`.
**Warning signs:** Key combination doesn't trigger paste in terminal.

### Pitfall 3: Window Focus Race Condition
**What goes wrong:** xdotool sends keystrokes to wrong window because focus hasn't transferred yet.
**Why it happens:** Window managers process focus changes asynchronously.
**How to avoid:** Use `xdotool windowactivate --sync WINDOW_ID` which waits for the window manager to confirm activation, OR use a delay after windowactivate before sending keystrokes.
**Warning signs:** Keystrokes appear in overlay instead of terminal.

### Pitfall 4: Wayland + XWayland Clipboard Confusion
**What goes wrong:** Clipboard written via xclip (X11) isn't visible in Wayland-native apps, or vice versa.
**Why it happens:** X11 and Wayland clipboards are separate; XWayland bridges them but with limitations.
**How to avoid:** On Wayland, prefer wl-copy. On XWayland (GDK_BACKEND=x11), xclip is fine because the whole app runs in X11 mode.
**Warning signs:** "Clipboard is empty" when trying to paste after xclip write.

### Pitfall 5: Missing Window ID for xdotool
**What goes wrong:** Need to target keystrokes at the terminal window, but only have PID from Phase 31.
**Why it happens:** xdotool `windowactivate` takes a window ID, not a PID.
**How to avoid:** Use `xdotool search --pid PID` to find window IDs for a given PID, OR use `xdotool key` without window targeting (sends to focused window) -- but then must ensure correct window is focused first via `xdotool search --pid PID windowactivate`.
**Warning signs:** Keystrokes sent to wrong window.

## Code Examples

### Full X11 Paste Flow
```rust
#[cfg(target_os = "linux")]
fn paste_to_terminal_linux(
    app: &AppHandle,
    command: &str,
    pid: i32,
    tools: &LinuxToolAvailability,
) -> Result<String, String> {
    let display = detect_display_server();

    // Write to clipboard first (always needed)
    write_to_clipboard_linux(command, tools, &display);

    // Check if we can auto-paste (X11 + xdotool available)
    let can_auto_paste = matches!(display, DisplayServer::X11) && tools.has_xdotool;

    if can_auto_paste {
        // Activate terminal window via xdotool
        let search_output = std::process::Command::new("xdotool")
            .args(["search", "--pid", &pid.to_string()])
            .output()
            .map_err(|e| format!("xdotool search failed: {}", e))?;

        let window_id = String::from_utf8_lossy(&search_output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();

        if !window_id.is_empty() {
            // Activate the window (--sync waits for WM confirmation)
            let _ = std::process::Command::new("xdotool")
                .args(["windowactivate", "--sync", &window_id])
                .output();
        }

        // Clear current line
        let _ = std::process::Command::new("xdotool")
            .args(["key", "ctrl+u"])
            .output();

        // 50ms delay for shell to process Ctrl+U
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Paste from clipboard
        let _ = std::process::Command::new("xdotool")
            .args(["key", "ctrl+shift+v"])
            .output();

        eprintln!("[paste] Linux X11 auto-paste succeeded | pid={} | chars={}", pid, command.len());
        Ok("auto".to_string())
    } else {
        // Fallback: clipboard already written, return hint
        eprintln!("[paste] Linux fallback: clipboard written, showing hint | display={:?}", display);
        Ok("clipboard_hint".to_string())
    }
}
```

### Confirm Command (X11)
```rust
#[cfg(target_os = "linux")]
fn confirm_command_linux(
    pid: i32,
    tools: &LinuxToolAvailability,
) -> Result<String, String> {
    let display = detect_display_server();
    let can_auto = matches!(display, DisplayServer::X11) && tools.has_xdotool;

    if can_auto {
        let _ = std::process::Command::new("xdotool")
            .args(["key", "Return"])
            .output();
        eprintln!("[paste] Linux confirm succeeded via xdotool");
        Ok("auto".to_string())
    } else {
        Ok("confirm_hint".to_string())
    }
}
```

### Frontend Hint Display
```typescript
// In store, after invoke("paste_to_terminal"):
invoke<string>("paste_to_terminal", { command: fullText })
  .then((result) => {
    if (result === "clipboard_hint") {
      set({ pasteHint: "Copied to clipboard \u2014 press Ctrl+Shift+V to paste" });
    }
  })
  .catch((err) => {
    console.error("[store] paste failed:", err);
  })
  .finally(() => {
    set({ isPasting: false });
  });
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| xdotool type for text entry | Clipboard + Ctrl+Shift+V | Always preferred for terminals | Handles special chars, Unicode, large text |
| xsel for clipboard | xclip or wl-copy | xclip is more common in modern distros | Better Wayland compat via wl-copy |
| Manual Wayland detection | WAYLAND_DISPLAY + XDG_SESSION_TYPE | Standardized ~2020 | Reliable across all major compositors |

**Deprecated/outdated:**
- `xdotool type`: Unreliable for special characters and multi-byte Unicode
- `xsel`: Less commonly installed than xclip on modern distros

## Open Questions

1. **xdotool search --pid may return multiple window IDs**
   - What we know: A process can own multiple X11 windows. `xdotool search --pid` returns all of them.
   - What's unclear: Which window ID to pick for terminals with multiple windows.
   - Recommendation: Take the first (most recently focused) window ID. For most terminals, there's only one X11 window per process. If needed, add `--name` filter later.

2. **Return type change for paste_to_terminal**
   - What we know: Currently returns `Result<(), String>`. Need to communicate hint mode.
   - What's unclear: Whether changing to `Result<String, String>` breaks frontend contract.
   - Recommendation: Change to `Result<String, String>` -- the frontend currently ignores the `Ok` value (only handles `.catch()`), so returning a string is backward-compatible. macOS/Windows paths return `"auto"`.

## Sources

### Primary (HIGH confidence)
- xdotool man page -- key syntax, windowactivate vs windowfocus, --sync flag
- xclip man page -- `-selection clipboard` for system clipboard
- wl-clipboard GitHub -- wl-copy usage and Wayland clipboard semantics
- Existing codebase: `paste.rs`, `hotkey.rs`, `state.rs` -- established patterns

### Secondary (MEDIUM confidence)
- [xdotool GitHub](https://github.com/jordansissel/xdotool) -- Wayland limitations documented
- [Ubuntu xdotool manpage](https://manpages.ubuntu.com/manpages/trusty/man1/xdotool.1.html) -- windowactivate recommended over windowfocus
- [Rust by Example - Pipes](https://doc.rust-lang.org/rust-by-example/std_misc/process/pipe.html) -- stdin piping pattern
- [wl-clipboard GitHub](https://github.com/bugaevc/wl-clipboard) -- wl-copy forks to background for clipboard persistence

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - xdotool/xclip/wl-copy are universally used Linux CLI tools
- Architecture: HIGH - follows exact same patterns as macOS/Windows in existing codebase
- Pitfalls: HIGH - well-documented X11 clipboard and xdotool behavior

**Research date:** 2026-03-14
**Valid until:** 2026-04-14 (stable tools, no fast-moving changes)
