# Phase 5: Safety Layer - Context

**Gathered:** 2026-02-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Detect destructive commands in AI-generated output and display a visual warning badge. Users are informed but not blocked -- copy behavior is unchanged. This phase does NOT add syntax highlighting, command editing, or new interaction patterns beyond the warning badge.

</domain>

<decisions>
## Implementation Decisions

### Warning presentation
- Red "Destructive" badge positioned on the right side of the results header, next to the terminal detection badge
- Badge only -- no changes to command text appearance (no tint, no background change)
- Badge appears after command generation is complete (not during streaming)
- Subtle fade-in animation (~200ms) when badge appears
- Badge is dismissable -- user can click it to hide for that command
- Tooltip on hover shows AI-generated plain-language explanation of what the command does

### Confirmation flow
- No confirmation required -- copy shortcut works the same for destructive and non-destructive commands
- No extra behavior after copying a destructive command (no toast, no flash)
- Badge serves as informational warning only, not a gate
- Update roadmap success criteria: change "User must explicitly confirm" to "Warning badge informs user of destructive commands"

### Detection scope & sensitivity
- Curated pattern list for detection (not AI-assisted classification)
- Comprehensive list covering:
  - File destruction: rm -rf, rm -r, shred, unlink, rmdir with contents
  - Git force operations: git push --force, git reset --hard, git clean -fd, git branch -D
  - Database mutations: DROP TABLE, DROP DATABASE, TRUNCATE, DELETE without WHERE
  - System/permission changes: chmod 777, chown, sudo rm, mkfs, dd if=, shutdown, reboot
  - Any other clearly destructive patterns identified during research
- On/off toggle only -- no sensitivity levels
- Toggle lives in the Settings panel (not tray menu)

### Plain-language explanations
- AI-generated per command via embedded system prompt (not pattern-based templates)
- Explanation is command-specific (e.g., "Recursively deletes all files in /home/user/projects")
- Explanation appears in the badge tooltip on hover
- Eager loading: separate API call fires as soon as destructive command is detected
- If user hovers before explanation loads, tooltip shows a spinner, then swaps to text when ready

### Claude's Discretion
- Exact curated pattern list completeness (researcher can expand during investigation)
- Tooltip styling and spinner design
- API call structure for the explanation request
- Badge dismiss interaction details (click vs X button)

</decisions>

<specifics>
## Specific Ideas

- Badge style mirrors the shell-type badge (zsh/bash) but in red, positioned to its right in the results header
- The AI explanation prompt should be embedded in the existing system prompt so the model can provide context-aware explanations

</specifics>

<deferred>
## Deferred Ideas

- Syntax highlighting for generated commands (Atom IDE-style) -- applies to all commands, not safety-specific. Add to backlog as a UI enhancement phase or standalone task.

</deferred>

---

*Phase: 05-safety-layer*
*Context gathered: 2026-02-23*
