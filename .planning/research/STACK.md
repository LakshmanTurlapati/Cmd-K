# Technology Stack: Linux Support & Smart Terminal Context

**Project:** CMD+K v0.3.9
**Researched:** 2026-03-13
**Overall confidence:** HIGH (X11), MEDIUM (Wayland -- known protocol gaps)

> This document supersedes the v0.2.8 STACK.md. The validated stack (Tauri v2, React, TypeScript, macOS NSPanel + Accessibility API, Windows Win32 + UIA, 5 AI providers, Ed25519 updater) is not re-researched. Focus is strictly on NEW libraries and changes needed for Linux platform support and smart terminal context truncation.

---

## Executive Summary

Linux support requires solving five problems that macOS and Windows handle via platform-specific APIs (NSPanel, Accessibility API, UIA, AppleScript, CGEvent, SendInput). Linux splits across X11 and Wayland with compositor-specific behaviors. The core strategy is: **ship X11-first with graceful Wayland degradation**.

Key stack additions: 3 new Rust crates (`x11rb`, `arboard` with Wayland feature, `atspi`), direct `/proc` filesystem reads (no crate), and `xdotool` as a runtime subprocess dependency. No frontend changes needed -- the overlay UI, AI streaming, and provider system are platform-independent.

---

## Recommended Stack Additions

### 1. Process Inspection: CWD, Shell Type, Process Tree

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `/proc` filesystem (direct) | N/A (kernel) | Read CWD, exe path, child PIDs | Zero dependencies. `std::fs::read_link("/proc/PID/cwd")` for CWD, `std::fs::read_link("/proc/PID/exe")` for binary path, `std::fs::read_to_string("/proc/PID/task/PID/children")` for child PIDs. ~50 lines of safe Rust. Mirrors the macOS `libproc` FFI approach. |

**Confidence:** HIGH -- `/proc` is standard on all Linux distros, documented at `man 5 proc`.

**Why not the `procfs` crate (v0.17)?** Adds 15+ transitive dependencies for functionality achievable in ~50 lines. The codebase pattern is direct platform API calls (macOS `libproc` FFI, Windows `NtQueryInformationProcess`), not wrapper crates. Consistency matters.

**Integration point:** Add `#[cfg(target_os = "linux")]` block in `src/terminal/process.rs`. Same `ProcessInfo` return struct, same function signatures. The existing `get_foreground_info()` dispatches by platform -- add the Linux arm.

**Implementation sketch:**
```rust
#[cfg(target_os = "linux")]
fn get_cwd(pid: i32) -> Option<String> {
    std::fs::read_link(format!("/proc/{}/cwd", pid))
        .ok()
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(target_os = "linux")]
fn get_process_name(pid: i32) -> Option<String> {
    std::fs::read_link(format!("/proc/{}/exe", pid))
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
}

#[cfg(target_os = "linux")]
fn get_child_pids(ppid: i32) -> Vec<i32> {
    std::fs::read_to_string(format!("/proc/{}/task/{}/children", ppid, ppid))
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect()
}
```

**Permissions note:** `/proc/PID/cwd` and `/proc/PID/exe` require the reader to have the same UID as the target process OR have `CAP_SYS_PTRACE`. For CMD+K reading its own user's terminal processes, this always works. No elevated permissions needed.

---

### 2. Terminal Text Reading: Visible Output via AT-SPI2

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `atspi` | 0.22+ | Read terminal text via AT-SPI2 D-Bus protocol | Pure Rust AT-SPI2 client from the Odilia screen reader project. Linux equivalent of macOS Accessibility API and Windows UIA. Uses `zbus` for D-Bus (async, tokio-compatible -- tokio already in deps). Provides typed `Text::get_text()` interface for reading accessible text from terminal emulators. |

**Confidence:** MEDIUM -- AT-SPI2 text reading works for GTK-based terminals (GNOME Terminal, Tilix, xfce4-terminal) and Qt-based terminals (Konsole). GPU-rendered terminals (Alacritty, kitty, WezTerm) do NOT expose AT-SPI2 text. This is the same limitation as macOS AX API and Windows UIA -- already documented and accepted in the codebase.

**Integration point:** New file `src/terminal/atspi_reader.rs` gated with `#[cfg(target_os = "linux")]`. Follows the pattern of `ax_reader.rs` (macOS) and `uia_reader.rs` (Windows). Returns `Option<String>` of filtered visible text.

**AT-SPI2 reading approach:**
1. Connect to AT-SPI2 bus via `atspi::AccessibilityBus`
2. Find the accessible object for the terminal application by PID
3. Navigate to the terminal widget (role: `Terminal` or `Text`)
4. Call `Text::get_text(0, -1)` to read all visible text
5. Filter through existing `filter::filter_sensitive()`

**Why not read the PTY via `/proc/PID/fd`?** The PTY master FD belongs to the terminal emulator process, not the shell. Reading it requires `ptrace` or root privilege, and captures raw byte streams without ANSI escape resolution. Not viable.

---

### 3. Clipboard Write & Paste Action

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `arboard` | 3 | Clipboard write (X11 + Wayland) | Already used on Windows. Add to Linux deps with `wayland-data-control` feature for Wayland support. Falls back to X11 automatically when Wayland data-control is unavailable. |
| `xdotool` (subprocess) | System package | Paste keystroke on X11 | Posts `Ctrl+Shift+V` to focused window. Single subprocess call (~5ms), same pattern as macOS `osascript`. Installed on most X11 desktops or available via `apt install xdotool`. |
| `wtype` (subprocess) | System package | Paste keystroke on Wayland | Wayland equivalent of xdotool. Posts `Ctrl+Shift+V` to focused window via Wayland protocols. Available via `apt install wtype`. |

**Confidence:** HIGH (clipboard via arboard), HIGH (paste via xdotool on X11), MEDIUM (paste via wtype on Wayland -- compositor-dependent)

**Why Ctrl+Shift+V, not Ctrl+V?** Linux terminals universally use Ctrl+Shift+V for paste. Ctrl+V is captured by the shell as "literal next character" (verbatim insert). This is different from Windows (Ctrl+V) and macOS (Cmd+V).

**Integration point:** `src/commands/paste.rs` -- add `#[cfg(target_os = "linux")]` block:
```rust
#[cfg(target_os = "linux")]
fn paste_to_terminal_linux(command: &str) -> Result<(), String> {
    // 1. Write to clipboard via arboard
    write_to_clipboard(command);

    // 2. Wait for focus restoration
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 3. Send paste keystroke
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session_type == "wayland" {
        // wtype for Wayland
        std::process::Command::new("wtype")
            .args(["-M", "ctrl", "-M", "shift", "-k", "v"])
            .output()
            .map_err(|e| format!("wtype failed: {}. Install with: sudo apt install wtype", e))?;
    } else {
        // xdotool for X11
        std::process::Command::new("xdotool")
            .args(["key", "ctrl+shift+v"])
            .output()
            .map_err(|e| format!("xdotool failed: {}. Install with: sudo apt install xdotool", e))?;
    }
    Ok(())
}
```

**Arboard Linux clipboard note:** X11 and Wayland clipboard semantics require the source app to remain alive to serve paste requests. Since CMD+K is a long-running background daemon, this works naturally. The `arboard` crate handles the clipboard ownership protocol.

---

### 4. Focused Window Detection (Pre-Overlay PID Capture)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `x11rb` | 0.13+ | X11 protocol for active window + PID query | Pure Rust X11 client. Query `_NET_ACTIVE_WINDOW` on root window for active window ID, then `_NET_WM_PID` on that window for the owning PID. Direct protocol call (~2ms) vs `xdotool getactivewindow getwindowpid` subprocess (~50ms). Critical for the capture-before-show pattern. |

**Confidence:** HIGH for X11 -- `_NET_ACTIVE_WINDOW` and `_NET_WM_PID` are EWMH standard, supported by all X11 window managers.

**Wayland focused-window detection:** No universal protocol exists. Each compositor has different mechanisms:
- GNOME: `gdbus call -e -d org.gnome.Shell -o /org/gnome/Shell -m org.gnome.Shell.Eval "global.display.focus_window.get_pid()"`
- KDE: `qdbus org.kde.KWin /KWin org.kde.KWin.activeWindow`
- Sway: `swaymsg -t get_tree | jq '.. | select(.focused?) | .pid'`
- Hyprland: `hyprctl activewindow -j | jq '.pid'`

**Recommendation for v0.3.9:** X11-only for active window PID. On Wayland, degrade gracefully:
- Skip PID capture (set `previous_app_pid` to None)
- No per-window history (use global history)
- No terminal-specific context detection
- Document as known Wayland limitation

The `XDG_SESSION_TYPE` env var reliably distinguishes X11 from Wayland at runtime.

**Integration point:** `src/commands/hotkey.rs` -- add `#[cfg(target_os = "linux")]` `get_frontmost_pid()`:
```rust
#[cfg(target_os = "linux")]
fn get_frontmost_pid() -> Option<i32> {
    let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session == "x11" {
        get_frontmost_pid_x11()
    } else {
        None // Wayland: no universal active window PID protocol
    }
}
```

---

### 5. Overlay Window

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri built-in | 2.x | Transparent, undecorated window | Already handles `transparent: true`, `decorations: false`, `skipTaskbar: true` on Linux via GTK/tao. No additional crate. |

**Confidence:** HIGH (window creation), HIGH (X11 always-on-top), NOT SUPPORTED (Wayland always-on-top)

**X11 always-on-top:** Tauri's `set_always_on_top(true)` works via `_NET_WM_STATE_ABOVE` EWMH hint. Window managers treat this as a suggestion but all mainstream WMs (GNOME/Mutter, KDE/KWin, i3, Sway-XWayland) honor it.

**Wayland always-on-top:** No Wayland protocol for always-on-top exists. Tauri's `tao` explicitly documents this as unsupported. The workaround is running under XWayland (set `GDK_BACKEND=x11` environment variable).

**No vibrancy on Linux:** The `window-vibrancy` crate does not support Linux. Blur/vibrancy is compositor-controlled. Use CSS semi-transparent background instead (`background: rgba(30, 30, 30, 0.92)`). This is the standard approach for Linux overlay apps.

---

### 6. Global Hotkey

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| `tauri-plugin-global-shortcut` | 2 (existing) | System-wide Ctrl+K hotkey | Already in deps. Works on X11 via X11 RECORD extension. |

**Confidence:** HIGH (X11), NOT SUPPORTED (Wayland)

**Wayland global hotkey status:** Tauri's `global-hotkey` crate explicitly disables hotkeys on Wayland to prevent libX11 segfaults. The `xdg-desktop-portal` GlobalShortcuts portal spec exists but:
- GNOME: NOT implemented (feature request open, no timeline)
- KDE: Implemented
- Hyprland: Implemented via `xdg-desktop-portal-hyprland`
- Sway: NOT implemented

A Wayland support PR (#162) was opened in `tauri-apps/global-hotkey` in Sep 2025 but merge status is unknown.

**Recommendation:** Ship as X11-first. Wayland users should run with `GDK_BACKEND=x11` to force XWayland. This is the same approach taken by OBS Studio, Discord, Slack, and other apps needing global hotkeys on Linux.

---

### 7. AppImage Distribution & Auto-Update

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri built-in | 2.x | AppImage bundle + updater | Tauri v2 natively builds AppImage, generates `.tar.gz` + `.sig` for updater. Same Ed25519 signing already configured. |

**Confidence:** HIGH -- explicitly documented in Tauri v2 docs.

**tauri.conf.json addition:**
```json
{
  "bundle": {
    "linux": {
      "appimage": {
        "bundleMediaFramework": false
      }
    }
  }
}
```

**CI/CD addition:** Third job in `release.yml`:
```yaml
build-linux:
  runs-on: ubuntu-22.04
  steps:
    - uses: actions/checkout@v4
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y \
          libwebkit2gtk-4.1-dev \
          libayatana-appindicator3-dev \
          librsvg2-dev \
          libatk-bridge2.0-dev \
          libatspi2.0-dev \
          libgtk-3-dev \
          libglib2.0-dev \
          libx11-dev \
          libxdo-dev
    - name: Build AppImage
      run: cargo tauri build --bundles appimage
```

**Updater:** The existing `latest.json` endpoint structure supports multi-platform. Tauri uses `{{target}}` and `{{arch}}` variables. The updater auto-selects the Linux AppImage `.tar.gz` bundle.

---

### 8. Smart Terminal Context Truncation

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| No new crate | N/A | Intelligent truncation of terminal output for AI context | Pure Rust logic. Character-based truncation (4 chars ~ 1 token) is sufficient. Keep most recent output (bottom), discard oldest (top). |

**Confidence:** HIGH -- application logic, not platform-specific.

**Why not `tiktoken-rs`?** Adds ~5MB of BPE vocabulary data and a C dependency. Character-based approximation is adequate for truncation decisions. The AI providers handle actual tokenization.

**Integration point:** New `src/terminal/truncate.rs` or function in `src/terminal/filter.rs`:
```rust
/// Truncate terminal output to fit within AI context budget.
/// Keeps the LAST (most recent) lines, discards older output from the top.
pub fn truncate_for_context(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    let lines: Vec<&str> = text.lines().collect();
    let mut result = Vec::new();
    let mut total = 0;
    for line in lines.iter().rev() {
        if total + line.len() + 1 > max_chars {
            break;
        }
        result.push(*line);
        total += line.len() + 1;
    }
    result.reverse();
    let truncated_count = lines.len() - result.len();
    if truncated_count > 0 {
        format!("[... {} lines truncated ...]\n{}", truncated_count, result.join("\n"))
    } else {
        result.join("\n")
    }
}
```

**Context budget:** Default to ~4000 chars (~1000 tokens). Configurable via settings. Applied in `src/commands/terminal.rs` before sending to AI. Applies to ALL platforms, not just Linux.

---

## Full Dependency Changes

### Cargo.toml

```toml
[target.'cfg(target_os = "linux")'.dependencies]
keyring = { version = "3", features = ["linux-native"] }   # EXISTING
arboard = { version = "3", features = ["wayland-data-control"] }  # NEW
x11rb = { version = "0.13", features = ["randr"] }               # NEW
atspi = { version = "0.22", features = ["tokio"] }                # NEW
```

### No changes needed for:
- `tauri`, `tauri-plugin-global-shortcut`, `tauri-plugin-updater`, `tauri-plugin-store`, `tauri-plugin-positioner` -- already support Linux
- `window-vibrancy` -- keep as-is, no Linux calls (CSS fallback instead)
- `serde`, `serde_json`, `tokio`, `regex`, `once_cell`, `eventsource-stream`, `futures-util` -- platform-independent
- `tauri-plugin-http` -- platform-independent

### No new npm/pnpm packages. No frontend changes.

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Process info | Direct `/proc` reads | `procfs` crate v0.17 | 15+ transitive deps for ~50 lines of `std::fs` calls. Consistency with raw FFI pattern. |
| Process info | Direct `/proc` reads | `sysinfo` crate | Massive dependency. We need 3 simple reads. |
| Terminal text | `atspi` crate | Raw `dbus`/`zbus` + manual AT-SPI2 | `atspi` provides typed interfaces (Text, Accessible). Raw D-Bus requires manual message construction. |
| Terminal text | `atspi` crate | PTY read via `/proc/PID/fd` | Requires `ptrace`/root. Raw bytes without ANSI rendering. Not viable. |
| Clipboard | `arboard` | `xclip`/`wl-copy` subprocess | `arboard` is already a dep. Feature flag is cleaner than subprocess. |
| Paste keystroke | `xdotool`/`wtype` subprocess | `enigo` crate | `enigo` has Wayland issues and adds compile-time complexity. Subprocess is reliable and follows the macOS `osascript` pattern. |
| Active window | `x11rb` direct protocol | `xdotool` subprocess | `x11rb` is faster (~2ms vs ~50ms). Critical for hotkey handler latency. |
| Active window | X11-only | All compositor-specific Wayland APIs | Fragmented (GNOME/KDE/Sway/Hyprland each different). Not feasible for v1. |
| Global hotkey | Tauri plugin (X11) | `evdev` kernel input | Requires `input` group membership. Not appropriate for user-facing app. |
| Vibrancy | CSS semi-transparent | KDE blur hint / GNOME extension | Compositor-specific, unreliable. Solid translucent background is standard on Linux. |
| Truncation | Character-based | `tiktoken-rs` token counting | ~5MB BPE data + C dep. 4 chars/token approximation is sufficient for truncation. |

---

## What Tauri Already Handles (DO NOT add libraries)

| Capability | Tauri Component | Notes |
|------------|----------------|-------|
| Window creation (X11+Wayland) | `tao` | Transparent, undecorated, skip-taskbar all work |
| Tray icon | `tauri` tray-icon feature | Uses `libayatana-appindicator` on Linux |
| HTTP streaming | `tauri-plugin-http` | Platform-independent |
| Config persistence | `tauri-plugin-store` | Filesystem-based, works everywhere |
| Auto-update | `tauri-plugin-updater` | Supports AppImage `.tar.gz` bundles |
| Keyring | `keyring` linux-native | Uses `secret-service` D-Bus (GNOME Keyring / KWallet) |
| Global shortcuts (X11) | `tauri-plugin-global-shortcut` | X11 RECORD extension |
| AppImage bundling | `tauri` CLI | Built-in bundle target |

---

## Platform Support Matrix

| Feature | macOS | Windows | Linux X11 | Linux Wayland |
|---------|-------|---------|-----------|---------------|
| Overlay always-on-top | NSPanel Floating | Win32 TOPMOST | EWMH `_ABOVE` | NO (use XWayland) |
| Global hotkey | Tauri plugin | Tauri plugin | Tauri plugin | NO (use XWayland) |
| Terminal text | AX API | UIA | AT-SPI2 | AT-SPI2 |
| CWD detection | `libproc` FFI | PEB read | `/proc/PID/cwd` | `/proc/PID/cwd` |
| Process tree | `libproc` FFI | Toolhelp32Snapshot | `/proc/PID/children` | `/proc/PID/children` |
| Clipboard write | `pbcopy` | `arboard` | `arboard` (X11) | `arboard` (wayland-data-control) |
| Paste action | AppleScript/CGEvent | SendInput Ctrl+V | `xdotool` Ctrl+Shift+V | `wtype` Ctrl+Shift+V |
| Active window PID | NSWorkspace FFI | GetForegroundWindow | `x11rb` EWMH | Degraded (no PID) |
| Vibrancy/blur | `window-vibrancy` | `window-vibrancy` | CSS fallback | CSS fallback |
| Distribution | DMG | NSIS | AppImage | AppImage |
| Auto-update | Tauri updater | Tauri updater | Tauri updater | Tauri updater |
| Smart truncation | Yes | Yes | Yes | Yes |

---

## System Dependencies

### Build-time (CI: ubuntu-22.04)
```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libatk-bridge2.0-dev \
  libatspi2.0-dev \
  libgtk-3-dev \
  libglib2.0-dev \
  libx11-dev \
  libxdo-dev
```

### Runtime (user system)
| Dependency | Purpose | Default installed? | Install command |
|------------|---------|-------------------|-----------------|
| `at-spi2-core` | Terminal text reading | Yes (GNOME/KDE/XFCE) | `sudo apt install at-spi2-core` |
| `xdotool` | Paste on X11 | Usually yes on X11 | `sudo apt install xdotool` |
| `wtype` | Paste on Wayland | No | `sudo apt install wtype` |
| `libayatana-appindicator3` | Tray icon | Yes (most DEs) | `sudo apt install libayatana-appindicator3-1` |

---

## Sources

- [Tauri v2 AppImage distribution](https://v2.tauri.app/distribute/appimage/) -- HIGH confidence
- [Tauri v2 Global Shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/) -- HIGH confidence
- [Tauri v2 Updater plugin](https://v2.tauri.app/plugin/updater/) -- HIGH confidence
- [Tauri tao Wayland always-on-top issue](https://github.com/tauri-apps/tao/issues/1134) -- HIGH confidence
- [Tauri global-hotkey Wayland issue](https://github.com/tauri-apps/global-hotkey/issues/28) -- HIGH confidence
- [window-vibrancy Linux status](https://github.com/tauri-apps/window-vibrancy) -- HIGH confidence
- [arboard Linux/Wayland support](https://github.com/1Password/arboard) -- HIGH confidence
- [atspi Rust crate](https://crates.io/crates/atspi) -- MEDIUM confidence (crate verified, terminal text reading needs empirical testing)
- [AT-SPI2 architecture](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/architecture.html) -- HIGH confidence
- [proc_pid_cwd man page](https://man7.org/linux/man-pages/man5/proc_pid_cwd.5.html) -- HIGH confidence
- [GNOME GlobalShortcuts portal request](https://discourse.gnome.org/t/feature-request-globalshortcuts-portal/15343) -- MEDIUM confidence (not implemented)
- [Wayland xdotool fragmentation](https://www.semicomplete.com/blog/xdotool-and-exploring-wayland-fragmentation/) -- HIGH confidence
- [xdg-activation protocol](https://wayland.app/protocols/xdg-activation-v1) -- HIGH confidence
- [x11rb crate](https://github.com/psychon/x11rb) -- HIGH confidence
- Existing codebase: `terminal/process.rs`, `terminal/mod.rs`, `commands/paste.rs`, `commands/hotkey.rs`, `Cargo.toml` -- HIGH confidence

---
*Stack research for: CMD+K v0.3.9 Linux Support & Smart Terminal Context*
*Researched: 2026-03-13*
