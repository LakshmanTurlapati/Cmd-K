# Phase 9: Arrow Key History Navigation - Research

**Researched:** 2026-03-01
**Domain:** React textarea keyboard event handling, Zustand transient UI state, shell-like history navigation UX
**Confidence:** HIGH

## Summary

Phase 9 adds arrow-key history navigation to the overlay input. When the user presses Arrow-Up, the input text is replaced with the previous query from the per-window history (already stored in Rust AppState and fetched into Zustand `windowHistory` by Phase 8). Arrow-Down moves forward through history; going past the newest entry restores the user's draft. The behavior mirrors bash/zsh shell history.

The existing codebase provides everything needed on the data side: `windowHistory: HistoryEntry[]` is populated in Zustand on every overlay open via `get_window_history` IPC (Phase 8, Plan 02). Each `HistoryEntry` contains `query: string` which is the text to recall. The history is ordered oldest-first (front of deque = oldest, matching `VecDeque::push_back` order).

The implementation is entirely frontend -- no Rust changes needed. The work centers on:
1. Adding history navigation state (index, draft) to the Zustand store or as local state in `CommandInput`
2. Intercepting Arrow-Up and Arrow-Down keydown events in the textarea `handleKeyDown` handler
3. Detecting cursor position for multi-line input (Arrow-Up should only trigger history when cursor is on the first line)
4. Visual feedback: dimmed text color for recalled history entries, returning to normal on edit

**Primary recommendation:** Add a custom React hook `useHistoryNavigation` that encapsulates the history index, draft preservation, and keyboard event logic. Wire it into `CommandInput.tsx` alongside the existing `handleKeyDown`. Use Zustand for the `windowHistory` data (already there) but keep the navigation index and draft as local component state (they reset on every overlay open, which is correct behavior). Apply dimmed text via a CSS class toggle on the textarea based on whether the currently displayed text came from history recall.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Inline replace: Arrow-Up replaces input text with previous query directly (like bash/zsh), no dropdown
- Multi-line handling (Shift+Enter text): Arrow-Up moves cursor up within multi-line text first, only triggers history when cursor is on the first line
- Arrow-Down in multi-line: always navigates history forward (does NOT do cursor-down-first like Arrow-Up)
- Works with text in input: Arrow-Up triggers history even when there is typed text (draft is saved first)
- Draft text is saved when user starts navigating history, restored when Arrow-Down past newest entry
- Restore text only -- cursor goes to end, no cursor position preservation
- Editing a history entry and submitting it logs as a new history entry; navigating away without submitting discards edits
- Draft is cleared when the overlay closes (no persistence across open/close)
- History index resets on submit -- next Arrow-Up starts from the most recent query
- Arrow-Up at oldest entry: stay on oldest, no feedback (like bash)
- Arrow-Down past newest: restore draft and stop, no cycling
- No history exists: Arrow-Up is a silent no-op
- Escape key always closes the overlay entirely, even during history navigation (no two-stage escape)
- No history position indicator (no "2/7" counter)
- Instant text swap with no animation or transition
- Recalled history entries show as slightly dimmed/lighter text color
- Dimmed text returns to normal once the user starts editing the recalled entry

### Claude's Discretion
- Exact dimmed text color/opacity value
- How to detect cursor position in multi-line input for Arrow-Up boundary detection
- Internal state management for history index tracking

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| HIST-01 | User can press Arrow-Up in the overlay input to recall the previous query for the active terminal window | Intercept ArrowUp keydown in CommandInput handleKeyDown; check `windowHistory` from Zustand; replace `inputValue` with `windowHistory[index].query`; manage navigation index as local state |
| HIST-02 | User can press Arrow-Down to navigate forward through history, restoring current draft at end | Intercept ArrowDown keydown; increment index; when index exceeds history length, restore saved draft text and reset index to "no history" sentinel |
| HIST-03 | Current draft text is preserved when user starts navigating history and restored when they return | Save `inputValue` to a `draft` variable on first Arrow-Up; restore it when Arrow-Down goes past newest entry; clear draft on overlay close (Zustand reset in show() handles this) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| React (hooks) | 19.1.0 (already in package.json) | useState/useRef/useCallback for history state management | Already the project's UI framework; no additional dependency |
| Zustand | 5.0.11 (already in package.json) | Read windowHistory array from store; update inputValue | Already the state management solution; Phase 8 already populates windowHistory |
| Tailwind CSS | 4.2.0 (already in package.json) | text-white/50 or text-white/60 opacity class for dimmed history text | Already used throughout all components for styling |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| None needed | - | - | - |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Local useState for history index | Zustand store fields | Zustand fields would persist across component unmount/remount, but history index SHOULD reset on every overlay open -- local state achieves this naturally via the show() reset. Adding to Zustand adds complexity with no benefit. |
| Custom hook (useHistoryNavigation) | Inline logic in CommandInput handleKeyDown | Inline would work but makes the already 107-line CommandInput harder to read. A hook isolates the history concern cleanly. |
| CSS class toggle for dimming | Inline style opacity | CSS class is consistent with the project's Tailwind convention (all styling via className). No performance difference. |

**Installation:**
No new packages needed. All functionality uses existing React, Zustand, and Tailwind CSS.

## Architecture Patterns

### Recommended Changes to Existing Structure
```
src/
  components/
    CommandInput.tsx    # MODIFY: add ArrowUp/ArrowDown handling, dimmed text class toggle
  hooks/
    useHistoryNavigation.ts  # NEW: encapsulates history index, draft, and navigation logic
  store/
    index.ts           # MINOR MODIFY: add historyIndex reset in submitQuery (optional)
```

### Pattern 1: History Navigation Hook (useHistoryNavigation)
**What:** A custom React hook that manages the history cursor position, draft preservation, and provides a `handleHistoryKey` callback.
**When to use:** Called once inside `CommandInput`, returns state and handlers consumed by the textarea.
**Why a hook:** Isolates the entire history navigation concern from the input's existing keyboard handling (Enter, Tab, Shift+Enter). The hook can be tested independently.

```typescript
// src/hooks/useHistoryNavigation.ts
import { useState, useCallback, useRef } from "react";
import { useOverlayStore, HistoryEntry } from "@/store";

interface UseHistoryNavigationReturn {
  /** Whether the currently displayed text is from history recall (for dimming) */
  isRecalled: boolean;
  /** Call this on every ArrowUp/ArrowDown keydown event; returns true if the event was consumed */
  handleHistoryKey: (
    e: React.KeyboardEvent<HTMLTextAreaElement>,
    textareaEl: HTMLTextAreaElement
  ) => boolean;
  /** Call this when a query is submitted (resets index) */
  resetOnSubmit: () => void;
  /** Call this when the user edits the input (clears recalled state) */
  markEdited: () => void;
}

export function useHistoryNavigation(): UseHistoryNavigationReturn {
  // History index: -1 means "not navigating" (showing draft or fresh input)
  // 0 = most recent query, 1 = second most recent, etc.
  // Maps to windowHistory[windowHistory.length - 1 - historyIndex]
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [isRecalled, setIsRecalled] = useState(false);
  const draftRef = useRef<string>("");

  const handleHistoryKey = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>, textareaEl: HTMLTextAreaElement): boolean => {
      const state = useOverlayStore.getState();
      const history = state.windowHistory;

      if (e.key === "ArrowUp") {
        // Multi-line check: only trigger history if cursor is on the first line
        if (isCursorOnFirstLine(textareaEl)) {
          e.preventDefault();
          if (history.length === 0) return true; // no-op

          if (historyIndex === -1) {
            // First time pressing up: save current input as draft
            draftRef.current = state.inputValue;
          }

          const newIndex = Math.min(historyIndex + 1, history.length - 1);
          setHistoryIndex(newIndex);
          const entry = history[history.length - 1 - newIndex];
          useOverlayStore.getState().setInputValue(entry.query);
          setIsRecalled(true);
          return true;
        }
        return false; // let default cursor movement happen
      }

      if (e.key === "ArrowDown") {
        // Arrow-Down ALWAYS navigates history (asymmetric with Arrow-Up)
        if (historyIndex > 0) {
          e.preventDefault();
          const newIndex = historyIndex - 1;
          setHistoryIndex(newIndex);
          const entry = history[history.length - 1 - newIndex];
          useOverlayStore.getState().setInputValue(entry.query);
          setIsRecalled(true);
          return true;
        } else if (historyIndex === 0) {
          // Past newest: restore draft
          e.preventDefault();
          setHistoryIndex(-1);
          useOverlayStore.getState().setInputValue(draftRef.current);
          setIsRecalled(false);
          return true;
        }
        // historyIndex === -1 and not navigating: let default behavior
        return false;
      }

      return false;
    },
    [historyIndex]
  );

  const resetOnSubmit = useCallback(() => {
    setHistoryIndex(-1);
    setIsRecalled(false);
    draftRef.current = "";
  }, []);

  const markEdited = useCallback(() => {
    if (isRecalled) {
      setIsRecalled(false);
    }
  }, [isRecalled]);

  return { isRecalled, handleHistoryKey, resetOnSubmit, markEdited };
}

function isCursorOnFirstLine(el: HTMLTextAreaElement): boolean {
  const value = el.value;
  const cursorPos = el.selectionStart;
  // If there's no newline before the cursor, cursor is on the first line
  return !value.substring(0, cursorPos).includes("\n");
}
```

### Pattern 2: CommandInput Integration
**What:** Modify `CommandInput.tsx` to use the history navigation hook and apply dimmed styling.
**When to use:** The hook is called at the top of the component; its `handleHistoryKey` is invoked at the start of the existing `handleKeyDown`.

```typescript
// In CommandInput.tsx handleKeyDown, ADD before existing Tab/Enter handling:
const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
  // History navigation (Arrow-Up/Down)
  if (e.key === "ArrowUp" || e.key === "ArrowDown") {
    const consumed = handleHistoryKey(e, textareaRef.current!);
    if (consumed) {
      // Also auto-resize textarea after text replacement
      const el = textareaRef.current!;
      el.style.height = "auto";
      el.style.height = `${el.scrollHeight}px`;
      return;
    }
    // Not consumed: let default behavior (cursor movement in multi-line)
    return;
  }

  // Mark as edited when any non-navigation key is pressed while showing recalled text
  if (isRecalled && e.key !== "ArrowUp" && e.key !== "ArrowDown") {
    markEdited();
  }

  // Existing Tab/Enter handling...
};
```

### Pattern 3: Dimmed Text Styling
**What:** Apply a slightly dimmed text color when the textarea shows a recalled history entry.
**When to use:** When `isRecalled` is true, add a CSS class that reduces text opacity.
**Why opacity not color:** The existing text color is `text-white`. Using `text-white/60` (60% opacity white) provides visible dimming without introducing a new color. This is consistent with how the project uses opacity variants throughout (e.g., `text-white/40` for placeholder, `text-white/20` for ghost suggestions).

Recommended dimmed value: `text-white/60` (noticeably dimmer than `text-white` but still readable).

```tsx
// In CommandInput.tsx textarea className:
className={[
  "w-full",
  "bg-transparent",
  isRecalled ? "text-white/60" : "text-white",  // dimmed when showing history
  "text-sm",
  // ... rest unchanged
].join(" ")}
```

### Pattern 4: History Index Reset on Submit
**What:** When the user submits a query (via Enter), the history index resets so the next Arrow-Up starts from the most recent entry.
**When to use:** In the `onSubmit` callback path inside `CommandInput`.
**Why:** Per user decision: "History index resets on submit -- next Arrow-Up starts from the most recent query."

```typescript
// In CommandInput handleKeyDown, within the Enter submission block:
if (inputValue.trim()) {
  resetOnSubmit(); // reset history navigation state
  onSubmit(inputValue);
}
```

### Pattern 5: Cursor-to-End on History Recall
**What:** After replacing the input text with a history entry, move the cursor to the end of the text.
**When to use:** After every `setInputValue` call from history navigation.
**Why:** Per user decision: "Restore text only -- cursor goes to end, no cursor position preservation."

```typescript
// After setting input value from history, move cursor to end:
requestAnimationFrame(() => {
  const el = textareaRef.current;
  if (el) {
    el.selectionStart = el.selectionEnd = el.value.length;
  }
});
```

### Pattern 6: Multi-Line Cursor Detection
**What:** Determine if the cursor is on the first line of a multi-line textarea.
**When to use:** Before deciding whether Arrow-Up triggers history or native cursor movement.
**Why:** The user decision says "Arrow-Up moves cursor up within multi-line text first, only triggers history when cursor is on the first line." This prevents Arrow-Up from hijacking normal cursor navigation in multi-line prompts (created via Shift+Enter).

The detection is simple: check if there is a newline character (`\n`) before the cursor position. If not, the cursor is on the first line.

```typescript
function isCursorOnFirstLine(el: HTMLTextAreaElement): boolean {
  const value = el.value;
  const cursorPos = el.selectionStart;
  return !value.substring(0, cursorPos).includes("\n");
}
```

Note: Arrow-Down is asymmetric -- it ALWAYS navigates history (does not do cursor-down-first). This is an intentional user decision.

### Anti-Patterns to Avoid
- **Do NOT add a dropdown or popup for history selection:** Decision says "Inline replace, no dropdown."
- **Do NOT persist history index across overlay open/close:** The index resets naturally because show() resets inputValue and the component re-mounts. Local state handles this automatically.
- **Do NOT cycle history (wrap around):** Arrow-Up at oldest stays on oldest; Arrow-Down past newest restores draft and stops.
- **Do NOT add animation or transition to text swaps:** Decision says "Instant text swap with no animation or transition."
- **Do NOT add a position indicator:** Decision says "No history position indicator (no '2/7' counter)."
- **Do NOT modify Rust backend:** All functionality is purely frontend. History data is already in Zustand via Phase 8.
- **Do NOT use controlled component pattern for the textarea during history:** The existing `inputValue` in Zustand + `setInputValue` is already the controlled value. History navigation just calls `setInputValue` with the recalled query -- no need for a separate controlled state.
- **Do NOT save edited history entries back to the history store:** Per decision: "Editing a history entry and submitting it logs as a new history entry; navigating away without submitting discards edits." The history array is read-only during navigation.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| First-line cursor detection | Complex position calculation using getBoundingClientRect | `el.value.substring(0, el.selectionStart).includes("\n")` check | The simple string check is correct for HTMLTextAreaElement; no need for pixel-based position detection |
| Textarea auto-resize after history recall | Custom ResizeObserver | Reuse the existing `el.style.height = "auto"; el.style.height = el.scrollHeight + "px"` pattern from handleChange | Already implemented in CommandInput; just call the same 2 lines after text replacement |
| History data retrieval | New IPC call for each arrow press | Read from `windowHistory` already in Zustand (fetched once on show()) | Phase 8 already loads history into Zustand on every overlay open; no need for repeated IPC calls |

## Common Pitfalls

### Pitfall 1: Arrow-Down Interferes with Native Textarea Cursor Movement
**What goes wrong:** In a multi-line textarea, Arrow-Down normally moves the cursor down one line. If history navigation always intercepts ArrowDown, users cannot navigate multi-line text.
**Why it happens:** The natural keyboard event for ArrowDown in a textarea moves the cursor.
**How to avoid:** Per the user decision, Arrow-Down ALWAYS triggers history navigation when in history mode (historyIndex >= 0), regardless of cursor position. When NOT in history mode (historyIndex === -1), do NOT intercept ArrowDown -- let the default textarea behavior happen. This is asymmetric with Arrow-Up by design.
**Warning signs:** Users unable to move cursor down in multi-line input when NOT navigating history.

### Pitfall 2: History Index Stale After New Query Submitted
**What goes wrong:** User navigates to history[2], edits it, and submits. The submitted text is added to history via `add_history_entry`. But `windowHistory` in Zustand is stale (it was fetched on overlay open). The next Arrow-Up may show the wrong entry.
**Why it happens:** `windowHistory` is fetched once in `show()` and not refreshed after `submitQuery()` adds a new entry.
**How to avoid:** After a query is submitted, the user returns to result mode (streaming -> result). If they press Enter again (follow-up), `returnToInput` is called and `displayMode` goes back to "input". At this point, history in Zustand is stale. Two options:
1. **Recommended:** After `submitQuery` completes and adds the entry to Rust history, also append the new entry to the local `windowHistory` in Zustand. This is a simple one-liner: `setWindowHistory([...windowHistory, newEntry])`.
2. **Alternative:** Re-fetch history from Rust after each submit. This is slower and unnecessary since we know exactly what was added.
The recommended approach keeps the Zustand state in sync without IPC overhead. The new entry can be constructed from the query text and an empty response placeholder (the response is not needed for Arrow-Up recall -- only `entry.query` is used).
**Warning signs:** Arrow-Up after submitting a query does not show the just-submitted query as the most recent entry.

### Pitfall 3: Draft Lost When Component Re-Renders
**What goes wrong:** The user types "hello", presses Arrow-Up (draft saved), then a React re-render occurs and the draft ref loses its value.
**Why it happens:** If the draft is stored in a useState that triggers re-render, or if the component unmounts.
**How to avoid:** Use `useRef` for the draft, not `useState`. The draft does not need to trigger re-renders -- it is only read when Arrow-Down returns past the newest entry. Using `useRef` avoids re-renders and survives React reconciliation within the same mount cycle. The overlay close/reopen naturally creates a new component mount, which resets the ref (correct behavior per "Draft is cleared when the overlay closes").
**Warning signs:** After Arrow-Up then Arrow-Down past newest, the input is empty instead of showing the user's draft.

### Pitfall 4: Textarea Does Not Auto-Resize After History Text Replacement
**What goes wrong:** User recalls a long history entry that is multi-line, but the textarea stays at its single-line height. The text overflows.
**Why it happens:** The textarea auto-resize logic in `handleChange` only fires on user input events (onChange). Programmatic value changes via `setInputValue` do not trigger onChange.
**How to avoid:** After replacing the text via `setInputValue(entry.query)`, manually trigger the auto-resize logic:
```typescript
requestAnimationFrame(() => {
  const el = textareaRef.current;
  if (el) {
    el.style.height = "auto";
    el.style.height = `${el.scrollHeight}px`;
  }
});
```
Use `requestAnimationFrame` to ensure the DOM has updated with the new text before measuring scrollHeight.
**Warning signs:** Long recalled history entries appear truncated or overflow the textarea.

### Pitfall 5: History Navigation Conflicts with Existing ArrowUp/ArrowDown Default Behavior
**What goes wrong:** The browser's default ArrowUp behavior in a textarea (move cursor up one line) fires alongside the history navigation, causing the cursor to jump and text to change simultaneously.
**Why it happens:** If `e.preventDefault()` is not called when history navigation consumes the event.
**How to avoid:** When `handleHistoryKey` consumes an ArrowUp or ArrowDown event, it MUST call `e.preventDefault()` to suppress the native textarea behavior. The current pattern already does this (see Pattern 1 code example).
**Warning signs:** Cursor jumps to weird positions when recalling history entries.

### Pitfall 6: handleChange Fires After setInputValue, Incorrectly Clearing isRecalled
**What goes wrong:** When `setInputValue` is called from history navigation, React updates the textarea's value. In some cases, this can trigger the `onChange` handler, which in turn would call `markEdited()` and clear the dimmed state prematurely.
**Why it happens:** React's controlled component pattern: setting the value programmatically re-renders the textarea, but does NOT fire the onChange event (onChange only fires from user interaction). However, if the implementation accidentally uses an `onInput` handler or if the auto-resize logic triggers a state update, it could interfere.
**How to avoid:** The existing `handleChange` fires only on user interaction (React's onChange is from user input, not programmatic value changes). As long as `markEdited` is called from `handleKeyDown` (on non-arrow key presses) rather than from `handleChange`, this pitfall is avoided. Alternatively, `markEdited` can check if the change was user-initiated by looking at the key event rather than the change event.
**Warning signs:** Recalled history text briefly flashes dimmed then immediately returns to normal color without user editing.

### Pitfall 7: windowHistory Order Confusion
**What goes wrong:** History index mapping is inverted -- Arrow-Up shows the oldest entry instead of the most recent.
**Why it happens:** `windowHistory` in Zustand is ordered oldest-first (index 0 = oldest, matching the Rust VecDeque `push_back` order). Arrow-Up should show the MOST RECENT entry first.
**How to avoid:** The index mapping must reverse: `historyIndex = 0` maps to `windowHistory[windowHistory.length - 1]` (most recent). The formula is `windowHistory[windowHistory.length - 1 - historyIndex]`.
**Warning signs:** First Arrow-Up press shows a very old query instead of the most recent one.

## Code Examples

### Complete useHistoryNavigation Hook
```typescript
import { useState, useCallback, useRef } from "react";
import { useOverlayStore } from "@/store";

export interface UseHistoryNavigationReturn {
  isRecalled: boolean;
  handleHistoryKey: (
    e: React.KeyboardEvent<HTMLTextAreaElement>,
    textareaEl: HTMLTextAreaElement
  ) => boolean;
  resetOnSubmit: () => void;
  markEdited: () => void;
}

export function useHistoryNavigation(): UseHistoryNavigationReturn {
  // -1 = not navigating (draft/fresh input)
  // 0 = most recent history entry, 1 = second most recent, etc.
  const [historyIndex, setHistoryIndex] = useState(-1);
  const [isRecalled, setIsRecalled] = useState(false);
  const draftRef = useRef<string>("");

  const handleHistoryKey = useCallback(
    (
      e: React.KeyboardEvent<HTMLTextAreaElement>,
      textareaEl: HTMLTextAreaElement
    ): boolean => {
      const history = useOverlayStore.getState().windowHistory;

      if (e.key === "ArrowUp") {
        // Multi-line: only trigger history when cursor is on the first line
        const value = textareaEl.value;
        const cursorPos = textareaEl.selectionStart;
        const isFirstLine = !value.substring(0, cursorPos).includes("\n");

        if (!isFirstLine) {
          return false; // let native cursor-up happen
        }

        e.preventDefault();
        if (history.length === 0) return true; // silent no-op

        if (historyIndex === -1) {
          // Save draft on first navigation
          draftRef.current = useOverlayStore.getState().inputValue;
        }

        const newIndex = Math.min(historyIndex + 1, history.length - 1);
        if (newIndex === historyIndex && historyIndex === history.length - 1) {
          // Already at oldest entry, stay put (no wrap)
          return true;
        }
        setHistoryIndex(newIndex);
        const entry = history[history.length - 1 - newIndex];
        useOverlayStore.getState().setInputValue(entry.query);
        setIsRecalled(true);
        return true;
      }

      if (e.key === "ArrowDown") {
        // Arrow-Down always navigates history when in history mode (asymmetric)
        if (historyIndex > 0) {
          e.preventDefault();
          const newIndex = historyIndex - 1;
          setHistoryIndex(newIndex);
          const entry = history[history.length - 1 - newIndex];
          useOverlayStore.getState().setInputValue(entry.query);
          setIsRecalled(true);
          return true;
        } else if (historyIndex === 0) {
          // Past newest: restore draft
          e.preventDefault();
          setHistoryIndex(-1);
          useOverlayStore.getState().setInputValue(draftRef.current);
          setIsRecalled(false);
          return true;
        }
        // Not in history mode: let native cursor-down happen
        return false;
      }

      return false;
    },
    [historyIndex]
  );

  const resetOnSubmit = useCallback(() => {
    setHistoryIndex(-1);
    setIsRecalled(false);
    draftRef.current = "";
  }, []);

  const markEdited = useCallback(() => {
    if (isRecalled) {
      setIsRecalled(false);
    }
  }, [isRecalled]);

  return { isRecalled, handleHistoryKey, resetOnSubmit, markEdited };
}
```

### CommandInput.tsx Integration
```typescript
// At top of CommandInput component:
const { isRecalled, handleHistoryKey, resetOnSubmit, markEdited } =
  useHistoryNavigation();

// In handleKeyDown, BEFORE existing Tab/Enter logic:
const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
  // History navigation
  if (e.key === "ArrowUp" || e.key === "ArrowDown") {
    const consumed = handleHistoryKey(e, textareaRef.current!);
    if (consumed) {
      // Auto-resize textarea after text replacement
      requestAnimationFrame(() => {
        const el = textareaRef.current;
        if (el) {
          el.style.height = "auto";
          el.style.height = `${el.scrollHeight}px`;
          // Move cursor to end
          el.selectionStart = el.selectionEnd = el.value.length;
        }
      });
      return;
    }
    return; // not consumed, default behavior
  }

  // Any other key pressed while showing recalled text: mark as edited
  if (isRecalled) {
    markEdited();
  }

  // Existing Tab/Enter handling below...
  if (e.key === "Tab" && suggestion) { ... }
  if (e.key === "Enter" && !e.shiftKey) { ... }
};

// In the Enter submission block, before onSubmit call:
if (inputValue.trim()) {
  resetOnSubmit();
  onSubmit(inputValue);
}
```

### Textarea Dimmed Styling
```tsx
<textarea
  ref={textareaRef}
  rows={1}
  value={inputValue}
  onChange={handleChange}
  onKeyDown={handleKeyDown}
  onMouseUp={handleMouseUp}
  placeholder="Ask anything..."
  className={[
    "w-full",
    "bg-transparent",
    isRecalled ? "text-white/60" : "text-white",
    "text-sm",
    "leading-relaxed",
    "resize-none",
    "outline-none",
    "ring-0",
    "border-none",
    "max-h-[200px]",
    "overflow-y-auto",
    "placeholder:text-white/40",
    "scrollbar-thin",
    "relative",
  ].join(" ")}
  style={{
    minHeight: "24px",
    caretColor: displayMode === "input" ? undefined : "transparent",
  }}
/>
```

### Keeping windowHistory in Sync After Submit
```typescript
// In store/index.ts submitQuery(), after the add_history_entry invoke:
// Append the new entry locally so Arrow-Up immediately sees it without re-fetching
const historySync: HistoryEntry = {
  query,
  response: fullText,
  timestamp: Date.now(),
  terminal_context: historyCtx,
  is_error: false,
};
const currentHistory = useOverlayStore.getState().windowHistory;
useOverlayStore.getState().setWindowHistory([...currentHistory, historySync]);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No history navigation | Arrow-Up/Down recalls per-window query history | Phase 9 (this phase) | Users can quickly re-submit or modify previous queries without retyping |
| Static input text | Dimmed text for recalled entries, normal for typed/edited text | Phase 9 (this phase) | Visual distinction between user-typed text and recalled history |
| windowHistory fetched once on show(), stale after submit | windowHistory synced locally after each submit | Phase 9 (this phase) | Arrow-Up after submit immediately sees the just-submitted query |

**Deprecated/outdated:**
- Nothing is deprecated by this phase. The existing `turnHistory` in Zustand (for AI streaming context) is unrelated to query history navigation and remains unchanged. Phase 10 will address `turnHistory` persistence.

## Open Questions

1. **Textarea cursor position after history recall with long text**
   - What we know: After replacing the textarea value with a long history entry, `requestAnimationFrame` + setting `selectionStart/selectionEnd = el.value.length` moves the cursor to the end.
   - What is unclear: Whether the textarea scrolls to show the cursor position when the recalled text exceeds `max-h-[200px]`.
   - Recommendation: Test during implementation. If the textarea does not scroll to the cursor, add `el.scrollTop = el.scrollHeight` after setting the cursor position.

2. **markEdited firing on modifier keys (Shift, Ctrl, Alt, Meta)**
   - What we know: Pressing Shift, Ctrl, Alt, or Meta alone should NOT clear the recalled state.
   - What is unclear: Whether the current "any non-arrow key" check inadvertently triggers on modifier-only presses.
   - Recommendation: Filter out modifier-only keys in the `markEdited` check: `if (isRecalled && !["ArrowUp", "ArrowDown", "Shift", "Control", "Alt", "Meta", "CapsLock", "Tab", "Escape"].includes(e.key)) { markEdited(); }`. Tab is already handled separately (tab completion). Escape closes the overlay. This ensures only actual text-changing keys clear the dimmed state.

3. **History sync after error queries**
   - What we know: Error queries are also persisted to Rust history with `isError: true`. The error path in submitQuery also calls `add_history_entry`.
   - What is unclear: Whether the error path should also sync the local windowHistory.
   - Recommendation: Yes, append the error entry locally too, so Arrow-Up can recall failed queries within the same overlay session.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `src/components/CommandInput.tsx` (159 lines), `src/store/index.ts` (600 lines), `src/hooks/useKeyboard.ts` (58 lines) -- direct examination of existing keyboard handling and state management
- React documentation: `useState`, `useRef`, `useCallback` hooks -- stable React 19 APIs
- MDN Web Docs: `HTMLTextAreaElement.selectionStart` (https://developer.mozilla.org/en-US/docs/Web/API/HTMLTextAreaElement/selectionStart) -- standard DOM API for cursor position detection
- MDN Web Docs: `KeyboardEvent.key` values (https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key/Key_Values) -- ArrowUp, ArrowDown key identifiers
- Zustand documentation (https://docs.pmnd.rs/zustand/getting-started/introduction) -- store access patterns via `useOverlayStore.getState()`

### Secondary (MEDIUM confidence)
- Tailwind CSS opacity utilities (https://tailwindcss.com/docs/opacity) -- `text-white/60` syntax for opacity variants
- Phase 8 implementation summaries (08-01-SUMMARY.md, 08-02-SUMMARY.md) -- confirms windowHistory is populated in Zustand and entries are ordered oldest-first

### Tertiary (LOW confidence)
- None needed -- this phase is entirely frontend React/TypeScript with no platform-specific concerns.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- Uses only existing React hooks + Zustand + Tailwind CSS; zero new dependencies
- Architecture: HIGH -- Follows established project patterns (custom hooks in `src/hooks/`, Zustand state in `src/store/`, component logic in `src/components/`); CommandInput.tsx is a well-structured 159-line component with clear extension points
- Pitfalls: HIGH -- All 7 pitfalls identified through direct analysis of the textarea behavior, React controlled component semantics, and the history data flow
- Multi-line detection: HIGH -- `selectionStart` + newline check is the standard approach for textarea first-line detection; no platform-specific quirks on macOS WebKit (Tauri uses WKWebView)

**Research date:** 2026-03-01
**Valid until:** 2026-04-01 (30 days -- stable domain, no rapidly changing APIs)
