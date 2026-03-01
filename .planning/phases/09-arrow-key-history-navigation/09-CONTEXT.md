# Phase 9: Arrow Key History Navigation - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can navigate their per-window query history using Arrow-Up/Down keys in the overlay input, like shell history. This phase implements HIST-01, HIST-02, HIST-03. The history store (HIST-04, up to 7 queries per window, in-memory) already exists from Phase 8.

</domain>

<decisions>
## Implementation Decisions

### Arrow Key Behavior
- Inline replace: Arrow-Up replaces input text with previous query directly (like bash/zsh), no dropdown
- Multi-line handling (Shift+Enter text): Arrow-Up moves cursor up within multi-line text first, only triggers history when cursor is on the first line
- Arrow-Down in multi-line: always navigates history forward (does NOT do cursor-down-first like Arrow-Up)
- Works with text in input: Arrow-Up triggers history even when there's typed text (draft is saved first)

### Draft Preservation
- Draft text is saved when user starts navigating history, restored when Arrow-Down past newest entry
- Restore text only -- cursor goes to end, no cursor position preservation
- Editing a history entry and submitting it logs as a new history entry; navigating away without submitting discards edits
- Draft is cleared when the overlay closes (no persistence across open/close)
- History index resets on submit -- next Arrow-Up starts from the most recent query

### History Boundary Behavior
- Arrow-Up at oldest entry: stay on oldest, no feedback (like bash)
- Arrow-Down past newest: restore draft and stop, no cycling
- No history exists: Arrow-Up is a silent no-op
- Escape key always closes the overlay entirely, even during history navigation (no two-stage escape)

### Visual Feedback
- No history position indicator (no "2/7" counter)
- Instant text swap with no animation or transition
- Recalled history entries show as slightly dimmed/lighter text color
- Dimmed text returns to normal once the user starts editing the recalled entry

### Claude's Discretion
- Exact dimmed text color/opacity value
- How to detect cursor position in multi-line input for Arrow-Up boundary detection
- Internal state management for history index tracking

</decisions>

<specifics>
## Specific Ideas

- Should feel like bash/zsh shell history -- familiar muscle memory, no surprises
- Asymmetric multi-line behavior is intentional: Arrow-Up respects cursor position (cursor-first), Arrow-Down always goes to history (history-first)

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 09-arrow-key-history-navigation*
*Context gathered: 2026-03-01*
