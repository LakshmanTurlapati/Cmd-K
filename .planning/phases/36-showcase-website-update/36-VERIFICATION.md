---
phase: 36-showcase-website-update
verified: 2026-03-15T20:00:00Z
status: passed
score: 16/16 must-haves verified
re_verification: false
---

# Phase 36: Showcase Website Update Verification Report

**Phase Goal:** Showcase website reflects v0.3.9 with Linux support — updated version numbers, platform-specific download buttons (macOS/Windows/Linux AppImage), and privacy policy page with version history
**Verified:** 2026-03-15
**Status:** PASSED
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | All version strings on index.html show v0.3.9 (footer, demo settings, download URLs) | VERIFIED | `data-version>v0.3.9` at lines 377 and 529 of index.html; VERSION = '0.3.9' in main.js; zero hardcoded v0.2.8 found |
| 2 | Visitor on macOS sees macOS download button highlighted as primary | VERIFIED | `data-platform="macos"` on hero button (line 66); JS adds `platform-detected` class via detectOS() |
| 3 | Visitor on Linux sees Linux download button highlighted as primary | VERIFIED | `data-platform="linux"` on hero button (line 75); same OS auto-detect path in main.js |
| 4 | Clicking Linux download shows a lightweight popup with x86_64 and aarch64 choices | VERIFIED | `.arch-popup#hero-arch-popup` div (lines 79-88) with `data-download="linux_x86"` and `data-download="linux_arm"` anchor elements; JS popup toggle at main.js lines 208-221 |
| 5 | All three platforms (macOS, Windows, Linux) are visible as download options | VERIFIED | `data-download="macos"` (line 66), `data-download="windows"` (line 70), `.linux-download-btn` (line 75) in hero section |
| 6 | Feature cards mention Linux alongside macOS and Windows | VERIFIED | System-Wide Overlay card (line 163): "Always one keystroke away on macOS, Windows, and Linux"; Multi-Terminal Support card (line 187): "macOS, Windows, and Linux -- including GNOME Terminal..." |
| 7 | GNOME Terminal appears in the terminal carousel | VERIFIED | Line 447 (original set) and line 464 (aria-hidden duplicate set) both contain GNOME Terminal entry with SVG icon |
| 8 | A Linux platform badge appears in the hero section | VERIFIED | `.platform-badges` div at lines 96-110 with macOS, Windows, and Linux badge spans |
| 9 | CTA section download button uses the data-download pattern for OS-aware URL | VERIFIED | Line 490: `<a href="#" data-download="macos" ...>` inside the `.cta-buttons` block |
| 10 | Privacy policy shows 'Last updated: March 15, 2026' | VERIFIED | Line 59 of privacy.html: `Last updated: March 15, 2026` |
| 11 | Permissions section covers macOS, Windows, and Linux with platform-specific details | VERIFIED | Lines 155-163 of privacy.html: three platform `<li>` items with macOS Accessibility API, Windows Win32 APIs, and Linux /proc + xdotool + AT-SPI2 |
| 12 | Linux-specific data handling is documented (/proc, xdotool, AT-SPI2) | VERIFIED | Line 161 of privacy.html contains all three: `/proc`, `xdotool`, `AT-SPI2` |
| 13 | Credential store mentions Linux system keyring via libsecret | VERIFIED | Line 139 of privacy.html: "system keyring via libsecret on Linux" |
| 14 | March 9, 2026 policy is preserved verbatim in a collapsible section | VERIFIED | Lines 182-286: `<details class="policy-version">` with `March 9, 2026` summary and full original policy body including all original sections |
| 15 | Terminal Mode data list includes recent terminal text with budget mention | VERIFIED | Line 83 of privacy.html: "Recent terminal text (visible buffer content, up to approximately 12% of the model's context window)" |
| 16 | Footer version badge uses data-version attribute for JS population | VERIFIED | privacy.html line 332: `<span data-version>v0.3.9</span>`; main.js loaded at line 337 |

**Score:** 16/16 truths verified

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `showcase/js/main.js` | VERSION constant, URLS map, OS detection, data-attribute population, arch popup toggle | VERIFIED | All 8 required patterns confirmed: VERSION, DOWNLOAD_BASE, detectOS, data-download, data-version, data-platform, arch-popup, linux-download-btn |
| `showcase/index.html` | Download buttons with data-download attrs, Linux badge, updated feature cards, GNOME Terminal in carousel | VERIFIED | All HTML patterns confirmed; zero v0.2.8 strings remain; 2 v0.3.9 fallback values present |
| `showcase/css/home.css` | Arch popup styles, platform-detected highlight, download-secondary row | VERIFIED | arch-popup at line 1197, platform-detected at line 1192, linux-download-wrapper at line 1187, platform-badges at line 1170 |
| `showcase/privacy.html` | Updated privacy policy with Linux details and version history | VERIFIED | All 10 required patterns found; 2 policy-version details blocks; zero v0.2.8 strings |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `showcase/js/main.js` | `showcase/index.html` | `data-download` and `data-version` attributes | WIRED | `querySelectorAll('[data-download]')` at main.js line 191 iterates all download anchors; `querySelectorAll('[data-version]')` at line 194 populates version badges |
| `showcase/js/main.js` | `showcase/index.html` | linux-arch-popup toggle | WIRED | `querySelectorAll('.linux-download-btn')` at main.js line 208; toggle via `popup.classList.toggle('open')` |
| `showcase/js/main.js` | `showcase/privacy.html` | `data-version` attribute on footer badge | WIRED | privacy.html loads `<script src="js/main.js">` at line 337; footer span has `data-version` at line 332 |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| WEB-01-VERSION | 36-01-PLAN.md | Version strings updated to v0.3.9 as single source of truth | SATISFIED | `var VERSION = '0.3.9'` in main.js; data-version attributes on index.html and privacy.html footers; zero hardcoded v0.2.8 in either file |
| WEB-02-DOWNLOADS | 36-01-PLAN.md | Platform-specific download buttons (macOS/Windows/Linux AppImage) | SATISFIED | Three download buttons with data-download attrs; Linux arch popup with x86_64 and aarch64 links; URLS map pointing to GitHub Releases with correct filenames |
| WEB-03-CONTENT | 36-01-PLAN.md | Content reflects Linux support (feature cards, carousel, badges) | SATISFIED | Both feature cards mention Linux; GNOME Terminal in carousel (original and aria-hidden sets); platform-badges in hero with Linux badge |
| WEB-04-PRIVACY | 36-02-PLAN.md | Privacy policy updated with Linux details and version history | SATISFIED | March 15 date, Linux permissions section (/proc, xdotool, AT-SPI2), libsecret credential store, 12% budget mention, March 9 policy preserved verbatim in collapsible section |

**Note on WEB-* requirement IDs:** These IDs (WEB-01-VERSION through WEB-04-PRIVACY) are declared in the plan frontmatter and referenced in ROADMAP.md but are not registered in `.planning/REQUIREMENTS.md`. REQUIREMENTS.md covers the v0.3.9 technical requirements (LPROC, LOVRL, LPST, LTXT, SCTX, APKG series). The WEB-* IDs are phase-local labels for the showcase website work and are fully covered by the plan must_haves. No orphaned requirements found within REQUIREMENTS.md for Phase 36.

---

## Anti-Patterns Found

None found. Scan of all three modified files:

- `showcase/js/main.js` — No TODO/FIXME/placeholder patterns; VERSION is a real constant; detectOS() is real implementation; all handlers are wired to DOM
- `showcase/index.html` — No placeholder content; data-download attributes populated by JS; all three platform buttons present and substantive
- `showcase/css/home.css` — Arch popup CSS is fully specified with display, positioning, z-index, and `.open` state; platform-detected uses actual box-shadow rule
- `showcase/privacy.html` — No stub content; Linux permissions section is detailed and substantive; collapsible history blocks contain full verbatim policy text

---

## Human Verification Required

The following behaviors require browser testing to confirm fully:

### 1. OS Auto-Detection Highlight

**Test:** Open showcase/index.html in a browser on Linux, macOS, and Windows respectively
**Expected:** The download button for the visitor's platform gains a visible highlight ring (box-shadow with --accent color)
**Why human:** `navigator.userAgent` detection and CSS class application cannot be confirmed from static file analysis alone

### 2. Linux Arch Popup Click Behavior

**Test:** Open showcase/index.html in a browser, click the Linux download button
**Expected:** A popup appears below the button showing two options: "x86_64 / Intel / AMD (most common)" and "aarch64 / ARM (Raspberry Pi, etc.)". Clicking outside closes the popup.
**Why human:** JavaScript event listeners and popup toggle are wired in code but click interaction requires browser execution

### 3. Download Link Resolution

**Test:** Click each platform download link (macOS, Windows, Linux x86_64, Linux aarch64, CTA button)
**Expected:** Browser navigates to the correct GitHub Releases URL for v0.3.9 with the correct artifact filename
**Why human:** The URLs are JS-constructed at runtime from DOWNLOAD_BASE + VERSION; static analysis confirms the construction pattern but not the live URL validity

### 4. Privacy Policy Collapsible Expansion

**Test:** Open showcase/privacy.html, expand the "March 9, 2026" collapsible section
**Expected:** Full original policy text is visible including all original sections (TLDR, what is collected, what is not sent, data storage, permissions, open source)
**Why human:** Correct rendering of `<details>/<summary>` with full content requires visual inspection

### 5. Footer Version Badge Population

**Test:** Open both showcase/index.html and showcase/privacy.html; inspect the footer
**Expected:** Footer displays "v0.3.9" (fallback text is present; JS population should match it)
**Why human:** JS population of data-version attributes fires on DOMContentLoaded and cannot be verified without browser execution

---

## Gaps Summary

No gaps found. All 16 observable truths are verified by direct codebase inspection. All artifacts exist, are substantive (not stubs), and are properly wired. All four requirement IDs claimed by the phase plans are satisfied by implementation evidence. The three commits documented in the summaries (7937c9d, 54cccbd, 2b944dc) exist in git history with appropriate content.

---

_Verified: 2026-03-15_
_Verifier: Claude (gsd-verifier)_
