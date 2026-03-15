# Phase 36: Showcase Website Update - Research

**Researched:** 2026-03-15
**Domain:** Static website update (HTML/CSS/JS, no framework)
**Confidence:** HIGH

## Summary

This phase updates the existing showcase website (`showcase/`) to reflect v0.3.9 with Linux support. The site is plain HTML/CSS/JS with no build step or framework -- all changes are direct file edits. The current site has 5 hardcoded instances of `v0.2.8` scattered across `index.html` (hero download URLs, demo settings version, footer badge) and `privacy.html` (footer badge). Download buttons currently serve only macOS DMG and Windows NSIS installer.

The primary work involves: (1) centralizing the version string into a single JS variable and dynamically populating all download URLs and version badges, (2) adding a Linux download button with an architecture chooser popup, (3) adding OS auto-detection to highlight the visitor's platform, (4) updating feature card text to mention Linux, (5) adding GNOME Terminal to the terminal carousel, (6) updating privacy.html with Linux-specific data handling details and collapsible version history, and (7) adding a Linux platform badge in the hero section.

**Primary recommendation:** Use a single `const VERSION = '0.3.9'` at the top of `main.js`, populate all download URLs and version badges via `querySelectorAll` with data attributes, and use `navigator.userAgent` for OS detection with graceful fallback.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Auto-detect visitor's OS via JavaScript, show primary download button for detected platform
- Secondary row shows all three platforms (macOS, Windows, Linux) so users can pick manually
- Linux download button opens a popup/modal with two choices: x86_64 and aarch64
- macOS links to universal DMG, Windows links to x64 NSIS installer (same as current)
- Single JS variable `const VERSION = '0.3.9'` at top of main.js (or inline script)
- All download URLs, demo version badge, and footer badge populated from this variable
- Demo UI mockup updated to show v0.3.9
- No more hardcoded version strings scattered across HTML
- Update "Multi-Terminal Support" and "System-Wide Overlay" cards to mention Linux alongside macOS/Windows
- Don't change card structure or add new cards -- just update text to reflect three-platform support
- Expandable "Previous Versions" section at bottom of privacy.html
- Current policy displays normally at the top (dated March 15, 2026)
- Collapsible section at bottom shows prior version (March 9, 2026) -- full text preserved
- Content updates for v0.3.9: Linux-specific privacy details (/proc, xdotool, AT-SPI2), supported platforms, AI model names to Claude 4.5/4.6, "Last Updated" date
- Add GNOME Terminal and kitty to the terminal selector list (kitty already present, just GNOME Terminal needed)
- Keep existing terminal entries, just append Linux ones
- Add Linux/penguin platform badge alongside existing macOS/Windows badges
- No Linux-specific tech details in the stack section -- just the badge
- Hero subtitle stays platform-agnostic
- Platform badges near download buttons in hero area are sufficient

### Claude's Discretion
- Exact popup/modal styling for the Linux arch chooser
- How to structure the version variable injection (inline script vs module)
- Collapsible section implementation for privacy policy history
- OS detection logic (navigator.platform, navigator.userAgent, or userAgentData)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

## Existing Codebase Analysis

### File Structure
```
showcase/
  index.html          (504 lines) -- hero, features, demo, tech stack, CTA, footer
  privacy.html        (228 lines) -- privacy policy with one existing version history entry
  css/
    main.css          (480 lines) -- global styles, .btn, .btn-primary, .btn-secondary
    home.css          (900 lines) -- hero, features, demo, tech stack, CTA
    privacy.css       (185 lines) -- privacy page prose, .policy-version collapsible styles
  js/
    main.js           (172 lines) -- theme, nav, scroll reveal, carousel, active nav
    demo.js           (852 lines) -- hero overlay animation + macOS desktop demo animation
  assets/             -- images (K.png, K-white.png)
```

### Hardcoded Version Locations (v0.2.8)
All 5 instances that must be updated:

| File | Line | Context |
|------|------|---------|
| `index.html` | 66 | Hero macOS download URL: `releases/download/v0.2.8/CMD+K-0.2.8-universal.dmg` |
| `index.html` | 70 | Hero Windows download URL: `releases/download/v0.2.8/CMD+K-0.2.8-windows-x64.exe` |
| `index.html` | 346 | Demo settings version badge: `<span class="demo-settings-version">v0.2.8</span>` |
| `index.html` | 457 | CTA download URL (macOS only): `releases/download/v0.2.8/CMD+K-0.2.8-universal.dmg` |
| `index.html` | 496 | Footer version badge: `<span>v0.2.8</span>` |
| `privacy.html` | 222 | Footer version badge: `<span>v0.2.8</span>` |

### Current Download Button Structure (Lines 65-78)
```html
<div class="hero-buttons reveal reveal-delay-3">
  <a href="...CMD+K-0.2.8-universal.dmg" class="btn btn-primary">
    <svg class="btn-icon" ...><!-- Apple icon --></svg>
    Download for macOS
  </a>
  <a href="...CMD+K-0.2.8-windows-x64.exe" class="btn btn-primary">
    <svg class="btn-icon" ...><!-- Windows icon --></svg>
    Download for Windows
  </a>
  <a href="https://github.com/LakshmanTurlapati/Cmd-K" class="btn btn-secondary">
    <svg class="btn-icon" ...><!-- GitHub icon --></svg>
    View Source
  </a>
</div>
```

### Download URL Patterns (from release.yml)
```
macOS:   https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v{VERSION}/CMD+K-{VERSION}-universal.dmg
Windows: https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v{VERSION}/CMD+K-{VERSION}-windows-x64.exe
Linux:   https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v{VERSION}/CMD+K-{VERSION}-linux-x86_64.AppImage
Linux:   https://github.com/LakshmanTurlapati/Cmd-K/releases/download/v{VERSION}/CMD+K-{VERSION}-linux-aarch64.AppImage
```

### Terminal Carousel (Lines 400-431)
Currently 15 terminals/IDEs in order: Terminal.app, iTerm2, Alacritty, kitty, WezTerm, Hyper, Warp, Ghostty, Rio, VS Code, Cursor, Zed, Windsurf, IntelliJ IDEA, Sublime Text. The list is duplicated (aria-hidden="true") for seamless CSS carousel loop. **kitty is already present** -- only GNOME Terminal needs to be added.

### Privacy Policy Version History (Lines 175-186)
Already uses `<details>` / `<summary>` pattern with `.policy-version` CSS class. One existing entry (Feb 27, 2026). The CSS for collapsible sections is already complete in `privacy.css` (lines 124-168) -- includes arrow rotation, border styling, body padding.

### Demo Settings Model List (Lines 309-313)
Currently shows xAI models (grok-3, grok-3-mini, grok-4, grok-4-fast, grok-4-fast-non-reasoning). No Claude model names shown in the demo -- updating model names to Claude 4.5/4.6 is a privacy.html task, not a demo task.

### CTA Section Download (Lines 457-460)
Single download button pointing to macOS DMG. Should be updated to use auto-detect or point to primary platform.

## Architecture Patterns

### Version Variable Injection
**Recommendation:** Add `const VERSION = '0.3.9'` at the top of `main.js` (inside the IIFE), then use it in a `DOMContentLoaded` handler to populate all version-dependent elements.

**Pattern:**
```javascript
const VERSION = '0.3.9';
const DOWNLOAD_BASE = 'https://github.com/LakshmanTurlapati/Cmd-K/releases/download';

// In DOMContentLoaded:
const URLS = {
  macos: `${DOWNLOAD_BASE}/v${VERSION}/CMD+K-${VERSION}-universal.dmg`,
  windows: `${DOWNLOAD_BASE}/v${VERSION}/CMD+K-${VERSION}-windows-x64.exe`,
  linux_x86: `${DOWNLOAD_BASE}/v${VERSION}/CMD+K-${VERSION}-linux-x86_64.AppImage`,
  linux_arm: `${DOWNLOAD_BASE}/v${VERSION}/CMD+K-${VERSION}-linux-aarch64.AppImage`
};

// Set all download links
document.querySelectorAll('[data-download]').forEach(function(el) {
  var platform = el.getAttribute('data-download');
  if (URLS[platform]) el.href = URLS[platform];
});

// Set all version badges
document.querySelectorAll('[data-version]').forEach(function(el) {
  el.textContent = 'v' + VERSION;
});
```

**Why `main.js` not inline:** main.js is loaded on both `index.html` and `privacy.html`, so the version variable and URL population logic works across all pages. The footer version badge on privacy.html gets updated automatically.

### OS Auto-Detection
**Recommendation:** Use `navigator.userAgent` (broadest compatibility). `navigator.platform` is deprecated. `navigator.userAgentData` is Chromium-only.

**Pattern:**
```javascript
function detectOS() {
  var ua = navigator.userAgent;
  if (/Linux/.test(ua) && !/Android/.test(ua)) return 'linux';
  if (/Mac/.test(ua)) return 'macos';
  if (/Win/.test(ua)) return 'windows';
  return null; // unknown -- show all equally
}
```

**Confidence:** HIGH -- `navigator.userAgent` is the industry standard for OS detection on static websites. It works across all browsers. The `/Linux/` test naturally matches desktop Linux (excluding Android which contains "Linux" in UA).

### Linux Arch Chooser Popup
**Recommendation:** Lightweight CSS popover anchored to the Linux download button. Not a full modal -- just a small dropdown/tooltip-like element. Use JS to show/hide on click with click-outside-to-dismiss.

**Pattern:**
```html
<div class="arch-popup" id="linux-arch-popup" style="display:none">
  <a href="#" data-download="linux_x86" class="arch-option">
    x86_64 <span class="arch-desc">Intel/AMD</span>
  </a>
  <a href="#" data-download="linux_arm" class="arch-option">
    aarch64 <span class="arch-desc">ARM</span>
  </a>
</div>
```

**CSS:** Position absolute relative to Linux button, small border-radius card with `var(--bg-card)` background, subtle border, two clickable rows. Matches existing site design tokens.

### Download Button Layout
**Recommendation:** Two-tier layout:
1. **Primary row:** Single auto-detected platform button (large, `btn-primary`) + "View Source" button
2. **Secondary row:** Three small platform buttons (all platforms, `btn-secondary` style or text links)

**Fallback:** If OS detection returns null, show all three as equal `btn-primary` buttons (same as current two-button layout but with Linux added).

### Platform Badge in Hero
**Recommendation:** Add a small badge row near the download buttons showing macOS, Windows, Linux icons. Similar to how many landing pages show "Available for" badges. Use small SVG icons (Apple, Windows, Tux/penguin).

### Privacy Policy Collapsible History
**Already implemented** -- the `<details>` / `<summary>` / `.policy-version` pattern is already in use for the Feb 27 entry. For the March 9 version, add another `<details class="policy-version">` block with the full current policy text (before v0.3.9 updates). The CSS is complete and handles the arrow rotation, open state, and body styling.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| OS detection | Custom UA parsing with many edge cases | Simple regex on `navigator.userAgent` | 3 patterns cover 99%+ of desktop visitors |
| Collapsible sections | Custom JS accordion | Native `<details>` / `<summary>` HTML | Already used in privacy.html, zero JS needed, accessible by default |
| Click-outside-dismiss | Complex event delegation | Simple document click listener | One event listener covers the arch popup |

## Common Pitfalls

### Pitfall 1: Carousel Duplication Mismatch
**What goes wrong:** Adding GNOME Terminal to the first set of carousel items but forgetting the duplicated `aria-hidden="true"` set, causing the seamless loop to break.
**How to avoid:** The carousel has two identical sets of terminal tags. When adding GNOME Terminal, add it to BOTH sets. The second set has `aria-hidden="true"` on each `<span>`.

### Pitfall 2: Privacy Policy Full Text Preservation
**What goes wrong:** Summarizing the March 9 version instead of preserving the exact text.
**How to avoid:** The CONTEXT.md explicitly says "preserve the exact text, not summarized." Copy the current full policy text into the collapsible section BEFORE making any v0.3.9 edits.

### Pitfall 3: Version in Privacy.html Footer
**What goes wrong:** Updating index.html footer version but forgetting privacy.html has its own footer with `v0.2.8`.
**How to avoid:** Both pages load `main.js`. The data-attribute approach populates all `[data-version]` elements across both pages automatically.

### Pitfall 4: CTA Section Download Link
**What goes wrong:** Updating hero download buttons but forgetting the CTA section at the bottom also has a download link (line 457) pointing to the old macOS DMG URL.
**How to avoid:** Use `data-download="macos"` attribute on this link too, so the JS version injection handles it.

### Pitfall 5: Linux UA Contains "Linux" for Android
**What goes wrong:** Detecting Linux when user is on Android (Android UA includes "Linux").
**How to avoid:** Check for Android first: `if (/Linux/.test(ua) && !/Android/.test(ua)) return 'linux'`.

### Pitfall 6: kitty Already in Carousel
**What goes wrong:** Adding kitty as a "new" entry when it is already entry #4 in the carousel.
**How to avoid:** CONTEXT.md says "Add GNOME Terminal and kitty" but kitty is already present (verified at line 404). Only GNOME Terminal needs to be added.

## Code Examples

### Version Variable and Download URL Population (main.js addition)
```javascript
// At top of IIFE in main.js:
var VERSION = '0.3.9';
var DOWNLOAD_BASE = 'https://github.com/LakshmanTurlapati/Cmd-K/releases/download';
var URLS = {
  macos: DOWNLOAD_BASE + '/v' + VERSION + '/CMD+K-' + VERSION + '-universal.dmg',
  windows: DOWNLOAD_BASE + '/v' + VERSION + '/CMD+K-' + VERSION + '-windows-x64.exe',
  linux_x86: DOWNLOAD_BASE + '/v' + VERSION + '/CMD+K-' + VERSION + '-linux-x86_64.AppImage',
  linux_arm: DOWNLOAD_BASE + '/v' + VERSION + '/CMD+K-' + VERSION + '-linux-aarch64.AppImage'
};

// In DOMContentLoaded handler:
document.querySelectorAll('[data-download]').forEach(function(el) {
  var key = el.getAttribute('data-download');
  if (URLS[key]) el.href = URLS[key];
});
document.querySelectorAll('[data-version]').forEach(function(el) {
  el.textContent = 'v' + VERSION;
});
```

### OS Detection and Primary Button Highlight
```javascript
function detectOS() {
  var ua = navigator.userAgent;
  if (/Linux/.test(ua) && !/Android/.test(ua)) return 'linux';
  if (/Mac/.test(ua)) return 'macos';
  if (/Win/.test(ua)) return 'windows';
  return null;
}

// Highlight the detected platform's download button
var os = detectOS();
if (os) {
  var primaryBtn = document.querySelector('[data-platform="' + os + '"]');
  if (primaryBtn) primaryBtn.classList.add('platform-detected');
}
```

### Linux Arch Popup (HTML)
```html
<button class="btn btn-primary" id="linux-download-btn" data-platform="linux">
  <svg class="btn-icon" ...><!-- Tux/penguin icon --></svg>
  Download for Linux
</button>
<div class="arch-popup" id="linux-arch-popup">
  <a data-download="linux_x86" class="arch-option">
    <strong>x86_64</strong>
    <span>Intel / AMD (most common)</span>
  </a>
  <a data-download="linux_arm" class="arch-option">
    <strong>aarch64</strong>
    <span>ARM (Raspberry Pi, etc.)</span>
  </a>
</div>
```

### Linux Arch Popup (CSS)
```css
.arch-popup {
  display: none;
  position: absolute;
  top: calc(100% + 8px);
  left: 50%;
  transform: translateX(-50%);
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 8px;
  min-width: 220px;
  box-shadow: var(--shadow-lg);
  z-index: 10;
}

.arch-popup.open { display: block; }

.arch-option {
  display: flex;
  flex-direction: column;
  padding: 10px 14px;
  border-radius: var(--radius-md);
  color: var(--text-primary);
  text-decoration: none;
  transition: background var(--duration-fast);
}

.arch-option:hover {
  background: var(--bg-code);
}

.arch-option span {
  font-size: 0.8rem;
  color: var(--text-muted);
}
```

### GNOME Terminal Carousel Entry
```html
<span class="terminal-tag"><svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M2 4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v16a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V4zm4.5 1.5l4.5 4.5-4.5 4.5-1-1L9 10 5.5 6.5l1-1zM11 15h6v1.5h-6V15z"/></svg> GNOME Terminal</span>
```
Note: Use a generic terminal icon SVG -- GNOME Terminal does not have a widely available branded SVG path. A simple terminal prompt icon works well.

### Privacy Policy Version History Entry (March 9 version)
```html
<!-- Policy History -->
<h2>Policy History</h2>
<details class="policy-version">
  <summary>
    <span class="policy-version-date">March 9, 2026</span>
    Multi-provider support &mdash; macOS and Windows only
  </summary>
  <div class="policy-version-body">
    <!-- Full text of the March 9 policy preserved verbatim -->
  </div>
</details>
<details class="policy-version">
  <summary>
    <span class="policy-version-date">February 27, 2026</span>
    Initial version &mdash; xAI-only provider support
  </summary>
  <div class="policy-version-body">
    <!-- Existing Feb 27 entry content -->
  </div>
</details>
```

### Linux Platform Badge (SVG Tux icon)
```html
<span class="platform-badge" title="Linux">
  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.323 1.396-1.743 2.817-1.659 1.717-3.203 4.093-3.203 5.795 0 2.153 1.222 3.705 2.996 4.627-.097.205-.158.413-.158.652 0 .921.738 1.79 2.243 1.79 1.262 0 2.088-.679 2.506-1.309.42.63 1.244 1.309 2.506 1.309 1.505 0 2.243-.869 2.243-1.79 0-.239-.061-.447-.158-.652C17.78 18.774 19 17.222 19 15.069c0-1.702-1.544-4.078-3.203-5.795-1.42-1.421-1.667-1.725-1.743-2.817-.065-1.491 1.056-5.965-3.17-6.298-.165-.013-.325-.021-.48-.021z"/></svg>
</span>
```

## Privacy Policy Content Updates (v0.3.9)

### Sections Requiring Updates

1. **TLDR** -- Add: "Data goes directly from your device to your selected provider's API, nowhere else." (already correct, no change needed)

2. **Terminal Mode data sent** -- Add bullet:
   - "Recent terminal text (visible buffer content, up to ~12% of model context window)"

3. **Permissions section** -- Currently macOS-only. Update to:
   - macOS: Accessibility API in System Settings
   - Windows: No special permissions needed (standard Win32 APIs)
   - Linux: `/proc` filesystem for process detection (standard, no special permissions), `xdotool` for keystroke simulation (X11), AT-SPI2 accessibility API for reading terminal text from VTE-based terminals

4. **Where Your Data Is Stored** -- Update credential store line:
   - macOS Keychain, Windows Credential Manager, **Linux: system keyring via libsecret**

5. **AI model names** -- Update from current names to include Claude 4.5 Haiku / Claude 4.6 Opus (Anthropic) references where mentioned. Note: model names are NOT listed in privacy.html currently -- the CONTEXT.md mention of "AI model names to current Claude 4.5/4.6 family" likely refers to keeping the provider list current rather than listing specific model names.

6. **Last Updated date** -- Change from "March 9, 2026" to "March 15, 2026"

## Open Questions

1. **Linux Tux SVG icon**
   - What we know: Need a penguin/Tux icon for the platform badge and download button
   - What's unclear: Whether to use a detailed Tux SVG or a simplified penguin outline
   - Recommendation: Use a simplified Tux SVG path that matches the visual weight of the Apple and Windows icons already used (14x14 viewBox, single `<path>`)

2. **GNOME Terminal SVG icon**
   - What we know: GNOME Terminal does not have a simple branded SVG like VS Code or kitty
   - What's unclear: Whether to use the GNOME foot icon or a generic terminal icon
   - Recommendation: Use a simple terminal-prompt style SVG similar to Terminal.app/Rio icons already in the carousel

3. **CTA section download button behavior**
   - What we know: Currently a single macOS-only download link
   - What's unclear: Whether to make it OS-aware or keep it simple
   - Recommendation: Make it OS-aware using the same `data-download` pattern, or change to "Download" with the detected platform

## Sources

### Primary (HIGH confidence)
- Direct code inspection of `showcase/index.html` (504 lines)
- Direct code inspection of `showcase/privacy.html` (228 lines)
- Direct code inspection of `showcase/js/main.js` (172 lines)
- Direct code inspection of `showcase/js/demo.js` (852 lines)
- Direct code inspection of `showcase/css/privacy.css` (185 lines)
- Direct code inspection of `showcase/css/home.css`
- Direct code inspection of `.github/workflows/release.yml` (AppImage naming pattern)

### Secondary (MEDIUM confidence)
- `navigator.userAgent` OS detection patterns -- well-established web standard

## Metadata

**Confidence breakdown:**
- Codebase analysis: HIGH -- direct code inspection of all files
- Version injection pattern: HIGH -- standard JS DOM manipulation
- OS detection: HIGH -- navigator.userAgent is well-established
- Download URL patterns: HIGH -- verified from release.yml
- Privacy content updates: MEDIUM -- need to verify exact Linux data handling details against actual Rust code

**Research date:** 2026-03-15
**Valid until:** 2026-04-15 (stable domain, no external dependencies)
