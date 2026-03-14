# Project Research Summary

**Project:** CMD+K v0.3.9 -- Linux Support & Smart Terminal Context
**Domain:** Cross-platform desktop app (Tauri v2) -- adding third platform + new AI context feature
**Researched:** 2026-03-13
**Confidence:** MEDIUM-HIGH

## Executive Summary

CMD+K v0.3.9 adds Linux platform support and smart terminal context truncation to an existing macOS + Windows Tauri v2 app. The codebase already has a clean `#[cfg(target_os)]` gating pattern with stub implementations for unsupported platforms. Linux work replaces these stubs with real implementations. The core strategy is **X11-first with graceful Wayland degradation** -- this is not a compromise but the industry standard approach used by OBS Studio, Discord, Slack, and similar apps. No frontend changes are needed; all work is in the Rust backend and CI/CD pipeline.

The recommended approach adds only 3 new Rust crates (`x11rb`, `arboard` with Wayland feature, `atspi`) plus direct `/proc` filesystem reads (zero dependencies, ~50 lines of `std::fs` code). External tools (`xdotool`, `wtype`) are invoked via subprocess, matching the macOS `osascript` pattern already established. Smart terminal context truncation is a new cross-platform module that replaces the current naive 25-line limit with character-budget-based intelligent truncation, ANSI stripping, and blank line collapsing. This benefits all three platforms.

The primary risks are Wayland limitations -- global hotkeys, always-on-top, focused window PID capture, and synthetic paste all lack standard Wayland protocols. These are not solvable at the application level; they are protocol-level gaps. The mitigation is XWayland fallback (`GDK_BACKEND=x11`) and clear user documentation. For terminal text reading, Linux has no universal API equivalent to macOS AX or Windows UIA. The recommendation is to ship without terminal text reading initially (return None, matching GPU terminal behavior on other platforms) and add AT-SPI2 + terminal-specific APIs (kitty, WezTerm) as enhancements. The AppImage must be built on Ubuntu 22.04 to avoid glibc floor issues on LTS distributions.

## Key Findings

### Recommended Stack

Three new Rust crates, direct `/proc` reads, and two external CLI tools. No frontend changes, no new npm packages.

**Core technologies:**
- `/proc` filesystem (direct `std::fs`): CWD, shell type, process tree -- zero dependencies, simpler than macOS libproc FFI or Windows PEB reads
- `x11rb` 0.13+: Active window PID via EWMH `_NET_ACTIVE_WINDOW` + `_NET_WM_PID` -- ~2ms vs xdotool's ~50ms subprocess call
- `arboard` 3 (with `wayland-data-control` feature): Clipboard write for both X11 and Wayland -- already used on Windows
- `xdotool` / `wtype` (subprocess): Paste keystroke simulation -- `Ctrl+Shift+V` on Linux (NOT `Ctrl+V`, which is "literal next char" in terminals)
- `atspi` 0.22+: AT-SPI2 terminal text reading (deferred to post-initial-release) -- works for GTK/Qt terminals, not GPU-rendered ones
- Tauri built-in: AppImage bundling, auto-update, tray icon, global hotkey (X11 only) -- no additional crates needed

### Expected Features

**Must have (table stakes):**
- System-wide Ctrl+K hotkey on X11 (Tauri plugin, zero platform code)
- Always-on-top overlay on X11 (EWMH `_NET_WM_STATE_ABOVE`)
- CWD detection via `/proc/PID/cwd` (trivially simple, easier than macOS/Windows)
- Shell type detection via `/proc/PID/exe` basename
- Process tree walking via `/proc/PID/task/PID/children`
- Clipboard write + paste via arboard + xdotool
- Focus capture/restore via xdotool on X11
- AppImage distribution with auto-update
- Smart terminal context truncation (cross-platform, replaces 25-line hardcoded limit)

**Should have (differentiators):**
- Zero-config terminal detection (no .bashrc modification -- core product differentiator vs Fig, Warp)
- Automatic X11/Wayland detection via `$XDG_SESSION_TYPE`
- Command-output pairing in truncation (semantic segments, not arbitrary line cuts)
- Error prioritization in truncation (keep stack traces, drop verbose logs)

**Defer (v2+):**
- Wayland-native layer-shell integration (wait for Tauri upstream)
- Snap/Flatpak distribution (sandbox conflicts with /proc access)
- Full tmux/screen scrollback extraction
- AT-SPI2 terminal text reading (complex, deferred to enhancement phase)

### Architecture Approach

No architectural changes needed. The existing `#[cfg(target_os)]` gating pattern absorbs Linux naturally. Every platform-specific function already has a catch-all stub returning None/empty -- Linux replaces these stubs. Smart context truncation is a new cross-platform module (`terminal/context.rs`) that sits between text capture and AI prompt construction.

**Major components:**
1. `terminal/process.rs` (Linux cfg) -- `/proc` reads for CWD, process name, child PIDs (replaces stubs)
2. `terminal/detect_linux.rs` -- App identification via `/proc/PID/exe`, terminal/IDE classification (new)
3. `commands/hotkey.rs` (Linux cfg) -- Frontmost PID via x11rb/xdotool, window ID capture (replaces stubs)
4. `commands/paste.rs` (Linux cfg) -- arboard clipboard + xdotool/wtype keystroke (replaces stubs)
5. `terminal/context.rs` -- Smart truncation: ANSI stripping, dedup, char-budget truncation (new, cross-platform)
6. `lib.rs` (Linux cfg) -- Overlay setup: always-on-top, skip-taskbar, no vibrancy (new block)
7. CI/CD `release.yml` -- AppImage build job on Ubuntu 22.04 (new job)

### Critical Pitfalls

1. **Wayland has no global hotkey protocol** -- Tauri's global-shortcut silently fails on pure Wayland. Ship as X11-first, require XWayland for Wayland users, display warning on pure Wayland sessions. This is the Linux platform gate equivalent of macOS Accessibility permission.
2. **`always_on_top` broken on Wayland** -- Confirmed in Tauri issue trackers (tao#1134, tauri#3117). No Wayland protocol for client-controlled stacking. Use EWMH on X11, accept limitation or investigate gtk-layer-shell for Wayland.
3. **Frontmost PID not queryable on Wayland** -- Wayland prevents clients from inspecting other clients' state by design. Without PID, entire terminal context pipeline breaks. X11 path works via `_NET_ACTIVE_WINDOW` + `_NET_WM_PID`.
4. **AppImage glibc floor** -- Building on Ubuntu 24.04 creates AppImages that crash on Ubuntu 22.04. CI MUST use Ubuntu 22.04 runners, NOT `ubuntu-latest`.
5. **`find_shell_pid` must maintain 3-arg arity** -- Linux branch must use same signature as macOS (pid, focused_cwd, snapshot=None) to avoid breaking `compute_window_key` call sites.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Linux Foundation (Process Detection via /proc)
**Rationale:** All other Linux features depend on process detection. /proc reads are the simplest part of the entire project -- easier than the macOS and Windows equivalents already built. No external dependencies. Gets the build compiling with real Linux code paths immediately.
**Delivers:** CWD detection, shell type detection, process tree walking, app identification, Linux system prompt
**Addresses:** Table stakes: CWD, shell type, process tree, IDE terminal detection
**Avoids:** Pitfall 7 (proc parsing quirks), Pitfall 15 (function arity consistency), Pitfall 12 (terminal identification list)
**Uses:** Direct `/proc` reads via `std::fs`, existing `process.rs` stub replacement

### Phase 2: Linux Detection Pipeline (Hotkey + PID Capture + Overlay)
**Rationale:** Depends on Phase 1 process functions. Wires up the hotkey handler, PID capture, overlay show/hide, and focus management. This is the phase that makes the app actually usable on Linux X11.
**Delivers:** Working hotkey -> overlay -> terminal context -> AI response flow on X11
**Addresses:** Table stakes: hotkey, overlay, focus capture/restore
**Avoids:** Pitfall 1 (Wayland hotkey -- detect and warn), Pitfall 2 (always-on-top -- X11 works, Wayland documented), Pitfall 3 (frontmost PID -- X11 via x11rb)
**Uses:** `x11rb` for active window PID, `xdotool` for focus restore, Tauri overlay APIs

### Phase 3: Linux Paste + Clipboard
**Rationale:** Depends on Phase 2 (needs focus capture + window ID). Completes the user-facing flow: hotkey -> context -> AI -> paste command back.
**Delivers:** Full end-to-end Linux X11 workflow
**Addresses:** Table stakes: clipboard write, paste into terminal (Ctrl+Shift+V), confirm (Enter), focus restore
**Avoids:** Pitfall 6 (Wayland clipboard focus -- use X11 path first, wl-copy fallback)
**Uses:** `arboard` (with wayland-data-control feature), `xdotool` for X11 paste, `wtype` for Wayland paste

### Phase 4: Smart Terminal Context (Cross-Platform)
**Rationale:** Independent of Linux-specific work. Can start in parallel with Phases 1-3. Benefits all three platforms immediately. Replaces the naive 25-line hardcoded limit with intelligent truncation.
**Delivers:** ANSI stripping, blank line collapsing, duplicate line dedup, character-budget truncation, truncation indicator
**Addresses:** Differentiators: token-aware context budgeting, command-output pairing
**Avoids:** Pitfall 11 (token explosion from scrollback), Pitfall 14 (ANSI codes wasting tokens)
**Uses:** Pure Rust logic in new `terminal/context.rs`, no platform-specific code

### Phase 5: AppImage Distribution + CI/CD
**Rationale:** Needs working Linux build from Phases 1-3. Third CI job parallel to macOS/Windows. Must use Ubuntu 22.04 for glibc floor.
**Delivers:** Downloadable AppImage with auto-update, CI pipeline producing Linux artifacts
**Addresses:** Table stakes: AppImage distribution, auto-update parity
**Avoids:** Pitfall 9 (glibc floor -- build on 22.04), Pitfall 13 (headless CI -- xvfb for GUI tests)
**Uses:** Tauri built-in AppImage bundling, existing Ed25519 updater signing

### Phase 6 (Enhancement): Terminal Text Reading
**Rationale:** Deferred from initial release. AT-SPI2 is complex and unreliable across terminals. CWD + shell type are the critical context. Ship without visible_output on Linux, add incrementally.
**Delivers:** Terminal text reading for GTK terminals (GNOME Terminal, Tilix) and terminals with remote APIs (kitty, WezTerm)
**Addresses:** Differentiators: multi-terminal-emulator text reading
**Avoids:** Pitfall 5 (no universal API -- accept per-terminal approach)
**Uses:** `atspi` crate for GTK terminals, subprocess calls for kitty/WezTerm remote control

### Phase Ordering Rationale

- **Phases 1-3 are sequential**: each depends on the prior phase's outputs (process functions -> detection pipeline -> paste actions)
- **Phase 4 is parallel**: smart context is cross-platform Rust with no Linux dependencies, can be developed alongside Phases 1-3
- **Phase 5 must follow Phases 1-3**: needs a working Linux binary to package
- **Phase 6 is explicitly deferred**: high complexity, low initial impact (CWD/shell are the critical context), follows the same pattern as GPU terminals on macOS/Windows returning None for visible_output
- **Wayland is treated as degraded throughout**: every phase has an X11 implementation and a Wayland fallback (usually "return None" or "display warning"). No phase blocks on Wayland support.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** X11 active window capture via x11rb needs implementation research (EWMH property queries, connection management)
- **Phase 3:** Wayland paste mechanism needs testing across compositors (wtype vs ydotool reliability)
- **Phase 6:** AT-SPI2 terminal text reading needs empirical testing per terminal emulator

Phases with standard patterns (skip research-phase):
- **Phase 1:** /proc filesystem reads are trivially documented, existing codebase has the exact patterns to follow
- **Phase 4:** Smart truncation is pure application logic, well-understood problem space
- **Phase 5:** Tauri AppImage + CI/CD is officially documented with examples

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified on crates.io. /proc is kernel-stable ABI. Tauri Linux support is documented. Only atspi crate needs empirical validation. |
| Features | HIGH | Feature landscape is clear. Table stakes map directly to existing macOS/Windows implementations. Differentiators are well-scoped. |
| Architecture | HIGH | No architectural changes needed. Existing cfg-gating pattern absorbs Linux cleanly. Smart context is a clean new module. |
| Pitfalls | MEDIUM-HIGH | Wayland limitations confirmed via official Tauri issue trackers. /proc quirks documented in man pages. Integration risks identified from codebase patterns. Unknown: AT-SPI2 reliability across terminal emulators. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Wayland global hotkey**: No solution exists. Must decide between "X11 only" and "compositor-specific D-Bus registration" during Phase 2 planning. Recommendation: X11 only for v0.3.9.
- **AT-SPI2 terminal text reliability**: The atspi crate is verified but actual terminal text reading has not been empirically tested. Defer to Phase 6 with explicit testing per terminal emulator.
- **x11rb vs xdotool for PID capture**: STACK.md recommends x11rb (2ms) over xdotool (50ms). Architecture uses xdotool in code examples. Must decide in Phase 2 -- recommend x11rb for latency-critical hotkey path, xdotool for focus restore (latency-tolerant).
- **Wayland paste tool**: wtype works on wlroots compositors only (Sway, Hyprland), NOT on GNOME Wayland. ydotool works everywhere but requires daemon + uinput permission. Must test both during Phase 3.
- **Linux-specific AppState field**: Architecture suggests `previous_xid: Mutex<Option<u64>>` for X11 window ID. May be able to reuse existing `previous_hwnd` (both are integer window IDs). Decide during Phase 2 implementation.

## Sources

### Primary (HIGH confidence)
- [Tauri v2 AppImage distribution docs](https://v2.tauri.app/distribute/appimage/)
- [Tauri v2 Global Shortcut plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri v2 Updater plugin](https://v2.tauri.app/plugin/updater/)
- [tauri-apps/tao#1134](https://github.com/tauri-apps/tao/issues/1134) -- always_on_top Wayland limitation
- [tauri-apps/tauri#3117](https://github.com/tauri-apps/tauri/issues/3117) -- always_on_top Wayland bug
- [tauri-apps/global-hotkey#28](https://github.com/tauri-apps/global-hotkey/issues/28) -- Wayland hotkey limitation
- [proc_pid_cwd(5)](https://man7.org/linux/man-pages/man5/proc_pid_cwd.5.html) -- /proc filesystem specification
- [Linux /proc filesystem kernel docs](https://www.kernel.org/doc/html/latest/filesystems/proc.html)
- [x11rb crate](https://github.com/psychon/x11rb) -- Pure Rust X11 client
- [arboard crate](https://github.com/1Password/arboard) -- Cross-platform clipboard
- [AT-SPI2 architecture](https://gnome.pages.gitlab.gnome.org/at-spi2-core/devel-docs/architecture.html)

### Secondary (MEDIUM confidence)
- [atspi Rust crate](https://crates.io/crates/atspi) -- needs empirical terminal testing
- [kitty remote control protocol](https://sw.kovidgoyal.net/kitty/remote-control/)
- [WezTerm CLI get-text](https://wezterm.org/cli/cli/get-text.html)
- [wtype Wayland keyboard simulation](https://www.linuxlinks.com/wtype-xdotool-type-wayland/)
- [Wayland xdotool fragmentation analysis](https://www.semicomplete.com/blog/xdotool-and-exploring-wayland-fragmentation/)

### Project-Internal (HIGH confidence)
- Existing codebase: `terminal/process.rs`, `terminal/mod.rs`, `commands/paste.rs`, `commands/hotkey.rs`
- Established patterns: cfg-gating, subprocess invocation, graceful degradation via Option types
- Memory: `find_shell_pid` unified 3-arg signature convention

---
*Research completed: 2026-03-13*
*Ready for roadmap: yes*
