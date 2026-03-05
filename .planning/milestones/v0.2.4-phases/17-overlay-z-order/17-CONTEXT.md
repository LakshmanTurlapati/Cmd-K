# Phase 17: Overlay Z-Order - Context

**Gathered:** 2026-03-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Change the NSPanel window level so macOS system UI elements (permission dialogs, Notification Center, Spotlight, accessibility popups) can appear above the CMD+K overlay, while preserving the overlay's ability to float above all normal application windows and fullscreen apps.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion
- **Window level choice**: Find the right NSPanel level that sits above normal apps but below system UI. Currently `PanelLevel::Status` (level 25) which is too high. Claude should determine the correct level (e.g., `NSFloatingWindowLevel`, `NSModalPanelWindowLevel`, or a custom value) that satisfies both constraints.
- **Overlay reaction to system dialogs**: Whether the overlay stays visible behind system dialogs or auto-dismisses. User did not specify a preference — choose the approach that provides the best UX.
- **Priority tradeoffs**: If lowering the window level causes some third-party floating windows (like other always-on-top apps) to appear above the overlay, that's acceptable as long as the overlay remains above all standard application windows.
- **Fullscreen behavior preservation**: The overlay must continue to work above fullscreen apps. Current `full_screen_auxiliary()` + `can_join_all_spaces()` collection behavior should be preserved or adapted as needed.

</decisions>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The user's core ask: "system overlays for accessibility or other permission popups are blocked by the overlay... so they can be on top of overlay."

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `tauri_nspanel` crate: Provides `PanelLevel`, `CollectionBehavior`, `StyleMask`, and the `OverlayPanel` custom subclass
- `position_overlay()` in `commands/window.rs`: Handles monitor-aware logical coordinate positioning

### Established Patterns
- NSPanel conversion in `lib.rs:89-91`: `window.to_panel::<OverlayPanel>()`
- Level set in `lib.rs:95`: `panel.set_level(PanelLevel::Status.value())` — this is the line to change
- Collection behavior in `lib.rs:98-103`: `full_screen_auxiliary()` + `can_join_all_spaces()`
- Style mask in `lib.rs:107`: `nonactivating_panel()` — must be preserved for non-focus-stealing behavior
- Shadow disabled in `lib.rs:113`: CSS shadow used instead of native panel shadow

### Integration Points
- `lib.rs` setup block (lines 82-114): Where NSPanel is configured — the change is localized here
- `show_overlay()` / `hide_overlay()` in `commands/window.rs`: May need no changes if only the level changes
- Windows path (`lib.rs:121-155`): Uses `set_always_on_top(true)` — separate from macOS, unaffected

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 17-overlay-z-order*
*Context gathered: 2026-03-03*
