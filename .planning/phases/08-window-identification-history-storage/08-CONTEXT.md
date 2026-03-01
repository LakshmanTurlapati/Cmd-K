# Phase 8: Window Identification & History Storage - Context

**Gathered:** 2026-03-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Stable per-terminal-window/tab identity key and Rust-side per-window history map that survives overlay open/close cycles. Every overlay invocation knows which terminal triggered it. History is in-memory only (no disk persistence). VS Code integrated terminal detection is included.

</domain>

<decisions>
## Implementation Decisions

### Window vs Tab identity
- Per-tab history: each terminal tab gets its own independent history (not per-window)
- Split panes within a tab share history (per-tab, not per-pane)
- If tab detection fails for a terminal, fall back to per-window identity (not global)
- Support all major terminals for tab detection: iTerm2, Terminal.app, Alacritty, Kitty, WezTerm
- Add VS Code as a detected terminal (integrated terminal support)

### History persistence
- In-memory only: history resets when the app quits -- no disk persistence
- History for closed tabs/windows is kept in memory until app quits (not cleaned up on close)
- Cap at ~50 tracked windows/tabs total -- evict oldest window's history when exceeded
- 7 entries per window/tab -- 8th query evicts the oldest (per roadmap spec)

### History entry content
- Each entry stores: query text + full AI response text + metadata
- Full AI response stored (explanation, command, warnings -- not just extracted commands)
- Metadata per entry: timestamp + terminal context (CWD, shell, recent terminal output at time of query)
- Failed/error queries are saved to history (user might want to retry via arrow-key recall)

### Non-terminal fallback
- Non-terminal detection is already implemented in v0.1.0 -- no changes to detection UX
- Non-terminal invocations get per-app history (Finder gets its own bucket, Safari gets its own, etc.)
- Not one global bucket -- each non-terminal app is tracked separately

### Claude's Discretion
- Internal data structure design for the history map
- Window/tab identity key generation strategy (what accessibility APIs to use)
- Memory management and eviction implementation details
- How to detect VS Code's integrated terminal vs regular VS Code window

</decisions>

<specifics>
## Specific Ideas

- Terminal detection already exists in `src-tauri/src/terminal/detect.rs` with bundle ID matching -- extend this for tab-level identity
- Current `TERMINAL_BUNDLE_IDS` covers iTerm2, Terminal.app, Alacritty, Kitty, WezTerm -- VS Code needs to be added
- History entries are rich (query + full response + terminal context) to support Phase 9 arrow-key recall and Phase 10 AI follow-up context

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope

</deferred>

---

*Phase: 08-window-identification-history-storage*
*Context gathered: 2026-03-01*
