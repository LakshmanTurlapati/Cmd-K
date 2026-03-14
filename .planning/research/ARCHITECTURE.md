# Architecture Patterns

**Domain:** Linux platform support + smart terminal context for existing cross-platform Tauri v2 app
**Researched:** 2026-03-13

## Recommended Architecture

The existing codebase already has a clean `#[cfg(target_os = "...")]` gating pattern across all platform-specific modules. Linux support fits into this pattern with **no architectural changes** to the existing structure. Every platform-specific function already has a `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` stub that returns `None` or an empty value. Linux work replaces these stubs with real implementations.

Smart terminal context (scrollback + truncation) is a **new cross-platform module** that sits between context gathering and AI prompt building. It processes the output of the existing detection pipeline before it reaches the AI command in `commands/ai.rs`.

### High-Level Component Map

```
                    +------------------+
                    |   Frontend (React)|
                    |   Zustand Store   |
                    +--------+---------+
                             |  IPC (invoke)
                    +--------+---------+
                    |   Tauri Commands  |
                    |  commands/*.rs    |
                    +--------+---------+
                             |
          +------------------+------------------+
          |                  |                  |
   +------+------+   +------+------+   +------+------+
   |  macOS Path  |   | Windows Path|   | Linux Path  |  <-- NEW
   | NSPanel      |   | HWND/Win32  |   | X11/Wayland |
   | AX API       |   | UIA         |   | /proc fs    |
   | libproc FFI  |   | Toolhelp32  |   | xdotool/    |
   | AppleScript  |   | SendInput   |   | wl-clipboard|
   | CGEventPost  |   | arboard     |   | arboard     |
   +--------------+   +-------------+   +-------------+
                             |
                    +--------+---------+
                    | Smart Context     |  <-- NEW (cross-platform)
                    | terminal/context.rs|
                    | Truncation logic  |
                    +-------------------+
```

### Component Boundaries

| Component | Responsibility | Communicates With | Status |
|-----------|---------------|-------------------|--------|
| `terminal/detect_linux.rs` | App identification via `/proc`, `xdotool`, `xprop` | `terminal/mod.rs` | **NEW** |
| `terminal/process.rs` (Linux cfg) | `/proc` filesystem: CWD, process name, child PIDs, process tree | `terminal/mod.rs` | **MODIFY** (replace stubs) |
| `terminal/linux_reader.rs` | Terminal text reading via AT-SPI2/DBus | `terminal/mod.rs` | **NEW** |
| `commands/paste.rs` (Linux cfg) | Clipboard write via arboard + synthetic paste via xdotool/wtype | `commands/paste.rs` | **MODIFY** (replace stubs) |
| `commands/hotkey.rs` (Linux cfg) | Frontmost PID capture via xdotool/xprop | `commands/hotkey.rs` | **MODIFY** (replace stubs) |
| `commands/window.rs` (Linux cfg) | Overlay show/hide, always-on-top, focus management | `commands/window.rs` | **MODIFY** (minor -- already uses generic Tauri path) |
| `commands/ai.rs` (Linux cfg) | Linux-specific system prompt template | `commands/ai.rs` | **MODIFY** (replace generic stub) |
| `terminal/context.rs` | Smart truncation: scrollback capture, token-aware trimming | `commands/ai.rs`, `terminal/mod.rs` | **NEW** (cross-platform) |
| `lib.rs` | Linux-specific setup (always-on-top, skip-taskbar, no vibrancy) | All | **MODIFY** (add Linux setup block) |
| `Cargo.toml` | Linux dependencies (arboard with wayland-data-control) | Build | **MODIFY** |
| `.github/workflows/release.yml` | AppImage build job | CI/CD | **MODIFY** |
| `tauri.conf.json` | AppImage bundle config, updater linux platform entries | Build | **MODIFY** |

### Data Flow

**Existing flow (unchanged):**
```
Hotkey pressed
  -> capture frontmost PID (platform-specific)
  -> store in AppState.previous_app_pid
  -> toggle_overlay()
  -> frontend receives "overlay-shown" event
  -> frontend calls get_app_context()
  -> Rust: detect_app_context() runs platform-specific detection
  -> returns AppContext { app_name, terminal: TerminalContext { shell, cwd, visible_output, ... } }
  -> frontend sends to stream_ai_response with context JSON
  -> ai.rs builds system prompt + user message from context
  -> streams response back
```

**New Linux additions in this flow:**

1. **PID capture** (hotkey.rs): `get_frontmost_pid()` Linux impl uses `xdotool getactivewindow getwindowpid` (X11) or compositor-specific DBus query (Wayland)
2. **Detection** (terminal/mod.rs): `detect_app_context()` calls new Linux-specific functions via `#[cfg(target_os = "linux")]`
3. **Process info** (process.rs): reads `/proc/PID/cwd` symlink, `/proc/PID/exe` for process name, `/proc/PID/task/*/children` or `/proc/*/stat` for child PIDs
4. **Terminal text** (linux_reader.rs): AT-SPI2/DBus for accessible terminals (gnome-terminal, xfce4-terminal), returns None for GPU terminals
5. **Paste** (paste.rs): arboard for clipboard + xdotool (X11) or wtype (Wayland) for synthetic Ctrl+Shift+V
6. **Focus restore** (hotkey.rs): `xdotool windowactivate` (X11) to return focus to terminal after paste

**New smart context flow (cross-platform):**
```
detect_app_context() returns TerminalContext.visible_output (raw text)
  -> terminal/context.rs: truncate_for_ai(visible_output, max_chars)
  -> Strips ANSI escapes, collapses blanks, deduplicates repeated lines
  -> Keeps last N lines fitting within token budget
  -> ai.rs uses truncated output in user message (replaces current .lines().rev().take(25))
```

## New Components (Detailed)

### 1. `terminal/detect_linux.rs` -- Linux App Identification

**Purpose:** Equivalent of `detect.rs` (macOS bundle IDs) and `detect_windows.rs` (exe names) for Linux.

**Approach:** Use `/proc/PID/exe` symlink to identify the running application binary. This is the Linux analog of macOS bundle IDs and Windows exe names.

```rust
// Key functions needed:
pub fn get_app_identifier(pid: i32) -> Option<String>     // reads /proc/PID/exe -> basename
pub fn get_app_display_name(pid: i32) -> Option<String>   // reads /proc/PID/comm or /proc/PID/cmdline
pub fn is_known_terminal(app_id: &str) -> bool            // matches against known terminal exe names
pub fn is_ide_with_terminal(app_id: &str) -> bool         // code, cursor, etc.
pub fn is_known_browser(app_id: &str) -> bool             // chrome, firefox, etc.
pub fn clean_app_name(raw: &str) -> String                // strip paths, suffixes
```

**Known terminal identifiers (exe basenames):**
- `gnome-terminal-server`, `konsole`, `xfce4-terminal`, `mate-terminal` (GTK/Qt native)
- `alacritty`, `kitty`, `wezterm-gui`, `foot` (GPU-rendered)
- `xterm`, `rxvt-unicode`, `urxvt` (legacy X11)
- `tilix`, `terminator`, `guake`, `yakuake` (tiling/dropdown)

**Known IDE identifiers:**
- `code`, `code-insiders` (VS Code)
- `cursor` (Cursor)

**GPU terminal identifiers (returns None for visible_output):**
- `alacritty`, `kitty`, `wezterm-gui`, `foot`

### 2. `terminal/process.rs` -- Linux `/proc` Implementations

**Purpose:** Replace the `None`-returning stubs with real `/proc` filesystem reads.

**Functions to implement:**

```rust
#[cfg(target_os = "linux")]
fn get_process_cwd(pid: i32) -> Option<String> {
    std::fs::read_link(format!("/proc/{}/cwd", pid))
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
    // Permission: works for same-user processes, fails for root-owned (returns None)
}

#[cfg(target_os = "linux")]
fn get_process_name(pid: i32) -> Option<String> {
    // Primary: /proc/PID/exe symlink -> extract basename
    std::fs::read_link(format!("/proc/{}/exe", pid))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    // Fallback: /proc/PID/comm (first 15 chars of process name)
}

#[cfg(target_os = "linux")]
fn get_child_pids(pid: i32) -> Vec<i32> {
    // Primary: /proc/PID/task/PID/children (Linux 3.5+, fast, space-separated PIDs)
    // Fallback: scan /proc/*/stat, parse ppid (4th field) to find children
}
```

**Key difference from macOS/Windows:** Linux `/proc` is a filesystem, not an API. It is synchronous, fast (<1ms per read), and does not require FFI or special permissions for same-user processes. Pure Rust `std::fs` operations -- no new crate dependencies.

**Process tree walking** reuses the existing `find_shell_recursive()` function which is already cross-platform (not cfg-gated). Only the leaf functions (`get_process_cwd`, `get_process_name`, `get_child_pids`) need Linux implementations.

**`find_shell_pid` and `find_shell_by_ancestry` signature:**

The existing function signatures differ between macOS (3 args) and Windows (4 args with ProcessSnapshot). Linux should follow the macOS signature pattern (no snapshot needed since `/proc` reads are stateless).

```rust
#[cfg(target_os = "linux")]
pub fn find_shell_pid(app_pid: i32, focused_cwd: Option<&str>, _unused: Option<()>) -> Option<i32> {
    // Same logic as macOS: find_shell_recursive then find_shell_by_ancestry
}

#[cfg(target_os = "linux")]
fn find_shell_by_ancestry(app_pid: i32, focused_cwd: Option<&str>) -> Option<i32> {
    // Scan /proc for shell processes, walk parent chain to app_pid
    // Can reuse existing macOS pattern since both use pgrep-style scanning
}
```

### 3. `commands/hotkey.rs` -- Linux Frontmost PID

**Purpose:** Implement `get_frontmost_pid()` for Linux.

**X11 approach (primary, covers ~95% of Linux desktop users including XWayland):**
```rust
#[cfg(target_os = "linux")]
fn get_frontmost_pid() -> Option<i32> {
    let output = std::process::Command::new("xdotool")
        .args(["getactivewindow", "getwindowpid"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .ok()
}
```

**Wayland fallback:** Most Wayland compositors run XWayland, so xdotool works in practice. For pure Wayland without XWayland:
- GNOME: `gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell --method org.gnome.Shell.Eval "global.display.focus_window.get_pid()"`
- KDE: DBus call to `org.kde.KWin`
- wlroots: `wlr-foreign-toplevel-management` protocol

**Recommendation:** Start with xdotool. If it fails, try GNOME DBus. Other compositors return None (graceful degradation). This matches the app's philosophy of returning `Option<i32>` for all detection steps.

**Focus restoration:**
```rust
#[cfg(target_os = "linux")]
pub fn restore_focus(window_id: isize) -> bool {
    // X11: xdotool windowactivate <window_id>
    std::process::Command::new("xdotool")
        .args(["windowactivate", &window_id.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
```

**Note:** Linux needs a `previous_window_id` field in AppState (equivalent to `previous_hwnd` on Windows). This is the X11 window ID, captured via `xdotool getactivewindow` before showing the overlay.

### 4. `commands/paste.rs` -- Linux Paste Mechanism

**Clipboard write:**
```rust
#[cfg(target_os = "linux")]
fn write_to_clipboard(command: &str) {
    // Use arboard crate (same as Windows, with wayland-data-control feature)
    match arboard::Clipboard::new() {
        Ok(mut clipboard) => {
            match clipboard.set_text(command) {
                Ok(()) => eprintln!("[paste] clipboard written via arboard"),
                Err(e) => eprintln!("[paste] arboard set_text failed: {}", e),
            }
        }
        Err(e) => eprintln!("[paste] arboard Clipboard::new failed: {}", e),
    }
}
```

**Paste to terminal:**
```rust
#[cfg(target_os = "linux")]
fn paste_to_terminal_linux(app: &AppHandle, command: &str) -> Result<(), String> {
    let state = app.try_state::<AppState>()
        .ok_or("AppState not found")?;

    // 1. Get captured window ID
    let prev_wid = state.previous_window_id.lock()
        .map_err(|_| "mutex poisoned")?
        .ok_or("no previous window ID")?;

    // 2. Write to clipboard
    write_to_clipboard(command);

    // 3. Restore focus to terminal
    let _ = std::process::Command::new("xdotool")
        .args(["windowactivate", "--sync", &prev_wid.to_string()])
        .status();
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 4. Send Ctrl+Shift+V (Linux terminal paste shortcut)
    // CRITICAL: Linux terminals use Ctrl+SHIFT+V, not Ctrl+V
    // Ctrl+V sends a literal control character in terminals
    std::process::Command::new("xdotool")
        .args(["key", "--clearmodifiers", "ctrl+shift+v"])
        .status()
        .map_err(|e| format!("xdotool key failed: {}", e))?;

    Ok(())
}
```

**Wayland paste:** Use `wtype` instead of `xdotool`:
```rust
// Wayland fallback
std::process::Command::new("wtype")
    .args(["-M", "ctrl", "-M", "shift", "-P", "v", "-p", "v", "-m", "shift", "-m", "ctrl"])
    .status()
```

**Confirm (send Enter):**
```rust
#[cfg(target_os = "linux")]
fn confirm_command_linux(app: &AppHandle) -> Result<(), String> {
    // Restore focus + xdotool key Return
    // Same pattern as paste but with "Return" instead of "ctrl+shift+v"
}
```

### 5. `terminal/linux_reader.rs` -- Terminal Text Reading

**Purpose:** Read visible terminal output on Linux. Equivalent of AX reader (macOS) and UIA reader (Windows).

**Approach:** AT-SPI2 (Assistive Technology Service Provider Interface) is the Linux accessibility framework, equivalent to macOS AX API. GTK-based terminals expose text through it.

**Terminals that support AT-SPI2 text reading:**
- gnome-terminal (GTK, VTE-based)
- xfce4-terminal (GTK, VTE-based)
- mate-terminal (GTK, VTE-based)
- tilix (GTK, VTE-based)
- terminator (GTK, VTE-based)

**Terminals that return None (GPU-rendered, no accessibility text):**
- Alacritty, kitty, WezTerm, foot

**Implementation approach:** Use `atspi` DBus interface via subprocess:
```rust
#[cfg(target_os = "linux")]
pub fn read_terminal_text_linux(pid: i32) -> Option<String> {
    // AT-SPI2 via gdbus or atspi python bindings is complex.
    // Simpler approach for V1: use `xdotool getactivewindow` to get window ID,
    // then `xdg-terminal-exec` or direct AT-SPI2 DBus calls.
    //
    // For MVP, return None (same as GPU terminals on macOS/Windows).
    // Add AT-SPI2 reading in a follow-up phase.
    None
}
```

**Recommendation for initial release:** Return None for all Linux terminals. This matches the behavior of GPU terminals on macOS/Windows -- the app works without visible_output (CWD and shell type are the critical context). AT-SPI2 reading can be added later as an enhancement.

### 6. `terminal/context.rs` -- Smart Terminal Context (Cross-Platform)

**Purpose:** Intelligent truncation of terminal output to fit AI context windows.

**Current behavior:** `build_user_message()` in `ai.rs` takes the last 25 lines of visible_output. This is naive.

**New module design:**

```rust
/// Configuration for smart truncation.
pub struct ContextConfig {
    /// Maximum characters to include (rough token estimate: chars/4)
    pub max_chars: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_chars: 8000,  // ~2000 tokens, leaving room for prompt + response
        }
    }
}

/// Strip ANSI escape sequences from terminal output.
fn strip_ansi(text: &str) -> String {
    // Regex: \x1b\[[0-9;]*[a-zA-Z] and \x1b\].*?\x07
    // Use once_cell::Lazy<Regex> for compiled pattern (existing project pattern)
}

/// Collapse runs of 3+ blank lines to a single blank line.
fn collapse_blanks(text: &str) -> String { ... }

/// Deduplicate consecutive identical lines (e.g., progress bars, spinner frames).
fn dedup_lines(text: &str) -> String { ... }

/// Truncate terminal output intelligently for AI context.
///
/// Processing pipeline:
/// 1. Strip ANSI escape sequences
/// 2. Collapse blank line runs
/// 3. Deduplicate consecutive identical lines
/// 4. Take from bottom up until max_chars
/// 5. Prepend "[... earlier output truncated ...]" if truncated
pub fn truncate_for_ai(raw_output: &str, config: &ContextConfig) -> String { ... }
```

**Integration point:** Called in `ai.rs` `build_user_message()` before including `visible_output` in the prompt. Replaces the current hardcoded `.lines().rev().take(25)` logic.

### 7. `lib.rs` -- Linux Setup Block

**Current state:** macOS creates NSPanel with vibrancy. Windows applies Acrylic + WS_EX_TOOLWINDOW. No Linux block exists.

```rust
#[cfg(target_os = "linux")]
{
    // 1. Always-on-top (works on X11, best-effort on Wayland)
    window.set_always_on_top(true)
        .unwrap_or_else(|e| eprintln!("[setup] always_on_top failed: {}", e));

    // 2. Skip taskbar (Tauri supports via GTK)
    window.set_skip_taskbar(true)
        .unwrap_or_else(|e| eprintln!("[setup] skip_taskbar failed: {}", e));

    // 3. No vibrancy -- window-vibrancy crate does NOT support Linux
    //    CSS handles dark semi-transparent look via rgba() background
    //    transparent: true in tauri.conf.json is already set
}
```

**Vibrancy limitation:** The `window-vibrancy` crate explicitly does not support Linux. Behind-window blur depends on the compositor (KDE supports it via KWin rules, GNOME does not). The CSS-only approach (dark semi-transparent background) is the correct solution. Accept the visual difference from macOS/Windows.

### 8. `Cargo.toml` -- Linux Dependencies

```toml
[target.'cfg(target_os = "linux")'.dependencies]
keyring = { version = "3", features = ["linux-native"] }          # Already present
arboard = { version = "3", features = ["wayland-data-control"] }  # Clipboard (X11 + Wayland)
```

**Why arboard with wayland-data-control:** arboard's default Linux backend is X11 only. The `wayland-data-control` feature adds Wayland support via `wl-clipboard-rs`. When enabled, Wayland is prioritized but falls back to X11 automatically if Wayland init fails. This covers both display server protocols.

No other new crate dependencies needed. `/proc` access uses `std::fs`. External tools (`xdotool`, `xprop`, `wtype`) use `std::process::Command` -- same pattern as macOS's `osascript`.

### 9. `tauri.conf.json` -- Linux Bundle Configuration

The existing config has `"targets": "all"` which includes AppImage on Linux. Specific Linux configuration needed:

```json
{
  "bundle": {
    "linux": {
      "appimage": {
        "bundleMediaFramework": true
      }
    }
  }
}
```

### 10. CI/CD -- AppImage Build Job

Add a `build-linux` job to `release.yml` parallel to existing `build-macos` and `build-windows`:

```yaml
build-linux:
  runs-on: ubuntu-22.04
  steps:
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y \
          libwebkit2gtk-4.1-dev \
          libgtk-3-dev \
          libappindicator3-dev \
          librsvg2-dev \
          patchelf
    # Standard Rust + pnpm + Node setup (same as other jobs)
    - name: Build Tauri app
      env:
        TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
        TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
      run: pnpm tauri build
    # Artifacts: src-tauri/target/release/bundle/appimage/*.AppImage
```

Update `latest.json` assembly to include `linux-x86_64` platform entry for auto-updater.

Update release `needs:` to include `build-linux` alongside `build-macos` and `build-windows`.

### 11. `commands/ai.rs` -- Linux System Prompt

Replace the current generic fallback with a Linux-specific template:

```rust
#[cfg(target_os = "linux")]
const TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for Linux. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run. No explanations, no markdown, no code fences. \
     Just the raw command(s). If multiple commands are needed, separate them with && or use pipes. \
     Prefer common POSIX tools (grep, find, sed, awk) over modern alternatives (rg, fd, jq). \
     The user is on Linux with {shell_type} shell.";

#[cfg(target_os = "linux")]
const ASSISTANT_SYSTEM_PROMPT: &str =
    "You are a concise assistant accessed via a Linux overlay. Answer in 2-3 sentences maximum. \
     Be direct and helpful. No markdown formatting, no code fences unless the user explicitly asks for code.";
```

### 12. `commands/permissions.rs` -- Linux Accessibility

Linux does not require macOS-style Accessibility permissions. No dialog needed. The stub should indicate permissions are granted:

```rust
#[cfg(target_os = "linux")]
pub fn check_accessibility_permission() -> bool {
    true  // Linux doesn't require accessibility permission for xdotool/AT-SPI2
}
```

### 13. `state.rs` -- Linux Window ID Field

Add a field for the X11 window ID (equivalent to `previous_hwnd` on Windows):

```rust
/// X11 Window ID of the foreground window captured BEFORE showing overlay (Linux only).
/// Used by focus restoration and paste. u64 on X11 (Window type is unsigned long).
pub previous_xid: Mutex<Option<u64>>,
```

Alternatively, reuse `previous_hwnd` (both are integer window identifiers). The `isize` type works for X11 window IDs which are typically u32.

## Patterns to Follow

### Pattern 1: Platform cfg-gating (Existing)
**What:** Every platform-specific function has exactly one impl per platform, gated by `#[cfg(target_os = "...")]`.
**When:** Any new platform-specific code.
**Rule:** The `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` stubs become `#[cfg(target_os = "linux")]` implementations.

### Pattern 2: Subprocess Command Invocation (Existing)
**What:** Use `std::process::Command` for external tool calls.
**When:** xdotool, xprop, wtype calls on Linux.
**Example:** macOS uses `osascript` this way in paste.rs. Linux uses `xdotool` the same way.

### Pattern 3: Timeout-wrapped Detection (Existing)
**What:** All detection runs in a background thread with `mpsc::channel` + `recv_timeout`.
**When:** Linux detection pipeline.
**Key:** The outer `detect()` and `detect_full()` wrappers already apply timeouts. Linux detection code just needs to implement `detect_inner()` and `detect_app_context()` -- the timeout wrapper is unchanged.

### Pattern 4: Graceful Degradation (Existing)
**What:** Every detection step returns `Option`. Missing data produces `None`, not errors.
**When:** Linux tools may not be installed (xdotool absent on pure Wayland, AT-SPI2 not running).
**Example:** GPU terminals already return `None` for visible_output. Linux text reading follows the same pattern.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Adding a heavyweight `/proc` crate
**What:** Using `procfs` or `sysinfo` crate for simple `/proc` reads.
**Why bad:** Adds a dependency for what is 5-10 lines of `std::fs::read_link` and `std::fs::read_to_string`. The macOS path uses raw FFI for the same reason (avoiding dependency conflicts).
**Instead:** Use `std::fs` directly. `/proc` is a stable Linux ABI.

### Anti-Pattern 2: Fighting Wayland always-on-top limitations
**What:** Extensive compositor-specific hacks to force always-on-top on every Wayland compositor.
**Why bad:** Wayland's security model explicitly prevents clients from controlling their own stacking order. Fighting this is fragile and compositor-dependent.
**Instead:** Set `always_on_top: true` via Tauri API (works on X11, best-effort on Wayland via GTK hints). Accept that some Wayland compositors may not honor it -- the overlay still functions, it just might not float above other windows.

### Anti-Pattern 3: Duplicating detection orchestration
**What:** Writing a completely separate `detect_app_context_linux()` orchestrator that duplicates the macOS pattern.
**Why bad:** Creates maintenance burden. The macOS and Linux patterns are structurally identical (get app identifier, check if terminal, walk process tree, read text).
**Instead:** Use a shared detection path where possible. The `#[cfg(target_os = "linux")]` blocks should be minimal -- just the leaf functions that differ.

### Anti-Pattern 4: Blocking on external tools without timeout
**What:** Calling `xdotool` or `xprop` without timeout protection.
**Why bad:** If X11 is unresponsive, the hotkey handler blocks indefinitely.
**Instead:** The existing `mpsc` timeout wrapper handles this. Additionally, xdotool calls are in `get_frontmost_pid()` which is called within the timeout-protected detection pipeline.

### Anti-Pattern 5: Attempting AT-SPI2 terminal text reading in initial release
**What:** Building complex DBus/AT-SPI2 integration for terminal text reading before the core Linux features work.
**Why bad:** AT-SPI2 is complex, compositor-dependent, and not universally available. It blocks the entire Linux release for a feature that is optional (CWD + shell type are the critical context).
**Instead:** Return None for visible_output in the initial release. Add AT-SPI2 reading as a follow-up enhancement.

## Integration Points Summary

### Already Works on Linux (No Changes Needed)
- Frontend React/TypeScript -- fully cross-platform
- AI provider abstraction (3 streaming adapters) -- no platform code
- Settings/store persistence (tauri-plugin-store) -- cross-platform
- Hotkey registration (tauri-plugin-global-shortcut) -- cross-platform
- Destructive command detection (safety.rs) -- already has Linux patterns (SAFE-02)
- Auto-updater (tauri-plugin-updater) -- cross-platform (needs AppImage artifacts)
- Tray icon -- cross-platform via Tauri tray API
- History/state management -- pure Rust HashMap, no platform code
- Token tracking and usage stats -- no platform code

### Needs Linux Implementations (Replace Stubs)

| Stub Location | Current Return | Linux Implementation | Complexity |
|---------------|---------------|---------------------|-----------|
| `hotkey.rs:get_frontmost_pid()` | `None` | xdotool getactivewindow getwindowpid | Low |
| `process.rs:get_process_cwd()` | `None` | `/proc/PID/cwd` symlink | Low |
| `process.rs:get_process_name()` | `None` | `/proc/PID/exe` symlink basename | Low |
| `process.rs:get_child_pids()` | `Vec::new()` | `/proc/PID/task/PID/children` | Low |
| `process.rs:find_shell_by_ancestry()` | `None` | `/proc` scan + parent chain walk | Medium |
| `detect.rs:get_bundle_id()` | `None` | `/proc/PID/exe` basename | Low |
| `detect.rs:get_app_display_name()` | `None` | `/proc/PID/comm` | Low |
| `paste.rs:write_to_clipboard()` | no-op | arboard with wayland-data-control | Low |
| `paste.rs:paste_to_terminal()` | `Err(not implemented)` | xdotool key ctrl+shift+v | Medium |
| `paste.rs:confirm_terminal_command()` | `Err(not implemented)` | xdotool key Return | Low |
| `mod.rs:detect_app_context()` | `None` | Linux orchestrator using above | Medium |

### New Modules Needed

| Module | Purpose | Complexity | Priority |
|--------|---------|-----------|----------|
| `terminal/detect_linux.rs` | App identification, terminal lists, GPU check | Low | P0 (blocks detection) |
| `terminal/context.rs` | Smart truncation (cross-platform) | Medium | P1 (enhances all platforms) |
| `terminal/linux_reader.rs` | AT-SPI2 terminal text | High | P2 (defer to post-release) |

## Build Order (Dependency-Aware)

**Phase 1: Foundation** (no dependencies on other new code)
1. Linux `/proc` process functions in process.rs -- get_process_cwd, get_process_name, get_child_pids
2. Linux app identification in detect_linux.rs -- is_known_terminal, clean_app_name
3. Linux system prompt + assistant prompt in ai.rs

**Phase 2: Detection Pipeline** (depends on Phase 1)
4. Linux frontmost PID capture in hotkey.rs -- xdotool getactivewindow getwindowpid
5. Linux window ID capture + focus restore -- xdotool windowactivate
6. Linux detect_app_context in mod.rs -- wire Phase 1 components into detection pipeline
7. Linux window key computation in hotkey.rs -- needs detect_linux + process

**Phase 3: Overlay + Actions** (depends on Phase 2)
8. Linux overlay setup in lib.rs -- always-on-top, skip-taskbar
9. Linux clipboard + paste in paste.rs -- arboard + xdotool key ctrl+shift+v
10. Linux confirm (Enter) in paste.rs -- xdotool key Return

**Phase 4: Smart Context** (independent, cross-platform)
11. Smart context truncation module context.rs -- ANSI stripping, dedup, char-budget truncation
12. Integration with ai.rs build_user_message -- replace hardcoded 25-line limit

**Phase 5: Distribution**
13. AppImage CI/CD job in release.yml -- parallel to macOS/Windows builds
14. Updater config -- linux-x86_64 platform entry in latest.json assembly
15. Linux permissions stub in permissions.rs -- always returns true

**Dependency graph:**
```
Phase 1 (proc + detect_linux + ai prompts)
    |
    v
Phase 2 (PID capture + detect pipeline + window key)
    |
    v
Phase 3 (overlay setup + paste + confirm)

Phase 4 (smart context) -- independent, can start any time
    |
    v
Phase 5 (CI/CD + updater) -- needs working Linux build from Phases 1-3
```

## Scalability Considerations

| Concern | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Process info read | libproc FFI ~1ms | PEB ReadProcessMemory ~2ms | `/proc` fs read <1ms |
| Process tree scan | sysctl fallback ~5ms | Toolhelp32Snapshot ~1ms | `/proc/*/stat` scan ~3ms |
| PID capture | NSWorkspace ObjC ~1ms | GetForegroundWindow ~0.1ms | xdotool subprocess ~10ms |
| Clipboard write | pbcopy subprocess ~5ms | arboard ~1ms | arboard ~1ms |
| Paste simulation | CGEventPost ~1ms | SendInput ~1ms | xdotool subprocess ~10ms |
| Text reading | AX API ~50-200ms | UIA ~20-200ms | None (V1), AT-SPI2 ~50ms (V2) |
| CI build time | ~8min (signed+notarized) | ~6min (NSIS) | ~6min (AppImage, no signing) |
| Detection budget | 500/750ms timeout | 750ms timeout | 500/750ms timeout (same) |

**Key insight:** Linux detection is faster than macOS/Windows for process info (direct filesystem reads vs API calls) but slightly slower for PID capture and paste (subprocess invocation vs in-process API). All operations comfortably fit within the existing 500/750ms timeout budget.

## Sources

- [Tauri v2 Window API](https://docs.rs/tauri/latest/tauri/window/struct.Window.html) -- Window management, always_on_top, skip_taskbar
- [Tauri always-on-top Wayland issue #3117](https://github.com/tauri-apps/tauri/issues/3117) -- Wayland limitation confirmed
- [tao always-on-top Wayland issue #1134](https://github.com/tauri-apps/tao/issues/1134) -- Upstream windowing limitation
- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy) -- No Linux support confirmed
- [arboard crate](https://github.com/1Password/arboard) -- Linux X11 + Wayland clipboard via wayland-data-control feature
- [Linux /proc filesystem documentation](https://www.kernel.org/doc/html/latest/filesystems/proc.html) -- proc_pid_cwd, cmdline, stat, children
- [proc_pid_cwd(5)](https://man7.org/linux/man-pages/man5/proc_pid_cwd.5.html) -- CWD symlink specification
- [xdotool man page](https://manpages.ubuntu.com/manpages/trusty/man1/xdotool.1.html) -- getactivewindow, getwindowpid, key, windowactivate
- [xdotool getwindowpid issue #428](https://github.com/jordansissel/xdotool/issues/428) -- Known PID accuracy limitations
- [Tauri AppImage distribution](https://v2.tauri.app/distribute/appimage/) -- AppImage bundling guide
- [Tauri GitHub Actions pipeline](https://v2.tauri.app/distribute/pipelines/github/) -- CI/CD reference
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard) -- Wayland clipboard reference
- [GNOME DBus window management](https://gist.github.com/rbreaves/257c3edfa301786e66e964d7ac036269) -- Wayland focused window via DBus
- [KDE KWin window metadata](https://community.kde.org/KWin/Window_Metadata) -- KDE Wayland window PID via DBus
- [Arch Wiki Clipboard](https://wiki.archlinux.org/title/Clipboard) -- X11 vs Wayland clipboard ecosystem
- [sick.codes xdotool paste](https://sick.codes/paste-clipboard-linux-xdotool-ctrl-v-terminal-type/) -- xdotool paste patterns
- [Tauri AppImage CI issue #14796](https://github.com/tauri-apps/tauri/issues/14796) -- Known AppImage CI issues
