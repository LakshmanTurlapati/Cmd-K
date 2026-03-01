# Requirements: CMD+K

**Defined:** 2026-02-28
**Core Value:** The overlay must appear on top of the active application and feel instant

## v0.1.1 Requirements

Requirements for per-terminal-window command history and AI follow-up context.

### Window Identification

- [ ] **WKEY-01**: App computes a stable per-terminal-window key (bundle_id:shell_pid) before the overlay shows
- [ ] **WKEY-02**: Window key is captured synchronously in the hotkey handler alongside PID capture, before overlay steals focus
- [ ] **WKEY-03**: Non-terminal apps fall back to a global key so history still works outside terminals

### Command History

- [ ] **HIST-01**: User can press Arrow-Up in the overlay input to recall the previous query for the active terminal window
- [ ] **HIST-02**: User can press Arrow-Down to navigate forward through history, restoring the current draft at the end
- [ ] **HIST-03**: Current draft text is preserved when user starts navigating history and restored when they return
- [ ] **HIST-04**: History stores up to 7 queries per terminal window, session-scoped (in-memory only)

### AI Follow-up Context

- [ ] **CTXT-01**: AI conversation history (turnHistory) persists per terminal window across overlay open/close cycles within the same session
- [ ] **CTXT-02**: When overlay opens, turnHistory is restored from the per-window map so the AI can do follow-ups
- [ ] **CTXT-03**: Terminal context (CWD, shell, output) is included only in the first user message of a session to prevent token bloat in follow-ups

## Future Requirements

### History Enhancements

- **HIST-05**: Visual history position indicator (e.g., "2/7" counter)
- **HIST-06**: Ctrl+R fuzzy search through command history
- **HIST-07**: Persistent cross-session history with encryption and user opt-in

### Window Identification Enhancements

- **WKEY-04**: CGWindowID-based window key for more precise multi-window identification

## Out of Scope

| Feature | Reason |
|---------|--------|
| Persistent history to disk | Session-scoped only for v0.1.1; privacy concern, adds complexity |
| History search (Ctrl+R) | Requires separate UI surface; deferred to future |
| CGWindowID window key | Shell PID sufficient for v0.1.1; CGWindowID adds dependency and potential permission prompt |
| Export conversation thread | Future feature |
| Favorites/bookmarks | Not part of v0.1.1 history scope |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| WKEY-01 | Pending | Pending |
| WKEY-02 | Pending | Pending |
| WKEY-03 | Pending | Pending |
| HIST-01 | Pending | Pending |
| HIST-02 | Pending | Pending |
| HIST-03 | Pending | Pending |
| HIST-04 | Pending | Pending |
| CTXT-01 | Pending | Pending |
| CTXT-02 | Pending | Pending |
| CTXT-03 | Pending | Pending |

**Coverage:**
- v0.1.1 requirements: 10 total
- Mapped to phases: 0
- Unmapped: 10

---
*Requirements defined: 2026-02-28*
*Last updated: 2026-02-28 after initial definition*
