# Domain Pitfalls: Linux Support & Smart Terminal Context (v0.3.9)

**Domain:** Adding Linux platform support (1:1 macOS parity) and smart terminal context reading to an existing cross-platform Tauri v2 app
**Researched:** 2026-03-13
**Overall confidence:** MEDIUM-HIGH (based on Tauri issue tracker findings, Wayland protocol limitations confirmed via official sources, and project's own cross-platform detection history)

> This document supersedes the v0.2.8 PITFALLS.md. Prior pitfalls for Windows terminal detection (Pitfalls 1-14) remain valid for the Windows code path but are out of scope here. This document covers ONLY pitfalls specific to adding Linux platform support and smart terminal context: overlay behavior on X11/Wayland, /proc-based terminal context, clipboard/paste on Linux, scrollback reading, AppImage distribution, and CI testing.

---

## Critical Pitfalls

Mistakes that cause architectural rewrites, platform-wide feature breakage, or "works on my machine but not on theirs" failures.

---

### Pitfall 1: Wayland Has No Global Hotkey Protocol

**What goes wrong:** The `tauri-plugin-global-shortcut` / `global-hotkey` crate uses X11's XGrabKey mechanism to register system-wide hotkeys. On pure Wayland sessions (no XWayland), this silently fails -- no error is returned but the hotkey never fires. The entire app becomes unusable because users cannot summon the overlay.

**Why it happens:** Wayland deliberately does not expose a global hotkey protocol. The `wayland-protocols` repository has no accepted specification for global shortcuts. Each compositor implements its own mechanism: GNOME uses `org.gnome.Shell.Extensions` D-Bus, KDE uses `org.kde.kglobalaccel` D-Bus, Sway uses `bindsym` in its config. XWayland provides X11 compatibility but many modern distros default to pure Wayland sessions (Ubuntu 24.04+, Fedora 40+).

**Consequences:** CMD+K is completely non-functional on pure Wayland. This is a showstopper, not a cosmetic issue. The app launches, sits in the tray, but the hotkey never fires. No error is surfaced to the user.

**Prevention:**
- Detect the display server at startup: check `$XDG_SESSION_TYPE` (returns "x11" or "wayland") and `$WAYLAND_DISPLAY` (set on Wayland)
- On Wayland: display a prominent warning explaining the limitation, or implement compositor-specific D-Bus hotkey registration
- Fallback strategy: register a D-Bus service that listens for a custom shortcut configured in the user's compositor settings (Sway config, GNOME custom shortcut, KDE global shortcuts)
- Test on BOTH pure Wayland and XWayland sessions -- XWayland masks the bug because X11 XGrabKey works through XWayland
- Consider requiring XWayland for v0.3.9 and documenting pure Wayland as a known limitation if compositor-specific D-Bus integration is too complex

**Detection:** Launch the app on a pure Wayland session (GNOME on Ubuntu 24.04 default). If Ctrl+K does nothing and `journalctl` shows no errors, this pitfall is active.

**Phase relevance:** Must be addressed in the FIRST phase of Linux work. If the hotkey does not work, nothing else matters. This is the Linux equivalent of macOS Accessibility permission -- a platform gate.

---

### Pitfall 2: `always_on_top` Does Not Work on Wayland

**What goes wrong:** `tauri.conf.json` has `"alwaysOnTop": false` (because macOS uses NSPanel floating level instead), but the Linux overlay will need always-on-top behavior. Tauri's `set_always_on_top(true)` / `with_always_on_top(true)` does NOT work on Wayland. The window appears but can be covered by other windows. Confirmed in [tauri-apps/tao#1134](https://github.com/tauri-apps/tao/issues/1134) and [tauri-apps/tauri#3117](https://github.com/tauri-apps/tauri/issues/3117).

**Why it happens:** Wayland compositors do not allow clients to set their own stacking order. The `wlr-layer-shell` protocol exists for this purpose (used by panels, lock screens, notification daemons), but Tauri does not implement it. The GTK layer shell library (`gtk-layer-shell`) provides GTK integration but Tauri's tao/wry stack does not wire it up.

**Consequences:** The overlay appears but immediately goes behind the terminal window when the user clicks. This destroys the core UX -- the overlay must float above the active terminal. On X11 this works via `_NET_WM_STATE_ABOVE`, but X11 is being phased out.

**Prevention:**
- On X11: `set_always_on_top(true)` works via EWMH `_NET_WM_STATE_ABOVE`. Use this path.
- On Wayland: investigate `gtk-layer-shell` integration. The `gtk4-layer-shell` crate exists. Since Tauri on Linux uses GTK, it may be possible to access the underlying GtkWindow and apply layer shell properties. This is experimental and compositor-dependent (works on wlroots-based compositors like Sway/Hyprland, not guaranteed on GNOME mutter).
- Alternatively: set the window type hint to "dialog" or "utility" which some compositors keep above normal windows
- The macOS path uses NSPanel -- Linux has no equivalent. This is the single biggest architectural difference.
- Worst case: document that Wayland users may need to configure their compositor to keep CMD+K always on top (compositor-specific keybind)

**Detection:** Open a terminal, trigger CMD+K, then click on the terminal. If the overlay disappears behind the terminal, this pitfall is active.

**Phase relevance:** Overlay phase. Must be solved or documented as a limitation before other features are built on top.

---

### Pitfall 3: Getting the Frontmost App PID on Wayland Is Compositor-Dependent

**What goes wrong:** On macOS, `get_frontmost_pid()` uses NSWorkspace. On Windows, `GetForegroundWindow()` + `GetWindowThreadProcessId()`. On Linux X11, `_NET_ACTIVE_WINDOW` + `_NET_WM_PID` via X11 properties. On Wayland, there is NO standard protocol for "which window has focus?" from a client application. Each compositor has its own D-Bus interface or no interface at all.

**Why it happens:** Wayland's security model prevents clients from inspecting other clients' state. This is a deliberate design choice, not a bug. The compositor knows which window is focused but does not expose this to other clients.

**Consequences:** The pre-capture-before-show pattern (capture frontmost PID before overlay steals focus) that is critical for terminal detection cannot work on Wayland without compositor-specific code. Without the previous app's PID, we cannot read /proc/PID/cwd, cannot determine shell type, cannot read terminal output -- the entire terminal context pipeline breaks.

**Prevention:**
- On X11: use `xcb` or `x11rb` crate to read `_NET_ACTIVE_WINDOW` root property, then `_NET_WM_PID` on the active window. This is reliable and well-documented.
- On Wayland: the ext-foreign-toplevel-list protocol provides window information but requires compositor support. wlroots-based compositors (Sway, Hyprland) implement `zwlr_foreign_toplevel_manager_v1`. GNOME has `org.gnome.Shell.Eval` D-Bus (unreliable, locked down). KDE has `org.kde.KWin.Scripting`.
- Alternative approach: when the hotkey fires, the compositor knows which window was focused. If the hotkey is registered via compositor config (not X11 XGrabKey), the compositor can pass the focused window info. This requires compositor integration.
- Pragmatic fallback: use `xdotool getactivewindow getwindowpid` on X11/XWayland. Accept that pure Wayland may not support frontmost-app detection initially.
- The PID capture MUST happen synchronously in the hotkey handler, exactly as the macOS and Windows paths do. Any async approach risks the focus having already changed.

**Detection:** Trigger CMD+K on Wayland. If `previous_app_pid` is always None, the frontmost app detection is failing.

**Phase relevance:** Terminal context phase. This blocks ALL terminal detection on Wayland. Must be solved (even if X11-only initially) before CWD/shell/output reading.

---

### Pitfall 4: /proc Permission Restrictions in Snap/Flatpak/Container Environments

**What goes wrong:** Linux terminal context relies on reading `/proc/PID/cwd` (current directory), `/proc/PID/cmdline` (process name), and `/proc/PID/stat` (parent PID). In Snap-confined or Flatpak-sandboxed environments, these reads return EPERM or EACCES. If CMD+K itself is distributed as a Snap/Flatpak, OR if the target terminal is running inside a container, /proc access fails.

**Why it happens:** `/proc/PID/cwd` requires `PTRACE_MODE_READ_FSCREDS` permission check (see [proc_pid_cwd(5)](https://man7.org/linux/man-pages/man5/proc_pid_cwd.5.html)). Snap confinement restricts access to `/proc` entries of processes outside the snap's namespace. Flatpak sandbox similarly restricts /proc visibility. Even without containerization, reading /proc/PID/cwd of a process owned by a different user fails.

**Consequences:** CWD detection returns None. Shell type detection returns None. Process tree walking returns empty results. The entire terminal context is blank -- CMD+K generates commands without knowing the user's current directory or shell.

**Prevention:**
- CMD+K runs as a normal user-space process (NOT a Snap/Flatpak). AppImage is the correct distribution format -- it runs with the user's normal permissions and full /proc access.
- Document clearly: "CMD+K does not work when installed as a Snap or Flatpak" -- do not chase sandbox escape heuristics
- For reading /proc of the TARGET terminal: both processes (CMD+K and the terminal) run as the same user, so /proc access works. This only fails if the terminal is running as a different user (e.g., `sudo -i`) or inside a container.
- Add graceful fallback: if `/proc/PID/cwd` returns EPERM, log a warning and return None instead of crashing. The existing `Option<String>` return types already handle this.
- Test with: normal terminal, terminal running `sudo -i`, terminal inside Docker, terminal inside Toolbox/Distrobox

**Detection:** If CWD is always None on Linux but shell_type works, /proc/PID/cwd specifically is being denied. Check `ls -la /proc/<terminal_pid>/cwd` manually.

**Phase relevance:** Terminal context phase. Must handle gracefully but does not need special workarounds for the normal case (same-user AppImage).

---

### Pitfall 5: Linux Has No Equivalent to macOS Accessibility API or Windows UIA for Terminal Text

**What goes wrong:** On macOS, `ax_reader.rs` reads visible terminal text via the Accessibility API (AXUIElementCopyAttributeValue). On Windows, `uia_reader.rs` reads via UI Automation (TextPattern or tree walk). On Linux, there is NO system-wide API to read the text content of another application's window. AT-SPI2 (Linux's accessibility framework) exists but terminal emulators have inconsistent support, and it is far less reliable than macOS AX or Windows UIA.

**Why it happens:** Linux terminal emulators are diverse. GPU-rendered terminals (Alacritty, kitty, WezTerm) do not expose text via AT-SPI2 at all. GTK-based terminals (GNOME Terminal, Tilix) expose some text via AT-SPI2 but the API is complex and fragile. Qt-based terminals (Konsole) have different AT-SPI2 behavior. The ecosystem never converged on a standard accessibility text interface for terminals.

**Consequences:** `visible_output` will be None for most Linux terminals unless an alternative reading mechanism is implemented. This is the same situation as GPU terminals on macOS (Alacritty, kitty, WezTerm are already None) but extends to ALL terminals on Linux.

**Prevention:**
- Do NOT try to build an AT-SPI2 reader for Linux as the primary text reading mechanism. The effort-to-reliability ratio is terrible.
- Instead, consider terminal-specific mechanisms:
  - kitty: has a remote control protocol (`kitty @ get-text`) that returns buffer content
  - WezTerm: has a CLI (`wezterm cli get-text`) for the same purpose
  - tmux: `tmux capture-pane -p` captures the visible pane content
  - GNOME Terminal / VTE-based: libvte exposes buffer content but only to the hosting process
- For the "smart terminal context" feature, the scrollback approach (reading via terminal-specific APIs) IS the Linux solution. There is no generic cross-terminal approach.
- Start with the terminals that have APIs (kitty, WezTerm, tmux) and accept None for others. This matches the macOS pattern where GPU terminals already return None.

**Detection:** If visible_output is always None on Linux, this is expected for terminals without specific API support. Not a bug -- a known limitation to be documented.

**Phase relevance:** Terminal output reading phase. Determines the architecture of the smart context feature on Linux.

---

## Moderate Pitfalls

These cause incorrect behavior in specific scenarios but are recoverable without rewrites.

---

### Pitfall 6: Clipboard Write + Paste on Wayland Requires Focus

**What goes wrong:** On macOS, `pbcopy` writes to clipboard from any context. On Windows, `arboard` writes via Win32 clipboard APIs. On Wayland, clipboard access requires the client to have keyboard focus. If CMD+K writes to the clipboard while the overlay has focus, then hides the overlay and tries to paste into the terminal, the clipboard content may be lost because the overlay lost focus before the paste target received it.

**Why it happens:** Wayland's clipboard protocol (`wl_data_device`) ties clipboard ownership to the focused surface. When a surface loses focus, its clipboard offer can be revoked (depending on compositor behavior). The `wl-clipboard` tool works around this by spawning a background process that maintains clipboard ownership, but programmatic clipboard access from Rust via `arboard` or `copypasta` may not implement this workaround.

**Consequences:** Command is written to clipboard, overlay hides, terminal gets focus, but Ctrl+V pastes the PREVIOUS clipboard content (not the command). This is a race condition that manifests intermittently.

**Prevention:**
- On X11: `xclip` or `xsel` work reliably from any context. `arboard` uses X11 clipboard internally. No issue.
- On Wayland: use `wl-copy` (from wl-clipboard package) as the clipboard write mechanism, similar to how macOS uses `pbcopy`. wl-copy spawns a helper that maintains clipboard ownership even after the calling process exits.
- Alternatively: use `arboard` with its Wayland backend, but test the focus-loss-clipboard-revocation scenario explicitly
- The paste mechanism itself needs attention: xdotool does NOT work on Wayland. Alternatives:
  - `ydotool type --key ctrl+v` (requires ydotool daemon running as root, or uinput permission)
  - `wtype -M ctrl -k v -m ctrl` (Wayland-native, but only works on wlroots compositors)
  - Direct keyboard event injection is compositor-dependent
- On X11: `xdotool key ctrl+v` works reliably
- Pragmatic approach: detect X11 vs Wayland and use the appropriate tool. Require users to have `xdotool` (X11) or `ydotool`/`wtype` (Wayland) installed, or bundle them.

**Detection:** Write a command, dismiss overlay, check if Ctrl+V pastes the correct content. If it pastes old content, clipboard ownership was lost during the focus transition.

**Phase relevance:** Paste phase. Must be solved per display server. Test both X11 and Wayland paths.

---

### Pitfall 7: Linux Process Tree Walking Differs from macOS/Windows

**What goes wrong:** The existing `process.rs` uses `proc_listchildpids()` on macOS (libproc FFI) and `CreateToolhelp32Snapshot` on Windows. On Linux, process tree walking reads `/proc/PID/stat` (field 4 = ppid) and `/proc/PID/cmdline` (process name). The code paths are different but the logic should be equivalent. Mistakes in the Linux implementation will cause wrong shell detection.

**Why it happens:** Linux /proc parsing has quirks:
- `/proc/PID/cmdline` separates args with null bytes, not spaces
- `/proc/PID/exe` is a symlink to the binary, but may point to "(deleted)" after an update
- `/proc/PID/stat` has the process name in parentheses, and if the name contains spaces or parentheses, naive parsing breaks
- The process tree includes kernel threads (ppid=2) that should be filtered
- `login` and `sshd` are shell wrappers (already in SHELL_WRAPPERS) but Linux also has `systemd --user` as a session parent

**Consequences:** Shell is not found in the process tree, or the wrong process is identified as the shell. CWD reads from the wrong PID.

**Prevention:**
- Parse `/proc/PID/stat` carefully: the process name field (field 2) is enclosed in parentheses and can itself contain parentheses. Find the LAST `)` in the line, then parse fields after it.
- Read `/proc/PID/cmdline` as null-separated bytes, take the first element (argv[0]), extract the filename portion
- Filter kernel threads: any process with ppid=0 or ppid=2 (kthreadd) is a kernel thread
- The existing KNOWN_SHELLS, MULTIPLEXERS, and SHELL_WRAPPERS lists are portable and should work as-is
- Test with: bash, zsh, fish, tmux+zsh, login+bash (tty), sshd+zsh (remote), systemd-user+bash (DE terminal)
- `/proc/PID/cwd` on Linux is equivalent to macOS `PROC_PIDVNODEPATHINFO` -- just `readlink("/proc/PID/cwd")`

**Detection:** If shell_type is wrong on Linux but CWD is correct (or vice versa), the process tree walk is finding the wrong process.

**Phase relevance:** Terminal context phase. Core infrastructure that everything else depends on.

---

### Pitfall 8: System Tray Icon Missing on Wayland in Dev Mode

**What goes wrong:** The system tray icon works on X11 and in AppImage, but disappears on Wayland during development (`cargo tauri dev`). Confirmed in [tauri-apps/tauri#14234](https://github.com/tauri-apps/tauri/issues/14234). Users on Wayland may not see the tray icon and have no way to access settings or quit the app.

**Why it happens:** Wayland compositors implement the `StatusNotifierItem` D-Bus protocol for system tray icons, but the behavior differs between dev mode (where the binary name changes) and release mode (where it has a proper desktop file).

**Prevention:**
- Test tray icon on BOTH X11 and Wayland, in BOTH dev and release modes
- Ensure the `.desktop` file is properly configured with the correct icon path
- On GNOME Wayland, users need the "AppIndicator" extension (ubuntu-appindicator) for tray icons to appear at all -- document this requirement
- Add a "quit" menu item in the tray menu (already exists) because some Wayland compositors may not show the tray icon

**Detection:** Run `cargo tauri dev` on Ubuntu 24.04 with Wayland. If no tray icon appears, this pitfall is active.

**Phase relevance:** Tray/background daemon phase.

---

### Pitfall 9: AppImage glibc Floor Creates Silent Failures on Older Distros

**What goes wrong:** An AppImage built on Ubuntu 24.04 links against glibc 2.39. When run on Ubuntu 22.04 (glibc 2.35), it crashes with "GLIBC_2.38 not found" or similar. The error appears in stderr but the app simply does not start -- no GUI error is shown.

**Why it happens:** AppImage bundles application libraries but relies on the host system's glibc (glibc cannot be reliably bundled due to NSS plugin requirements). The AppImage works on systems with the same or newer glibc, not older.

**Consequences:** Users on LTS distributions (Ubuntu 22.04, Debian 11/12) cannot run the app. No error dialog -- the app just does not start.

**Prevention:**
- Build AppImages on the OLDEST target distribution. For 2026, Ubuntu 22.04 (glibc 2.35) is the floor.
- CI: use a Docker container based on Ubuntu 22.04 for AppImage builds, NOT the default `ubuntu-latest` (which is 24.04 on GitHub Actions as of 2026)
- WebKitGTK version: Tauri v2 requires webkit2gtk-4.1. Verify this is available on Ubuntu 22.04 (it is, from the default repos).
- Test the built AppImage on Ubuntu 22.04 in CI: add a test job that runs the AppImage on an older container (smoke test: does it launch?)
- Document minimum distro versions in README: "Requires glibc 2.35+ (Ubuntu 22.04+, Fedora 36+, Debian 12+)"

**Detection:** Download the AppImage on an older distro. If it crashes with a glibc error, the build container is too new.

**Phase relevance:** CI/CD and distribution phase. Must be configured correctly from the first Linux build.

---

### Pitfall 10: Transparent/Frosted Glass Window Broken on Linux

**What goes wrong:** The overlay uses `"transparent": true` in tauri.conf.json for the frosted glass effect. On Linux, window transparency behavior varies wildly: some compositors support ARGB windows, others do not. The `transparent: true` flag may produce a black background instead of transparency, or the transparency works but there is no blur effect (frosted glass requires compositor-specific blur protocols).

**Why it happens:** macOS has a system-wide vibrancy API (NSVisualEffectView). Windows has `SetWindowCompositionAttribute` for acrylic blur. Linux has no standard blur API. KDE/KWin supports `_KDE_NET_WM_BLUR_BEHIND_REGION` X11 atom. Sway/wlroots supports nothing (transparency yes, blur no). GNOME's mutter has no blur protocol.

**Consequences:** The overlay looks wrong -- either fully opaque (black background) or transparent but without blur (jarring, text hard to read against busy backgrounds).

**Prevention:**
- Test transparency on: GNOME (mutter), KDE (kwin), Sway (wlroots), i3 (with picom compositor)
- Accept that frosted glass (blur) is not achievable on most Linux compositors. Use a semi-transparent dark background as fallback.
- The CSS should use `backdrop-filter: blur()` for the WebView content but this only blurs web content behind the element, not the desktop behind the window. The window-level transparency is what shows the desktop through.
- Consider using a solid dark background with high opacity (e.g., rgba(0,0,0,0.85)) on Linux instead of trying to match macOS vibrancy. This is what most Linux overlay apps (rofi, dmenu, ulauncher) do.
- Do NOT waste time trying to achieve macOS-quality vibrancy on Linux. It is not possible with current protocols.

**Detection:** Launch overlay on GNOME Wayland. If background is black instead of transparent/blurred, window transparency is not working.

**Phase relevance:** Overlay appearance phase. Purely cosmetic but affects perceived quality.

---

### Pitfall 11: Smart Context Scrollback Can Exceed AI Token Limits

**What goes wrong:** The "smart terminal context" feature reads scrollback buffer content (hundreds or thousands of lines) and sends it as AI context. Without intelligent truncation, this can consume the entire context window (4K-128K tokens depending on model), leaving no room for the system prompt, turn history, or response generation. Cost also spikes because pricing is per-token.

**Why it happens:** Terminal scrollback buffers can contain 10K-100K+ lines. A naive "send everything" approach sends all of it. Even a "last 200 lines" approach can include large command outputs (e.g., `npm install` logs, `git log` output, build errors) that are mostly noise.

**Consequences:** Token limit exceeded (generation fails or gets truncated), excessive cost per query, or the useful context (CWD, recent commands, errors) is pushed out of the context window by noise.

**Prevention:**
- Implement a truncation strategy that prioritizes recent and relevant content:
  1. Always include the last prompt line (shows CWD + command being worked on)
  2. Include the last N lines of output (configurable, default 50)
  3. If output exceeds budget, keep first 10 lines + last 40 lines (captures both the start of output and recent errors)
  4. Total context budget: 2000-4000 tokens (approximately 1500-3000 words), configurable
- Count tokens BEFORE sending, not after. Use a rough heuristic: ~4 chars per token for English text, ~3 chars per token for code.
- Strip ANSI escape codes before counting and sending -- they waste tokens and confuse the AI
- Strip repeated blank lines and common noise patterns (progress bars, download percentages, spinner frames)
- The system prompt already says "Terminal context only in first user message to prevent token bloat" (CTXT-03) -- the smart context must respect this same principle

**Detection:** If AI responses start getting cut off or if cost per query spikes dramatically after enabling smart context, truncation is insufficient.

**Phase relevance:** Smart context phase. Must be designed before the scrollback reading mechanism, because the truncation strategy determines how much data to read.

---

## Minor Pitfalls

These cause suboptimal behavior or developer confusion but are not user-facing showstoppers.

---

### Pitfall 12: Linux Terminal Emulator Identification Is Process-Name-Based, Not Bundle-ID-Based

**What goes wrong:** macOS identifies apps by bundle ID (`com.googlecode.iterm2`). Windows identifies by exe name (`WindowsTerminal.exe`). Linux terminal emulators are identified by process name (`gnome-terminal-server`, `konsole`, `alacritty`, `kitty`, `wezterm-gui`, `tilix`, `xfce4-terminal`, `mate-terminal`, `terminology`, `st`, `urxvt`, `xterm`). The process name list is much longer than macOS bundle IDs because Linux has far more terminal emulators.

**Why it happens:** Linux has no unified app identity system like macOS bundle IDs. Desktop files (.desktop) have unique IDs but these are not accessible at runtime via PID. The binary name is the only reliable identifier available from /proc.

**Prevention:**
- Create a comprehensive `KNOWN_TERMINAL_BINARIES` list for Linux. Start with: `gnome-terminal-server`, `konsole`, `alacritty`, `kitty`, `wezterm-gui`, `tilix`, `xfce4-terminal`, `mate-terminal`, `terminology`, `st`, `urxvt`, `xterm`, `foot`, `sakura`, `terminator`, `guake`, `tilda`, `termite`, `cool-retro-term`, `contour`, `blackbox-terminal`
- Note: GNOME Terminal's process name is `gnome-terminal-server` (NOT `gnome-terminal`). The `gnome-terminal` command is a launcher that talks to the server process via D-Bus.
- For IDEs: `code` (VS Code), `cursor` (Cursor), `codium` (VS Codium). The process name may vary by installation method (snap: may have a wrapper name).
- Use `std::fs::read_link("/proc/PID/exe")` to get the full binary path, then extract the filename. This is more reliable than parsing `/proc/PID/cmdline` (which can be overwritten by the process).
- The `detect.rs` module's `is_known_terminal()` function needs a Linux branch that checks process names instead of bundle IDs.

**Detection:** If a common Linux terminal (GNOME Terminal, Konsole) is not detected as a terminal, its process name is not in the known list.

**Phase relevance:** Terminal detection phase. Mostly a list curation task.

---

### Pitfall 13: Headless CI Cannot Test Overlay or Hotkey Behavior

**What goes wrong:** GitHub Actions Linux runners do not have a display server. GUI tests (overlay positioning, hotkey registration, window focus) fail with "cannot open display" errors. Even with Xvfb, testing Wayland behavior is not possible.

**Why it happens:** CI runners are headless. Xvfb provides a virtual X11 display but not Wayland. Weston (Wayland reference compositor) can run headless but is fragile in CI and does not provide the same environment as a real compositor (GNOME, KDE, Sway).

**Prevention:**
- Use `xvfb-run` for X11-based GUI tests in CI. The `GabrielBB/xvfb-action` GitHub Action handles this.
- Separate tests into: unit tests (no display needed, run normally), integration tests (need display, run under xvfb-run), and manual tests (Wayland behavior, test on real hardware).
- For the Linux build job in CI: install `webkit2gtk-4.1` dev libraries, `libappindicator3-dev` (for tray), and `xvfb` (for GUI tests)
- Do NOT attempt to run Wayland tests in CI. Mark Wayland-specific behavior as requiring manual testing.
- The existing 3-job CI architecture (parallel macOS/Windows + sequential release) should become 4-job: add a Linux build job parallel to macOS and Windows.
- Set environment variables in CI: `WEBKIT_DISABLE_COMPOSITING_MODE=1` if WebKitGTK rendering fails in Xvfb

**Detection:** If the Linux CI job fails with "Gtk initialization failed" or "Cannot open display", the Xvfb setup is missing.

**Phase relevance:** CI/CD phase. Must be set up before any Linux-specific code lands to prevent regressions.

---

### Pitfall 14: ANSI Escape Codes in Terminal Output Waste Tokens and Confuse AI

**What goes wrong:** Terminal output contains ANSI escape sequences for colors, cursor movement, bold/underline, and terminal control. These sequences are meaningless to the AI model but consume tokens. A colorized `ls` output or a `git diff` with ANSI colors can be 2-3x larger than the actual text content.

**Why it happens:** Terminal emulators store raw VT100/ANSI escape sequences in their output buffer. When reading terminal text via scrollback APIs (kitty @ get-text, tmux capture-pane), the raw sequences are included unless specifically stripped.

**Prevention:**
- Strip ANSI escape codes BEFORE sending to AI. Regex: `\x1b\[[0-9;]*[a-zA-Z]` captures most CSI sequences. Also strip OSC sequences: `\x1b\][^\x07]*\x07` and `\x1b\][^\x1b]*\x1b\\`.
- Some terminal APIs have flags to strip escape codes: `tmux capture-pane -p -e` includes escapes, `tmux capture-pane -p` (without -e) strips them. Prefer the stripped version.
- `kitty @ get-text --ansi` includes ANSI, `kitty @ get-text` (default) includes ANSI. May need post-processing.
- Add the stripping function to the existing `filter.rs` module alongside `filter_sensitive()`.
- The existing `filter_sensitive` function should be extended to also strip ANSI codes, or a new `strip_ansi()` function should be added upstream in the pipeline.

**Detection:** If AI responses reference "color codes" or "[32m" strings, ANSI escape codes are leaking through.

**Phase relevance:** Smart context phase. Must be implemented alongside scrollback reading.

---

### Pitfall 15: find_shell_pid Linux Branch Must Use Same Arity as Other Platforms

**What goes wrong:** The existing `find_shell_pid` function has different signatures per platform (Windows takes 4 args including ProcessSnapshot, non-Windows takes 3 args). Adding a Linux branch that needs different parameters would break the unified call sites in `compute_window_key`.

**Why it happens:** This project already solved this problem: the memory note documents "Both Windows (3 args with ProcessSnapshot) and non-Windows (3 args, snapshot ignored) use same arity to avoid cross-platform compilation issues in non-cfg-gated callers like compute_window_key."

**Prevention:**
- The Linux branch of `find_shell_pid` MUST use the same 3-arg signature as macOS (pid, focused_cwd, snapshot=None)
- Linux does not need a ProcessSnapshot equivalent -- /proc is always readable, no need to snapshot. Pass None for snapshot and ignore it.
- The `cfg(not(target_os = "windows"))` blocks in `compute_window_key` already handle this -- Linux falls into the non-Windows path.
- Do NOT add Linux-specific parameters to find_shell_pid. If Linux needs extra context, pass it via a different mechanism (thread-local, global state, or a separate function).

**Detection:** If `cargo check --target x86_64-unknown-linux-gnu` fails with arity mismatches, this pitfall is active.

**Phase relevance:** Terminal context implementation phase. Maintain the established pattern.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation | Severity |
|-------------|---------------|------------|----------|
| Global hotkey registration | Pitfall 1 (Wayland has no global hotkey protocol) | X11 via XGrabKey, Wayland via compositor D-Bus or document limitation | Critical |
| Overlay always-on-top | Pitfall 2 (always_on_top broken on Wayland) | X11 EWMH _NET_WM_STATE_ABOVE, Wayland investigate gtk-layer-shell | Critical |
| Frontmost app detection | Pitfall 3 (compositor-dependent on Wayland) | X11 _NET_ACTIVE_WINDOW, Wayland via ext-foreign-toplevel or accept limitation | Critical |
| Terminal context via /proc | Pitfall 4, 7 (/proc permissions, parsing quirks) | Same-user assumption, careful /proc/PID/stat parsing | Moderate |
| Terminal output reading | Pitfall 5, 14 (no universal API, ANSI codes) | Terminal-specific APIs (kitty, WezTerm, tmux), strip ANSI | Moderate |
| Clipboard and paste | Pitfall 6 (Wayland focus-based clipboard) | wl-copy for Wayland, xdotool/ydotool for keystroke injection | Moderate |
| Tray icon on Wayland | Pitfall 8 (missing in dev mode, needs AppIndicator) | Test both modes, document GNOME extension requirement | Minor |
| AppImage distribution | Pitfall 9 (glibc floor) | Build on Ubuntu 22.04, test on 22.04 | Moderate |
| Window transparency | Pitfall 10 (no blur protocol on Linux) | Accept solid/semi-transparent background, no frosted glass | Minor |
| Smart context truncation | Pitfall 11 (token explosion from scrollback) | Token budget, intelligent truncation, ANSI stripping | Moderate |
| Terminal identification | Pitfall 12 (process name list curation) | Comprehensive KNOWN_TERMINAL_BINARIES list | Minor |
| CI testing | Pitfall 13 (headless, no Wayland in CI) | Xvfb for X11 tests, manual testing for Wayland | Minor |
| Function arity | Pitfall 15 (cross-platform signature consistency) | Follow established non-Windows 3-arg pattern | Minor |

---

## Integration Pitfalls (Adding Linux to Existing Cross-Platform System)

These are specific to the v0.3.9 scenario: adding a third platform to a working macOS + Windows codebase without breaking existing platforms.

---

### Integration Risk 1: Linux cfg Blocks Breaking macOS/Windows Compilation

**What goes wrong:** Adding `#[cfg(target_os = "linux")]` blocks can interact badly with existing `#[cfg(not(target_os = "macos"))]` and `#[cfg(target_os = "windows")]` blocks. The existing code uses `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` as a catch-all for "other platforms." Linux code placed in these catch-all blocks will be compiled for all non-macOS non-Windows targets, not just Linux.

**Prevention:**
- Replace catch-all `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` with explicit `#[cfg(target_os = "linux")]` when adding Linux-specific code
- Keep the catch-all blocks as stubs that return None or Err for truly unsupported platforms
- CI: the existing `cargo check` on Linux (WSL2) will now compile Linux code paths, not just stubs. This is a GOOD change -- it means Linux code gets type-checked.
- Verify that `cargo check` on macOS and `cargo build` on Windows still pass after every Linux code addition
- The existing pattern in `detect_inner()` (lines 282-287 in mod.rs) has a catch-all that returns None -- this becomes the Linux implementation site, not a dead-code stub

**Detection:** If macOS or Windows builds break after adding Linux code, a cfg gate is wrong.

**Phase relevance:** ALL phases. Every code change must be verified on all three platforms.

---

### Integration Risk 2: Different "Capture Before Show" Mechanism Per Platform

**What goes wrong:** The hotkey handler has three pre-capture paths: macOS (NSWorkspace PID + AX text pre-capture), Windows (HWND + PID from HWND + window key from HWND), and now Linux (X11 active window / Wayland compositor query). Each platform captures different data in a different order. Adding Linux without following the same pattern leads to missing pre-capture data.

**Prevention:**
- The Linux hotkey handler MUST capture the previous app's identity BEFORE calling toggle_overlay(), exactly as macOS and Windows do
- On X11: capture window ID + PID from _NET_WM_PID + compute window key SYNCHRONOUSLY in the hotkey callback
- On Wayland: capture whatever is available (may be nothing on some compositors)
- Store results in the SAME AppState fields: `previous_app_pid`, `current_window_key`. Do NOT add Linux-specific state fields.
- The pre-captured data flows into `detect_full` (or a new `detect_full_linux` variant). The pipeline should be: capture PID -> read /proc -> find shell -> read CWD -> (optionally) read terminal output via terminal-specific API

**Detection:** If terminal context is always empty on Linux but the hotkey works, the pre-capture is not storing the PID.

**Phase relevance:** Hotkey and terminal context phases. These must be implemented together.

---

### Integration Risk 3: Smart Context Feature Accidentally Enabled for macOS/Windows Before Linux

**What goes wrong:** The smart terminal context feature (scrollback reading + intelligent truncation) is being added alongside Linux support. If the feature flag/code path is not properly gated, it may accidentally change behavior on macOS/Windows before it is fully tested there. For example, if truncation logic is applied to macOS AX text that was previously sent verbatim, existing behavior changes.

**Prevention:**
- Implement smart context as a SEPARATE code path from existing visible_output reading
- The existing `visible_output: Option<String>` field in TerminalContext should remain as-is
- Add a new field or mechanism for scrollback/smart context, e.g., `scrollback_context: Option<String>`
- The truncation and ANSI stripping logic should be applied ONLY to the new scrollback data, not retroactively to existing AX/UIA text
- Feature-gate smart context: enable for Linux first (where it fills the gap left by no AX/UIA), then opt-in for macOS/Windows

**Detection:** If macOS visible_output behavior changes after smart context is merged, the feature is not properly isolated.

**Phase relevance:** Smart context phase. Must be additive, not modifying existing behavior.

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical verification for v0.3.9.

- [ ] **Hotkey works on X11:** Ctrl+K fires and overlay appears -- but verify it also works on pure Wayland (Pitfall 1)
- [ ] **Overlay appears on top:** Window shows above terminal -- but verify always-on-top on Wayland (Pitfall 2)
- [ ] **Terminal context detected:** CWD and shell_type populated -- but verify when terminal is a Snap (Pitfall 4)
- [ ] **Terminal output captured:** visible_output has text -- but verify ANSI codes are stripped (Pitfall 14)
- [ ] **Paste works on X11:** Command appears in terminal -- but verify on Wayland with wtype/ydotool (Pitfall 6)
- [ ] **AppImage launches on Ubuntu 22.04:** App starts -- but verify it was built on 22.04 container (Pitfall 9)
- [ ] **Tray icon visible:** Icon shows on KDE/GNOME -- but verify on GNOME Wayland without AppIndicator extension (Pitfall 8)
- [ ] **Smart context truncation:** Scrollback sent to AI -- but verify token count stays within budget after large `npm install` output (Pitfall 11)
- [ ] **macOS still works:** All macOS tests pass -- but verify AX text is not being modified by smart context code (Integration Risk 3)
- [ ] **Windows still works:** All Windows tests pass -- but verify cfg gates are correct for new Linux code (Integration Risk 1)
- [ ] **Linux CI passes:** Build succeeds in GitHub Actions -- but verify AppImage was built on correct Ubuntu version (Pitfall 9, 13)

---

## Sources

### Primary (HIGH confidence -- confirmed via official issue trackers and documentation)
- [tauri-apps/tao#1134](https://github.com/tauri-apps/tao/issues/1134): always_on_top not working on Wayland -- confirmed open issue
- [tauri-apps/tauri#3117](https://github.com/tauri-apps/tauri/issues/3117): Always on top not working on Wayland -- confirmed open bug
- [tauri-apps/global-hotkey#28](https://github.com/tauri-apps/global-hotkey/issues/28): Global Shortcut support on Wayland -- confirmed limitation, no protocol exists
- [tauri-apps/tauri#14234](https://github.com/tauri-apps/tauri/issues/14234): SystemTray icon missing on Wayland in dev mode
- [tauri-apps/tauri#14796](https://github.com/tauri-apps/tauri/issues/14796): AppImage build issues with linuxdeploy
- [Tauri v2 AppImage docs](https://v2.tauri.app/distribute/appimage/): AppImage bundles all deps except glibc
- [proc_pid_cwd(5)](https://man7.org/linux/man-pages/man5/proc_pid_cwd.5.html): /proc/PID/cwd requires ptrace permission check
- [wlr-layer-shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1): Wayland layer shell for overlay/panel windows

### Secondary (MEDIUM confidence -- community sources and ecosystem analysis)
- [KDE focus stealing prevention on Wayland](https://www.neowin.net/news/kde-plasma-prepares-crackdown-on-focus-stealing-window-behavior-under-wayland/): Wayland prevents focus stealing by design
- [ydotool as xdotool replacement](https://gadgeteer.co.za/ydotool-is-an-alternative-to-xdotool-that-works-on-both-x11-and-wayland/): requires uinput permission, not ideal
- [wl-clipboard](https://github.com/bugaevc/wl-clipboard): Wayland clipboard utility, handles focus issues with helper process
- [gtk-layer-shell](https://github.com/wmww/gtk-layer-shell): GTK library for wlr-layer-shell, potential path for overlay on Wayland
- [OpenWhispr#240](https://github.com/OpenWhispr/openwhispr/issues/240): xdotool silently fails on Wayland, relevant to paste mechanism
- [Tauri Linux packaging skill](https://playbooks.com/skills/dchuk/claude-code-tauri-skills/tauri-linux-packaging): Community patterns for Tauri Linux distribution

### Project-Internal (HIGH confidence -- this codebase's established patterns)
- `src-tauri/src/terminal/mod.rs`: Detection pipeline architecture, pre-capture-before-show pattern
- `src-tauri/src/commands/hotkey.rs`: Platform-specific PID/HWND capture in hotkey handler
- `src-tauri/src/commands/paste.rs`: Platform-specific clipboard write and keystroke injection
- `src-tauri/src/terminal/process.rs`: Process tree walking, find_shell_pid arity convention
- `src-tauri/src/commands/window.rs`: Overlay show/hide with platform cfg blocks
- `.planning/PROJECT.md`: Capture-before-show as key decision, zero-setup constraint

---
*Pitfalls research for: CMD+K v0.3.9 Linux Support & Smart Terminal Context*
*Researched: 2026-03-13*
