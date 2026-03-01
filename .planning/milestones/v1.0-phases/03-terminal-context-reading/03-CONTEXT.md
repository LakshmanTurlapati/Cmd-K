# Phase 3: Terminal Context Reading - Context

**Gathered:** 2026-02-21
**Updated:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect the active app's context when the overlay opens. For terminals: working directory, shell type, visible output, and running process. For browsers: detect DevTools console state. For all apps: resolve a display name for the badge. Context is captured on-demand and passed to AI. This phase does NOT generate commands or paste into terminals -- it only reads context.

</domain>

<decisions>
## Implementation Decisions

### Detection Scope
- Support all 5 terminals equally: Terminal.app, iTerm2, Alacritty, kitty, WezTerm
- Detect through tmux and screen sessions running within those terminals
- Detect the frontmost terminal only (whichever was last active before overlay opened)
- For iTerm2 with multiple tabs/panes, detect the active tab/pane specifically
- Detection is on-demand only -- triggered when Cmd+K is pressed, no background polling
- Detect shell type (bash, zsh, fish, etc.) in addition to CWD and output
- Detect running process (e.g., 'node server.js', 'python script.py')
- Also detect shells inside code editors (VS Code, Cursor) via process tree ancestry
- Silent fallback for unsupported/unknown terminals -- no user notification

### App-Aware Badges
- Every frontmost app gets a badge, not just terminals
- Badge text: resolve display name from macOS, then clean/shorten it (e.g., 'Chrome' not 'Google Chrome', 'Code' not 'Visual Studio Code')
- App name is stored and passed to AI as context (AI knows you're in Chrome/Figma/etc.)
- When a shell is detected inside an app (e.g., Cursor's integrated terminal), show shell badge only ('zsh'), not the app name
- When no shell detected, show the cleaned app name as the badge
- Badge priority: shell type > console > app name

### Browser Console Detection
- Supported browsers: Chrome, Safari, Firefox, Arc, Edge, Brave
- Detect whether DevTools/Console is open via Accessibility API
- When console IS detected: badge shows just 'Console' (drop the browser name)
- When console is NOT detected: badge shows the browser name ('Chrome', 'Safari', etc.)
- Same 'Console' badge for all browsers -- no per-browser distinction when console is active
- Chromium-based browsers (Arc, Edge, Brave) show their own name, not 'Chrome'
- Safari's Web Inspector treated same as Chrome DevTools for console detection
- Read visible console output but keep only the very last line (most recent error/log) -- avoid context bloat
- Last console line is passed to AI as context
- No URL or page title reading -- just app name and console state

### Context Capture Depth
- Capture visible screen content only (no scrollback)
- Include command prompts and typed commands along with output
- Structured parsing: identify individual commands and their outputs as separate entries
- Filter sensitive data (API keys, passwords, tokens) before sending to AI
- Use Accessibility tree (AXUIElement) as primary read method, with per-terminal fallbacks
- Hard timeout on detection -- overlay opens with whatever was captured within the time limit
- Do NOT read directory listings or git context -- just CWD path, visible output, shell type, and running process

### Overlay Integration
- Show detected shell type (e.g., 'zsh') OR app name OR 'Console' as subtle text below the input field
- Label appears to the left in monospace style
- CWD, terminal output, running process, console last line are captured internally for AI but NOT displayed in the overlay
- Subtle spinner during detection; if detection times out or fails, hide the badge area entirely
- Overlay height adjusts naturally based on whether a badge is shown
- No manual override of detected context
- AI works regardless of whether context was captured

### Fallback Behavior
- Partial detection: use whatever was captured (partial context better than none)
- Accessibility permission denied: persistent banner in overlay saying 'Enable Accessibility for terminal context' with click action that opens macOS System Settings directly
- Banner is always visible until Accessibility permission is granted (not dismissable)
- Permission re-checked each time overlay opens (banner disappears once granted)
- Debug logging for detection failures (internal, not user-facing)
- AI still works without any terminal context for general command questions

### Claude's Discretion
- Exact hard timeout duration for detection
- Accessibility tree traversal strategy per terminal and browser
- Structured parsing heuristics for identifying commands vs output
- Sensitive data pattern matching implementation
- Spinner design and animation details
- Exact bundle ID to display name mapping table
- Best-effort approach for Safari Web Inspector AX detection

</decisions>

<specifics>
## Specific Ideas

- Console badge is just 'Console' regardless of which browser -- keeps it clean
- Browser console output: only the very last line to avoid context window bloat
- Shell always wins over app name -- if zsh is found in Cursor, show 'zsh' not 'Cursor'

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 03-terminal-context-reading*
*Context gathered: 2026-02-21*
*Context updated: 2026-02-22*
