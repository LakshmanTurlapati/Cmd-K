# Feature Landscape

**Domain:** Linux platform support + smart terminal context for cross-platform AI terminal overlay
**Researched:** 2026-03-13

## Table Stakes

Features users expect. Missing = product feels incomplete.

### Linux Overlay (System-Wide Hotkey + Floating Window)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| System-wide Ctrl+K hotkey on X11 | macOS/Windows already have it; Linux parity is the milestone goal | Low | Tauri's `global-shortcut` plugin works on X11 out of the box. Same plugin API, no platform-specific code needed. |
| System-wide Ctrl+K hotkey on Wayland | Wayland is the default on Ubuntu 22.04+, Fedora 38+, most modern distros | Med | Tauri's global-shortcut plugin works on Wayland via compositor-specific protocols. May require portal-based shortcut registration on GNOME (GlobalShortcuts portal via xdg-desktop-portal). Needs testing per compositor. |
| Always-on-top overlay on X11 | Core product behavior -- overlay MUST float above active window | Low | `always_on_top(true)` works on X11. WM treats it as a "hint" but all major WMs (GNOME/Mutter, KDE/KWin, i3, Sway on XWayland) honor it. |
| Always-on-top overlay on Wayland | Wayland is increasingly the default display server | High | **Known Tauri limitation**: `always_on_top` does NOT work on native Wayland (tao issue #1134, tauri issue #3117). Workarounds: (1) Force XWayland via `GDK_BACKEND=x11` env var, (2) Use `wlr-layer-shell` protocol (requires compositor support, not in Tauri), (3) Accept that on some Wayland compositors the window may not stay on top. XWayland fallback is the pragmatic first approach. |
| Overlay dismissal with Escape | Consistent with macOS/Windows behavior | Low | Pure frontend behavior, already implemented. No platform-specific work. |
| Frosted glass / vibrancy effect | Differentiator on macOS, users expect visual polish | Med | No native vibrancy API on Linux (no NSVisualEffectView equivalent). Use CSS `backdrop-filter: blur()` with a semi-transparent background. GTK4 compositing supports this on most compositors. Visually close enough. |
| Overlay positioning over active window | Core UX -- overlay should appear where the user is working | Med | On X11: `xdotool getactivewindow getwindowgeometry` gets position and size of the active window before overlay appears. On Wayland: no standard API to query other windows' geometry. Fallback: center on primary monitor or use cursor position. |
| Tray icon / background daemon | Users expect to see the app is running | Low | Tauri's system tray plugin works on Linux (uses libappindicator/StatusNotifierItem on modern desktops). Already implemented cross-platform. |

### Linux Terminal Context (CWD, Shell Type, Output)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Detect focused terminal app | Must know which app was active to provide context | Med | X11: `xdotool getactivewindow getwindowpid` gives PID of focused window. Then `/proc/PID/exe` resolves to terminal binary. Wayland: compositor-specific -- GNOME D-Bus `org.gnome.Shell` (restricted since GNOME 41), KDE KWin scripting, or `wlr-foreign-toplevel-management` protocol. Fallback: track which window lost focus when overlay gained it. |
| Read CWD of foreground shell | 1:1 macOS parity -- CWD is core context for AI command generation | Low | `/proc/PID/cwd` is a symlink to the process CWD. `readlink("/proc/{shell_pid}/cwd")` in Rust via `std::fs::read_link`. Trivially simple compared to macOS libproc FFI. No permissions issues for same-user processes. |
| Detect shell type (bash/zsh/fish/etc.) | Must tell AI which shell syntax to use | Low | `/proc/PID/exe` is a symlink to the shell binary path (e.g., `/usr/bin/zsh`). Extract basename. Simpler than macOS `proc_pidpath` FFI. |
| Walk process tree to find shell child | Terminal emulators spawn shells as children; need to find the right one | Med | `/proc/PID/stat` field 4 (ppid) lets you walk the tree. Or scan all `/proc/*/stat` entries for matching ppid. Equivalent to macOS `proc_listchildpids`. Handle tmux/screen multiplexers by walking deeper. Pattern: terminal -> login/bash -> user_shell. |
| Detect running process inside shell | Show "node", "python", etc. when a command is running | Low | Same process tree walk as macOS. Find deepest child of the shell PID, check if it is a known shell (if not, it is the running process). |
| Read visible terminal output | Context for AI to understand what the user is looking at | High | **This is the hardest Linux feature.** No unified API like macOS Accessibility or Windows UIA. Options per terminal: (1) kitty: `kitty @ get-text` remote control (requires `allow_remote_control` in kitty.conf), (2) WezTerm: `wezterm cli get-text` (works out of box), (3) GNOME Terminal/xterm/others: AT-SPI2 over D-Bus (accessibility API, equivalent to macOS AX), (4) Alacritty: no remote API at all. AT-SPI2 is the universal fallback but quality varies by terminal. |
| Detect IDE integrated terminals | VS Code, Cursor have embedded terminals | Med | Same pattern as macOS/Windows. Detect process name (`code`, `cursor`), walk process tree for shell children. Linux process tree is easier to traverse than Windows ConPTY. |

### Linux Paste (Inject Command into Terminal)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Clipboard write | Must place generated command on clipboard | Low | X11: `xclip -selection clipboard` or `xsel --clipboard`. Wayland: `wl-copy`. Rust crate `arboard` handles both transparently (already used for Windows). |
| Paste into terminal (X11) | 1:1 macOS parity -- user expects command to appear in terminal | Med | Strategy: (1) Write to clipboard via arboard, (2) Activate terminal window via `xdotool windowactivate {window_id}`, (3) Simulate Ctrl+Shift+V via `xdotool key ctrl+shift+v` (Linux terminals use Ctrl+Shift+V, not Ctrl+V). Alternative: `xdotool type --clearmodifiers "{text}"` to type text directly (avoids clipboard, but slow for long text and special chars can cause issues). |
| Paste into terminal (Wayland) | Wayland is default on modern distros | High | Wayland's security model restricts synthetic input to other windows. Options: (1) `wtype` for keyboard simulation (wlroots compositors only, NOT GNOME), (2) `ydotool` uses kernel uinput (works everywhere but needs ydotoold daemon, may need root), (3) `wl-copy` + rely on user pressing Ctrl+Shift+V manually. Most reliable approach: clipboard write + focus restore + show "Press Ctrl+Shift+V to paste" hint if synthetic paste fails. |
| Clear current line before paste | Ctrl+U to clear line (same as macOS behavior) | Low | `xdotool key ctrl+u` on X11. Wayland: same tools as paste (wtype/ydotool). |
| Focus restore to terminal | After overlay dismissal, terminal must regain focus | Med | X11: `xdotool windowactivate {window_id}` (window_id captured before overlay). Wayland: no standard "activate other window" API. Hiding the overlay causes the previous window to naturally regain focus on most compositors. |
| Confirm command (Enter) | User presses Enter to execute the pasted command | Low | Same mechanism as paste. `xdotool key Return` on X11. On Wayland, same limitations as paste apply. |

### Smart Terminal Context (Scrollback + Intelligent Truncation) -- Cross-Platform

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Read full visible terminal text | Current behavior -- read what is on screen | Already exists for macOS/Windows | On Linux, this is the per-terminal-emulator text reading described above. |
| Read scrollback buffer beyond visible area | Users run long commands (build logs, test output) and expect AI to understand recent output | High | No universal API. Terminal-specific: kitty `get-text --extent all`, WezTerm `cli get-text --start-line -N`, GNOME Terminal via AT-SPI2 text interface. Alacritty has no programmatic scrollback access at all. Fallback: shell HISTFILE for command history (not output). |
| Intelligent truncation to fit context window | Sending 100K tokens of scrollback wastes money and degrades AI quality ("context rot") | Med | Must truncate scrollback to fit within AI model's context window minus prompt/response overhead. Strategy: keep last N lines (most recent output is most relevant), preserve the first few lines (often contain the command that produced the output), drop middle content. |
| Token-aware context budgeting | Different AI providers have different context windows (4K to 200K tokens) | Med | Already have model metadata with context window sizes. Budget ~10-15% of context window for terminal context. Rough heuristic: 1 token ~= 4 chars. For a 4K context model, ~1600 chars of terminal text. For 128K+ models, cap at ~8000 chars regardless (diminishing returns, terminal context beyond recent output rarely helps). |
| Preserve command prompts in truncation | Prompt lines (`user@host:~$`) show what commands were run -- critical context for AI | Med | When truncating, scan for prompt patterns (same regex patterns already used in WSL detection: `user@host:/path$`, `PS C:\>`, `$`, `#`) and preserve lines matching prompt patterns. This keeps the "what did the user do" narrative intact even when output between commands is trimmed. |
| Strip ANSI escape codes from captured text | Raw terminal output contains color codes, cursor movement, and other escape sequences | Low | Regex strip `\x1b\[[0-9;]*[a-zA-Z]` and OSC sequences. Use the `strip-ansi-escapes` crate. Already partially handled by the sensitive data filter on macOS/Windows. |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Zero-config terminal detection on Linux | Most Linux terminal tools require shell plugin installation (oh-my-zsh plugins, starship config, shell integration scripts). CMD+K reads /proc directly -- no .bashrc modification needed. This is the core product differentiator. | Low | Direct carry-over of zero-setup philosophy from macOS. Linux /proc makes it EASIER than macOS (symlinks vs raw FFI). Major advantage vs Fig, Warp, and others that require shell integration. |
| Multi-terminal-emulator support | Support GNOME Terminal, Konsole, kitty, Alacritty, WezTerm, Terminator, xterm, Tilix, foot, st -- not just one | High | CWD/shell/process detection works universally via /proc. Text reading is the variable -- each terminal may need a different strategy. Prioritize by popularity: GNOME Terminal > kitty > Alacritty > WezTerm > Konsole > others. |
| Automatic X11/Wayland detection | App works regardless of display server without user configuration | Med | Check `$XDG_SESSION_TYPE` env var (`x11` or `wayland`) or `$WAYLAND_DISPLAY` (set when running on Wayland). Route to X11 or Wayland code paths accordingly. Tauri/GTK already handles window creation internally, but paste/focus code needs explicit routing. |
| Smart context with command-output pairing | When truncating scrollback, pair each command prompt with its output block so the AI sees complete command-result pairs, not arbitrary line cuts | High | Parse scrollback into segments: [prompt + command] -> [output block]. Truncate by dropping oldest complete segments first. Preserves semantic coherence. This is meaningfully better than naive "last N lines" truncation and no existing tool does this well. |
| Scrollback error prioritization | When truncating, prioritize keeping lines containing error patterns (stderr markers, stack traces, "error:", "failed", exit codes) over routine output | Med | Scan captured text for error indicators. When budget is tight, keep: (1) last prompt + command, (2) error lines, (3) first/last few lines of output. Drop "passing" test output, verbose logs, progress bars. |
| Linux AppImage with auto-update | Single portable binary that updates itself, like macOS DMG + auto-updater | Med | Tauri v2 supports AppImage bundling natively. Auto-updater works with AppImage via the same `tauri-plugin-updater` used for macOS/Windows. CI/CD pipeline needs a Linux runner added to the existing 3-job architecture. |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Shell plugin / .bashrc integration | Violates core "zero setup" philosophy. Users hate modifying shell configs. Every competitor requires this. Our differentiator is NOT needing it. | Use /proc filesystem for CWD/shell detection. Use AT-SPI2/terminal remote control for text reading. Accept that some terminals (Alacritty) may have limited text reading without plugins. |
| PTY/pseudo-terminal interception | Intercepting the PTY fd to read terminal I/O would give perfect output capture but is extremely invasive, requires ptrace or LD_PRELOAD, breaks with sudo/su, and is a security concern. | Use terminal-emulator APIs (remote control) or accessibility APIs (AT-SPI2). Accept imperfect coverage over invasive approaches. |
| Auto-execution of commands | Never run commands directly. Always paste and let user press Enter. This is a safety invariant of the entire product. | Paste to terminal + provide confirm button that sends Enter keystroke. User always has the chance to review before execution. |
| Wayland-native layer-shell integration | Building a custom layer-shell client would bypass Tauri's window management entirely. Massive complexity, requires compositor-specific code, and marginal benefit over XWayland fallback. | Use XWayland fallback (`GDK_BACKEND=x11`) for Wayland sessions initially. Document the env var. Revisit when Tauri upstream adds native Wayland overlay support. |
| Universal scrollback via LD_PRELOAD | Injecting a shared library to intercept write() syscalls on the PTY would capture all terminal output universally. Too invasive, fragile across distros, and a security concern. | Use per-terminal remote control APIs where available. Fall back to AT-SPI2. Accept that Alacritty users get CWD/shell context but not scrollback text. |
| Snap/Flatpak distribution (initial release) | Sandboxing restrictions in Snap/Flatpak conflict with the app's need to read /proc, access clipboard tools (xclip, wl-copy), and interact with terminal emulators via D-Bus. Would require extensive permission manifests and testing. | Ship AppImage first (no sandbox restrictions). Consider Snap/Flatpak later with appropriate portal/permission configurations. |
| Full tmux/screen scrollback extraction | Extracting scrollback from within tmux panes requires `tmux capture-pane -p` which only works if the user is running tmux. Adding tmux-specific integration adds complexity for a niche use case. | Detect tmux in the process tree (already handled by multiplexer detection in process.rs). Use the outer terminal's text reading for visible content. If tmux is detected, note it in context so AI knows commands may need `tmux` prefix. |
| D-Bus GNOME Shell extension dependency | Requiring users to install a GNOME Shell extension for focused window detection is unacceptable UX friction. | Use xdotool on X11/XWayland (covers most cases). On native Wayland without XWayland, accept limited window identification and use process-based fallbacks. |

## Feature Dependencies

```
System-wide hotkey --> Overlay window (hotkey triggers overlay)
Overlay window --> Focus capture (must know previous window before showing overlay)
Focus capture --> Terminal detection (need PID to identify terminal)
Terminal detection --> Process tree walk (find shell child of terminal)
Process tree walk --> CWD reading (/proc/{shell_pid}/cwd)
Process tree walk --> Shell type detection (/proc/{shell_pid}/exe)
Terminal detection --> Text reading (AT-SPI2 / remote control / per-terminal API)
Text reading --> ANSI escape stripping (clean raw text)
ANSI escape stripping --> Smart truncation (truncate cleaned text)
Smart truncation --> AI prompt construction (include truncated context in system prompt)
Clipboard write --> Paste to terminal (write first, then simulate Ctrl+Shift+V)
Focus capture --> Focus restore (restore focus to captured window after overlay hides)
Focus restore --> Paste to terminal (must focus terminal before pasting)
X11/Wayland detection --> Paste method selection (xdotool vs wtype/ydotool)
X11/Wayland detection --> Focus capture method selection (xdotool vs compositor fallback)
X11/Wayland detection --> Overlay behavior (always-on-top vs XWayland fallback)
```

## MVP Recommendation

### Phase 1: Linux Core (X11-first)

Prioritize these -- they establish basic Linux parity with macOS:
1. **X11 hotkey + overlay** -- Tauri plugin handles this with zero platform-specific code
2. **Process detection via /proc** -- CWD, shell type, running process via symlink reads. Easiest of all three platforms.
3. **X11 focus capture** -- `xdotool getactivewindow` before overlay show, store window ID in AppState
4. **X11 paste via xdotool** -- Clipboard write (arboard) + `xdotool windowactivate` + `xdotool key ctrl+shift+v`
5. **AppImage distribution** -- Add Linux runner to CI/CD pipeline, Tauri bundles AppImage natively

### Phase 2: Terminal Text Reading

6. **AT-SPI2 text reading** -- Universal fallback for GNOME Terminal, xterm, Konsole, and other accessible terminals via D-Bus
7. **kitty remote control** -- `kitty @ get-text` for kitty users (second most popular terminal among power users)
8. **WezTerm CLI** -- `wezterm cli get-text` for WezTerm users

### Phase 3: Wayland Support

9. **Wayland detection** -- Check `$XDG_SESSION_TYPE` / `$WAYLAND_DISPLAY`, set `GDK_BACKEND=x11` for XWayland fallback
10. **Wayland clipboard** -- `wl-copy` via arboard (arboard handles this automatically)
11. **Wayland paste fallback** -- Try `ydotool` or `wtype`, gracefully degrade to "press Ctrl+Shift+V" hint

### Phase 4: Smart Context (Cross-Platform)

12. **ANSI escape stripping** -- Clean captured text from all platforms before sending to AI
13. **Token-aware truncation** -- Budget terminal context to ~10-15% of model context window, cap at ~8000 chars
14. **Command-output pairing** -- Parse prompts to create semantic segments for intelligent truncation
15. **Scrollback reading** -- Use terminal-specific APIs (kitty, WezTerm) to read beyond visible area on Linux; extend macOS AX and Windows UIA to request more text

Defer:
- **Wayland layer-shell**: Wait for Tauri upstream. XWayland covers all practical cases today.
- **Snap/Flatpak**: Sandbox conflicts. AppImage first.
- **tmux scrollback**: Detect tmux, note in context, but do not extract pane content.
- **Error prioritization in truncation**: Ship basic "last N lines" first, iterate with error-aware truncation later.

## Complexity Estimates: Linux vs Existing Platforms

| Feature Area | macOS Complexity | Windows Complexity | Linux Complexity | Notes |
|--------------|-----------------|-------------------|-----------------|-------|
| CWD detection | High (libproc FFI, raw C struct parsing) | High (PEB reading via ReadProcessMemory) | **Low** (`readlink /proc/PID/cwd`) | Linux is trivially the easiest platform |
| Shell detection | Med (proc_pidpath FFI) | Med (QueryFullProcessImageName) | **Low** (`readlink /proc/PID/exe`, basename) | Linux is simplest |
| Process tree walk | Med (proc_listchildpids FFI) | High (CreateToolhelp32Snapshot + ConPTY) | **Med** (scan /proc/*/stat for ppid) | Similar to macOS, much simpler than Windows |
| Visible text reading | High (Accessibility API, per-terminal AX) | High (UIA, 3-strategy cascade) | **High** (AT-SPI2 + per-terminal remote control) | Most fragmented on Linux due to terminal diversity |
| Paste into terminal | Med (AppleScript + CGEventPost) | Med (SendInput Ctrl+V) | **Med-High** (xdotool X11, much harder on Wayland) | Wayland's security model restricts synthetic input |
| Focus capture/restore | Low (NSWorkspace ObjC FFI) | Low (GetForegroundWindow/SetForegroundWindow) | **Med** (xdotool X11, compositor-specific Wayland) | Wayland is the complication |
| Overlay window | Low (NSPanel floating) | Med (WS_EX_TOPMOST + z-order) | **Med** (works X11, broken native Wayland without workaround) | XWayland fallback mitigates |
| Smart truncation | N/A (new feature) | N/A (new feature) | N/A (new feature) | **Med** -- Cross-platform Rust code, no platform-specific work |
| Scrollback reading | N/A (not yet built) | N/A (not yet built) | **High** (per-terminal APIs) | New feature, primarily benefits from Linux terminal APIs |

## Known Terminal Emulators: Linux Detection + Text Reading Matrix

| Terminal | Process Name | Text Reading Method | Scrollback Access | Popularity |
|----------|-------------|--------------------|--------------------|------------|
| GNOME Terminal | `gnome-terminal-server` | AT-SPI2 (good support) | AT-SPI2 Text interface | Very High (default on Ubuntu, Fedora) |
| Konsole | `konsole` | AT-SPI2 (good support) | AT-SPI2 Text interface | High (default on KDE) |
| kitty | `kitty` | `kitty @ get-text` (remote control) | `kitty @ get-text --extent all` | High (popular with power users) |
| Alacritty | `alacritty` | **None** (GPU-rendered, no API) | **None** | High (minimalists) |
| WezTerm | `wezterm-gui` | `wezterm cli get-text` | `wezterm cli get-text --start-line -N` | Medium |
| xterm | `xterm` | AT-SPI2 (basic support) | Limited | Low (legacy) |
| Terminator | `terminator` | AT-SPI2 (VTE-based) | AT-SPI2 Text interface | Medium |
| Tilix | `tilix` | AT-SPI2 (VTE-based) | AT-SPI2 Text interface | Medium |
| foot | `foot` | AT-SPI2 (if enabled) | Unknown | Low-Medium (Wayland-native) |
| st (suckless) | `st` | **None** (minimal, no accessibility) | **None** | Low (tinkerers) |
| VS Code terminal | `code` | AT-SPI2 (xterm.js accessibility) | Limited | Very High (IDE) |
| Cursor terminal | `cursor` | AT-SPI2 (xterm.js accessibility) | Limited | Medium (IDE) |

## Sources

- [Tauri global-shortcut plugin docs](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri AppImage distribution docs](https://v2.tauri.app/distribute/appimage/)
- [tao issue #1134 - Wayland always_on_top not working](https://github.com/tauri-apps/tao/issues/1134)
- [tauri issue #3117 - Always on top not working on Wayland](https://github.com/tauri-apps/tauri/issues/3117)
- [kitty remote control protocol](https://sw.kovidgoyal.net/kitty/remote-control/)
- [WezTerm CLI get-text](https://wezterm.org/cli/cli/get-text.html)
- [WezTerm scrollback docs](https://wezterm.org/scrollback.html)
- [AT-SPI2 accessibility protocol](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/)
- [Linux /proc filesystem man page](https://man7.org/linux/man-pages/man5/proc.5.html)
- [xdotool man page](https://man.archlinux.org/man/xdotool.1.en)
- [ydotool - X11/Wayland automation](https://github.com/ReimuNotMoe/ydotool)
- [wtype - Wayland keyboard simulation](https://www.linuxlinks.com/wtype-xdotool-type-wayland/)
- [xdotool paste approach](https://sick.codes/paste-clipboard-linux-xdotool-ctrl-v-terminal-type/)
- [GNOME window-calls extension](https://github.com/ickyicky/window-calls)
- [Context window management strategies](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/)
- [Context windows - Claude API Docs](https://platform.claude.com/docs/en/build-with-claude/context-windows)
- [Baeldung - Find CWD of running process](https://www.baeldung.com/linux/find-working-directory-of-running-process)
- [wlr-layer-shell protocol](https://wayland.app/protocols/wlr-layer-shell-unstable-v1)
