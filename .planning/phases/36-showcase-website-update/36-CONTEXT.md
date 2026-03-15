# Phase 36: Showcase Website Update - Context

**Gathered:** 2026-03-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Update the showcase website (showcase/) to reflect v0.3.9 with Linux support. Version numbers, platform-specific download buttons with OS auto-detection, privacy policy with version history, and content updates for Linux features. No new pages or sections — updating existing structure.

</domain>

<decisions>
## Implementation Decisions

### Platform download buttons
- Auto-detect visitor's OS via JavaScript, show primary download button for detected platform
- Secondary row shows all three platforms (macOS, Windows, Linux) so users can pick manually
- Linux download button opens a popup/modal with two choices: x86_64 and aarch64
- macOS links to universal DMG, Windows links to x64 NSIS installer (same as current)

### Version number management
- Single JS variable `const VERSION = '0.3.9'` at top of main.js (or inline script)
- All download URLs, demo version badge, and footer badge populated from this variable
- Demo UI mockup updated to show v0.3.9
- No more hardcoded version strings scattered across HTML

### Feature card updates
- Update "Multi-Terminal Support" and "System-Wide Overlay" cards to mention Linux alongside macOS/Windows
- Don't change card structure or add new cards — just update text to reflect three-platform support

### Privacy policy with version history
- Expandable "Previous Versions" section at bottom of privacy.html
- Current policy displays normally at the top (dated March 15, 2026)
- Collapsible section at bottom shows prior version (March 9, 2026) — full text preserved
- Content updates for v0.3.9:
  - Add Linux-specific privacy details: /proc filesystem access, xdotool keystroke simulation, AT-SPI2 accessibility API usage
  - Update supported platforms list to include Linux
  - Update AI model names to current Claude 4.5/4.6 family
  - Update "Last Updated" date

### Demo terminal selector
- Add GNOME Terminal and kitty to the terminal selector list
- Keep existing entries (Alacritty, WezTerm, Warp, VS Code, Zed, Sublime Text) — just append Linux ones

### Tech stack & hero section
- Add Linux/penguin platform badge alongside existing macOS/Windows badges
- No Linux-specific tech details in the stack section — just the badge
- Hero subtitle stays platform-agnostic ("A native desktop overlay...")
- Platform badges near download buttons in hero area are sufficient

### Claude's Discretion
- Exact popup/modal styling for the Linux arch chooser
- How to structure the version variable injection (inline script vs module)
- Collapsible section implementation for privacy policy history
- OS detection logic (navigator.platform, navigator.userAgent, or userAgentData)

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `showcase/js/main.js`: Navigation, theme switching, scroll effects — version variable can live here
- `showcase/css/main.css` + `showcase/css/home.css`: Existing button styles for download buttons — extend for three-button layout
- `showcase/js/demo.js`: Interactive demo with terminal/provider selectors — add entries to terminal list

### Established Patterns
- Plain HTML/CSS/JS — no framework, no build step
- Dark/light mode via system theme detection
- Intersection Observer for scroll animations
- Direct GitHub Release URLs for download links (`https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v{VERSION}/...`)

### Integration Points
- `showcase/index.html` — hero download buttons, feature cards, demo terminal selector, tech stack section, footer badge
- `showcase/privacy.html` — policy content, date, model names, expandable history section
- `showcase/js/main.js` — version variable, OS detection logic, arch popup
- `showcase/css/home.css` — download button layout for 3 platforms + auto-detect primary

</code_context>

<specifics>
## Specific Ideas

- Auto-detect OS with fallback to showing all three buttons equally (if detection fails)
- Linux arch popup should be lightweight — not a full modal, more like a small tooltip/popover
- Privacy policy previous versions should preserve the exact text, not summarized

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 36-showcase-website-update*
*Context gathered: 2026-03-15*
