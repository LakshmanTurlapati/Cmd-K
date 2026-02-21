# Phase 1: Foundation & Overlay - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

System-wide floating overlay that appears on global hotkey (Cmd+K), accepts text input, and dismisses with Escape or click-outside. Runs silently in background with menu bar icon. No AI generation, no terminal integration -- this phase delivers the shell that later phases fill.

</domain>

<decisions>
## Implementation Decisions

### Overlay style & position
- Spotlight/Raycast style floating panel, but positioned ~25% down from top of screen (not at the very top)
- Appears centered over the currently active window, not at a fixed screen position
- Frosted glass background with vibrancy/blur-behind effect, but with enough opacity to feel solid (translucent, not transparent) -- plus subtle drop shadow for depth
- Visual reference: Cursor's Cmd+K terminal assist is the direct competitor and primary design reference
- Quick fade-in with slight scale-up animation on appear/disappear (Spotlight-like)
- No background dimming -- overlay floats on top with shadow separation only

### Input field & keyboard flow
- Placeholder text: "Describe a task or type a command..." (signals both natural language and direct command input)
- Single-line input that grows/expands if user types a lot; Shift+Enter for newline, Enter to submit
- Auto-focus immediately when overlay appears -- start typing right away
- Click outside the overlay dismisses it (like Spotlight), in addition to Escape key

### Menu bar & hotkey config
- Use K.png from repo root as app icon, menu bar icon, and all branding
- Menu bar dropdown items: "Settings...", "Change Hotkey...", "About", "Quit CMD+K"
- Default hotkey: Cmd+K (matches app name)
- Hotkey configuration: preset dropdown of common options (Cmd+K, Cmd+Shift+K, Ctrl+Space, etc.) plus ability to record a custom shortcut by pressing desired key combo

### Phase 1 submit behavior
- When user submits text and API is not configured: show inline "API not configured" message with quick toggle/link to settings page
- This is a fallback -- the onboarding flow (Phase 2) will handle first-run setup, but this catches users who skip or haven't completed it
- Include empty results area below the input field, ready for AI command output when Phase 4 connects

### Claude's Discretion
- Exact overlay width (guided by Cursor Cmd+K reference)
- Typography and spacing details
- Exact animation timing and easing curves
- Results area placeholder/empty state appearance
- Error state handling for edge cases

</decisions>

<specifics>
## Specific Ideas

- "I want it very similar to how Cmd+K terminal assist on Cursor works -- it's a direct competitor"
- Overlay should feel like Spotlight/Raycast but positioned lower (25% down from top)
- K.png is the branding asset for everything -- app icon, menu bar, logo
- Background should be a mix of frosted glass vibrancy and solid panel with shadow -- translucent, not transparent

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation-overlay*
*Context gathered: 2026-02-21*
