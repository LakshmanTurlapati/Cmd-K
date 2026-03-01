# Architecture Research

**Domain:** Tauri v2 macOS overlay -- per-terminal-window command history and AI follow-up context
**Researched:** 2026-02-28
**Confidence:** HIGH

---

## Context: This is a Milestone Architecture Doc (v0.1.1)

This document is scoped to how the v0.1.1 features integrate with the existing v0.1.0 architecture. It does not re-document the base architecture -- it documents what changes, what stays the same, and where the seams are.

The three features to add:

1. Per-terminal-window command history (up to 7 entries, session-scoped)
2. Arrow key navigation in overlay input to recall previous prompts
3. AI follow-up context -- AI sees the full `turnHistory` for the active terminal window

---

## System Overview: What Changes in v0.1.1

```
┌─────────────────────────────────────────────────────────────────────┐
│                        macOS System Layer                            │
│  (Accessibility API, NSPanel, Global Hotkey, CGWindowListCopyInfo)  │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
┌────────────────────────────────────┴────────────────────────────────┐
│                     Tauri Core Process (Rust)                        │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  AppState (state.rs)                        [MODIFIED]         │  │
│  │  + previous_window_id: Mutex<Option<u32>>                     │  │
│  │  (existing: hotkey, previous_app_pid, pre_captured_text, ...) │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─────────────────────────┐  ┌──────────────────────────────────┐  │
│  │  commands/hotkey.rs     │  │  commands/window_id.rs  [NEW]    │  │
│  │  [MODIFIED]             │  │  get_terminal_window_id()        │  │
│  │  + capture window ID    │  │  Uses CGWindowListCopyWindowInfo │  │
│  │    before show_overlay  │  │  keyed by previous_app_pid       │  │
│  └─────────────────────────┘  └──────────────────────────────────┘  │
│                                                                      │
│  (all other command modules unchanged)                               │
└────────────────────────────────────┬────────────────────────────────┘
                                     │
                           Tauri IPC (Commands/Events)
                                     │
                                     ▼
┌────────────────────────────────────┴────────────────────────────────┐
│                    WebView Process (Web Frontend)                    │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  src/store/index.ts (Zustand)               [MODIFIED]         │  │
│  │                                                                │  │
│  │  NEW state:                                                    │  │
│  │    windowHistories: Map<string, WindowHistory>                 │  │
│  │    activeWindowKey: string | null                              │  │
│  │    historyNavIndex: number  (-1 = not navigating)             │  │
│  │                                                                │  │
│  │  CHANGED behavior:                                             │  │
│  │    show() -- calls get_terminal_window_id, sets key,          │  │
│  │              loads that window's turnHistory                   │  │
│  │    submitQuery() -- saves turnHistory to windowHistories map   │  │
│  │    turnHistory -- still current-session slice (unchanged use)  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─────────────────────────────┐  ┌──────────────────────────────┐  │
│  │  CommandInput.tsx           │  │  Overlay.tsx                 │  │
│  │  [MODIFIED]                 │  │  [UNCHANGED]                 │  │
│  │  + ArrowUp/ArrowDown key    │  │                              │  │
│  │    handlers using history   │  │                              │  │
│  │    navigation actions       │  │                              │  │
│  └─────────────────────────────┘  └──────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Window Identification Strategy

### The Problem

A user can have multiple Terminal.app windows open at once, or multiple iTerm2 sessions. The `previous_app_pid` captures the *application* PID, but a single application process can own multiple windows. We need a stable key that identifies "which specific terminal window was active."

### Approach: CGWindowListCopyWindowInfo for Window ID

macOS CoreGraphics provides `CGWindowListCopyWindowInfo`, which returns a list of all on-screen windows with their `kCGWindowNumber` (CGWindowID, a u32) and `kCGWindowOwnerPID`. The CGWindowID is stable for the lifetime of the window within the current user session -- it does not persist across app restarts, but that matches the "session-scoped" history requirement.

**The key derivation:** For the `previous_app_pid`, enumerate windows owned by that PID and pick the first on-screen window (layer 0, normal window). This gives a stable `u32` CGWindowID.

**Window key format:** `"{bundle_id}:{window_id}"`, e.g. `"com.apple.Terminal:14320"` or `"com.googlecode.iterm2:8891"`. Bundle ID is included so keys are human-readable in logs and debuggable.

**Fallback:** If `CGWindowListCopyWindowInfo` finds no windows for the PID (possible for GPU terminals that have unusual window structures), fall back to `"{bundle_id}:{pid}"`. This means a multi-window Terminal.app scenario degrades to per-application history, which is acceptable compared to no history at all.

**Why not AX-based window identification?** The AX `_AXUIElementGetWindow` function that converts AXUIElementRef to CGWindowID is a private API (underscore prefix). Using private macOS APIs risks breakage on OS updates. `CGWindowListCopyWindowInfo` is a public, stable CoreGraphics API. Both arrive at the same CGWindowID -- the public API is the correct path.

### Implementation: New Rust Function

New file: `src-tauri/src/commands/window_id.rs`

```rust
// Get a stable window key for the terminal window that was frontmost.
// Called in the hotkey handler BEFORE toggle_overlay (same timing as PID capture).
// Returns "bundle_id:window_id" or "bundle_id:pid" as fallback.
#[cfg(target_os = "macos")]
pub fn get_window_key(pid: i32) -> Option<String> {
    use crate::terminal::detect::get_bundle_id;
    use std::ffi::c_void;

    // CGWindowListCopyWindowInfo FFI
    extern "C" {
        fn CGWindowListCopyWindowInfo(
            option: u32,      // CGWindowListOption
            relativeToWindow: u32,  // CGWindowID, kCGNullWindowID = 0
        ) -> *mut c_void;   // CFArrayRef
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, idx: isize) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        // ... (CFDictionary key lookups, CFNumber extraction)
    }

    let bundle_id = get_bundle_id(pid)?;

    // kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements = 1 | 16 = 17
    // Enumerate on-screen windows, filter by owner PID
    // Extract kCGWindowNumber for the matching window
    // Return format: "com.apple.Terminal:14320"

    // Fallback if no window found: "com.apple.Terminal:{pid}"
    Some(format!("{}:{}", bundle_id, pid))  // placeholder, full impl in plan
}
```

The actual implementation uses CFDictionary key extraction with `CFDictionaryGetValue` and `CFNumberGetValue` -- the same FFI pattern already used extensively in `detect.rs` and `permissions.rs`.

### Integration Point: Hotkey Handler

`src-tauri/src/commands/hotkey.rs` -- the `on_shortcut` closure already captures `previous_app_pid` and `pre_captured_text` before calling `toggle_overlay`. Window ID capture slots in here:

```rust
// Existing code already does:
//   1. get_frontmost_pid() -> store in state.previous_app_pid
//   2. ax_reader::read_focused_text_fast(pid) -> store in state.pre_captured_text
//   3. toggle_overlay(...)

// v0.1.1 addition: after step 1, before step 3:
let window_key = window_id::get_window_key(pid);
if let Some(state) = app_handle.try_state::<AppState>() {
    if let Ok(mut wk) = state.previous_window_key.lock() {
        *wk = window_key;
    }
}
```

`AppState` gets one new field:

```rust
pub previous_window_key: Mutex<Option<String>>,
```

A new Tauri command `get_previous_window_key() -> Option<String>` exposes this to the frontend during context detection. No new complexity in AppState -- same Mutex pattern as `previous_app_pid`.

---

## Zustand Store Extension

### New Types

```typescript
// Add to src/store/index.ts

export interface WindowHistory {
  // Prompts the user entered for this window (for ArrowUp navigation)
  promptHistory: string[];   // max 7 entries, oldest first
  // Full AI conversation turns for follow-up context
  turnHistory: TurnMessage[];  // max 14 entries (7 pairs), oldest first
}
```

### New State Fields

```typescript
// Add to OverlayState interface:

windowHistories: Record<string, WindowHistory>;   // keyed by "bundle_id:window_id"
activeWindowKey: string | null;                    // key of current terminal window
historyNavIndex: number;                           // -1 = not navigating; 0..n = index in promptHistory
historyNavSnapshot: string;                        // saved input before nav started (to restore on Escape)
```

### Changed Behavior in `show()`

```typescript
show: () => {
  // ... existing reset logic stays identical ...
  set((state) => ({
    visible: true,
    // ... existing fields reset ...
    turnHistory: [],    // will be replaced after window key resolves
    historyNavIndex: -1,
    historyNavSnapshot: "",
  }));

  (async () => {
    try {
      const hasPermission = await invoke<boolean>("check_accessibility_permission");
      useOverlayStore.getState().setAccessibilityGranted(hasPermission);

      // NEW: resolve window key before loading history
      const windowKey = await invoke<string | null>("get_previous_window_key");
      const ctx = await invoke<AppContext | null>("get_app_context");

      useOverlayStore.getState().setAppContext(ctx);
      useOverlayStore.getState().setActiveWindowKey(windowKey);  // NEW action
    } catch (err) {
      console.error("[store] context detection error:", err);
    } finally {
      useOverlayStore.getState().setIsDetectingContext(false);
    }
  })();
},
```

### `setActiveWindowKey` action (new)

When the active window key is set, the store restores `turnHistory` from the persisted `windowHistories` map:

```typescript
setActiveWindowKey: (key: string | null) => set((state) => {
  if (!key) return { activeWindowKey: null };
  const existing = state.windowHistories[key];
  return {
    activeWindowKey: key,
    turnHistory: existing?.turnHistory ?? [],
  };
}),
```

### Changed Behavior in `submitQuery()`

After building `trimmedHistory`, save it back to the per-window map and also update the prompt history:

```typescript
// After building trimmedHistory (existing logic unchanged)...

// NEW: persist to per-window map
const currentKey = useOverlayStore.getState().activeWindowKey;
if (currentKey) {
  set((state) => {
    const existing = state.windowHistories[currentKey] ?? {
      promptHistory: [],
      turnHistory: [],
    };
    const updatedPrompts = [
      ...existing.promptHistory.filter(p => p !== query),  // deduplicate
      query,
    ].slice(-7);  // keep last 7
    return {
      windowHistories: {
        ...state.windowHistories,
        [currentKey]: {
          promptHistory: updatedPrompts,
          turnHistory: trimmedHistory,
        },
      },
    };
  });
}
```

### History Navigation State Machine

Arrow key navigation is local state in the `CommandInput` component, but uses store actions to read history. The navigation state lives in the store (not local component state) so it can be cleared on overlay hide.

```
historyNavIndex = -1  (not navigating, showing live inputValue)
    |
ArrowUp pressed (inputValue saved to historyNavSnapshot if index was -1)
    |
historyNavIndex = promptHistory.length - 1  (most recent entry)
    |
ArrowUp again -> index--  (older entry)
    |
ArrowDown -> index++  (newer entry)
    |
index reaches -1 -> restore historyNavSnapshot to inputValue
    |
Escape or Enter -> historyNavIndex resets to -1
```

New store actions:

```typescript
navigateHistoryUp: () => set((state) => {
  const key = state.activeWindowKey;
  if (!key) return {};
  const prompts = state.windowHistories[key]?.promptHistory ?? [];
  if (prompts.length === 0) return {};
  if (state.historyNavIndex === -1) {
    // Save current input before starting navigation
    const snapshot = state.inputValue;
    const newIndex = prompts.length - 1;
    return {
      historyNavSnapshot: snapshot,
      historyNavIndex: newIndex,
      inputValue: prompts[newIndex],
    };
  }
  const newIndex = Math.max(0, state.historyNavIndex - 1);
  return { historyNavIndex: newIndex, inputValue: prompts[newIndex] };
}),

navigateHistoryDown: () => set((state) => {
  if (state.historyNavIndex === -1) return {};
  const key = state.activeWindowKey;
  if (!key) return {};
  const prompts = state.windowHistories[key]?.promptHistory ?? [];
  const newIndex = state.historyNavIndex + 1;
  if (newIndex >= prompts.length) {
    // Past end: restore snapshot
    return {
      historyNavIndex: -1,
      inputValue: state.historyNavSnapshot,
      historyNavSnapshot: "",
    };
  }
  return { historyNavIndex: newIndex, inputValue: prompts[newIndex] };
}),

resetHistoryNav: () => set({
  historyNavIndex: -1,
  historyNavSnapshot: "",
}),
```

### Memory Bound

The `windowHistories` map grows unboundedly during a session (one entry per distinct terminal window). Terminals opened during a session are typically 1-5 windows. Even at 20 windows, each holding 7 prompts of ~200 chars and 14 turns of ~500 chars each, the total is under 100KB -- negligible. No eviction needed for v0.1.1.

The histories are in-memory only (not persisted to disk). A new app launch starts fresh. This is correct for the "session-scoped" requirement.

---

## Data Flow Changes

### Existing Flow (v0.1.0)

```
Cmd+K pressed
  -> hotkey.rs: capture PID, pre-capture AX text -> store in AppState
  -> toggle_overlay(show)
  -> frontend emit "overlay-shown"
  -> store.show() -> reset turnHistory=[], detect context
  -> get_app_context IPC -> returns AppContext
  -> AI query -> stream_ai_response -> build turnHistory locally
  -> next query uses same turnHistory (in-memory, until overlay show resets)
```

### New Flow (v0.1.1)

```
Cmd+K pressed
  -> hotkey.rs: capture PID -> capture window key (NEW) -> pre-capture AX text
  -> toggle_overlay(show)
  -> frontend emit "overlay-shown"
  -> store.show() -> reset historyNavIndex=-1, turnHistory=[] (will be replaced)
  -> PARALLEL:
       get_previous_window_key IPC (NEW, fast: reads AppState field)
       get_app_context IPC (unchanged)
       check_accessibility_permission IPC (unchanged)
  -> setActiveWindowKey(key) -> restores turnHistory from windowHistories[key]
  -> AI query -> stream_ai_response (unchanged IPC, uses restored turnHistory)
  -> submitQuery() -> saves updated turnHistory + promptHistory to windowHistories[key]
  -> Arrow keys -> navigate promptHistory for activeWindowKey
```

The critical insight: `turnHistory` in the store is still the single "current session's conversation." The only change is that on overlay open it is *restored* from the per-window map rather than always starting empty. The `stream_ai_response` IPC call is completely unchanged -- it still receives `history` as a `TurnMessage[]` array. No Rust changes needed for the AI command.

---

## Component Boundaries: New vs. Modified vs. Unchanged

### New Components

| File | What it is | Purpose |
|------|-----------|---------|
| `src-tauri/src/commands/window_id.rs` | New Rust module | `get_window_key(pid)` using CGWindowListCopyWindowInfo, plus `get_previous_window_key` Tauri command |

### Modified Components

| File | What changes | Scope of change |
|------|-------------|-----------------|
| `src-tauri/src/state.rs` | Add `previous_window_key: Mutex<Option<String>>` | 2-line addition |
| `src-tauri/src/lib.rs` | Register `get_previous_window_key` command; import `window_id` module | ~5 lines |
| `src-tauri/src/commands/hotkey.rs` | Call `window_id::get_window_key(pid)` after PID capture; store result in AppState | ~10 lines in closure |
| `src/store/index.ts` | Add `windowHistories`, `activeWindowKey`, `historyNavIndex/Snapshot` state; modify `show()` and `submitQuery()`; add navigation actions | Largest change: ~80 new lines, ~20 modified |
| `src/components/CommandInput.tsx` | Add ArrowUp/ArrowDown key handlers that call navigation actions; conditionally prevent default when history available | ~25 lines |

### Unchanged Components

| File | Why unchanged |
|------|--------------|
| `src-tauri/src/commands/ai.rs` | `stream_ai_response` receives `history: Vec<TurnMessage>` -- the caller (store) manages what it passes; no change needed |
| `src-tauri/src/commands/paste.rs` | Paste logic uses `previous_app_pid`, not window key |
| `src-tauri/src/commands/terminal.rs` | `get_app_context` unchanged -- window key is a separate concern |
| `src-tauri/src/terminal/*` | All detection modules unchanged |
| `src/components/Overlay.tsx` | No new UI elements needed for history/follow-up |
| `src/components/ResultsArea.tsx` | Follow-up context is implicit (turnHistory is populated) |
| All Settings/Onboarding components | Unrelated to this feature |

---

## Architectural Patterns

### Pattern 1: Capture-Before-Show (existing, extended)

**What:** All data about the external application state must be captured in the hotkey handler BEFORE `toggle_overlay` calls `show_and_make_key()`, because after that point `frontmostApplication` returns CMD+K itself.

**Extension for v0.1.1:** Window key is added to this pre-show capture sequence alongside PID and AX text. The hotkey handler captures: `(pid, window_key, pre_captured_text)` in that order before calling `toggle_overlay`.

**Trade-off:** Adds one more FFI call (`CGWindowListCopyWindowInfo`) in the hotkey handler critical path. This call is fast (enumerates on-screen windows from the window server, no AX tree walking) -- benchmark expectation under 5ms.

### Pattern 2: Window-Keyed Map in Zustand

**What:** `windowHistories: Record<string, WindowHistory>` is the central data structure. It accumulates per-window state across multiple overlay invocations without any manual lifecycle management.

**When to use:** When an app visits the same context repeatedly and needs to accumulate state over multiple visits. This is equivalent to a session-scoped cache keyed by an identity token.

**Trade-off:** The map grows monotonically during a session. For the expected use (1-10 terminal windows per session), this is trivially small. The simplicity of append-only writes without eviction makes the logic easier to reason about and test.

### Pattern 3: Restore-on-Activate

**What:** When the overlay shows, `setActiveWindowKey` both sets the active key and restores `turnHistory` from the map. The restore is synchronous within the Zustand action -- no async or effect needed.

**Why:** The AI streaming code in `submitQuery` reads `state.turnHistory` directly. By restoring at key-set time (which happens during the async context detection sequence, before any query), the existing streaming code sees the correct history with no changes.

**Trade-off:** There is a short window (~50-200ms) between `show()` resetting `turnHistory=[]` and `setActiveWindowKey` restoring it. During this window, `turnHistory` is empty. This is acceptable because no query can be submitted until the user finishes typing, and context detection completes in ~200-750ms (existing behavior).

### Pattern 4: Navigation Cursor in Store (not Component)

**What:** `historyNavIndex` lives in the Zustand store rather than local React state in `CommandInput`.

**Why:** The navigation index must be reset when the overlay hides (in `hide()`) and when a query is submitted. If it lived in component state, resetting it from store actions (which don't have component references) would require complex event wiring.

**Trade-off:** Navigation state is slightly over-centralized -- `historyNavIndex` is only consumed by `CommandInput`. But the reset-on-hide requirement makes store placement clearly correct.

---

## Data Flow Diagrams

### Window Key Resolution Flow

```
Hotkey fires (hotkey.rs)
    |
    v
get_frontmost_pid()   [existing, fast ObjC FFI]
    |
    v
window_id::get_window_key(pid)  [NEW, CGWindowListCopyWindowInfo]
    |-- finds windows for pid: returns "bundle_id:window_id"
    |-- no windows found: returns "bundle_id:pid"  (fallback)
    |
    v
AppState.previous_window_key <- Some(key)   [new Mutex field]
    |
    v
[existing: pre_captured_text capture, then toggle_overlay]
```

### History Restore Flow (on overlay open)

```
store.show() called
    |
    +-- set({ turnHistory: [], historyNavIndex: -1, ... })  [sync reset]
    |
    +-- async IIFE starts:
         |
         PARALLEL:
         |-- invoke("get_previous_window_key")  [reads AppState, ~1ms]
         |-- invoke("get_app_context")          [AX detection, ~200-750ms]
         |-- invoke("check_accessibility_permission")
         |
         both resolve:
         |
         v
         setActiveWindowKey(windowKey)
           |-- windowHistories[key] exists?
           |     YES -> restore turnHistory from map
           |     NO  -> turnHistory stays []
         setAppContext(ctx)
         setIsDetectingContext(false)
```

### History Save Flow (on query submit)

```
submitQuery(query) called
    |
    [... existing streaming + destructive check logic (unchanged) ...]
    |
    v
build trimmedHistory   [existing, slices to 14 entries]
    |
    v
save to windowHistories[activeWindowKey]:  [NEW]
    promptHistory <- deduplicated + appended query, sliced to 7
    turnHistory   <- trimmedHistory
    |
    v
set({ turnHistory: trimmedHistory, ... })  [existing update]
```

### Arrow Key Navigation Flow

```
User presses ArrowUp in CommandInput (textarea)
    |
    v
handleKeyDown in CommandInput.tsx
    |
    check: is cursor at start of textarea?  (or textarea is single-line empty)
    YES ->
        e.preventDefault()
        store.navigateHistoryUp()
            |-- historyNavIndex == -1?
            |     save inputValue to historyNavSnapshot
            |     index <- promptHistory.length - 1
            |-- else
            |     index <- max(0, index - 1)
            set inputValue <- promptHistory[newIndex]
    NO  -> default textarea behavior (cursor moves)
```

---

## Integration Points

### Rust Backend

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `hotkey.rs` -> `window_id.rs` | Direct function call (same crate) | Called in hotkey closure before toggle |
| `window_id.rs` -> `AppState` | `app_handle.try_state::<AppState>()` | Same mutex pattern as `previous_app_pid` |
| Frontend -> `get_previous_window_key` | Tauri IPC invoke | New command, reads `AppState.previous_window_key` |
| `window_id.rs` -> `CoreGraphics` | Raw C FFI via `extern "C"` | Same pattern as existing libproc FFI |

### Frontend

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `store.show()` -> Rust | `invoke("get_previous_window_key")` | Parallel with existing IPC calls |
| `CommandInput` -> store | `navigateHistoryUp/Down` actions | Arrow key handlers |
| `store.submitQuery()` -> store | Zustand state update | Writes to `windowHistories` map |

---

## Build Order (considering dependencies)

The dependency chain determines implementation order:

1. **`state.rs`** -- add `previous_window_key` field. Zero dependencies. All subsequent Rust work depends on this.

2. **`commands/window_id.rs`** -- new file with `get_window_key()` and `get_previous_window_key` Tauri command. Depends on `state.rs` (step 1) and `terminal/detect.rs` (existing, `get_bundle_id`). The CGWindowListCopyWindowInfo FFI is self-contained.

3. **`commands/hotkey.rs`** -- add window key capture call. Depends on `window_id.rs` (step 2). Small change to existing closure.

4. **`lib.rs`** -- register new command, import new module. Depends on steps 1-3. Compile gate: `cargo build` must pass here before moving to frontend.

5. **`store/index.ts` -- new state types and fields** -- add `WindowHistory`, `windowHistories`, `activeWindowKey`, `historyNavIndex/Snapshot`. Add `setActiveWindowKey`, `navigateHistoryUp`, `navigateHistoryDown`, `resetHistoryNav` actions. Modify `show()` to call `get_previous_window_key` and invoke `setActiveWindowKey`. Modify `submitQuery()` to persist to `windowHistories`. Depends on step 4 (new IPC command must exist).

6. **`CommandInput.tsx` -- ArrowUp/Down handlers** -- add keyboard navigation. Depends on step 5 (store actions must exist). This is the final user-visible piece.

---

## Anti-Patterns

### Anti-Pattern 1: Storing Window ID as the Primary Key in AppState

**What people do:** Replace `previous_app_pid` with `previous_window_id` everywhere, including in paste and context detection logic.

**Why it's wrong:** The existing context detection (`get_app_context`) and paste (`paste_to_terminal`) use `previous_app_pid` because they need to make ObjC/AX calls that are PID-scoped. `CGWindowID` is not a substitute for PID in these APIs. Conflating the two causes silent failures.

**Do this instead:** Keep `previous_app_pid` unchanged for all existing functionality. Add `previous_window_key` as a separate field solely for history keying.

### Anti-Pattern 2: Persisting `windowHistories` to tauri-plugin-store

**What people do:** Persist window histories to disk so they survive app restarts, using the existing tauri-plugin-store (already in the app for API key/hotkey storage).

**Why it's wrong:** CGWindowIDs are not stable across app restarts -- a new launch of Terminal.app gets new window IDs. Persisted histories would never match live window keys, creating orphaned data that grows forever. Additionally, terminal command history may contain sensitive data (API keys accidentally typed as commands, passwords, etc.) that should not persist to disk without explicit user consent.

**Do this instead:** Keep `windowHistories` in-memory only (Zustand state). Fresh start on each app launch. If disk persistence is desired in a future milestone, design it around user-controlled export, not automatic persistence.

### Anti-Pattern 3: Using ArrowUp in a Multi-Line Textarea for Navigation Unconditionally

**What people do:** Intercept ArrowUp/ArrowDown on the textarea and always override with history navigation, breaking multi-line input cursor movement.

**Why it's wrong:** The `CommandInput` textarea already supports multi-line input (shift+Enter). A user editing a multi-line prompt expects ArrowUp to move the cursor between lines, not jump to history.

**Do this instead:** Only intercept ArrowUp when the cursor is at position 0 of a single-line input (or the textarea has only one line and cursor is at start). When there are multiple lines, let default textarea cursor movement happen. This matches the behavior of bash/zsh history navigation (which also only works when the line is single-line).

### Anti-Pattern 4: Calling CGWindowListCopyWindowInfo Inside `get_app_context` (Async)

**What people do:** Add window ID resolution inside the `get_app_context` Tauri command handler, since that's already called asynchronously after the overlay shows.

**Why it's wrong:** `get_app_context` runs after the overlay has taken focus. At that point, the frontmost window of the terminal app may no longer be reported as "on screen" in the same way, or the window list may have changed. The window key must be captured in the hotkey handler BEFORE `show_and_make_key()`, following the established Capture-Before-Show pattern.

**Do this instead:** Capture window key in `hotkey.rs` alongside PID capture, store in `AppState`, expose via a fast dedicated IPC command (`get_previous_window_key`) that just reads the stored value.

---

## Scaling Considerations

This feature is entirely session-scoped and in-memory. There is no backend state, no database, no network calls added. Scaling dimensions that matter:

| Concern | Expected (1-5 windows) | Edge case (20 windows) | Impact |
|---------|----------------------|----------------------|--------|
| `windowHistories` map size | < 5KB | < 100KB | None |
| CGWindowListCopyWindowInfo call cost | < 5ms | < 5ms (window count bounded by OS) | None |
| `get_previous_window_key` IPC latency | < 1ms | < 1ms | None |
| Arrow key navigation render | O(1) array index | O(1) | None |

No scaling concerns exist for the target usage of this application (single-user macOS app).

---

## Sources

- `src-tauri/src/state.rs` -- existing AppState structure showing Mutex field pattern
- `src-tauri/src/commands/hotkey.rs` -- existing capture-before-show pattern
- `src-tauri/src/terminal/detect.rs` -- existing `get_bundle_id` ObjC FFI pattern
- `src/store/index.ts` -- existing Zustand store showing `turnHistory` and `show()` behavior
- [CGWindowListCopyWindowInfo Apple Developer Documentation](https://developer.apple.com/documentation/coregraphics/1455137-cgwindowlistcopywindowinfo)
- [pdubs -- CGWindowListCopyWindowInfo for PID-to-window mapping](https://github.com/mikesmithgh/pdubs)
- [GetWindowID -- CGWindowID retrieval utility](https://github.com/smokris/GetWindowID)
- [macOS window enumeration patterns](https://www.symdon.info/posts/1729078231/)
- [Accessibility API: AXUIElement to window discussion](https://developer.apple.com/forums/thread/121114)

---

*Architecture research for: CMD+K v0.1.1 per-terminal-window history and AI follow-up context*
*Researched: 2026-02-28*
