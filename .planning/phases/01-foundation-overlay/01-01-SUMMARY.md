---
phase: 01-foundation-overlay
plan: 01
subsystem: ui
tags: [tauri, rust, nspanel, macos, react, typescript, vibrancy, global-hotkey, tray-icon, tailwind, shadcn]

requires: []

provides:
  - NSPanel-based floating overlay window with frosted glass HudWindow vibrancy
  - Global hotkey (Super+K) registration with 200ms debounce
  - System tray icon with K.png branding and 4-item menu
  - Tauri v2 app with no Dock icon (ActivationPolicy::Accessory)
  - IPC commands: show_overlay, hide_overlay, register_hotkey
  - React + TypeScript + Vite frontend with Tailwind v4 and shadcn/ui

affects:
  - 01-02 (frontend overlay UI will consume show_overlay/hide_overlay/register_hotkey commands)
  - 01-03 (hotkey config UI builds on register_hotkey command)
  - all subsequent phases that depend on the app running

tech-stack:
  added:
    - tauri v2.10.x (Rust desktop framework)
    - tauri-nspanel v2.1 (git, NSPanel overlay behavior)
    - tauri-plugin-global-shortcut v2.3 (system-wide hotkey)
    - tauri-plugin-positioner v2.3 (window placement helpers)
    - tauri-plugin-store v2.4 (persistent config)
    - window-vibrancy v0.6 (NSVisualEffectView frosted glass)
    - React 19 + TypeScript 5.8 + Vite 7
    - Tailwind CSS v4 (CSS-first, no tailwind.config.js)
    - shadcn/ui (neutral palette, Tailwind v4 compatible)
    - zustand v5 (frontend state)
    - tw-animate-css (Tailwind v4 replacement for tailwindcss-animate)
    - lucide-react (icon library)
  patterns:
    - NSPanel created via tauri_panel! macro (can_become_key_window=true, is_floating_panel=true)
    - ActivationPolicy::Accessory set FIRST in setup() to hide Dock and fix Stage Manager
    - Physical pixel coordinates divided by scale_factor() before LogicalPosition (Retina support)
    - Global hotkey with 200ms debounce using Mutex<Option<Instant>> in AppState
    - Tray icon loaded from K.png with resource_dir() fallback chain for dev/prod
    - Collection behavior: FullScreenAuxiliary + CanJoinAllSpaces for above-fullscreen overlay

key-files:
  created:
    - src-tauri/src/lib.rs (Tauri builder, OverlayPanel macro, NSPanel setup, vibrancy, tray, hotkey)
    - src-tauri/src/main.rs (entry point calling cmd_k_lib::run())
    - src-tauri/src/state.rs (AppState with hotkey, debounce, visibility Mutex fields)
    - src-tauri/src/commands/mod.rs (module re-exports)
    - src-tauri/src/commands/window.rs (show_overlay, hide_overlay, toggle_overlay, position_overlay)
    - src-tauri/src/commands/hotkey.rs (register_hotkey with unregister_all + debounce)
    - src-tauri/src/commands/tray.rs (setup_tray with K.png and 4 menu items)
    - src-tauri/tauri.conf.json (transparent, no decorations, hidden, 640x400)
    - src-tauri/capabilities/default.json (IPC permissions for all commands)
    - src-tauri/entitlements.plist (apple-events, no sandbox)
    - src/styles.css (Tailwind v4, tw-animate-css, shadcn theme, overlay keyframes)
    - src/App.tsx (minimal transparent root with click-outside dismiss)
    - src/main.tsx (React entry importing styles.css)
    - vite.config.ts (Tailwind v4 plugin, @/* path alias)
    - tsconfig.json (paths for @/* alias)
  modified: []

key-decisions:
  - "Used tauri_panel! macro with can_become_key_window=true instead of raw to_panel() - required by tauri-nspanel v2.1 API which removed set_can_become_key_window() setter"
  - "Set window level to PanelLevel::Status (25 = NSMainMenuWindowLevel+1) for above-fullscreen behavior"
  - "Added CollectionBehavior::FullScreenAuxiliary+CanJoinAllSpaces for overlay appearing on fullscreen apps"
  - "Used StyleMask::nonactivating_panel() so overlay accepts keyboard input without activating the app"
  - "Pinned time crate to 0.3.36 to satisfy rustc 1.85.0 minimum version requirement"
  - "K.png icon loading uses resource_dir() fallback chain (resource dir -> ../K.png -> ./K.png) for dev/prod compatibility"
  - "NSPanel hide() used (not order_out) - tauri-nspanel v2.1 Panel trait uses hide() method"

patterns-established:
  - "Rust command pattern: #[tauri::command] functions return Result<(), String> for IPC error propagation"
  - "AppState uses Mutex<T> for all shared state accessed from hotkey callback (different thread)"
  - "All window operations gated with #[cfg(target_os = 'macos')] where platform-specific"
  - "Tray event handler uses app.emit() to notify frontend of menu actions (settings, hotkey config, about)"

requirements-completed:
  - OVRL-01
  - OVRL-02
  - OVRL-05

duration: 11min
completed: 2026-02-21
---

# Phase 1 Plan 1: Foundation & Overlay Backend Summary

**Tauri v2 app scaffolded with NSPanel overlay, HudWindow vibrancy, Super+K global hotkey with 200ms debounce, K.png tray icon, and ActivationPolicy::Accessory for no-Dock menu bar daemon**

## Performance

- **Duration:** 11 min
- **Started:** 2026-02-21T08:57:32Z
- **Completed:** 2026-02-21T09:08:40Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments

- Tauri v2 project scaffolded at repo root with React + TypeScript + Vite; all Rust and frontend dependencies installed and compiling
- NSPanel-based floating overlay with frosted glass HudWindow vibrancy (12px radius), positioned at 25% down from top of current monitor using logical coordinates
- System-wide hotkey Super+K registered with 200ms debounce, toggles overlay show/hide, supports runtime re-registration
- Menu bar tray icon with K.png branding and Settings/Change Hotkey/About/Quit menu items emitting frontend events
- App runs with no Dock icon via ActivationPolicy::Accessory; overlay floats above fullscreen apps via Status window level + FullScreenAuxiliary collection behavior

## Task Commits

Each task was committed atomically:

1. **Task 1: Scaffold Tauri v2 project with React + TypeScript and install all dependencies** - `a4bd166` (feat)
2. **Task 2: Implement Rust backend -- NSPanel, vibrancy, global hotkey, tray icon, and window positioning** - `c2590e3` (feat)

**Plan metadata:** (docs commit after SUMMARY)

## Files Created/Modified

- `src-tauri/src/lib.rs` - Tauri builder with OverlayPanel macro, all 4 plugins, vibrancy, setup closure
- `src-tauri/src/state.rs` - AppState with hotkey (Mutex<String>), debounce (Mutex<Option<Instant>>), visibility (Mutex<bool>)
- `src-tauri/src/commands/window.rs` - show_overlay (positions + shows_and_make_key), hide_overlay, toggle_overlay, position_overlay
- `src-tauri/src/commands/hotkey.rs` - register_hotkey with unregister_all, ShortcutState::Pressed check, 200ms debounce
- `src-tauri/src/commands/tray.rs` - setup_tray with K.png loading fallback chain, 4 menu items, event emissions
- `src-tauri/tauri.conf.json` - transparent: true, decorations: false, visible: false, 640x400
- `src-tauri/capabilities/default.json` - IPC permissions for all commands and plugins
- `src-tauri/entitlements.plist` - apple-events entitlement, no sandbox
- `src/styles.css` - Tailwind v4, tw-animate-css, shadcn theme variables, overlay-in/out keyframes
- `vite.config.ts` - @tailwindcss/vite plugin, @/* path alias
- `tsconfig.json` - paths for @/* alias added
- `package.json` - zustand, lucide-react, tw-animate-css, tailwindcss, shadcn deps added

## Decisions Made

- Used `tauri_panel!` macro with `can_become_key_window: true` since tauri-nspanel v2.1 removed the `set_can_become_key_window()` setter in favor of the macro-based approach
- `PanelLevel::Status` (value 25, one above NSMainMenuWindowLevel) chosen for overlay level to float above all normal windows and fullscreen apps
- `CollectionBehavior::FullScreenAuxiliary + CanJoinAllSpaces` added so the overlay can appear on Mission Control spaces and over fullscreen app windows
- Pinned `time` crate to `0.3.36` since `time@0.3.47` requires rustc 1.88.0 but system has 1.85.0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tauri-nspanel v2.1 API differences from research documentation**
- **Found during:** Task 2 (Rust backend implementation)
- **Issue:** Research referenced `set_can_become_key_window(true)`, `order_out(None)`, and `tauri_nspanel::cocoa::appkit::NSMainMenuWindowLevel` which don't exist in the v2.1 branch. The v2.1 API uses the `tauri_panel!` macro for configuration, `panel.hide()` instead of `order_out`, and `PanelLevel` enum instead of raw cocoa constants.
- **Fix:** Used `tauri_panel!` macro to define `OverlayPanel` with `can_become_key_window: true`, replaced `order_out(None)` with `panel.hide()`, used `PanelLevel::Status.value()` for window level
- **Files modified:** src-tauri/src/lib.rs, src-tauri/src/commands/window.rs
- **Verification:** cargo check, cargo build, cargo clippy all pass with no errors
- **Committed in:** c2590e3 (Task 2 commit)

**2. [Rule 1 - Bug] Missing trait imports for Tauri v2 API**
- **Found during:** Task 2 (cargo check)
- **Issue:** Tauri v2 requires explicit trait imports: `use tauri::Emitter` for `.emit()`, `use tauri::Manager` for `.try_state()`, `use tauri_nspanel::WebviewWindowExt` for `.to_panel()`
- **Fix:** Added required trait imports to window.rs, hotkey.rs, tray.rs, and lib.rs
- **Files modified:** src-tauri/src/commands/window.rs, src-tauri/src/commands/hotkey.rs, src-tauri/src/commands/tray.rs
- **Verification:** cargo check passes
- **Committed in:** c2590e3 (Task 2 commit)

**3. [Rule 3 - Blocking] Pinned time crate version for rustc 1.85.0 compatibility**
- **Found during:** Task 1 (cargo check)
- **Issue:** time@0.3.47 requires rustc 1.88.0; system has rustc 1.85.0
- **Fix:** `cargo update time@0.3.47 --precise 0.3.36`
- **Files modified:** src-tauri/Cargo.lock
- **Verification:** cargo check passes after pinning
- **Committed in:** c2590e3 (Task 2 commit, Cargo.lock included)

---

**Total deviations:** 3 auto-fixed (2 Rule 1 bugs, 1 Rule 3 blocking)
**Impact on plan:** All auto-fixes were necessary for compilation. The tauri-nspanel v2.1 API differences were the primary deviation -- the macro-based panel configuration is actually more structured than the imperative approach in the research docs. No scope creep.

## Issues Encountered

- `pnpm create tauri-app` created a `cmd-k/` subdirectory instead of scaffolding at root; resolved by copying all files to repo root and removing the subdirectory
- shadcn init required the styles.css file to already exist with `@import "tailwindcss"` before it could validate the Tailwind v4 configuration
- `tauri-plugin-opener` was in the scaffolded Cargo.toml but not needed for this plan; removed in favor of the required plugins

## User Setup Required

None - no external service configuration required. The app runs locally with no API keys needed for Phase 1.

## Next Phase Readiness

- Rust backend is complete and all IPC commands are registered and compiling
- Frontend is minimal placeholder (App.tsx) ready for Phase 1 Plan 2 (overlay UI)
- All commands (show_overlay, hide_overlay, register_hotkey) are available for frontend consumption
- Tray events (open-settings, open-hotkey-config, open-about) are emitted and ready for frontend listeners

---
*Phase: 01-foundation-overlay*
*Completed: 2026-02-21*
