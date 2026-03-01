# Feature Research

**Domain:** macOS overlay input history navigation and AI conversation context per terminal window
**Milestone:** v0.1.1 Command History and Follow-up Context
**Researched:** 2026-02-28
**Confidence:** HIGH (codebase verified + established UX patterns)

---

## Milestone Scope

This research is scoped to the v0.1.1 milestone only. Existing v0.1.0 features (overlay, AI generation, auto-paste, destructive warnings, terminal context detection) are already built and out of scope. The question is: what does per-terminal-window history navigation and AI follow-up context require to feel complete?

---

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist once "command history" is advertised. Missing any of these makes the feature feel broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Arrow-up navigates to previous query | Universal CLI muscle memory (bash, zsh, fish, readline all do this). Users will press arrow-up on first use and expect it to work. | LOW | Only applies in `displayMode: "input"` when cursor is at column 0 of the first line. Multi-line input needs guard: arrow-up should only trigger history if cursor is on the first line. Existing `handleKeyDown` in `CommandInput.tsx` is the right place. |
| Arrow-down navigates forward (toward present) | Paired with arrow-up. Users who go back expect to go forward again. Must restore empty input when navigating past the most recent entry. | LOW | When user reaches index 0 (most recent) and presses arrow-down again, restore the draft they were typing before they started navigating. |
| Current draft preserved during navigation | When user is mid-type and presses arrow-up, their current input should be saved as a "draft" and restored when they arrow-down past all history entries. Shell behavior: bash/zsh both preserve the draft. | LOW | Store draft string in local state (`historyDraft`) when user first starts navigating. Reset draft when user submits or closes overlay. |
| History scoped to the active terminal window | If the user switches from one terminal tab or window to another and presses Cmd+K, history should reflect commands run in THAT window's context, not a global list. | MEDIUM | See Window Identification section below. This is the central challenge of the milestone. |
| Session-scoped (not persistent across app restarts) | History disappears when CMD+K is quit. No file writes, no ~/.cmd_k_history. Users do not expect an overlay tool to persist query history long-term. Raycast history is session-scoped within a search context, not global persistent. | LOW | In-memory only. Store in Zustand store or a module-level Map. No tauri-plugin-store writes needed for history. |
| AI sees prior turns in follow-up queries | After user gets a command result and submits a follow-up query in the same overlay session, the AI receives prior turns as conversation context. This is already implemented for `turnHistory` within a single overlay open/close cycle. The v0.1.1 requirement extends this to persist `turnHistory` across overlay open/close cycles for the same terminal window. | MEDIUM | `turnHistory` currently resets in `show()` on every overlay open. The fix is: instead of resetting, load the saved `turnHistory` for the current window key, and save it back on hide/submit. |
| History capped at 7 entries per window | PROJECT.md specifies "up to 7 entries, session-scoped." Users do not need infinite history in an overlay tool. Caps prevent memory growth and keep the navigation fast. | LOW | Trim oldest entry when adding beyond 7. Match the existing 14-message (7 turn) cap already in `ai.rs`. |

### Differentiators (Competitive Advantage)

Features that elevate the experience beyond what users expect.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Per-window AI conversation continuity | Most launchers (Raycast, Alfred) treat each activation as a fresh session. CMD+K remembers the full conversation thread for each terminal window across overlay open/close cycles. User can open iTerm2 tab 1, ask "how do I list files by size", get an answer, dismiss, do other work, re-open CMD+K in the same tab, and ask "now filter by .log extension" -- the AI understands the prior context without the user repeating it. | MEDIUM | Requires window key derivation (see below) and storing `turnHistory` per window key. The AI infrastructure already supports this (history param in `stream_ai_response`). |
| History index resets cleanly on submit | When user navigates to a history entry, edits it, and submits, the edited query is added as a new entry (not overwriting the original). History grows forward. Users can freely edit recalled entries. | LOW | This is how bash/zsh handle it. No special logic needed: just call `addToHistory(query)` on every submit. |
| Graceful degradation when window key unavailable | If the app cannot determine which terminal window is active (e.g., non-terminal app, GPU terminal with no AX, permission denied), fall back to a single shared history bucket for that app PID. History still works, just not window-granular. | LOW | Use `appPid` as fallback key when no window-specific key can be derived. |

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Persistent cross-session history (written to disk) | "Remember what I asked last week" | Commands contain sensitive paths, tokens, project names. Writing history to disk violates the zero-footprint expectation of an overlay tool. Adds file I/O, storage concerns, privacy liability. | Session-scoped in-memory only. Users who want persistent history use their shell's history (zsh_history). |
| Cross-window shared history (global history) | Simpler to implement (no window identification needed) | Destroys the value of per-window context. If working on three different projects in three terminal tabs, global history mixes queries from different contexts. Arrow-up in project A tab shows a query from project B. | Per-window scoping is the correct model. Do the work to identify windows. |
| Arrow-up in result or streaming mode | Feels intuitive to some users | Overlay is not a REPL. In result mode, the input area shows the previous query for follow-up editing. Arrow-up would be ambiguous: navigate history OR move cursor in a multiline follow-up? | Only activate history navigation in `displayMode: "input"` and only when cursor is on line 1, col 0. Match how IDEs handle this (e.g., VS Code's terminal input). |
| Infinite history per window | "More is better" | Memory cost grows unboundedly in long sessions. Navigation becomes slow. Users forget what they queried 50 entries ago. | Cap at 7 entries per window as specified. This aligns with the 7-turn AI context cap already in place. |
| Persisting follow-up context across app restarts | "Resume where I left off" | Context from a previous session is stale. Terminal state (CWD, running processes, visible output) will have changed. Resuming a stale conversation leads to irrelevant AI responses. | Session-scoped only. Fresh start on each CMD+K launch. |
| History search (Ctrl+R style fuzzy find in history) | Power-user feature from shell | Scope explosion. v0.1.1 is navigation only. Fuzzy search requires a separate UI surface and filtering logic. | Arrow key navigation is sufficient for 7-entry history. Add search in a future milestone if demand exists. |

---

## Window Identification Strategy

This is the core technical challenge for the milestone. Here is the full analysis.

### The Problem

The `AppState.previous_app_pid` captures the PID of the frontmost app (e.g., `iTerm2`, PID 1234). But iTerm2 may have 10 tabs and 3 windows open. All tabs share the same app PID. History must be keyed to the specific terminal window or tab, not just the app.

### What Exists in the Codebase

The current context detection (`detect_full` in `terminal/mod.rs`) returns:
- `app_name` (cleaned display name, e.g., "iTerm2")
- `terminal.cwd` (current working directory of the shell)
- `terminal.shell_type` (e.g., "zsh")
- `terminal.shell_pid` is NOT returned but is derived internally in `process.rs`

The `AppContext` returned to the frontend has everything except a stable window identifier.

### Candidate Window Key Strategies

**Strategy 1: CWD + App Name composite key**
Key: `"{app_name}:{cwd}"`

Example: `"iTerm2:/Users/alice/projects/web-app"`

- Pros: Already available from `AppContext`. No new Rust code needed. Works for the common case where each terminal tab is in a different directory.
- Cons: Two tabs in the same directory get the same key (shares history). CWD changes as user navigates -- after `cd ~/other-project`, key changes and history appears empty.
- Verdict: Acceptable as a pragmatic fallback. Most developer workflows have tabs scoped to projects, so same-CWD collision is infrequent.

**Strategy 2: Shell PID as key**
Key: shell PID (e.g., 5678 for the specific zsh process)

- Pros: Each terminal tab spawns a unique shell process. Shell PIDs are stable for the lifetime of that tab. Already calculated in `process.rs:find_shell_pid()` but not returned from `detect_full`.
- Cons: Requires exposing shell PID in `AppContext` (one new field in Rust + TypeScript types). Shell PID changes when user opens a new tab in the same directory.
- Verdict: Best technical solution. Low Rust implementation cost. Shell PID is the most precise identifier for "this terminal session."

**Strategy 3: CGWindowID via CoreGraphics**
Key: CGWindowID of the frontmost window

- Pros: Truly unique per OS window.
- Cons: Requires ObjC FFI using `CGWindowListCopyWindowInfo`. Screen recording permission may be required (the same concern addressed in existing research). Adds a new permission request for a non-critical feature. Overkill for v0.1.1.
- Verdict: Do not use. Permission cost exceeds the marginal accuracy improvement over shell PID.

**Strategy 4: AX window title**
Key: window title read via Accessibility API

- Pros: Unique per terminal window/tab in apps that expose tab titles (iTerm2 tab titles often show current directory or running process).
- Cons: AX text reading is already in the critical path and has 500ms timeout. Adding another AX call increases latency. Tab titles are user-configurable and not stable. Does not work for GPU terminals (Alacritty, kitty, WezTerm).
- Verdict: Do not use. Fragile and adds latency.

### Recommended Approach

Use **shell PID** as the primary window key. Fall back to **`app_name:cwd`** when shell PID is unavailable (non-terminal apps, GPU terminals where shell detection may be partial).

Implementation: Add `shell_pid: Option<i32>` to `TerminalContext` struct. Populate it in `process::get_foreground_info()` by returning the PID that was found. This is a minimal Rust change since `find_shell_pid()` already returns the PID -- it just isn't passed through to the output struct.

Frontend key derivation:
```typescript
function deriveWindowKey(appContext: AppContext | null): string {
  if (!appContext) return "global";
  const terminal = appContext.terminal;
  if (terminal?.shell_pid) return `pid:${terminal.shell_pid}`;
  if (terminal?.cwd && appContext.app_name) {
    return `cwd:${appContext.app_name}:${terminal.cwd}`;
  }
  return `app:${appContext.app_name ?? "unknown"}`;
}
```

---

## Feature Dependencies

```
Arrow-up/down history navigation
    └──requires──> per-window history store (Map<windowKey, string[]>)
                       └──requires──> window key derivation
                                          └──requires──> shell_pid in TerminalContext
                                                             └──requires──> expose shell_pid from process.rs

AI follow-up context across open/close cycles
    └──requires──> per-window turnHistory store (Map<windowKey, TurnMessage[]>)
                       └──requires──> window key derivation (same as above)
                       └──requires──> load turnHistory from map in show() instead of reset
                       └──requires──> save turnHistory to map in submitQuery() after each turn

Window key derivation
    └──requires──> AppContext available (already set in store on show())
    └──enhances──> History navigation (makes it per-window vs global)
    └──enhances──> AI follow-up context (makes it per-window vs session-only)

Draft preservation during navigation
    └──requires──> Arrow navigation feature (no value without it)
    └──enhances──> Arrow navigation (prevents accidental loss of in-progress query)
```

### Dependency Notes

- **Shell PID requires minimal Rust change:** `TerminalContext` needs one new optional field. `ProcessInfo` in `process.rs` already has the PID from `find_shell_pid()`. Pass it through. Update TypeScript interface to match.
- **turnHistory per-window conflicts with current show() reset:** The `show()` action currently resets `turnHistory: []`. This must change to: load from the per-window map using the current window key. The window key must be derived after `appContext` is set (which is async). This means turn history loading happens in the async context detection callback, not in the synchronous `show()` call.
- **Arrow navigation conflicts with textarea cursor behavior:** On a single-line input, arrow-up always navigates history. On a multi-line input (the textarea grows), arrow-up should only navigate history when cursor is on the first line (selectionStart <= first line length). Check `textarea.selectionStart === 0` or more accurately check that there's no newline before the cursor position.

---

## MVP Definition

### Launch With (v0.1.1)

Minimum to ship the milestone with all specified behaviors working correctly.

- [ ] `shell_pid` exposed in `TerminalContext` and `AppContext` Rust structs -- required for per-window key
- [ ] `shell_pid` added to TypeScript `TerminalContext` interface in `store/index.ts`
- [ ] `deriveWindowKey()` utility function in frontend (shell_pid primary, cwd:app fallback)
- [ ] Per-window history store (`Map<string, string[]>`) in Zustand store, capped at 7 entries per key
- [ ] `addQueryToHistory(windowKey, query)` called on every successful AI response
- [ ] Arrow-up in `CommandInput.tsx` navigates backward through per-window history when `displayMode === "input"` and cursor is at start of first line
- [ ] Arrow-down navigates forward, restoring in-progress draft when past the end of history
- [ ] Per-window `turnHistory` store (`Map<string, TurnMessage[]>`) in Zustand store
- [ ] `show()` loads `turnHistory` from per-window map (async, after `appContext` is set) instead of resetting to `[]`
- [ ] `submitQuery()` saves updated `turnHistory` back to per-window map after each AI response
- [ ] Graceful fallback: all features work when window key is `"global"` (no terminal detected)

### Add After Validation (v1.x)

- [ ] Tab-specific key for GPU terminals (Alacritty, kitty, WezTerm) -- currently CWD fallback is sufficient
- [ ] History navigation visual indicator (e.g., subtle "3/7" counter) -- adds discoverability but not essential
- [ ] Persist window key mapping across hotkey triggers within same session for apps that respawn shell PIDs

### Future Consideration (v2+)

- [ ] Persistent cross-session history with encryption and explicit user opt-in
- [ ] History search (Ctrl+R style) as a separate UI surface
- [ ] Export conversation thread per window

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Arrow-up/down query history | HIGH | LOW | P1 |
| Per-window history scoping | HIGH | MEDIUM | P1 |
| Draft preservation during navigation | HIGH | LOW | P1 |
| AI follow-up context across cycles | HIGH | MEDIUM | P1 |
| Shell PID in TerminalContext | HIGH (enables all above) | LOW | P1 |
| History capped at 7 entries | LOW (constraint, not a feature) | LOW | P1 |
| History navigation visual counter | MEDIUM | LOW | P2 |
| CGWindowID-based window key | LOW (marginal over shell PID) | HIGH | P3 |
| History search | MEDIUM | HIGH | P3 |

**Priority key:**
- P1: Must have for v0.1.1 launch
- P2: Should have, add if time permits in this milestone
- P3: Future milestone

---

## Competitor Feature Analysis

| Feature | Shell (bash/zsh) | Raycast | Warp AI | CMD+K v0.1.1 |
|---------|-----------------|---------|---------|--------------|
| Arrow-up navigates history | Yes, always | N/A (search-driven) | Yes, per shell session | Yes, per terminal window session |
| History scope | Per shell process (session or file) | Global recent commands | Per terminal session | Per terminal window (shell PID keyed) |
| Draft preservation | Yes (bash/zsh both preserve draft) | N/A | Yes | Yes |
| AI context across queries | N/A | No (stateless search) | Yes, within Warp session | Yes, per terminal window session (v0.1.1) |
| Context persists across launcher close | N/A | No | Yes (terminal stays open) | Yes (per window, session-scoped) |
| Window-level history separation | N/A | N/A | Yes (terminal tabs are separate) | Yes (shell PID key) |

---

## Implementation Complexity Notes

### Low-complexity items (same-session, frontend-only)

Arrow key navigation and draft preservation are purely frontend changes to `CommandInput.tsx` and the Zustand store. No new Tauri commands needed. No Rust changes. Estimated: 2-4 hours.

### Medium-complexity items (cross-session context, requires Rust change)

Exposing `shell_pid` in `TerminalContext` requires:
1. Add `pub shell_pid: Option<i32>` to `TerminalContext` struct in `terminal/mod.rs`
2. Populate it in `detect_inner()` and `detect_app_context()` by capturing the return value of `find_shell_pid()`
3. Update TypeScript `TerminalContext` interface in `store/index.ts`
4. Add `deriveWindowKey()` in frontend
5. Add per-window Maps to Zustand store
6. Change `show()` to load rather than reset `turnHistory` (requires async timing fix)

Estimated: 4-6 hours for all Rust + TypeScript changes.

### Timing challenge for per-window turnHistory

`show()` runs synchronously but `appContext` arrives asynchronously (500ms detection). The per-window `turnHistory` can only be loaded after `appContext` is known. Current `show()` pattern already handles this for accessibility checking -- the same pattern applies: after `setAppContext(ctx)` is called in the async callback, immediately derive the window key and set `turnHistory` from the per-window map. This avoids any new architectural patterns; it extends the existing async context detection flow.

---

## Sources

- Codebase: `/src-tauri/src/terminal/process.rs` -- `find_shell_pid()` already computes shell PID
- Codebase: `/src-tauri/src/terminal/mod.rs` -- `TerminalContext` struct definition
- Codebase: `/src/store/index.ts` -- existing `turnHistory`, `show()`, `submitQuery()` patterns
- Codebase: `/src/components/CommandInput.tsx` -- existing `handleKeyDown` for arrow key extension
- Codebase: `/src-tauri/src/state.rs` -- `AppState` with `previous_app_pid` for window identification context
- [Navigate your command history with ease - Devlog](https://vonheikemen.github.io/devlog/tools/navigate-command-history/) -- shell history UX patterns
- [ZSH-style up/down arrows in Bash/Readline - DEV Community](https://dev.to/onethingwell/zsh-style-updown-arrows-in-bashreadline-linuxunix-series-3mid) -- readline arrow behavior
- [CGWindowListCopyWindowInfo - Apple Developer Documentation](https://developer.apple.com/documentation/coregraphics/1455137-cgwindowlistcopywindowinfo) -- window ID API research
- [Multi-line Input Navigation: Up Arrow issue - opencode GitHub](https://github.com/anomalyco/opencode/issues/9659) -- multi-line arrow conflict pattern
- [Add Up Arrow Navigation for Chat History - Cursor Forum](https://forum.cursor.com/t/add-up-arrow-navigation-for-chat-history/101523) -- expected behavior from user perspective
- PROJECT.md: v0.1.1 specification ("up to 7 entries, session-scoped")

---

*Feature research for: macOS overlay per-terminal-window command history and AI follow-up context*
*Milestone: v0.1.1*
*Researched: 2026-02-28*
