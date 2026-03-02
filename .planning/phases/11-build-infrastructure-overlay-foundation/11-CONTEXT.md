# Phase 11: Build Infrastructure and Overlay Foundation - Context

**Gathered:** 2026-03-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Windows build compiles cross-platform without breaking macOS, and the overlay window appears with native frosted glass vibrancy on Ctrl+Shift+K with correct focus management. Terminal context detection, paste execution, and output reading are separate phases (12-14).

</domain>

<decisions>
## Implementation Decisions

### Overlay Appearance
- Match macOS frosted glass look as closely as possible -- pixel-match the blur radius and tint opacity
- Use Acrylic blur on Windows (closest to macOS NSVisualEffectView behind-window blur)
- Match macOS edge treatment: rounded corners and subtle shadow replicating the macOS floating panel
- Minimum Windows version: Win10 1903+ (Acrylic support required) -- no fallback for older builds
- Full DPI awareness required: overlay renders crisply at any scaling factor (100%, 150%, 200%)

### Sizing and Positioning
- Match macOS dimensions: 320px fixed width, dynamic height based on content
- Horizontal position: centered on current monitor (not primary monitor)
- Vertical position: 25% down from top of current monitor (same as macOS)
- Multi-monitor: use current/active monitor (same as macOS `current_monitor()` approach) -- Claude's discretion on implementation details
- Height resizes dynamically via frontend (same ResizeObserver pattern as macOS)

### Activation and Dismiss
- Hotkey re-press: toggle behavior (same as macOS) -- pressing Ctrl+Shift+K while visible dismisses overlay
- Click-outside dismiss: yes, overlay dismisses when user clicks outside (same as macOS blur/click handling)
- Guard dismissal during paste operations and AI streaming (same guards as macOS)
- Show animation: scale 0.96->1, opacity 0->1, 120ms ease-out (same CSS keyframes as macOS)
- Hide animation: scale 1->0.96, opacity 1->0, 100ms ease-in (same CSS keyframes as macOS)
- 200ms debounce on hotkey to prevent double-fire (same as macOS)
- Hotkey configurable from day one (same as macOS) -- default Ctrl+Shift+K, changeable via settings

### Focus Management
- Use AllowSetForegroundWindow / AttachThreadInput workaround to ensure reliable focus restoration (Windows-specific)
- Overlay captures HWND of ANY foreground window, not terminals only (same as macOS which captures any frontmost app PID)
- Overlay appears from ANY application, not just terminals (system-wide launcher, same as macOS)
- If non-terminal: still show overlay, capture app context instead of terminal context
- Window closure during overlay: context reads fail silently, overlay remains functional (same graceful degradation as macOS)

### Build Infrastructure
- Cargo.toml platform-gates macOS-only deps (cocoa, objc, etc.) behind `[target.'cfg(target_os = "macos")']`
- Windows-only deps (windows-sys, etc.) behind `[target.'cfg(target_os = "windows")']`
- Project must compile on both platforms without `cfg` breakage

### Window Properties
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

</decisions>

<specifics>
## Specific Ideas

- "Same as Mac" is the guiding principle -- every visual and behavioral detail should match the macOS overlay experience as closely as Windows allows
- macOS uses NSPanel with `is_floating_panel`, `can_become_key_window`, and nonactivating style -- Windows equivalent should achieve the same "floats above everything, accepts input, doesn't permanently steal focus" behavior
- macOS overlay CSS animations are shared via the frontend (styles.css, Overlay.tsx) -- these should work on Windows without changes since Tauri WebView renders them
- macOS hotkey uses tauri_plugin_global_shortcut -- same plugin should work on Windows

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 11-build-infrastructure-overlay-foundation*
*Context gathered: 2026-03-02*
