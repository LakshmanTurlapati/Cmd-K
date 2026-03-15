# Requirements: CMD+K

**Defined:** 2026-03-14
**Core Value:** The overlay must appear on top of the active application and feel instant

## v0.3.9 Requirements

Requirements for Linux Support & Smart Terminal Context milestone. Each maps to roadmap phases.

### Linux Process Detection

- [x] **LPROC-01**: CWD detected via `/proc/PID/cwd` readlink for the active terminal's shell process
- [x] **LPROC-02**: Shell type detected via `/proc/PID/exe` readlink (bash, zsh, fish, etc.)
- [x] **LPROC-03**: Process tree walking via `/proc/PID/children` to find shell process from terminal emulator PID

### Linux Overlay & Hotkey

- [x] **LOVRL-01**: System-wide Ctrl+K hotkey registers and triggers overlay on X11
- [x] **LOVRL-02**: Overlay appears as floating window above active application on X11
- [x] **LOVRL-03**: Wayland users can run with `GDK_BACKEND=x11` (XWayland) for full overlay functionality
- [x] **LOVRL-04**: Active window PID captured before overlay shows (capture-before-show pattern)
- [x] **LOVRL-05**: CSS-only frosted glass fallback (no window-vibrancy on Linux)

### Linux Paste

- [x] **LPST-01**: Auto-paste into active terminal via xdotool keystroke simulation on X11
- [x] **LPST-02**: Wayland graceful fallback — copies to clipboard with "press Ctrl+Shift+V" hint
- [x] **LPST-03**: Destructive command detection works with Linux-specific patterns (already built)

### Linux Terminal Text Reading

- [x] **LTXT-01**: AT-SPI2 D-Bus integration reads terminal text from VTE-based terminals (GNOME Terminal, Tilix, Terminator)
- [x] **LTXT-02**: kitty remote control (`kitty @ get-text`) reads terminal text from kitty
- [x] **LTXT-03**: WezTerm CLI (`wezterm cli get-text`) reads terminal text from WezTerm
- [x] **LTXT-04**: Graceful None return for terminals without text reading support (Alacritty, st)

### Smart Terminal Context

- [x] **SCTX-01**: ANSI escape sequence stripping from terminal output before sending to AI
- [x] **SCTX-02**: Token budget allocation — terminal context uses ~10-15% of model's context window
- [x] **SCTX-03**: Command-output pairing — truncation removes oldest complete command+output segments, not arbitrary lines
- [x] **SCTX-04**: Cross-platform module — smart truncation applies to macOS, Windows, and Linux equally

### AppImage Distribution

- [x] **APKG-01**: AppImage built via Tauri bundler with ubuntu-22.04 CI base for glibc compatibility
- [x] **APKG-02**: Third CI job in release.yml builds Linux AppImage alongside macOS DMG and Windows NSIS
- [x] **APKG-03**: Auto-updater supports Linux AppImage (Ed25519 signed, latest.json manifest)
- [x] **APKG-04**: GitHub Release includes Linux AppImage artifact with SHA256 checksum

## Future Requirements

### Deferred

- **Wayland native overlay** — Wayland protocol lacks always-on-top and global hotkey support; revisit when protocols mature
- **Wayland native paste** — wtype/ydotool have significant limitations; revisit when tooling stabilizes
- **Terminal scrollback access** — Direct PTY scrollback reading for terminals that don't expose text via API

## Out of Scope

| Feature | Reason |
|---------|--------|
| Native Wayland always-on-top | Protocol-level gap, no solution exists |
| Native Wayland global hotkey | Protocol-level gap, XWayland is industry standard fallback |
| .deb/.rpm packages | AppImage covers all distros with single binary |
| Snap/Flatpak packaging | Sandboxing conflicts with /proc access and xdotool |
| Terminal multiplexer integration (tmux/screen) | Complex, different UX model, defer to future |
| Linux window-vibrancy effect | window-vibrancy crate doesn't support Linux |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| LPROC-01 | Phase 30 | Complete |
| LPROC-02 | Phase 30 | Complete |
| LPROC-03 | Phase 30 | Complete |
| LOVRL-01 | Phase 31 | Complete |
| LOVRL-02 | Phase 31 | Complete |
| LOVRL-03 | Phase 31 | Complete |
| LOVRL-04 | Phase 31 | Complete |
| LOVRL-05 | Phase 31 | Complete |
| LPST-01 | Phase 32 | Complete |
| LPST-02 | Phase 32 | Complete |
| LPST-03 | Phase 32 | Complete |
| SCTX-01 | Phase 33 | Complete |
| SCTX-02 | Phase 33 | Complete |
| SCTX-03 | Phase 33 | Complete |
| SCTX-04 | Phase 33 | Complete |
| LTXT-01 | Phase 34 | Complete |
| LTXT-02 | Phase 34 | Complete |
| LTXT-03 | Phase 34 | Complete |
| LTXT-04 | Phase 34 | Complete |
| APKG-01 | Phase 35 | Complete |
| APKG-02 | Phase 35 | Complete |
| APKG-03 | Phase 35 | Complete |
| APKG-04 | Phase 35 | Complete |

**Coverage:**
- v0.3.9 requirements: 23 total
- Mapped to phases: 23
- Unmapped: 0

---
*Requirements defined: 2026-03-14*
*Last updated: 2026-03-14 after roadmap creation*
