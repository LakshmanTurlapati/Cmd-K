# Phase 32: Linux Paste - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Complete the Ctrl+K workflow on Linux — after the AI generates a command, paste it into the active terminal. Covers X11 auto-paste via xdotool, Wayland graceful fallback with clipboard + user hint, and confirm (Enter) command. Destructive command detection already works for Linux commands (existing patterns).

</domain>

<decisions>
## Implementation Decisions

### X11 paste method
- Clipboard + Ctrl+Shift+V approach (not xdotool type)
- Write to clipboard via xclip (`xclip -selection clipboard`)
- Clipboard selection only (not primary/middle-click selection)
- xdotool sends `key ctrl+shift+v` to simulate terminal paste shortcut
- After paste, overlay stays visible and refocuses (same as macOS/Windows)

### Line clearing and confirm
- Clear current shell line with `xdotool key ctrl+u` before pasting (matches macOS/Windows Ctrl+U pattern)
- 50ms delay between Ctrl+U and Ctrl+Shift+V for shell processing
- Confirm command uses `xdotool key Return` (same pattern as macOS CGEventPost Return, Windows SendInput Return)

### Wayland detection and fallback
- Detect Wayland via `WAYLAND_DISPLAY` env var + `XDG_SESSION_TYPE=wayland`
- If `GDK_BACKEND=x11` is set, treat as X11 (XWayland path) even on Wayland
- Wayland fallback: copy to clipboard, show inline hint in overlay "press Ctrl+Shift+V to paste"
- Wayland clipboard: try `wl-copy` first, fall back to xclip (via XWayland)
- Hint persists until user dismisses overlay (Escape or Ctrl+K)
- Wayland confirm fallback: show "press Enter" hint (same inline pattern)

### Tool availability and fallback chain
- Check xdotool and xclip/wl-copy availability once at app startup, cache results
- If xdotool missing on X11: graceful fallback to clipboard + hint (same as Wayland path)
- If xclip/wl-copy both missing: use Tauri clipboard API as final fallback
- No hard errors — app always has a working paste path (worst case: clipboard + manual paste hint)

### Claude's Discretion
- Exact xdotool window activation approach (windowactivate vs windowfocus)
- Focus restoration timing and delays
- How to structure the Linux cfg block in paste.rs (single function vs helper modules)
- Error logging granularity

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `paste.rs`: Full macOS and Windows implementations — Linux follows same `#[cfg(target_os = "linux")]` pattern
- `write_to_clipboard`: Existing stub at line 162 — replace with xclip implementation
- `paste_to_terminal`: Existing stub at line 217 — replace with xdotool implementation
- `confirm_terminal_command`: Existing stub at line 626 — replace with xdotool Return
- `AppState.previous_app_pid`: Already populated by Phase 31's x11rb PID capture
- `hotkey.rs`: `get_active_window_pid()` via x11rb — provides the PID for focus restoration

### Established Patterns
- macOS: activate terminal (osascript) → Ctrl+U (CGEventPost) → paste → re-acquire key window
- Windows: restore focus (SetForegroundWindow) → clipboard (arboard) → Ctrl+V (SendInput)
- Linux should follow: activate terminal (xdotool) → Ctrl+U (xdotool) → clipboard (xclip) → Ctrl+Shift+V (xdotool)
- Platform detection: `#[cfg(target_os = "linux")]` blocks at function level, not module level

### Integration Points
- `paste_to_terminal` Tauri command: Frontend calls this — Linux path needs to work with same API contract
- `confirm_terminal_command` Tauri command: Same — Linux path returns Ok/Err like other platforms
- `AppState`: May need Linux-specific fields (e.g., cached tool availability flags)
- Frontend `Overlay.tsx`: Needs to handle "clipboard + hint" response from backend for Wayland/fallback paths

</code_context>

<specifics>
## Specific Ideas

- Fallback chain should be seamless: xdotool auto-paste > clipboard + hint > Tauri clipboard + hint
- The inline hint in overlay should feel like a natural part of the UI, not an error state
- "Copied to clipboard — press Ctrl+Shift+V to paste" is the expected hint text

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 32-linux-paste*
*Context gathered: 2026-03-14*
