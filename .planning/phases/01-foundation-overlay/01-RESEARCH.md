# Phase 1: Foundation & Overlay - Research

**Researched:** 2026-02-21
**Domain:** Tauri v2 macOS overlay window, global hotkey, menu bar, NSPanel, vibrancy
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Overlay style & position:** Spotlight/Raycast style floating panel, positioned ~25% down from top of screen. Appears centered over the currently active window, not at a fixed screen position.
- **Visual style:** Frosted glass background with vibrancy/blur-behind effect, enough opacity to feel solid (translucent, not transparent), plus subtle drop shadow. Design reference: Cursor's Cmd+K terminal assist.
- **Animation:** Quick fade-in with slight scale-up on appear/disappear (Spotlight-like). No background dimming.
- **Input field:** Single-line input that grows/expands; Shift+Enter for newline, Enter to submit. Placeholder: "Describe a task or type a command...". Auto-focus immediately on appear.
- **Dismiss:** Click outside OR Escape key dismisses.
- **Menu bar:** K.png used as app icon, menu bar icon, and all branding. Menu items: "Settings...", "Change Hotkey...", "About", "Quit CMD+K".
- **Default hotkey:** Cmd+K (matches app name). Configuration: preset dropdown of common options (Cmd+K, Cmd+Shift+K, Ctrl+Space, etc.) plus record-a-custom-shortcut by pressing desired key combo.
- **Phase 1 submit behavior:** When user submits and API is not configured, show inline "API not configured" message with toggle/link to settings. Include empty results area below input, ready for Phase 4.

### Claude's Discretion

- Exact overlay width (guided by Cursor Cmd+K reference)
- Typography and spacing details
- Exact animation timing and easing curves
- Results area placeholder/empty state appearance
- Error state handling for edge cases

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| OVRL-01 | System-wide Cmd+K hotkey triggers the overlay from any application | tauri-plugin-global-shortcut 2.x; runtime re-registration for configurable hotkey; debounce required for double-fire bug |
| OVRL-02 | Overlay appears as a floating panel on top of the currently active window | tauri-nspanel (branch v2.1) for NSPanel; window-vibrancy for frosted glass; manual set_position with LogicalPosition for 25%-down placement; alwaysOnTop + NSWindowLevel for above-fullscreen |
| OVRL-03 | User can dismiss overlay with Escape key without affecting underlying application | Keyboard event listener in frontend + tauri command to hide window; click-outside via React pointer event; NSPanel show_and_make_key + hide pattern |
| OVRL-04 | User can configure the trigger hotkey to avoid conflicts | Runtime unregister + re-register via app.global_shortcut(); preset list + record-custom-shortcut UI; store preference in Tauri store or local config |
| OVRL-05 | App runs as background daemon with menu bar icon (no dock icon) | ActivationPolicy::Accessory; TrayIconBuilder with Menu; K.png as template icon; no dock entry |
</phase_requirements>

---

## Summary

Phase 1 delivers the structural skeleton of CMD+K: a macOS system-wide overlay window that appears on a configurable global hotkey, accepts keyboard input, and disappears on Escape or click-outside, while the app runs silently as a menu bar item with no Dock icon. No AI, no terminal integration -- just the shell that later phases fill.

The core technology stack is well-validated: Tauri v2 (current stable 2.10.x) with React + TypeScript + Vite for the frontend, `tauri-nspanel` (v2.1 branch) for correct NSPanel floating behavior, `tauri-plugin-global-shortcut` for system-wide hotkey, `window-vibrancy` for the frosted glass effect, and `TrayIconBuilder` for the menu bar presence. All components have official or well-maintained community support as of February 2026.

The two highest-risk items for Phase 1 are: (1) correct window level control so the overlay floats above fullscreen apps (Tauri's built-in `alwaysOnTop` is insufficient; NSPanel via `tauri-nspanel` solves this), and (2) transparent window rendering glitches on macOS Sonoma with Stage Manager (solved by setting `ActivationPolicy::Accessory` which also hides the Dock icon -- a required behavior anyway). The default hotkey `Cmd+K` conflicts with many apps; the architecture must support runtime re-registration from day one.

**Primary recommendation:** Use `tauri-nspanel` (not raw NSWindowLevel Cocoa code) for the overlay panel. It provides the correct macOS panel semantics with a maintained Tauri v2 API and is the established community solution for this exact pattern.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Tauri | 2.10.x | Desktop framework | Current stable Feb 2026. Native macOS integration, 2-2.5MB bundle, Rust security model. |
| tauri-nspanel | git v2.1 branch | NSPanel overlay behavior | Only maintained plugin that correctly handles macOS panel semantics for overlay apps. Replaces manual Cocoa bindings. |
| tauri-plugin-global-shortcut | 2.3.x | System-wide hotkey | Official Tauri plugin. Supports runtime register/unregister for configurable hotkeys. |
| window-vibrancy | 0.5.x | Frosted glass NSVisualEffectView | Official Tauri-maintained crate. apply_vibrancy() with NSVisualEffectMaterial options. |
| React | 18.3+ | Frontend UI | Component-based, TypeScript support, mature Tauri integration. |
| TypeScript | 5.7+ | Type safety | Essential for IPC type safety across Rust/frontend boundary. |
| Vite | 6.x | Build tool | Official Tauri recommendation, fast HMR. |
| Tailwind CSS | 4.x | Styling | CSS-first config in v4. Works with shadcn/ui (v4 compatible as of 2026). |
| shadcn/ui | Latest | UI components | v4 compatible. Input, Button primitives. data-slot attributes for styling. |
| Zustand | 5.x | Frontend state | Minimal (3KB), hooks-based, sufficient for overlay state. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-plugin-positioner | 2.3.x | Predefined window positions | Use for initial center-screen placement; custom set_position needed for 25%-down offset |
| tauri-plugin-store | 2.x | Persistent config storage | Saving hotkey preference, overlay width setting |
| Lucide React | Latest | Icon library | Menu bar dropdown icons, close button, settings icons |
| tw-animate-css | Latest | Animation utilities | Replaces tailwindcss-animate in Tailwind v4. Fade + scale animation for overlay appear/disappear |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tauri-nspanel | Raw cocoa crate + NSWindowLevel | tauri-nspanel is higher-level, maintained, avoids unsafe Cocoa boilerplate. Raw cocoa needed only if nspanel insufficient. |
| window-vibrancy | CSS backdrop-filter | CSS backdrop-filter does not use native NSVisualEffectView; looks noticeably different from Spotlight/Raycast. Native vibrancy is required to match design spec. |
| Tailwind v4 + shadcn | Custom CSS | shadcn/ui already Tailwind v4 compatible; avoid the migration complexity by starting on v4 |
| tauri-plugin-store | JSON file via fs plugin | tauri-plugin-store provides structured key-value persistence; simpler API for hotkey config |

**Installation:**
```bash
# Frontend deps
pnpm create tauri-app cmd-k --template react-ts
pnpm add zustand lucide-react tw-animate-css
pnpm dlx shadcn@latest init   # initializes shadcn/ui with Tailwind v4

# Tauri plugins (Cargo.toml)
cargo add tauri-plugin-global-shortcut
cargo add tauri-plugin-positioner
cargo add tauri-plugin-store
cargo add window-vibrancy

# tauri-nspanel (git dep, not on crates.io)
# In Cargo.toml:
# tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
```

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/
  src/
    lib.rs            # tauri builder, plugin init, setup
    commands/
      window.rs       # show_overlay, hide_overlay, set_position
      hotkey.rs       # register_hotkey, unregister_hotkey commands
      tray.rs         # tray menu event handling
    state.rs          # AppState struct (Mutex-wrapped)
  Cargo.toml
  tauri.conf.json
  capabilities/
    default.json      # IPC capability grants

src/
  main.tsx            # React entry
  App.tsx             # Root overlay layout
  components/
    Overlay.tsx       # Panel root: frosted glass, positioning, animation
    CommandInput.tsx  # Auto-focus input field, Shift+Enter handling
    ResultsArea.tsx   # Empty results area placeholder (Phase 4 hook)
    MenuBarIcon.tsx   # (handled in Rust, not frontend)
  store/
    index.ts          # Zustand store: overlay visibility, hotkey config
  hooks/
    useKeyboard.ts    # Escape key listener, submit handler
```

### Pattern 1: NSPanel Show/Hide via tauri-nspanel

**What:** Use tauri-nspanel to create a floating panel that accepts keyboard input without stealing focus from the underlying app when dismissed. Show with `show_and_make_key()`, hide with a custom hide command.

**When to use:** Every overlay show/hide triggered by global hotkey or Escape key.

**Example:**
```rust
// Source: https://github.com/ahkohd/tauri-nspanel (v2.1 branch)
use tauri_nspanel::ManagerExt;

#[tauri::command]
pub fn show_overlay(app: tauri::AppHandle) {
    let panel = app.get_webview_panel("main").unwrap();
    panel.show_and_make_key();
}

#[tauri::command]
pub fn hide_overlay(app: tauri::AppHandle) {
    let panel = app.get_webview_panel("main").unwrap();
    panel.order_out(None);
}
```

```rust
// In setup: convert main window to NSPanel
tauri::Builder::default()
    .plugin(tauri_nspanel::init())
    .setup(|app| {
        // ActivationPolicy must be set BEFORE panel creation
        #[cfg(target_os = "macos")]
        app.set_activation_policy(tauri::ActivationPolicy::Accessory);

        // Create panel from window
        let window = app.get_webview_window("main").unwrap();
        let panel = window.to_panel().unwrap();
        panel.set_level(tauri_nspanel::cocoa::appkit::NSMainMenuWindowLevel + 1);
        panel.set_can_become_key_window(true);
        Ok(())
    })
```

### Pattern 2: Configurable Global Hotkey with Runtime Re-registration

**What:** Register hotkey at startup; unregister old and register new when user changes hotkey in settings. Debounce handler to prevent double-fire (known Tauri bug #10025).

**When to use:** App startup and every time user saves a new hotkey in "Change Hotkey..." dialog.

**Example:**
```rust
// Source: https://v2.tauri.app/plugin/global-shortcut/
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use std::sync::Mutex;

#[tauri::command]
pub fn register_hotkey(app: tauri::AppHandle, shortcut_str: String) -> Result<(), String> {
    // Unregister all existing shortcuts first
    app.global_shortcut().unregister_all()
        .map_err(|e| e.to_string())?;

    let shortcut: Shortcut = shortcut_str.parse()
        .map_err(|e: _| format!("Invalid shortcut: {}", e))?;

    app.global_shortcut().on_shortcut(shortcut, move |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            // Debounce: handled in Rust with timestamp check
            show_overlay_internal(app);
        }
    }).map_err(|e| e.to_string())?;

    Ok(())
}
```

### Pattern 3: Window Positioning at 25% Down from Screen Top

**What:** The positioner plugin's predefined positions do not include "25% from top, centered". Use the Tauri window API directly: get the current monitor size, compute the target position, then call `set_position`.

**When to use:** Every time the overlay is shown (position over the current active window).

**Example:**
```rust
// Source: Tauri v2 window API + monitor API
use tauri::{LogicalPosition, Manager};

fn position_overlay(app: &tauri::AppHandle) {
    let window = app.get_webview_window("main").unwrap();

    // Get primary monitor dimensions
    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen_size = monitor.size();
        let scale = monitor.scale_factor();
        let screen_w = screen_size.width as f64 / scale;
        let screen_h = screen_size.height as f64 / scale;

        // Overlay width: 640px (matches Cursor Cmd+K reference)
        let overlay_w = 640.0_f64;
        let overlay_x = (screen_w - overlay_w) / 2.0;
        let overlay_y = screen_h * 0.25;  // 25% down from top

        let _ = window.set_position(LogicalPosition::new(overlay_x, overlay_y));
    }
}
```

**Note:** For multi-monitor setups, use `current_monitor()` instead of `primary_monitor()` to position relative to the active display.

### Pattern 4: Frosted Glass Vibrancy Effect

**What:** Apply NSVisualEffectView with `UnderWindowBackground` material for the Spotlight-like translucent-but-solid appearance described in the design spec. Combine with transparent window configuration.

**When to use:** Applied once during window setup. Requires `transparent: true` in tauri.conf.json.

**Example:**
```rust
// Source: https://docs.rs/window-vibrancy/latest/window_vibrancy/
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

fn apply_frosted_glass(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    apply_vibrancy(
        window,
        NSVisualEffectMaterial::HudWindow,  // Solid-feeling translucency
        None,   // NSVisualEffectState::FollowsWindowActiveState (default)
        Some(12.0),  // Corner radius (match CSS border-radius)
    ).expect("Failed to apply vibrancy");
}
```

**Material choice guidance:**
- `HudWindow` -- dark frosted panel, matches Spotlight/Raycast feel, recommended
- `UnderWindowBackground` -- lighter, more transparent, see-through look
- `Popover` -- lighter material, typically used for small popups
- `Sidebar` -- matches sidebar panels in macOS apps

### Pattern 5: Menu Bar with TrayIconBuilder (Tauri v2)

**What:** Create a system tray icon using the Tauri v2 TrayIconBuilder API (note: v1 `SystemTray` API is removed in v2).

**When to use:** App startup in the `setup` closure.

**Example:**
```rust
// Source: https://v2.tauri.app/learn/system-tray/
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    image::Image,
};

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let change_hotkey = MenuItem::with_id(app, "change_hotkey", "Change Hotkey...", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit CMD+K", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings, &change_hotkey, &about, &quit])?;

    // K.png loaded as template icon for correct dark/light mode adaptation
    let icon = Image::from_path("../K.png")?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .menu_on_left_click(false)  // Only show menu on right-click (macOS convention)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => { /* open settings window */ }
            "change_hotkey" => { /* show hotkey recorder */ }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
```

**Note on template icon:** On macOS, menu bar icons should be PNG files that will be rendered as template images (adapts to dark/light menu bar). The icon should be a simple monochrome K glyph at 16x16 and 32x32 (@2x).

### Pattern 6: Overlay Animation (CSS + Tailwind v4)

**What:** Spotlight-like fade-in + slight scale-up on appear, fade-out on dismiss. Implemented in CSS with `tw-animate-css` (Tailwind v4 replacement for tailwindcss-animate).

**When to use:** Wrap the overlay panel root element.

**Example:**
```css
/* In global CSS -- tw-animate-css provides these utilities */
@keyframes overlay-in {
  from { opacity: 0; transform: scale(0.96) translateY(-4px); }
  to   { opacity: 1; transform: scale(1) translateY(0); }
}

@keyframes overlay-out {
  from { opacity: 1; transform: scale(1); }
  to   { opacity: 0; transform: scale(0.96) translateY(-4px); }
}
```

```tsx
// Overlay.tsx -- conditional render controls animation
export function Overlay({ visible }: { visible: boolean }) {
  return (
    <div
      data-visible={visible}
      className="
        w-[640px] rounded-xl shadow-2xl
        data-[visible=true]:animate-[overlay-in_120ms_ease-out]
        data-[visible=false]:animate-[overlay-out_100ms_ease-in]
      "
    >
      {/* content */}
    </div>
  );
}
```

### Anti-Patterns to Avoid

- **Tauri setAlwaysOnTop without NSPanel:** `alwaysOnTop: true` in tauri.conf.json does NOT float above fullscreen apps on macOS (Issue #11488). Always use tauri-nspanel for the panel level.
- **SystemTray v1 API:** `tauri::SystemTray` is removed in Tauri v2. Use `TrayIconBuilder` from `tauri::tray`.
- **CSS backdrop-filter for vibrancy:** Does not use NSVisualEffectView, looks different from Spotlight. Use window-vibrancy crate.
- **Polling for hotkey changes:** Do not set up a timer to check hotkey settings. Use a Tauri command invoked by the UI when the user saves a new hotkey.
- **Window hide via frontend JS:** Do not call `appWindow.hide()` from the frontend when Escape is pressed. Route through a Tauri command to ensure NSPanel state is correctly managed.
- **Hardcoded Cmd+K with no fallback:** Cmd+K registration will fail silently in many apps. The architecture must handle registration failure from day one.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| NSPanel floating window | Custom unsafe Cocoa FFI bindings | tauri-nspanel | tauri-nspanel handles panel level, key window semantics, all macOS quirks. The unsafe code is already tested. |
| Frosted glass / NSVisualEffectView | NSWindow subclassing or CSS hacks | window-vibrancy apply_vibrancy() | Native NSVisualEffectView via well-tested Tauri-maintained crate. |
| Global hotkey registration | OS-level Carbon/CGEvent APIs | tauri-plugin-global-shortcut | Official Tauri plugin handles cross-platform, permissions, and the double-fire workaround. |
| Window position persistence | Custom JSON config file | tauri-plugin-store | Structured key-value store, atomic writes, type-safe Rust API. |
| Menu bar icon + menu | Manual NSStatusBar AppKit bindings | TrayIconBuilder (Tauri v2 built-in) | First-class Tauri v2 API, handles dark/light mode, template images automatically. |
| UI components (Input, Button) | Hand-written form elements | shadcn/ui (copy-paste) | Accessible, keyboard-native, Tailwind v4 compatible. Pattern matches Spotlight/Raycast. |

**Key insight:** Every "custom macOS binding" problem in this phase has a maintained community solution. The project should use all of them rather than writing unsafe Rust/Cocoa code.

---

## Common Pitfalls

### Pitfall 1: Transparent Window Rendering Glitches on Sonoma (Stage Manager)

**What goes wrong:** macOS Sonoma 14.0+ breaks transparent Tauri window rendering on focus change when Stage Manager is enabled. Shadow artifacts and border glitches persist.

**Why it happens:** Sonoma's Stage Manager changed window compositing APIs; Tauri v2 has not fully adapted (Issue #8255, open as of Feb 2026).

**How to avoid:** Set `ActivationPolicy::Accessory` in setup. This hides the Dock icon (desired behavior anyway) and removes the window from Stage Manager's management scope, which eliminates the rendering issue.

**Warning signs:** Broken shadows or blurry borders after Cmd+Tab. Test on macOS 14+ with Stage Manager enabled.

### Pitfall 2: Global Hotkey Registration Fails Silently

**What goes wrong:** Cmd+K is used by Safari, VS Code, Slack, and others. The plugin may fail to register without notifying the user. The handler never fires.

**Why it happens:** macOS gives focused-app shortcuts priority. When `tauri-plugin-global-shortcut` fails, it returns an `Err` but the app continues silently.

**How to avoid:** Always handle the `Result` from `register()`. If it fails, show a notification (or menu bar indicator) and prompt user to change hotkey. Build the entire hotkey pipeline as configurable from the start -- do not treat "default hotkey" as always-available.

**Warning signs:** Hotkey works when tested alone but not when other apps are open.

### Pitfall 3: Global Shortcut Fires Twice

**What goes wrong:** Known Tauri bug (#10025) causes the hotkey handler to fire twice per keypress on macOS.

**Why it happens:** Undocumented Tauri v2 behavior, present as of current version.

**How to avoid:** Add a debounce in the Rust handler using a timestamp stored in AppState:

```rust
use std::time::{Instant, Duration};

// In hotkey handler:
let mut last_trigger = state.last_hotkey_trigger.lock().unwrap();
let now = Instant::now();
if now.duration_since(*last_trigger) < Duration::from_millis(200) {
    return;  // Ignore duplicate
}
*last_trigger = now;
// Proceed to show overlay
```

**Warning signs:** Overlay flashes open then immediately closes when hotkey is pressed.

### Pitfall 4: NSPanel Focus Steals from Underlying App

**What goes wrong:** `focusable: false` in tauri.conf.json does not work on macOS (Issue #14102). The overlay panel steals focus from the app the user was using.

**Why it happens:** Tauri's window configuration does not properly translate to NSPanel focus semantics.

**How to avoid:** Use `tauri-nspanel`'s `show_and_make_key()` -- NSPanel is the correct macOS type for auxiliary panels that accept input without permanently stealing app focus. When the overlay is dismissed, the previous app regains focus automatically. Do NOT use `focusable: false` as a substitute.

**Warning signs:** After dismissing overlay, user's original app (terminal, browser) does not regain focus.

### Pitfall 5: Window Positioning Uses Wrong Coordinate Space

**What goes wrong:** On multi-monitor setups with mixed DPI, positioning the window at 25% of screen height using physical pixels places it at the wrong position on retina displays.

**Why it happens:** macOS uses logical (point-based) coordinates for window positioning, but Tauri's `monitor.size()` returns physical pixels. Failing to divide by `scale_factor()` results in the window appearing at half the intended position.

**How to avoid:** Always convert monitor size to logical units before computing the 25% offset:
```rust
let logical_height = monitor.size().height as f64 / monitor.scale_factor();
let y = logical_height * 0.25;
window.set_position(LogicalPosition::new(x, y)).unwrap();
```

**Warning signs:** Window appears at wrong position on Retina MacBook Pro, correct on external 1x display.

### Pitfall 6: Tailwind v4 CSS Configuration Breaking Change

**What goes wrong:** Tailwind v4 removes `tailwind.config.js`. Attempting to use v3 config patterns causes a build failure or silent style stripping.

**Why it happens:** Tailwind v4 is CSS-first: all configuration moves into the main CSS file via `@theme` directive. The `tailwindcss-animate` plugin is also removed in favor of `tw-animate-css`.

**How to avoid:** Use `pnpm dlx shadcn@latest init` which configures v4 correctly. Do not copy v3 `tailwind.config.js` patterns from documentation written before 2025.

**Warning signs:** Animations don't work, theme colors missing, build warnings about unrecognized config.

### Pitfall 7: Sandboxing Must Be Disabled

**What goes wrong:** If `com.apple.security.app-sandbox` entitlement is enabled, the Accessibility API (needed in Phase 3) is blocked and global shortcuts may behave unexpectedly.

**Why it happens:** Apple's sandbox blocks inter-application accessibility features, even with user permission. This is not recoverable -- it requires a full distribution strategy change if discovered late.

**How to avoid:** Ensure sandboxing is disabled from Phase 1. Use Developer ID distribution (notarize only). Add the apple-events entitlement:

```xml
<!-- src-tauri/entitlements.plist -->
<key>com.apple.security.automation.apple-events</key>
<true/>
```

**Warning signs:** Any attempt to enable App Store distribution will hit this wall in Phase 3+.

---

## Code Examples

Verified patterns from official sources:

### Tauri App Setup with NSPanel and Tray

```rust
// src-tauri/src/lib.rs
use tauri::{Manager, menu::{Menu, MenuItem}, tray::TrayIconBuilder};
use tauri_nspanel::ManagerExt;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_nspanel::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // Hide dock icon (must be FIRST on macOS)
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Convert window to NSPanel
            let window = app.get_webview_window("main").unwrap();
            apply_frosted_glass(&window);
            let panel = window.to_panel().unwrap();
            panel.set_can_become_key_window(true);

            // Register default hotkey
            register_hotkey_internal(app.handle(), "Super+K");

            // Setup tray
            setup_tray(app)?;

            // Hide window initially
            window.hide().unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            show_overlay,
            hide_overlay,
            register_hotkey,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn apply_frosted_glass(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    apply_vibrancy(window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))
        .expect("Failed to apply vibrancy");
}
```

### tauri.conf.json Window Configuration

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "CMD+K",
        "width": 640,
        "height": 400,
        "minWidth": 480,
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": false,
        "skipTaskbar": true,
        "visible": false,
        "center": true,
        "resizable": false,
        "focus": false
      }
    ]
  }
}
```

**Note:** `alwaysOnTop: false` here because NSPanel level control handles this instead of the Tauri built-in (which does not work above fullscreen apps).

### IPC Types (Rust + TypeScript)

```rust
// src-tauri/src/state.rs
use std::sync::Mutex;
use std::time::Instant;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct AppState {
    pub hotkey: Mutex<String>,
    pub last_hotkey_trigger: Mutex<Option<Instant>>,
}

#[derive(Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub shortcut: String,
}
```

```typescript
// src/store/index.ts
import { create } from 'zustand';

interface OverlayState {
  visible: boolean;
  hotkeyConfig: string;
  show: () => void;
  hide: () => void;
  setHotkey: (shortcut: string) => void;
}

export const useOverlayStore = create<OverlayState>((set) => ({
  visible: false,
  hotkeyConfig: 'Super+K',
  show: () => set({ visible: true }),
  hide: () => set({ visible: false }),
  setHotkey: (shortcut) => set({ hotkeyConfig: shortcut }),
}));
```

### Input Field with Auto-focus and Grow Behavior

```tsx
// src/components/CommandInput.tsx
import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface CommandInputProps {
  onSubmit: (value: string) => void;
}

export function CommandInput({ onSubmit }: CommandInputProps) {
  const ref = useRef<HTMLTextAreaElement>(null);

  // Auto-focus when overlay appears
  useEffect(() => {
    ref.current?.focus();
  }, []);

  // Listen for Escape key to dismiss
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        invoke('hide_overlay');
      }
    };
    window.addEventListener('keydown', handler);
    return () => window.removeEventListener('keydown', handler);
  }, []);

  return (
    <textarea
      ref={ref}
      rows={1}
      placeholder="Describe a task or type a command..."
      className="
        w-full resize-none bg-transparent outline-none
        text-base text-white placeholder:text-white/40
        overflow-hidden leading-relaxed
      "
      style={{ height: 'auto' }}
      onChange={(e) => {
        // Auto-grow: reset height to auto then set to scrollHeight
        e.target.style.height = 'auto';
        e.target.style.height = `${e.target.scrollHeight}px`;
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          onSubmit(e.currentTarget.value);
        }
        // Shift+Enter falls through to default (newline in textarea)
      }}
    />
  );
}
```

### Click-Outside Dismiss

```tsx
// src/App.tsx
import { useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

export function App() {
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (panelRef.current && !panelRef.current.contains(e.target as Node)) {
        invoke('hide_overlay');
      }
    };
    document.addEventListener('mousedown', handler);
    return () => document.removeEventListener('mousedown', handler);
  }, []);

  return (
    <div ref={panelRef} className="overlay-panel">
      {/* content */}
    </div>
  );
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| cocoa crate + NSWindowLevel (unsafe) | tauri-nspanel plugin (safe Rust API) | 2024-2025 | No more unsafe Cocoa boilerplate for panels |
| tauri v1 SystemTray API | TrayIconBuilder (v2 built-in) | Tauri v2 stable (Oct 2024) | v1 tray code does not compile on v2 |
| tailwindcss-animate plugin | tw-animate-css package | Tailwind v4 release (2025) | Must use new package; old plugin not compatible |
| tailwind.config.js | CSS-first @theme in main CSS file | Tailwind v4 release (2025) | No JS config file; all theme config in CSS |
| tauri::WindowBuilder (v1) | WebviewWindowBuilder (v2) | Tauri v2 stable (Oct 2024) | API renamed; build patterns changed |

**Deprecated/outdated:**
- `tauri::SystemTray` and `SystemTrayMenu`: Removed in Tauri v2. Use `TrayIconBuilder`.
- `cocoa` crate for NSWindowLevel: Still works, but `tauri-nspanel` is the maintained higher-level alternative.
- `tailwindcss-animate`: Replaced by `tw-animate-css` in Tailwind v4 projects.
- `tauri-plugin-global-shortcut` v1 API: v2 plugin has different registration pattern (`Builder::new().with_handler(...).build()` vs v1's simpler form).

---

## Open Questions

1. **tauri-nspanel v2.1 branch stability**
   - What we know: The plugin is maintained by `ahkohd`, has a v2.1 branch, and is the community-standard solution for NSPanel in Tauri v2.
   - What's unclear: Whether the v2.1 branch is production-ready or still experimental. No crates.io release exists -- only git dependency.
   - Recommendation: Pin the git dependency to a specific commit hash for reproducibility. Prototype the show/hide/focus cycle early in Phase 1 to validate.

2. **Cmd+K default hotkey registration on macOS**
   - What we know: Cmd+K conflicts with Safari, VS Code, Slack. Registration may fail silently.
   - What's unclear: Whether macOS allows a global `Cmd+K` override at all, or whether it is always blocked when any app claiming it is running.
   - Recommendation: Default to `Cmd+K` as the product name implies, but detect registration failure immediately and show a menu bar badge/indicator. Present the "Change Hotkey..." flow as the first-run resolution path.

3. **Overlay width for "Cursor Cmd+K reference"**
   - What we know: Claude's Discretion per CONTEXT.md. Cursor's Cmd+K terminal assist panel is approximately 640px wide.
   - What's unclear: Exact current Cursor Cmd+K dimensions (UI may have changed).
   - Recommendation: Start with 640px width, 400px initial height. These match common Spotlight/Raycast proportions and can be adjusted based on visual testing.

4. **K.png as menu bar template icon**
   - What we know: K.png is in the repo root as the branding asset. macOS menu bar icons should be template images (monochrome, transparent background).
   - What's unclear: Whether K.png is already a suitable monochrome template or needs a separate icon variant. Template icons must be grayscale-only.
   - Recommendation: Create `K-template.png` as a 16x16 and 32x32 monochrome version of K.png for the menu bar. Use K.png full-color for the app icon in the About dialog and title bar.

---

## Sources

### Primary (HIGH confidence)

- [tauri-nspanel GitHub](https://github.com/ahkohd/tauri-nspanel) - v2.1 branch API, PanelBuilder, show_and_make_key
- [Tauri v2 Global Shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/) - registration API, Builder pattern, permissions
- [Tauri v2 System Tray docs](https://v2.tauri.app/learn/system-tray/) - TrayIconBuilder, Menu, MenuItem, event handling
- [window-vibrancy crate docs.rs](https://docs.rs/window-vibrancy/latest/window_vibrancy/) - apply_vibrancy, NSVisualEffectMaterial variants
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) - transparent, decorations, alwaysOnTop config
- [shadcn/ui Tailwind v4 docs](https://ui.shadcn.com/docs/tailwind-v4) - v4 compatibility, data-slot attributes, tw-animate-css

### Secondary (MEDIUM confidence)

- [Tauri Issue #11488](https://github.com/tauri-apps/tauri/issues/11488) - visibleOnAllWorkspaces / alwaysOnTop above fullscreen broken (confirms NSPanel approach needed)
- [Tauri Issue #8255](https://github.com/tauri-apps/tauri/issues/8255) - Transparent window glitch Sonoma (confirms ActivationPolicy::Accessory workaround)
- [Tauri Issue #10025](https://github.com/tauri-apps/tauri/issues/10025) - Global shortcut fires twice (confirms debounce requirement)
- [Tauri Issue #14102](https://github.com/tauri-apps/tauri/issues/14102) - focusable: false broken on macOS (confirms NSPanel approach needed)
- [tauri-plugin-positioner v2 docs](https://v2.tauri.app/plugin/positioner/) - move_window, WindowExt trait

### Tertiary (LOW confidence, flag for validation)

- [tauri-plugin-spotlight](https://crates.io/crates/tauri-plugin-spotlight) - Alternative all-in-one spotlight plugin; not chosen because tauri-nspanel gives finer control, but worth monitoring as a simpler alternative if nspanel proves unstable
- [tauri-plugin-liquid-glass](https://crates.io/crates/tauri-plugin-liquid-glass) - macOS 26 (Tahoe) liquid glass effect; monitor for future macOS version support but not needed for v1

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All major libraries verified against official docs and community sources
- Architecture: HIGH - NSPanel pattern confirmed via tauri-nspanel, IPC patterns confirmed via Tauri docs
- Pitfalls: HIGH - All critical pitfalls sourced from verified Tauri GitHub issues and Apple documentation

**Research date:** 2026-02-21
**Valid until:** 2026-03-21 (30 days -- stable ecosystem, no breaking changes expected soon)
