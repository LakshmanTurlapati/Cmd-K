# Phase 11: Build Infrastructure and Overlay Foundation - Research

**Researched:** 2026-03-02
**Domain:** Windows Win32 window management, Acrylic/Mica vibrancy, cross-platform Rust build infrastructure, focus management
**Confidence:** HIGH (core APIs are stable Win32; window-vibrancy crate is Tauri-owned and well-documented)

## Summary

Phase 11 requires two major workstreams: (1) restructuring the Cargo.toml and Rust source to compile cleanly on both macOS and Windows, and (2) creating a Windows overlay window with frosted glass vibrancy, always-on-top behavior, taskbar/Alt+Tab hiding, and correct focus capture/restore.

The existing codebase already has extensive `#[cfg(target_os = "macos")]` / `#[cfg(not(target_os = "macos"))]` stubs for cross-compilation, but lib.rs unconditionally imports `tauri_nspanel` and `window_vibrancy::NSVisualEffectMaterial`, which will fail on Windows. The Cargo.toml lists macOS-only crates (`tauri-nspanel`, `accessibility-sys`, `core-foundation-sys`) as unconditional dependencies. These must be platform-gated.

On Windows, the overlay architecture differs fundamentally from macOS. macOS uses NSPanel (a non-activating floating panel that accepts keyboard input without permanently stealing focus). Windows has no direct equivalent -- WS_EX_NOACTIVATE prevents keyboard input entirely. The practical approach used by launchers (Raycast Windows, PowerToys Run, etc.) is: the overlay DOES become the foreground window (accepting keyboard input normally), and on dismiss, focus is explicitly restored to the previously captured HWND via SetForegroundWindow with the AllowSetForegroundWindow/AttachThreadInput workaround.

**Primary recommendation:** Use `windows-sys` for raw Win32 API bindings (fast compile, zero overhead, sufficient for the C-style APIs needed). Use `window-vibrancy` 0.5+ (already in the project) for Acrylic/Mica vibrancy (it supports Windows natively). Upgrade `window-vibrancy` to 0.7.x for latest fixes. On Windows, use a standard Tauri window with WS_EX_TOOLWINDOW style (manual Win32 call) rather than trying to replicate NSPanel behavior.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Match macOS frosted glass look as closely as possible -- pixel-match the blur radius and tint opacity
- Use Acrylic blur on Windows (closest to macOS NSVisualEffectView behind-window blur)
- Match macOS edge treatment: rounded corners and subtle shadow replicating the macOS floating panel
- Minimum Windows version: Win10 1903+ (Acrylic support required) -- no fallback for older builds
- Full DPI awareness required: overlay renders crisply at any scaling factor (100%, 150%, 200%)
- Match macOS dimensions: 320px fixed width, dynamic height based on content
- Horizontal position: centered on current monitor (not primary monitor)
- Vertical position: 25% down from top of current monitor (same as macOS)
- Multi-monitor: use current/active monitor (same as macOS current_monitor() approach) -- Claude's discretion on implementation details
- Height resizes dynamically via frontend (same ResizeObserver pattern as macOS)
- Hotkey re-press: toggle behavior (same as macOS) -- pressing Ctrl+Shift+K while visible dismisses overlay
- Click-outside dismiss: yes, overlay dismisses when user clicks outside (same as macOS blur/click handling)
- Guard dismissal during paste operations and AI streaming (same guards as macOS)
- Show animation: scale 0.96->1, opacity 0->1, 120ms ease-out (same CSS keyframes as macOS)
- Hide animation: scale 1->0.96, opacity 1->0, 100ms ease-in (same CSS keyframes as macOS)
- 200ms debounce on hotkey to prevent double-fire (same as macOS)
- Hotkey configurable from day one (same as macOS) -- default Ctrl+Shift+K, changeable via settings
- Use AllowSetForegroundWindow / AttachThreadInput workaround to ensure reliable focus restoration (Windows-specific)
- Overlay captures HWND of ANY foreground window, not terminals only (same as macOS which captures any frontmost app PID)
- Overlay appears from ANY application, not just terminals (system-wide launcher, same as macOS)
- If non-terminal: still show overlay, capture app context instead of terminal context
- Window closure during overlay: context reads fail silently, overlay remains functional (same graceful degradation as macOS)
- Cargo.toml platform-gates macOS-only deps (cocoa, objc, etc.) behind [target.'cfg(target_os = "macos")']
- Windows-only deps (windows-sys, etc.) behind [target.'cfg(target_os = "windows")']
- Project must compile on both platforms without cfg breakage
- Always-on-top: equivalent to macOS NSPanel floating level (PanelLevel::Status = 25)
- Skip taskbar: WS_EX_TOOLWINDOW style (equivalent to macOS panel not appearing in Dock)
- Skip Alt+Tab: WS_EX_TOOLWINDOW achieves this (equivalent to macOS panel not appearing in Cmd+Tab)
- Accepts keyboard input without permanently stealing focus (equivalent to macOS nonactivating panel style)

### Claude's Discretion
- Exact Windows API calls for Acrylic composition (DwmSetWindowAttribute vs SetWindowCompositionAttribute)
- Rust crate choice for Windows APIs (windows-sys vs windows-rs)
- How to structure cross-platform code (separate modules, cfg blocks, traits)
- Exact AttachThreadInput sequence for focus workaround
- Window class registration details

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| WOVL-01 | Overlay window appears with Acrylic (Win10) or Mica (Win11) frosted glass vibrancy | `window-vibrancy` crate provides `apply_acrylic()` and `apply_mica()` with runtime OS version detection via `windows-version` crate |
| WOVL-02 | Overlay floats above all windows with always-on-top and skip-taskbar behavior | Tauri's `set_always_on_top(true)` + manual WS_EX_TOOLWINDOW via Win32 SetWindowLongPtrW |
| WOVL-03 | Overlay does not appear in Alt+Tab or taskbar (WS_EX_TOOLWINDOW) | WS_EX_TOOLWINDOW extended style hides from both Alt+Tab and taskbar; Tauri's built-in skipTaskbar is buggy (issue #10422), use direct Win32 call |
| WOVL-04 | Previous window HWND captured before overlay shows for focus restoration | `GetForegroundWindow()` called BEFORE showing overlay (same pattern as macOS PID capture in hotkey.rs) |
| WOVL-05 | Focus returns to previous terminal window on overlay dismiss (SetForegroundWindow) | AttachThreadInput + SetForegroundWindow sequence; AllowSetForegroundWindow as fallback; SendInput(ALT) as last resort |
| WOVL-06 | Ctrl+Shift+K default hotkey triggers overlay system-wide (configurable) | `tauri_plugin_global_shortcut` works cross-platform; existing hotkey.rs architecture applies directly |
| WOVL-07 | Escape dismisses overlay without executing | Frontend Overlay.tsx already handles Escape; hide_overlay command needs Windows path |
| WBLD-01 | Cargo.toml platform-gates macOS-only deps and adds Windows-only deps | `[target.'cfg(target_os = "macos")'.dependencies]` and `[target.'cfg(target_os = "windows")'.dependencies]` sections |
| WBLD-02 | Project compiles on both macOS and Windows without regressions | All macOS-only imports (tauri_nspanel, accessibility_sys, core_foundation_sys) gated behind #[cfg(target_os = "macos")]; non-macOS stubs already exist for most functions |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| windows-sys | 0.59+ | Raw Win32 API bindings (GetForegroundWindow, SetForegroundWindow, SetWindowLongPtrW, etc.) | Zero-overhead, fast compile, maintained by Microsoft, sufficient for C-style Win32 APIs needed here |
| window-vibrancy | 0.7.x | Acrylic and Mica frosted glass effects | Already in project (0.5); Tauri-owned crate; handles all DwmSetWindowAttribute/SetWindowCompositionAttribute complexity internally |
| windows-version | 0.1+ | Runtime Windows version detection (Win10 vs Win11) | Dependency of window-vibrancy; use directly for Acrylic-vs-Mica branching |
| tauri | 2.x | Window management, WebView, IPC | Already in project; provides set_always_on_top(), WebviewWindow, HWND access via HasWindowHandle |
| tauri-plugin-global-shortcut | 2.x | System-wide hotkey registration | Already in project; works cross-platform including Windows |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| keyring | 3.x | Credential storage | Already in project with "apple-native" feature; needs "windows-native" feature added for Windows Credential Manager |
| raw-window-handle | 0.6+ | Cross-platform window handle trait | Transitive dependency; WebviewWindow implements HasWindowHandle for HWND access |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| windows-sys | windows (full crate) | More ergonomic but slower compile times; not needed since all APIs here are simple C-style Win32 calls |
| Manual WS_EX_TOOLWINDOW | Tauri's set_skip_taskbar() | Tauri's implementation is buggy on Windows (issue #10422, still open); direct Win32 call is reliable |
| Manual Acrylic setup | window-vibrancy | window-vibrancy handles the undocumented SetWindowCompositionAttribute for Win10 and DwmSetWindowAttribute for Win11 internally; no reason to hand-roll |

**Cargo.toml changes needed:**
```toml
# Move macOS-only deps to platform-gated section
[target.'cfg(target_os = "macos")'.dependencies]
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
accessibility-sys = "0.2"
core-foundation-sys = "0.8"

# Add Windows-only deps
[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "Win32_System_Threading",
] }

# Keep cross-platform deps in main [dependencies]
# window-vibrancy stays cross-platform (handles both macOS and Windows)
# keyring needs both features:
keyring = { version = "3", features = ["apple-native", "windows-native"] }

# Upgrade window-vibrancy for latest Windows fixes
window-vibrancy = "0.7"
```

## Architecture Patterns

### Recommended Project Structure

The existing codebase uses inline `#[cfg]` gating on individual functions with macOS implementations and non-macOS stubs. This pattern should continue for Phase 11, with Windows-specific code added as new implementations alongside existing stubs.

```
src-tauri/src/
  lib.rs              # Platform-branching setup (NSPanel on macOS, standard window on Windows)
  state.rs            # Cross-platform state (change previous_app_pid to handle HWND on Windows)
  commands/
    window.rs         # show_overlay / hide_overlay -- platform-branching for panel vs window
    hotkey.rs         # get_frontmost_pid -> get_foreground_hwnd on Windows
    paste.rs          # Phase 13 scope, but cfg gates already present
    ...
  platform/           # NEW: platform-specific modules (optional, or inline in existing files)
    mod.rs            # Re-exports platform API
    windows.rs        # Windows-specific: focus capture, WS_EX_TOOLWINDOW, vibrancy
    macos.rs          # Extract existing macOS-specific code (refactor, not required for phase 11)
```

### Pattern 1: Platform-Branching in lib.rs Setup

**What:** The Tauri setup code in lib.rs must branch between NSPanel (macOS) and standard Tauri window (Windows) for the overlay.

**When to use:** In the `.setup()` closure where the window is created and configured.

**Example:**
```rust
// lib.rs setup() -- macOS path (existing)
#[cfg(target_os = "macos")]
{
    use tauri_nspanel::{CollectionBehavior, PanelLevel, StyleMask, WebviewWindowExt};
    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

    apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))
        .expect("Failed to apply vibrancy");

    let panel = window.to_panel::<OverlayPanel>()
        .expect("Failed to convert to NSPanel");
    panel.set_level(PanelLevel::Status.value());
    panel.set_collection_behavior(/* ... */);
    panel.set_style_mask(StyleMask::empty().nonactivating_panel().into());
    panel.set_has_shadow(false);
}

// lib.rs setup() -- Windows path (new)
#[cfg(target_os = "windows")]
{
    use window_vibrancy::{apply_acrylic, apply_mica};

    // Apply vibrancy: try Mica first (Win11), fall back to Acrylic (Win10)
    let mica_result = apply_mica(&window, Some(true)); // dark mode
    if mica_result.is_err() {
        apply_acrylic(&window, Some((18, 18, 18, 125)))
            .expect("Failed to apply Acrylic -- requires Win10 1903+");
    }

    // Set always on top
    window.set_always_on_top(true).expect("Failed to set always-on-top");

    // Apply WS_EX_TOOLWINDOW to hide from Alt+Tab and taskbar
    apply_tool_window_style(&window);
}
```

### Pattern 2: HWND Capture Before Overlay Show (Windows Focus Management)

**What:** Before showing the overlay, capture the HWND of the currently focused window so we can restore focus on dismiss.

**When to use:** In the hotkey handler, BEFORE toggle_overlay().

**Example:**
```rust
// In hotkey.rs, Windows equivalent of get_frontmost_pid()
#[cfg(target_os = "windows")]
fn get_foreground_hwnd() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        None
    } else {
        Some(hwnd as isize)
    }
}
```

### Pattern 3: Focus Restoration via AttachThreadInput + SetForegroundWindow

**What:** On overlay dismiss, restore focus to the previously captured HWND using the AttachThreadInput workaround to bypass SetForegroundWindow restrictions.

**When to use:** In hide_overlay() on Windows.

**Example:**
```rust
#[cfg(target_os = "windows")]
fn restore_focus(target_hwnd: isize) -> Result<(), String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;

    unsafe {
        let hwnd = target_hwnd as HWND;
        let target_thread = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());
        let current_thread = GetCurrentThreadId();

        if target_thread != current_thread {
            AttachThreadInput(current_thread, target_thread, 1); // TRUE
        }

        let result = SetForegroundWindow(hwnd);

        if target_thread != current_thread {
            AttachThreadInput(current_thread, target_thread, 0); // FALSE
        }

        if result == 0 {
            // Fallback: simulate ALT key press to enable SetForegroundWindow
            // (SendInput technique from research)
            Err("SetForegroundWindow failed".into())
        } else {
            Ok(())
        }
    }
}
```

### Pattern 4: WS_EX_TOOLWINDOW Application

**What:** Apply extended window style to hide from Alt+Tab and taskbar.

**When to use:** During window setup on Windows (Tauri's skipTaskbar is buggy).

**Example:**
```rust
#[cfg(target_os = "windows")]
fn apply_tool_window_style(window: &tauri::WebviewWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32) = handle.as_raw() {
            let hwnd = win32.hwnd.get() as isize;
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                let new_style = (style | WS_EX_TOOLWINDOW as isize)
                    & !(WS_EX_APPWINDOW as isize);
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
            }
        }
    }
}
```

### Anti-Patterns to Avoid
- **Using WS_EX_NOACTIVATE for the overlay:** This prevents ALL keyboard input. The overlay needs keyboard input for the command input field. Instead, let the overlay become the foreground window and explicitly restore focus on dismiss.
- **Relying on Tauri's skipTaskbar on Windows:** Known broken (GitHub issue #10422, open since July 2024). Use direct Win32 WS_EX_TOOLWINDOW call instead.
- **Unconditional imports of platform crates:** `use tauri_nspanel::*` at module top level without `#[cfg]` will break Windows compilation. Every import of macOS-only crates must be behind `#[cfg(target_os = "macos")]`.
- **Storing PID for Windows focus restore:** macOS stores a PID (i32) for the frontmost app. Windows must store an HWND (isize/pointer). Do NOT try to convert between them. Use a platform-generic type or separate fields in AppState.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Acrylic/Mica vibrancy | Manual DwmSetWindowAttribute / SetWindowCompositionAttribute calls | `window-vibrancy` crate (`apply_acrylic()`, `apply_mica()`) | Handles undocumented Win10 API, Win11 differences, version detection, error handling internally |
| OS version detection | Manual registry reads or GetVersionEx | `windows-version` crate (`OsVersion::current()`) | Avoids manifest/compatibility mode issues that plague GetVersionEx |
| Global hotkey | Manual RegisterHotKey / Windows hooks | `tauri-plugin-global-shortcut` | Already in project, works cross-platform, handles registration/unregistration lifecycle |
| Window handle access | Manual FindWindow / EnumWindows | `raw-window-handle` trait on WebviewWindow | Tauri implements HasWindowHandle; gives direct HWND access safely |

**Key insight:** The window-vibrancy crate is Tauri-owned and handles the most complex parts (the undocumented SetWindowCompositionAttribute on Win10, DwmSetWindowAttribute on Win11, version branching). The Win32 calls we need to make manually are simple and well-documented (GetForegroundWindow, SetForegroundWindow, SetWindowLongPtrW, AttachThreadInput).

## Common Pitfalls

### Pitfall 1: Unconditional macOS Imports Break Windows Build
**What goes wrong:** `use tauri_nspanel::*` or `use window_vibrancy::NSVisualEffectMaterial` at the top of lib.rs without cfg gating causes compilation failure on Windows.
**Why it happens:** The current lib.rs imports tauri_nspanel types and NSVisualEffectMaterial unconditionally.
**How to avoid:** Wrap ALL macOS-only imports in `#[cfg(target_os = "macos")]` blocks. Move the `tauri_nspanel::tauri_panel!` macro invocation inside a `#[cfg(target_os = "macos")]` module.
**Warning signs:** `cargo check --target x86_64-pc-windows-msvc` fails with "unresolved import" errors.

### Pitfall 2: SetForegroundWindow Silently Fails
**What goes wrong:** After hiding the overlay, `SetForegroundWindow(hwnd)` flashes the taskbar button instead of restoring focus.
**Why it happens:** Windows restricts which processes can set the foreground window. Once the overlay is foreground, calling SetForegroundWindow for another process requires special handling.
**How to avoid:** Use the AttachThreadInput workaround: attach our thread to the target window's thread, call SetForegroundWindow, then detach. As fallback, use the SendInput(ALT key) technique which has near-100% reliability.
**Warning signs:** Focus does not return to the terminal after pressing Escape or accepting a command.

### Pitfall 3: Tauri skipTaskbar Bug on Windows
**What goes wrong:** Setting `skipTaskbar: true` in tauri.conf.json or calling `set_skip_taskbar(true)` does not actually hide the window from the Windows taskbar.
**Why it happens:** Known Tauri bug (GitHub issue #10422, open since July 2024). The Tauri maintainers rejected the WS_EX_TOOLWINDOW workaround as a built-in fix because it changes other window behaviors.
**How to avoid:** Apply WS_EX_TOOLWINDOW manually via SetWindowLongPtrW in the setup code. This achieves both taskbar hiding AND Alt+Tab hiding in one call.
**Warning signs:** Overlay appears in taskbar or Alt+Tab switcher during testing.

### Pitfall 4: DPI Mismatch on High-DPI Monitors
**What goes wrong:** Overlay appears at wrong size or position on displays with 125%, 150%, or 200% scaling.
**Why it happens:** Physical vs logical pixel confusion in positioning calculations.
**How to avoid:** Tauri's WebView2 runtime handles DPI awareness for rendering. For positioning, use Tauri's `current_monitor()` which returns logical coordinates. The existing position_overlay() function already handles scale_factor() correctly -- ensure the same pattern is used on Windows.
**Warning signs:** Overlay appears off-center or at wrong vertical position on scaled displays.

### Pitfall 5: Window Vibrancy Version Mismatch
**What goes wrong:** `apply_acrylic()` or `apply_mica()` fails with trait mismatch errors.
**Why it happens:** Current project uses window-vibrancy 0.5 which uses `HasRawWindowHandle`. Version 0.7+ uses `HasWindowHandle`. Tauri v2 provides `HasWindowHandle`.
**How to avoid:** Upgrade window-vibrancy to 0.7.x. The function signatures accept `impl HasWindowHandle` which Tauri's WebviewWindow implements.
**Warning signs:** Compilation errors about trait bounds on window-vibrancy function calls.

### Pitfall 6: Rounded Corners on Win10 vs Win11
**What goes wrong:** Overlay has sharp corners on Windows 10, looks inconsistent with macOS.
**Why it happens:** Windows 11 automatically rounds top-level window corners via DWM. Windows 10 does not.
**How to avoid:** On Win10, rely on the CSS border-radius in the WebView content (the existing overlay CSS already has rounded corners). The transparent window background allows the CSS radius to show through. On Win11, DWM rounds automatically -- optionally set `DWMWCP_ROUND` via `DwmSetWindowAttribute(DWMWA_WINDOW_CORNER_PREFERENCE)` for explicit control.
**Warning signs:** Sharp corners visible around the overlay on Windows 10.

### Pitfall 7: AppState Type Incompatibility (PID vs HWND)
**What goes wrong:** AppState stores `previous_app_pid: Mutex<Option<i32>>` which works for macOS PID but not for Windows HWND (which is a pointer-sized integer, isize).
**Why it happens:** macOS identifies apps by PID (i32), Windows identifies windows by HWND (isize/pointer).
**How to avoid:** Either: (a) add a separate `previous_hwnd: Mutex<Option<isize>>` field for Windows, or (b) change to a platform-abstracted type. Option (a) is simpler and avoids refactoring macOS code.
**Warning signs:** Type errors when trying to store HWND in the i32 PID field.

## Code Examples

Verified patterns from official sources:

### Acrylic/Mica Application with Runtime Version Detection
```rust
// Source: window-vibrancy docs + windows-version docs
#[cfg(target_os = "windows")]
fn apply_windows_vibrancy(window: &tauri::WebviewWindow) {
    use window_vibrancy::{apply_acrylic, apply_mica};

    // Try Mica first (Win11 only), fall back to Acrylic (Win10 1903+)
    match apply_mica(window, Some(true)) {
        Ok(_) => eprintln!("[vibrancy] Mica applied (Windows 11)"),
        Err(_) => {
            // Acrylic with dark semi-transparent tint
            // RGBA: (18, 18, 18, 125) matches macOS HudWindow darkness
            apply_acrylic(window, Some((18, 18, 18, 125)))
                .expect("Acrylic requires Win10 1903+");
            eprintln!("[vibrancy] Acrylic applied (Windows 10)");
        }
    }
}
```

### GetForegroundWindow for HWND Capture
```rust
// Source: windows-sys docs, Microsoft Learn SetForegroundWindow docs
#[cfg(target_os = "windows")]
fn capture_foreground_hwnd() -> Option<isize> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd == 0 {
        None
    } else {
        Some(hwnd)
    }
}
```

### WS_EX_TOOLWINDOW via SetWindowLongPtrW
```rust
// Source: Microsoft Learn Extended Window Styles docs, Tauri issue #10422 workaround
#[cfg(target_os = "windows")]
fn apply_tool_window_ex_style(hwnd: isize) {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_style = (ex_style | WS_EX_TOOLWINDOW as isize)
            & !(WS_EX_APPWINDOW as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_style);
    }
}
```

### Focus Restoration Sequence
```rust
// Source: Microsoft Learn SetForegroundWindow docs, AttachThreadInput bypass gist
#[cfg(target_os = "windows")]
fn restore_foreground(target_hwnd: isize) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    use windows_sys::Win32::System::Threading::GetCurrentThreadId;

    unsafe {
        let target_thread = GetWindowThreadProcessId(target_hwnd, std::ptr::null_mut());
        let our_thread = GetCurrentThreadId();

        // Attach our thread to the target's thread input queue
        if target_thread != 0 && target_thread != our_thread {
            AttachThreadInput(our_thread, target_thread, 1); // TRUE = attach
        }

        let ok = SetForegroundWindow(target_hwnd);

        // Always detach
        if target_thread != 0 && target_thread != our_thread {
            AttachThreadInput(our_thread, target_thread, 0); // FALSE = detach
        }

        ok != 0
    }
}
```

### Platform-Gated Cargo.toml Dependencies
```toml
# Source: Cargo reference, Tauri platform configuration docs
[dependencies]
# Cross-platform dependencies (both macOS and Windows)
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-global-shortcut = "2"
tauri-plugin-positioner = "2"
tauri-plugin-store = "2"
window-vibrancy = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tauri-plugin-http = { version = "2", features = ["stream"] }
eventsource-stream = "0.2"
futures-util = "0.3"
tokio = { version = "1", features = ["time"] }
regex = "1"
once_cell = "1"

[target.'cfg(target_os = "macos")'.dependencies]
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
accessibility-sys = "0.2"
core-foundation-sys = "0.8"
keyring = { version = "3", features = ["apple-native"] }

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "Win32_System_Threading",
] }
keyring = { version = "3", features = ["windows-native"] }
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SetWindowCompositionAttribute (undocumented) for Acrylic | DwmSetWindowAttribute with DWMWA_SYSTEMBACKDROP_TYPE (Win11 22H2+) | Windows 11 22H2 (2022) | window-vibrancy handles both; no manual change needed |
| HasRawWindowHandle (raw-window-handle 0.5) | HasWindowHandle (raw-window-handle 0.6) | 2024 | window-vibrancy 0.5+ requires 0.6 traits; Tauri v2 provides them |
| GetVersionEx for OS detection | windows-version crate (RtlGetVersion internally) | Ongoing | GetVersionEx lies in compatibility mode; windows-version is reliable |
| Tauri skipTaskbar for Alt+Tab hiding | Manual WS_EX_TOOLWINDOW via SetWindowLongPtrW | Tauri issue #10422, July 2024 | Tauri's built-in method is broken on Windows; direct Win32 call works |

**Deprecated/outdated:**
- `winapi` crate: Replaced by Microsoft's official `windows-sys` / `windows` crates
- `GetVersionEx`: Returns fake version numbers due to Windows compatibility manifests; use `windows-version` crate instead
- `HasRawWindowHandle` trait: Superseded by `HasWindowHandle` in raw-window-handle 0.6

## Open Questions

1. **WS_EX_TOOLWINDOW + Keyboard Input Interaction with WebView2**
   - What we know: WS_EX_TOOLWINDOW changes window behavior (smaller title bar, not in Alt+Tab). It does NOT prevent keyboard input (unlike WS_EX_NOACTIVATE).
   - What's unclear: Whether WebView2 inside a WS_EX_TOOLWINDOW window has any input quirks. The STATE.md notes: "WS_EX_NOACTIVATE + WebView keyboard input interaction needs prototyping on Windows hardware."
   - Recommendation: WS_EX_TOOLWINDOW (not WS_EX_NOACTIVATE) is the correct choice. It should work fine with WebView2 keyboard input, but this should be validated early in implementation. If issues arise, the fallback is to not use WS_EX_TOOLWINDOW and hide the taskbar entry via other means.

2. **Acrylic Performance on Window Resize**
   - What we know: window-vibrancy docs note "performance issues on Win10 v1903+ and Win11 build 22000 during window resizing or dragging." The overlay uses dynamic height via ResizeObserver.
   - What's unclear: Whether the dynamic height resizing (which changes WebView size) triggers the Acrylic performance issue.
   - Recommendation: Monitor performance during height transitions. If stuttering occurs, consider using a fixed maximum height with internal scrolling, or switching to Mica (Win11) which does not have this issue.

3. **Click-Outside Dismiss on Windows**
   - What we know: macOS uses NSPanel's key window resignation (blur event) to detect click-outside. Windows has no direct equivalent for a standard window.
   - What's unclear: Exact mechanism for detecting clicks outside the overlay on Windows. Options: WM_ACTIVATE with WA_INACTIVE, global mouse hook, or WebView blur event.
   - Recommendation: Use WM_ACTIVATE notification (sent when the window is deactivated because user clicked elsewhere). Tauri likely forwards this as a focus-lost event. Alternatively, the frontend already listens for window blur events which should fire when the Tauri window loses focus.

4. **Keyring Feature Flag Compatibility**
   - What we know: Current Cargo.toml has `keyring = { version = "3", features = ["apple-native"] }`. This needs both `apple-native` (macOS) and `windows-native` (Windows) features.
   - What's unclear: Whether specifying both features in a non-platform-gated dependency causes build issues on each platform, or whether the keyring crate handles this gracefully with cfg.
   - Recommendation: Platform-gate keyring in the Cargo.toml (macOS section gets "apple-native", Windows section gets "windows-native") to avoid pulling unnecessary dependencies on each platform.

## Sources

### Primary (HIGH confidence)
- [window-vibrancy docs](https://docs.rs/window-vibrancy/latest/window_vibrancy/) - apply_acrylic, apply_mica API signatures, Windows version requirements
- [window-vibrancy crates.io](https://crates.io/crates/window-vibrancy) - Version 0.7.1, dependencies (windows-sys 0.60, windows-version 0.1)
- [Microsoft Learn: SetForegroundWindow](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) - Focus restrictions, AllowSetForegroundWindow, conditions for success
- [Microsoft Learn: Extended Window Styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles) - WS_EX_TOOLWINDOW, WS_EX_NOACTIVATE, WS_EX_APPWINDOW behavior
- [Microsoft Learn: DWM_WINDOW_CORNER_PREFERENCE](https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwm_window_corner_preference) - Rounded corners API for Win11
- [Kenny Kerr: windows vs windows-sys](https://kennykerr.ca/rust-getting-started/windows-or-windows-sys.html) - Choosing between Microsoft's Rust crate offerings
- [windows-version docs](https://docs.rs/windows-version/latest/windows_version/) - OsVersion::current() API for runtime detection

### Secondary (MEDIUM confidence)
- [Tauri issue #10422: skipTaskbar broken on Windows](https://github.com/tauri-apps/tauri/issues/10422) - Bug description, WS_EX_TOOLWINDOW workaround discussed, maintainer rejection of built-in fix
- [SetForegroundWindow bypass gist](https://gist.github.com/Aetopia/1581b40f00cc0cadc93a0e8ccb65dc8c) - Four bypass techniques: AllowSetForegroundWindow, AllocConsole, AttachThreadInput, SendInput(ALT)
- [window-vibrancy CHANGELOG](https://github.com/tauri-apps/window-vibrancy/blob/dev/CHANGELOG.md) - Version history, breaking changes between 0.5 and 0.7

### Tertiary (LOW confidence)
- [Raycast Windows keyboard-first architecture](https://windowsforum.com/threads/raycast-on-windows-a-keyboard-first-command-bar-for-faster-workflow.387394/) - Confirms launcher-style overlay pattern on Windows (hotkey invoke, keyboard input, dismiss-restores-focus)
- WS_EX_NOACTIVATE keyboard limitation - Multiple forum sources confirm it prevents keyboard input; validated against Microsoft's official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - window-vibrancy is Tauri-owned, windows-sys is Microsoft-maintained, all APIs are stable Win32
- Architecture: HIGH - The platform-branching pattern is well-established in the existing codebase (50+ cfg gates already present); the focus management pattern is documented by Microsoft
- Pitfalls: HIGH - Tauri skipTaskbar bug is documented with open issue; SetForegroundWindow restrictions are extensively documented by Microsoft; DPI handling is already solved in existing code

**Research date:** 2026-03-02
**Valid until:** 2026-04-02 (stable Win32 APIs, low churn)
