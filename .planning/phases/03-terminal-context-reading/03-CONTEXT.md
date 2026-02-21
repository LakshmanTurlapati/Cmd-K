# Phase 3: Terminal Context Reading - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect the active terminal's working directory, recent visible output, shell type, and running process without requiring shell plugins or configuration. Uses macOS Accessibility API. Context is captured on-demand when the overlay opens and passed to the AI for command generation. This phase does NOT generate commands or paste into terminals -- it only reads context.

</domain>

<decisions>
## Implementation Decisions

### Detection Scope
- Support all 5 terminals equally: Terminal.app, iTerm2, Alacritty, kitty, WezTerm
- Detect through tmux and screen sessions running within those terminals
- Detect the frontmost terminal only (whichever was last active before overlay opened)
- For iTerm2 with multiple tabs/panes, detect the active tab/pane specifically
- If the frontmost app is not a terminal, provide no context (overlay opens with empty context, AI still works)
- Detection is on-demand only -- triggered when Cmd+K is pressed, no background polling
- Detect shell type (bash, zsh, fish, etc.) in addition to CWD and output
- Detect running process (e.g., 'node server.js', 'python script.py')
- Silent fallback for unsupported/unknown terminals -- no user notification

### Context Capture Depth
- Capture visible screen content only (no scrollback)
- Include command prompts and typed commands along with output
- Structured parsing: identify individual commands and their outputs as separate entries
- Filter sensitive data (API keys, passwords, tokens) before sending to AI
- Use Accessibility tree (AXUIElement) as primary read method, with per-terminal fallbacks
- Hard timeout on detection -- overlay opens with whatever was captured within the time limit
- Do NOT read directory listings or git context -- just CWD path, visible output, shell type, and running process

### Overlay Integration
- Show only the detected shell type (e.g., 'zsh') as subtle text below the input field
- Shell type label appears to the left: `zsh` (no CWD path shown to user)
- CWD, terminal output, running process are captured internally for AI but NOT displayed in the overlay
- Subtle spinner during detection; if detection times out or fails, hide the shell area entirely
- When no context is available (non-terminal app), hide the context area completely -- no placeholder
- Overlay height adjusts: slightly taller when shell type is shown, shorter without it
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
- Accessibility tree traversal strategy per terminal
- Structured parsing heuristics for identifying commands vs output
- Sensitive data pattern matching implementation
- Spinner design and animation details

</decisions>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches for Accessibility API integration and terminal detection.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 03-terminal-context-reading*
*Context gathered: 2026-02-21*
