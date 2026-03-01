# Phase 10: AI Follow-up Context Per Window - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Per-window AI conversation history (turnHistory) that persists across overlay open/close cycles within the same app session. The AI sees prior exchanges for the active terminal window and can do follow-ups. Terminal context (CWD, shell, output) is included only in the first user message of a session to prevent token bloat.

</domain>

<decisions>
## Implementation Decisions

### Conversation scope
- Keep last 7 turns per window (1 turn = 1 user query + 1 AI response pair)
- History persists across overlay open/close but clears on app restart (in-memory only)
- Turn limit configurable via slider in preferences: range 5-50, default 7

### Follow-up detection
- Every query on the same window is always a follow-up -- no intent detection
- Always send full history (up to the turn limit) regardless of topic relevance
- No manual "start fresh" mechanism -- history flows until app restart
- No explicit follow-up signaling to the AI -- history is silently included

### Prompt construction
- Use native AI API message array format (alternating user/assistant roles)
- Store both user query and full AI response per turn (no trimming or summarizing)
- Terminal context (CWD, shell, recent output) handling in first vs follow-up: Claude's discretion

### History visibility
- No visual indicator in the overlay that AI has prior context -- it just works silently
- No separate conversation/chat log view -- arrow-up recall (Phase 9) serves as user-facing history
- Preferences panel: turn limit slider (5-50, default 7) + "Clear conversation history" button
- Clear button clears history for all windows at once

### Claude's Discretion
- Terminal context strategy for follow-up messages (omit entirely vs send minimal updates)
- Internal data structure for storing turn history per window
- How to handle edge cases (window ID changes, stale history)

</decisions>

<specifics>
## Specific Ideas

- Arrow key history (Phase 9) and AI conversation context (Phase 10) complement each other: arrow keys let users recall what they typed, AI context lets the model understand the conversation flow
- The feature should be invisible to the user -- no UI changes beyond the settings slider and clear button. The AI just naturally responds with awareness of prior exchanges.

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 10-ai-follow-up-context-per-window*
*Context gathered: 2026-03-01*
