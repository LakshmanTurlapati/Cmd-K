# Phase 31: Linux Overlay & Hotkey - Research

**Researched:** 2026-03-14
**Domain:** Linux X11 overlay window, global hotkey, PID capture, CSS frosted glass
**Confidence:** HIGH

## Summary

Phase 31 brings the CMD+K overlay to Linux X11. The existing codebase already has macOS (NSPanel) and Windows (Win32 APIs) overlay implementations. Linux needs: (1) global hotkey registration via Tauri's existing global-shortcut plugin, (2) active window PID capture before overlay shows, (3) always-on-top floating window, and (4) CSS-only frosted glass since window-vibrancy doesn't support Linux.

The project already uses `tauri-plugin-global-shortcut 2.3.1` which depends on `global-hotkey 0.7.0` -- the version that migrated to `x11rb` for X11 backend. This means global hotkeys on X11 should work with the existing dependency. The `x11rb 0.13.2` crate is already a transitive dependency in Cargo.lock. For PID capture, we need to query `_NET_ACTIVE_WINDOW` and `_NET_WM_PID` X11 properties, which can be done via `x11rb` directly (already available) or via a simple `xdotool getactivewindow getwindowpid` subprocess call.

The overlay window uses Tauri's standard `WebviewWindow` on non-macOS platforms (already implemented in `window.rs`). Linux needs `set_always_on_top(true)` (same as Windows) and CSS-only styling since `window-vibrancy` crate has no Linux support. The existing overlay CSS uses `bg-black/60` which already provides a dark semi-transparent background. Adding `backdrop-blur` will give frosted glass on WebKitGTK (supported since WebKitGTK 2.30.0, shipped with most distros since Ubuntu 20.04+).

**Primary recommendation:** Use `x11rb` (already in dependency tree) for PID capture. Mirror the Windows `#[cfg(target_os = "windows")]` pattern in `hotkey.rs` and `lib.rs` with `#[cfg(target_os = "linux")]` blocks. CSS frosted glass is a minor tweak to the existing Overlay component.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOVRL-01 | System-wide Ctrl+K hotkey registers and triggers overlay on X11 | Already works via `tauri-plugin-global-shortcut 2.3.1` + `global-hotkey 0.7.0` (x11rb backend). Default hotkey on Linux is already "Ctrl+K" per `lib.rs` line 219. |
| LOVRL-02 | Overlay appears as floating window above active application on X11 | `set_always_on_top(true)` works on X11 (same pattern as Windows in `lib.rs`). Tauri window config already has `transparent: true`, `decorations: false`. |
| LOVRL-03 | Wayland users can run with `GDK_BACKEND=x11` (XWayland) for full overlay functionality | XWayland presents as X11 to applications. No code changes needed -- just documentation. |
| LOVRL-04 | Active window PID captured before overlay shows (capture-before-show pattern) | New `get_active_window_pid()` function using `x11rb` to read `_NET_ACTIVE_WINDOW` + `_NET_WM_PID` atoms, called in hotkey handler before `toggle_overlay()`. |
| LOVRL-05 | CSS-only frosted glass fallback (no window-vibrancy on Linux) | WebKitGTK 2.30+ supports `backdrop-filter: blur()`. Add platform detection + CSS class. Existing `bg-black/60` is the base. |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| x11rb | 0.13.2 | X11 property queries (_NET_ACTIVE_WINDOW, _NET_WM_PID) | Already in dependency tree via global-hotkey. Pure Rust, no C bindings. |
| tauri-plugin-global-shortcut | 2.3.1 | System-wide hotkey registration on X11 | Already in project. Uses global-hotkey 0.7.0 with x11rb backend. |
| tauri (WebviewWindow) | 2.x | Overlay window with always-on-top, transparent, decorationless | Already in project. Non-macOS path in window.rs already uses WebviewWindow. |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| x11rb-protocol | 0.13.2 | Protocol definitions for X11 atoms | Transitive dep, needed for atom interning |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| x11rb for PID | `xdotool getactivewindow getwindowpid` subprocess | Simpler but adds runtime dependency on xdotool being installed |
| x11rb for PID | `xprop -root _NET_ACTIVE_WINDOW` + `xprop -id WID _NET_WM_PID` | Two subprocess calls, parsing required, xprop must be installed |
| x11rb direct | x11_get_windows crate | Thin wrapper but unmaintained, x11rb is already in tree |

**Installation:**
No new dependencies needed. `x11rb` is already available as a transitive dependency. To use it directly:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { version = "0.13", features = ["allow-unsafe-code"] }
```
Note: Check if the existing x11rb features are sufficient. The `allow-unsafe-code` feature enables connection setup. The `randr` or `xfixes` features are NOT needed.

## Architecture Patterns

### Recommended Changes (Files to Modify)

```
src-tauri/src/
├── commands/
│   └── hotkey.rs          # Add get_active_window_pid() for Linux, add #[cfg(linux)] block in handler
├── lib.rs                 # Add Linux setup block: always-on-top, skip taskbar (no vibrancy)
├── terminal/
│   └── detect_linux.rs    # Already has exe detection -- no changes needed
└── (no new files needed)

src/
├── components/
│   └── Overlay.tsx        # Add isLinux() platform check, add frosted glass CSS class
├── utils/
│   └── platform.ts        # Add isLinux() helper function
```

### Pattern 1: Capture-Before-Show on Linux (Mirrors macOS/Windows)
**What:** Get the PID of the active X11 window BEFORE showing the overlay
**When to use:** In the hotkey handler, only when overlay is about to show (not hide)
**Example:**
```rust
// Source: x11rb docs + _NET_WM_PID EWMH spec
#[cfg(target_os = "linux")]
fn get_active_window_pid() -> Option<i32> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;

    let (conn, screen_num) = x11rb::connect(None).ok()?;
    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Intern atoms
    let net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW").ok()?.reply().ok()?;
    let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID").ok()?.reply().ok()?;

    // Get active window ID from root window property
    let active_reply = conn.get_property(
        false, root, net_active_window.atom, AtomEnum::WINDOW, 0, 1
    ).ok()?.reply().ok()?;

    let window_id = active_reply.value32()?.next()?;
    if window_id == 0 { return None; }

    // Get PID from window's _NET_WM_PID property
    let pid_reply = conn.get_property(
        false, window_id, net_wm_pid.atom, AtomEnum::CARDINAL, 0, 1
    ).ok()?.reply().ok()?;

    let pid = pid_reply.value32()?.next()?;
    if pid > 0 { Some(pid as i32) } else { None }
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn get_active_window_pid() -> Option<i32> {
    None
}
```

### Pattern 2: Linux Setup in lib.rs (Mirrors Windows Block)
**What:** Configure overlay window properties at startup
**When to use:** In the `.setup()` callback
**Example:**
```rust
// Linux: always-on-top, no vibrancy (CSS-only frosted glass)
#[cfg(target_os = "linux")]
{
    window
        .set_always_on_top(true)
        .expect("Failed to set always-on-top");
    eprintln!("[setup] Linux: always-on-top set, CSS-only frosted glass");
}
```

### Pattern 3: Hotkey Handler Linux Block
**What:** Capture active window PID and compute window key on Linux
**When to use:** In `register_hotkey()` handler when overlay is about to show
**Example:**
```rust
#[cfg(target_os = "linux")]
{
    let pid = get_active_window_pid();
    eprintln!("[hotkey] Linux: capturing active window PID: {:?}", pid);
    if let Some(pid) = pid {
        if let Some(state) = app_handle.try_state::<AppState>() {
            if let Ok(mut prev) = state.previous_app_pid.lock() {
                *prev = Some(pid);
            }
        }
        // Compute window key from exe_name + shell_pid
        let window_key = compute_window_key_linux(pid);
        if let Some(state) = app_handle.try_state::<AppState>() {
            if let Ok(mut wk) = state.current_window_key.lock() {
                *wk = Some(window_key);
            }
        }
    }
}
```

### Pattern 4: compute_window_key_linux
**What:** Linux window key computation from X11 active window PID
**When to use:** In hotkey handler after PID capture
**Example:**
```rust
#[cfg(target_os = "linux")]
fn compute_window_key_linux(pid: i32) -> String {
    use crate::terminal::detect_linux;

    let exe_name = detect_linux::get_exe_name_for_pid(pid);
    let exe_str = exe_name.as_deref().unwrap_or("unknown");
    let is_terminal = detect_linux::is_known_terminal_exe(exe_str);
    let is_ide = detect_linux::is_ide_with_terminal_exe(exe_str);

    let key = if is_terminal || is_ide {
        match terminal::process::find_shell_pid(pid, None, None) {
            Some(shell_pid) => format!("{}:{}", exe_str, shell_pid),
            None => format!("{}:{}", exe_str, pid),
        }
    } else {
        format!("{}:{}", exe_str, pid)
    };

    eprintln!("[hotkey] computed Linux window_key: {}", &key);
    key
}
```

### Pattern 5: CSS Frosted Glass Fallback
**What:** Pure CSS frosted glass for Linux (no window-vibrancy)
**When to use:** On Linux platform only
**Example:**
```tsx
// In Overlay.tsx
const isLinuxPlatform = isLinux();

<div className={[
    panelWidth,
    isLinuxPlatform ? "rounded-lg" : isWindows() ? "rounded-md" : "rounded-xl",
    "shadow-2xl",
    isLinuxPlatform
      ? "bg-[#1a1a1c]/90 backdrop-blur-xl border border-white/10"
      : "bg-black/60",
    "p-4",
    "flex flex-col gap-2",
    animClass,
].filter(Boolean).join(" ")}
```

### Anti-Patterns to Avoid
- **Spawning xdotool subprocess for PID:** Adds runtime dependency, slower, parsing overhead. Use x11rb directly since it's already in the tree.
- **Trying window-vibrancy on Linux:** The crate explicitly does not support Linux. Don't even conditionally try it.
- **Opening a new X11 connection per hotkey press:** Cache the connection or accept the small overhead (connect + 3 roundtrips takes ~1-2ms on local X11). Given the hotkey fires at most once per 200ms (debounce), this is acceptable.
- **Attempting Wayland-native APIs:** Wayland has no global hotkey or always-on-top protocol. The project decision is X11-first with XWayland fallback.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| X11 property reading | Custom C FFI to Xlib | x11rb (already in tree) | Pure Rust, safe API, handles connection management |
| Global hotkey on X11 | Custom XGrabKey implementation | tauri-plugin-global-shortcut (already in project) | Handles key codes, modifiers, event loop integration |
| Always-on-top on X11 | Custom _NET_WM_STATE manipulation | Tauri's set_always_on_top() | Tauri/tao handles EWMH state changes internally |
| Frosted glass effect | Custom shader/canvas blur | CSS backdrop-filter: blur() | WebKitGTK 2.30+ supports it natively |

**Key insight:** Almost all infrastructure already exists. The Linux overlay is primarily about wiring up platform-specific `#[cfg]` blocks and one new X11 PID capture function.

## Common Pitfalls

### Pitfall 1: X11 Connection Lifetime
**What goes wrong:** Opening x11rb connection fails silently on Wayland without XWayland
**Why it happens:** No `DISPLAY` environment variable set on pure Wayland
**How to avoid:** Check `std::env::var("DISPLAY").is_ok()` before attempting X11 connection. Return None gracefully.
**Warning signs:** `x11rb::connect(None)` returns `Err` in logs

### Pitfall 2: _NET_WM_PID Not Set
**What goes wrong:** Some X11 windows don't set `_NET_WM_PID` property (e.g., some Java apps, very old xterm)
**Why it happens:** `_NET_WM_PID` is an EWMH convention, not mandatory
**How to avoid:** Return None gracefully. All major modern terminals (GNOME Terminal, kitty, Alacritty, etc.) and IDEs set this property correctly.
**Warning signs:** PID comes back as None for specific apps despite window being active

### Pitfall 3: Always-on-Top is a "Hint" on X11
**What goes wrong:** Some window managers may not honor always-on-top
**Why it happens:** X11 EWMH properties are hints, not mandates. Tiling WMs (i3, sway) may ignore or reinterpret them.
**How to avoid:** Document that floating WMs (GNOME, KDE, XFCE) provide full support. Tiling WM users may need WM-specific rules.
**Warning signs:** Overlay appears behind other windows on tiling WMs

### Pitfall 4: Transparent Window on Some Compositors
**What goes wrong:** Window background appears opaque black instead of transparent
**Why it happens:** Transparency requires a compositor running (picom, compton, or built-in like GNOME's mutter)
**How to avoid:** tauri.conf.json already has `"transparent": true`. Most modern DEs have compositors enabled by default. Document requirement.
**Warning signs:** Overlay has solid black background instead of semi-transparent

### Pitfall 5: Focus Restoration on Linux
**What goes wrong:** After hiding overlay, focus doesn't return to original window
**Why it happens:** Unlike macOS NSPanel and Windows SetForegroundWindow, Linux has no standard "restore focus to previous window" API
**How to avoid:** When overlay hides, the WM should auto-focus the previous window since the overlay was always-on-top. If not, can use `xdotool windowactivate` with the stored X11 window ID as a fallback.
**Warning signs:** After Escape, user has to click on the terminal to refocus it

### Pitfall 6: WebKitGTK Backdrop Filter Prefix
**What goes wrong:** `backdrop-filter: blur()` doesn't work
**Why it happens:** Older WebKitGTK versions may require `-webkit-backdrop-filter`
**How to avoid:** Tailwind's `backdrop-blur-xl` generates both prefixed and unprefixed versions. Verify with `npx tailwindcss --help` or check generated CSS.
**Warning signs:** Overlay background is flat dark without any blur effect

## Code Examples

### Active Window PID Capture (Full Implementation)
```rust
// Source: x11rb docs + EWMH _NET_ACTIVE_WINDOW / _NET_WM_PID spec
#[cfg(target_os = "linux")]
fn get_active_window_pid() -> Option<i32> {
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::*;

    // Check DISPLAY is set (fails gracefully on pure Wayland without XWayland)
    if std::env::var("DISPLAY").is_err() {
        eprintln!("[hotkey] DISPLAY not set, cannot capture X11 active window PID");
        return None;
    }

    let (conn, screen_num) = match x11rb::connect(None) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[hotkey] X11 connect failed: {}", e);
            return None;
        }
    };

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Intern atoms for EWMH properties
    let net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .ok()?.reply().ok()?;
    let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID")
        .ok()?.reply().ok()?;

    // Query root window for _NET_ACTIVE_WINDOW
    let active_reply = conn.get_property(
        false,
        root,
        net_active_window.atom,
        AtomEnum::WINDOW,
        0,
        1,
    ).ok()?.reply().ok()?;

    let window_id = active_reply.value32()?.next()?;
    if window_id == 0 || window_id == root {
        eprintln!("[hotkey] Active window is root or none");
        return None;
    }

    // Query active window for _NET_WM_PID
    let pid_reply = conn.get_property(
        false,
        window_id,
        net_wm_pid.atom,
        AtomEnum::CARDINAL,
        0,
        1,
    ).ok()?.reply().ok()?;

    let pid = pid_reply.value32()?.next()?;
    eprintln!("[hotkey] Linux active window 0x{:x} -> PID {}", window_id, pid);
    if pid > 0 { Some(pid as i32) } else { None }
}
```

### Platform Detection (Frontend)
```typescript
// Source: navigator.userAgent patterns
export function isLinux(): boolean {
  return navigator.userAgent.includes("Linux") && !navigator.userAgent.includes("Android");
}
```

### CSS Frosted Glass for Linux
```tsx
// Source: Tailwind CSS docs + existing Overlay.tsx pattern
// The key classes: bg-[#1a1a1c]/90 gives dark base, backdrop-blur-xl adds blur,
// border border-white/10 adds subtle edge definition
const frostedLinux = "bg-[#1a1a1c]/90 backdrop-blur-xl border border-white/10";
const frostedMacOS = "bg-black/60"; // vibrancy applied natively
const frostedWindows = "bg-black/60"; // acrylic applied natively

const bgClass = isLinux() ? frostedLinux : "bg-black/60";
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| global-hotkey with x11-dl | global-hotkey 0.7.0 with x11rb | May 2025 | Pure Rust X11 backend, likely fixes earlier X11 hotkey issues |
| xdotool for PID capture | x11rb direct property query | N/A | No runtime dependency on xdotool |
| No Linux support | X11-first with XWayland fallback | v0.3.9 roadmap | Industry standard approach for Linux desktop apps |

**Deprecated/outdated:**
- `x11-dl` crate for X11 bindings: replaced by `x11rb` in global-hotkey 0.7.0
- Wayland-native global hotkeys: still no protocol support, XWayland remains the standard fallback

## Open Questions

1. **X11 Connection Per Hotkey vs Cached Connection**
   - What we know: x11rb::connect() takes ~1ms on local X11. Hotkey fires at most every 200ms.
   - What's unclear: Whether keeping a persistent X11 connection has benefits or adds complexity
   - Recommendation: Open fresh connection per hotkey press (simple, reliable, no state management). The 1ms overhead is negligible.

2. **Focus Restoration on Linux**
   - What we know: macOS uses NSPanel (auto-restores focus). Windows uses SetForegroundWindow. Linux has no standard equivalent.
   - What's unclear: Whether hiding an always-on-top Tauri window auto-returns focus to the previously focused window across all WMs
   - Recommendation: Test on GNOME/KDE first. If focus is lost, store the X11 window ID alongside PID and use `x11rb` to call `set_input_focus()` on hide. This can be deferred to a follow-up if not needed.

3. **Tiling Window Manager Compatibility**
   - What we know: i3/sway may not honor always-on-top hints or may tile the overlay
   - What's unclear: Whether Tauri's window type or additional EWMH hints can prevent tiling
   - Recommendation: Out of scope for initial implementation. Document that floating WMs are the primary target. Tiling WM users can add WM rules (e.g., `for_window [class="cmd-k"] floating enable` in i3).

## Sources

### Primary (HIGH confidence)
- Cargo.lock analysis: `global-hotkey 0.7.0`, `x11rb 0.13.2`, `tauri-plugin-global-shortcut 2.3.1` versions confirmed
- Codebase analysis: `hotkey.rs`, `window.rs`, `lib.rs`, `detect_linux.rs` patterns examined
- x11rb docs: https://docs.rs/x11rb/latest/x11rb/ - property query API
- EWMH spec: _NET_ACTIVE_WINDOW and _NET_WM_PID are standard EWMH properties

### Secondary (MEDIUM confidence)
- global-hotkey releases: https://github.com/tauri-apps/global-hotkey/releases - v0.7.0 migrated to x11rb
- Tauri window customization: https://v2.tauri.app/learn/window-customization/ - Linux transparency and always-on-top
- WebKitGTK 2.30.0 release notes: backdrop-filter support confirmed since 2020

### Tertiary (LOW confidence)
- tao issue #501: X11 hotkey reliability concerns. May be resolved by x11rb migration in global-hotkey 0.7.0 but not explicitly confirmed.
- Tiling WM compatibility: No definitive testing data for Tauri overlays on i3/sway

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all dependencies already in project, versions confirmed from Cargo.lock
- Architecture: HIGH - follows exact same cfg-gate pattern used for macOS and Windows
- PID capture: HIGH - EWMH _NET_ACTIVE_WINDOW/_NET_WM_PID is well-documented standard
- CSS frosted glass: HIGH - WebKitGTK supports backdrop-filter, Tailwind generates prefixed CSS
- Pitfalls: MEDIUM - focus restoration and tiling WM behavior needs real-world testing

**Research date:** 2026-03-14
**Valid until:** 2026-04-14 (stable technologies, 30-day window)
