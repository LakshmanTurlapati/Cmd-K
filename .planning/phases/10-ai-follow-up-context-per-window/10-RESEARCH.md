# Phase 10: AI Follow-up Context Per Window - Research

**Researched:** 2026-03-01
**Domain:** Per-window AI conversation context persistence and prompt construction
**Confidence:** HIGH

## Summary

Phase 10 enables the AI to do follow-up responses by persisting conversation history (turnHistory) per terminal window across overlay open/close cycles. The existing codebase already stores per-window HistoryEntry records (query + response pairs) in Rust's AppState via the Phase 8 history system. However, the Zustand `show()` action currently resets `turnHistory: []` on every overlay open, destroying conversation context. The fix is straightforward: reconstruct turnHistory from the existing per-window HistoryEntry data on overlay open, and stop resetting it.

The second key change is prompt construction -- terminal context (CWD, shell, output) should only appear in the first user message of a window's session to prevent token bloat on follow-ups. The current `build_user_message` in `ai.rs` always includes full terminal context. This needs conditional logic based on whether the history array is empty (first message) or not (follow-up).

**Primary recommendation:** Reconstruct turnHistory from existing HistoryEntry data on overlay open, remove the `turnHistory: []` reset from `show()`, and conditionally include terminal context in `build_user_message` only when history is empty.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Keep last 7 turns per window (1 turn = 1 user query + 1 AI response pair)
- History persists across overlay open/close but clears on app restart (in-memory only)
- Turn limit configurable via slider in preferences: range 5-50, default 7
- Every query on the same window is always a follow-up -- no intent detection
- Always send full history (up to the turn limit) regardless of topic relevance
- No manual "start fresh" mechanism -- history flows until app restart
- No explicit follow-up signaling to the AI -- history is silently included
- Use native AI API message array format (alternating user/assistant roles)
- Store both user query and full AI response per turn (no trimming or summarizing)
- No visual indicator in the overlay that AI has prior context -- it just works silently
- No separate conversation/chat log view -- arrow-up recall (Phase 9) serves as user-facing history
- Preferences panel: turn limit slider (5-50, default 7) + "Clear conversation history" button
- Clear button clears history for all windows at once

### Claude's Discretion
- Terminal context strategy for follow-up messages (omit entirely vs send minimal updates)
- Internal data structure for storing turn history per window
- How to handle edge cases (window ID changes, stale history)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| CTXT-01 | AI conversation history (turnHistory) persists per terminal window across overlay open/close cycles within the same session | HistoryEntry in Rust AppState already stores query+response per window. TurnHistory can be reconstructed from this data on overlay open. The key change is removing `turnHistory: []` from `show()` and rebuilding from windowHistory. |
| CTXT-02 | When overlay opens, turnHistory is restored from the per-window map so the AI can do follow-ups | `show()` already fetches `get_window_history` -> `setWindowHistory()`. TurnHistory reconstruction is a pure frontend transformation: flatMap each HistoryEntry into `[{role:"user", content:entry.query}, {role:"assistant", content:entry.response}]`. |
| CTXT-03 | Terminal context (CWD, shell, output) is included only in the first user message of a session to prevent token bloat in follow-ups | `build_user_message` in `ai.rs` receives the `history` Vec. When `history.is_empty()`, include full terminal context. When `!history.is_empty()`, omit terminal context from the user message (just send the raw query). The system prompt already provides shell type. |
</phase_requirements>

## Standard Stack

### Core

No new libraries needed. Phase 10 is entirely implemented with existing dependencies.

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Zustand | (existing) | Frontend state management -- turnHistory storage and restoration | Already used for all overlay state |
| Tauri IPC | (existing) | Fetch per-window history from Rust AppState on overlay open | Already wired for `get_window_history` |
| tauri-plugin-store | (existing) | Persist turn limit preference to settings.json | Already used for hotkey, model, etc. |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @radix-ui/react-slider | (if not present) | Turn limit slider in Preferences tab | Only if HTML range input is insufficient |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Reconstructing turnHistory from HistoryEntry | Storing separate turnHistory array in Rust AppState | Adds redundant data structure; HistoryEntry already has all needed fields (query, response). Reconstruction is O(n) where n <= 50 (7 turns * 2 messages) -- negligible cost. |
| Omitting terminal context in follow-ups entirely | Sending CWD-only updates in follow-ups | CWD changes are unlikely within a window session (same terminal tab). Omitting entirely is simpler and saves more tokens. System prompt already has shell type. |

**Installation:**
```bash
# No new packages needed. If a slider component is desired:
npm install @radix-ui/react-slider
# But HTML <input type="range"> is sufficient for this use case.
```

## Architecture Patterns

### Recommended Changes (File-by-File)

```
src/store/index.ts         # Remove turnHistory reset from show(), add turn limit state,
                            # reconstruct turnHistory from windowHistory on open
src-tauri/src/commands/ai.rs  # Conditional terminal context in build_user_message
src/components/Settings/PreferencesTab.tsx  # Add turn limit slider
src/components/Settings/AdvancedTab.tsx     # Add "Clear conversation history" button
src-tauri/src/commands/history.rs  # Add clear_all_history IPC command
```

### Pattern 1: TurnHistory Reconstruction from HistoryEntry

**What:** Convert per-window HistoryEntry[] into TurnMessage[] on overlay open
**When to use:** Every time `show()` fires and windowHistory is fetched
**Example:**
```typescript
// In show() async block, after setWindowHistory(history):
const turnMessages: TurnMessage[] = history
  .filter(e => !e.is_error)          // skip failed queries
  .flatMap(e => [
    { role: "user" as const, content: e.query },
    { role: "assistant" as const, content: e.response },
  ]);
// Apply turn limit (from preferences, default 7 turns = 14 messages)
const turnLimit = useOverlayStore.getState().turnLimit ?? 7;
const maxMessages = turnLimit * 2;
const trimmed = turnMessages.length > maxMessages
  ? turnMessages.slice(turnMessages.length - maxMessages)
  : turnMessages;
set({ turnHistory: trimmed });
```

### Pattern 2: Conditional Terminal Context in Prompt

**What:** Only include terminal context (CWD, shell, output) in the first message of a session
**When to use:** In `build_user_message` when history array is non-empty
**Example:**
```rust
// ai.rs -- build_user_message signature change:
fn build_user_message(query: &str, ctx: &AppContextView, is_follow_up: bool) -> String {
    if is_follow_up {
        // Follow-up: just the query, no terminal context
        // System prompt already has shell type from the initial message
        return if ctx.terminal.as_ref().and_then(|t| t.shell_type.as_ref()).is_some() {
            format!("Task: {}", query)
        } else {
            query.to_string()
        };
    }
    // First message: full terminal context (existing behavior)
    // ... existing code unchanged ...
}
```

### Pattern 3: Turn Limit Preference Persistence

**What:** Store configurable turn limit in tauri-plugin-store settings.json
**When to use:** When user changes slider in Preferences tab
**Example:**
```typescript
// Load on startup (in App.tsx checkOnboarding):
const turnLimit = await store.get<number>("turnLimit");
useOverlayStore.getState().setTurnLimit(turnLimit ?? 7);

// Save on change (in PreferencesTab):
const handleTurnLimitChange = async (value: number) => {
  setTurnLimit(value);
  const store = await Store.load("settings.json");
  await store.set("turnLimit", value);
  await store.save();
};
```

### Pattern 4: Clear All History Button

**What:** Wipe all per-window history from Rust AppState and reset frontend turnHistory
**When to use:** When user clicks "Clear conversation history" in Preferences
**Example:**
```rust
// New IPC command in history.rs:
#[tauri::command]
pub fn clear_all_history(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    let state = app
        .try_state::<crate::state::AppState>()
        .ok_or("AppState not found")?;
    let mut history = state
        .history
        .lock()
        .map_err(|_| "History mutex poisoned".to_string())?;
    history.clear();
    Ok(())
}
```

### Anti-Patterns to Avoid

- **Separate turn history data structure in Rust:** HistoryEntry already stores query + response. Adding a parallel data structure for turnHistory would create sync issues. Reconstruct on the frontend instead.
- **Including terminal context in every follow-up message:** This wastes tokens and confuses the AI with redundant context. The system prompt already specifies shell type; CWD is stable within a terminal tab session.
- **Using HistoryEntry.terminal_context for follow-up detection:** The presence/absence of terminal_context on individual entries is unreliable (non-terminal apps have null context). Use `history.is_empty()` on the message array passed to `stream_ai_response`.
- **Resetting turnHistory on overlay hide:** The whole point of Phase 10 is that turnHistory survives hide/show cycles. Only reset when the window key changes (different terminal) or user explicitly clears.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Slider UI component | Custom drag-handle slider | HTML `<input type="range">` or Radix Slider | Accessibility, keyboard support, range validation are non-trivial |
| Turn limit enforcement | Custom array slicing logic in multiple places | Single `applyTurnLimit(messages, limit)` utility | Ensures consistent capping everywhere (reconstruction, append, AI call) |
| Conversation context detection | NLP-based follow-up detection | Always treat same-window queries as follow-ups | User decision: no intent detection, every query is a follow-up |

**Key insight:** This phase is primarily about wiring -- connecting existing data (HistoryEntry) to existing consumers (turnHistory in submitQuery). There is very little new logic to build.

## Common Pitfalls

### Pitfall 1: TurnHistory Reset in show()

**What goes wrong:** The current `show()` action resets `turnHistory: []` on every overlay open. If this is not removed, conversation context is destroyed.
**Why it happens:** Original design (v0.1.0) did not have per-window history persistence -- each overlay open was a fresh session.
**How to avoid:** Remove `turnHistory: []` from the `show()` setter. Instead, reconstruct turnHistory from windowHistory in the async block after `setWindowHistory()` completes.
**Warning signs:** AI gives generic responses with no awareness of prior exchanges after overlay dismiss/reopen.

### Pitfall 2: Token Bloat from Repeated Terminal Context

**What goes wrong:** Each follow-up message includes full terminal context (CWD, shell type, last 25 lines of output), multiplying token usage by 2-5x per turn.
**Why it happens:** `build_user_message` always includes terminal context regardless of history.
**How to avoid:** Check `history.is_empty()` in `stream_ai_response` and pass an `is_follow_up` flag to `build_user_message`. Only include terminal context when history is empty (first message).
**Warning signs:** AI responses become slow or hit token limits after 3-4 follow-ups. API costs spike.

### Pitfall 3: Turn Limit vs History Entry Limit Confusion

**What goes wrong:** The existing `MAX_HISTORY_PER_WINDOW = 7` in state.rs caps HistoryEntry storage (for arrow-key recall). The new turn limit (configurable 5-50) caps AI conversation context. These are DIFFERENT limits with DIFFERENT purposes.
**Why it happens:** Both deal with "7 items per window" but serve different features (command recall vs AI context).
**How to avoid:** Keep the two limits separate. HistoryEntry cap stays at 7 for arrow-key recall. Turn limit caps how many HistoryEntry records are converted to turnHistory messages for the AI. When turn limit > 7, the AI only sees up to 7 turns anyway (bounded by storage). When turn limit < 7, fewer entries are sent to the AI even though 7 are stored.
**Warning signs:** Changing the turn limit slider affects arrow-key history or vice versa.

### Pitfall 4: Hardcoded 14-Message Cap in submitQuery and ai.rs

**What goes wrong:** Both `submitQuery` in store/index.ts (line 462-465) and `stream_ai_response` in ai.rs (line 193) have hardcoded `14` as the history message cap (7 turns x 2 messages). This must be updated to use the configurable turn limit.
**Why it happens:** Original implementation used a fixed cap before the configurable slider was added.
**How to avoid:** Pass the turn limit from frontend to `stream_ai_response` or do all capping on the frontend before sending. The frontend already controls turnHistory construction -- cap there and send the pre-capped array.
**Warning signs:** User sets turn limit to 20 but AI only sees 7 turns.

### Pitfall 5: Error Entries in TurnHistory

**What goes wrong:** HistoryEntry records with `is_error: true` have empty response strings. Including them in turnHistory sends incomplete context to the AI.
**Why it happens:** Error entries are stored for arrow-key recall (user can retry failed queries).
**How to avoid:** Filter out `is_error: true` entries when reconstructing turnHistory from windowHistory.
**Warning signs:** AI receives messages like `{role: "assistant", content: ""}` which may confuse it.

### Pitfall 6: HistoryEntry Cap Lower Than Turn Limit

**What goes wrong:** If user sets turn limit to 50 but MAX_HISTORY_PER_WINDOW is 7, only 7 turns of context are ever available.
**Why it happens:** The Rust-side HistoryEntry storage evicts entries beyond 7.
**How to avoid:** The turn limit slider (5-50) and the HistoryEntry cap (7) need to be reconciled. Option A: increase MAX_HISTORY_PER_WINDOW to match the max turn limit (50). Option B: keep storage at 7 and let the slider only go up to 7. **Recommendation: Option A** -- increase storage cap to match the slider max (50 entries). This aligns with the user's decision that turn limit should go up to 50. The per-window VecDeque can hold 50 entries without meaningful memory impact (~50 * avg_entry_size; even at 2KB per entry, that's 100KB per window, 5MB for 50 windows).
**Warning signs:** User moves slider to 20 but AI context stops at 7 turns.

## Code Examples

### Example 1: Reconstructing turnHistory in show()

```typescript
// In show() async block, after windowHistory is fetched and set:
if (windowKey) {
  const history = await invoke<HistoryEntry[]>("get_window_history", { windowKey });
  useOverlayStore.getState().setWindowHistory(history);

  // Reconstruct turnHistory from stored entries (CTXT-01, CTXT-02)
  const turnLimit = useOverlayStore.getState().turnLimit;
  const turnMessages: TurnMessage[] = history
    .filter(e => !e.is_error && e.response)
    .flatMap(e => [
      { role: "user" as const, content: e.query },
      { role: "assistant" as const, content: e.response },
    ]);
  const maxMessages = turnLimit * 2;
  const trimmed = turnMessages.length > maxMessages
    ? turnMessages.slice(turnMessages.length - maxMessages)
    : turnMessages;
  useOverlayStore.getState().setTurnHistory(trimmed);
}
```

### Example 2: Conditional Terminal Context (ai.rs)

```rust
// In stream_ai_response, determine if this is a follow-up:
let is_follow_up = !history.is_empty();

// Build user message with conditional context:
let user_message = build_user_message(&query, &ctx, is_follow_up);

// In build_user_message:
fn build_user_message(query: &str, ctx: &AppContextView, is_follow_up: bool) -> String {
    let is_terminal_mode = ctx.terminal.as_ref()
        .and_then(|t| t.shell_type.as_ref())
        .is_some();

    if is_follow_up {
        // Follow-up: minimal message, no terminal context (CTXT-03)
        if is_terminal_mode {
            format!("Task: {}", query)
        } else {
            query.to_string()
        }
    } else {
        // First message: full context (existing behavior)
        // ... existing code ...
    }
}
```

### Example 3: Turn Limit Slider in PreferencesTab

```typescript
// Slider with value display and persistence
<div className="flex flex-col gap-2">
  <div className="flex items-center justify-between">
    <span className="text-white/70 text-xs">AI conversation memory</span>
    <span className="text-white/40 text-xs font-mono">{turnLimit} turns</span>
  </div>
  <input
    type="range"
    min={5}
    max={50}
    value={turnLimit}
    onChange={(e) => handleTurnLimitChange(Number(e.target.value))}
    className="w-full accent-blue-500"
  />
</div>
```

### Example 4: Clear All History IPC + Frontend

```typescript
// In AdvancedTab or PreferencesTab:
const handleClearHistory = async () => {
  try {
    await invoke("clear_all_history");
    // Reset frontend state
    useOverlayStore.getState().setWindowHistory([]);
    useOverlayStore.getState().setTurnHistory([]);
  } catch (err) {
    console.error("[settings] Failed to clear history:", err);
  }
};
```

### Example 5: Updated show() Without turnHistory Reset

```typescript
show: () => {
  clearRevealTimer();
  set((state) => ({
    visible: true,
    mode: state.mode === "onboarding" && !state.onboardingComplete
      ? "onboarding" : "command",
    hotkeyConfigOpen: false,
    inputValue: "",
    submitted: false,
    showApiWarning: false,
    appContext: null,
    isDetectingContext: true,
    windowKey: null,
    windowHistory: [],
    // DO NOT reset turnHistory here -- it will be reconstructed from windowHistory
    // turnHistory: [],  <-- REMOVED
    streamingText: "",
    isStreaming: false,
    displayMode: "input",
    previousQuery: "",
    streamError: null,
    isDestructive: false,
    destructiveExplanation: null,
    destructiveDismissed: false,
    isPasting: false,
  }));
  // ... async block reconstructs turnHistory after fetching windowHistory
},
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Reset turnHistory on every overlay open | Persist and reconstruct from per-window HistoryEntry | Phase 10 (this phase) | Enables follow-up conversations |
| Always include full terminal context in every message | Terminal context only in first message of a session | Phase 10 (this phase) | Reduces token usage by 30-60% on follow-ups |
| Fixed 7-turn history cap | Configurable turn limit (5-50) via preferences slider | Phase 10 (this phase) | User control over AI memory depth |

**Deprecated/outdated:**
- The hardcoded `14` message cap in `submitQuery` (store/index.ts line 462) and `stream_ai_response` (ai.rs line 193) should be replaced with the configurable turn limit.

## Open Questions

1. **Should MAX_HISTORY_PER_WINDOW be increased to match the turn limit slider max (50)?**
   - What we know: Current cap is 7 entries in Rust. Turn limit slider goes up to 50. If storage stays at 7, the slider range beyond 7 is meaningless.
   - What's unclear: User intent -- is 50 the real desired max, or was 7 the intended practical limit with 50 as a "nice to have" number?
   - Recommendation: Increase `MAX_HISTORY_PER_WINDOW` to 50 to match the slider max. Memory impact is negligible. This makes the full slider range functional.

2. **Should the "Clear conversation history" button also clear arrow-key command history?**
   - What we know: Both use the same HistoryEntry data in Rust AppState. Clearing HistoryEntry would remove both AI context and arrow-key recall data.
   - What's unclear: User expects "conversation history" to mean AI context only, or everything?
   - Recommendation: Clear everything (single `history.clear()`) since they share the same backing store. The button label "Clear conversation history" is accurate -- arrow-key history IS the conversation history (queries the user typed). Simpler than maintaining separate stores.

3. **Should setTurnHistory be added as a store action?**
   - What we know: turnHistory is currently set inline within submitQuery. Phase 10 needs to set it during show() reconstruction too.
   - Recommendation: Yes, add a `setTurnHistory` action to the store for clean reconstruction.

## Sources

### Primary (HIGH confidence)

- **Codebase analysis** -- Direct reading of all relevant source files:
  - `src/store/index.ts` -- Zustand store with turnHistory state and show()/submitQuery() actions
  - `src-tauri/src/commands/ai.rs` -- `stream_ai_response` and `build_user_message` prompt construction
  - `src-tauri/src/state.rs` -- AppState with per-window HistoryEntry HashMap
  - `src-tauri/src/commands/history.rs` -- IPC commands for window history CRUD
  - `src/hooks/useHistoryNavigation.ts` -- Arrow-key history navigation hook
  - `src/components/Settings/PreferencesTab.tsx` -- Current preferences UI
  - `src/components/Settings/AdvancedTab.tsx` -- Current advanced settings UI

### Secondary (MEDIUM confidence)

- **Phase 8 verification report** -- Confirmed HistoryEntry persistence across overlay cycles, window key stability, and IPC command wiring
- **Phase 10 CONTEXT.md** -- User decisions on turn limits, follow-up detection, prompt construction, and history visibility

### Tertiary (LOW confidence)

- None -- all findings are from direct codebase analysis

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new libraries needed, direct codebase analysis confirms all patterns
- Architecture: HIGH -- changes are straightforward wiring of existing data structures
- Pitfalls: HIGH -- identified from direct code reading (hardcoded caps, reset in show(), error entries, storage vs display limit mismatch)

**Research date:** 2026-03-01
**Valid until:** 2026-03-31 (stable -- no external dependency changes expected)
