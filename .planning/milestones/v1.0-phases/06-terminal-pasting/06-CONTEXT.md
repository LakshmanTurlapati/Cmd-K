# Phase 6: Terminal Pasting - Context

**Gathered:** 2026-02-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Auto-paste generated commands into the user's active terminal input line. Safe commands paste automatically; destructive commands (flagged by Phase 5 safety layer) do not auto-paste. Fallback to silent clipboard copy when the terminal doesn't support auto-paste. This phase does not add new command generation or safety detection -- it wires the paste delivery mechanism.

</domain>

<decisions>
## Implementation Decisions

### Paste trigger flow
- Safe commands auto-paste immediately on generation -- no user action required
- Destructive commands (those with destructive badge from Phase 5) do NOT auto-paste -- user copies manually via existing copy button
- Linked to safety toggle in Settings (from Phase 2):
  - Toggle ON (default, ships this way): auto-paste safe commands, skip destructive
  - Toggle OFF: no auto-paste for any command
  - Warn user when they attempt to toggle OFF

### Terminal focus behavior
- Overlay stays open after auto-paste (does not dismiss)
- Focus shifts to terminal after paste -- terminal comes to foreground
- No visual paste confirmation indicator in the overlay -- the command appearing in terminal is confirmation enough
- Overlay preserves state when user brings it back to focus -- same command visible, can re-paste

### Fallback experience
- When auto-paste is unavailable (unsupported terminal): silently auto-copy to clipboard, no notification
- Best-effort terminal detection: try to paste to whatever terminal is active, fall back on failure
- Reuse terminal detection from Phase 3 (Terminal Context Reading) rather than re-detecting at paste time

### Command placement
- Never auto-execute -- always place command in input line, user presses Enter
- Replace any existing text in terminal input line (clear and paste)
- Only single-line commands (with pipes) -- multi-line is not applicable
- Paste to most recently active terminal window (not a specific tab/pane)

### Claude's Discretion
- AppleScript implementation details for Terminal.app and iTerm2
- Exact mechanism for clearing existing input line text
- How to handle edge cases where terminal detection from Phase 3 is stale

</decisions>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 06-terminal-pasting*
*Context gathered: 2026-02-23*
