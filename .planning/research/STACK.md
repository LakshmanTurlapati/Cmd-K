# Stack Research

**Domain:** Per-terminal-window command history, arrow key navigation, AI follow-up context (v0.1.1 milestone additions)
**Researched:** 2026-02-28
**Confidence:** HIGH

> This document supersedes the v0.1.0 STACK.md. The validated stack (Tauri v2, React 19, TypeScript 5.8, Vite 7, Zustand 5, Radix UI, xAI/Grok, NSPanel, libproc FFI, Accessibility API) is not re-researched. Focus is strictly on what the new milestone adds.

---

## What Changes in v0.1.1

Three new capabilities are needed:

1. **Terminal window identification** -- a stable key to separate per-window histories
2. **In-memory history store** -- up to 7 entries per window, session-scoped (no persistence)
3. **Arrow key history navigation** -- up/down in the textarea cycles history, respecting cursor position semantics
4. **Conversation context per window** -- AI follow-up context (`turnHistory`) scoped to window ID

All of this is implemented within the existing Tauri v2 + Rust + React + Zustand stack. No new frameworks. No new persistence libraries.

---

## Recommended Stack

### Core Technologies

No changes to core technologies. Existing versions already in use:

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Tauri v2 | 2.x (in use) | IPC bridge between Rust and React | Already integrated; new Tauri commands follow existing patterns |
| Rust | 1.88+ (in use) | Window ID detection via Core Graphics FFI | Unsafe FFI already used for libproc and AX API; same approach for CGWindowListCopyWindowInfo |
| React 19 + TypeScript 5.8 | in use | History navigation UI in CommandInput.tsx | Already handles keyboard events in CommandInput; arrow key logic extends existing onKeyDown handler |
| Zustand 5 | 5.0.11 (in use) | Per-window history and conversation state | Already used for all overlay state; extend with Map-keyed history |

### New Rust Capability: Window ID via Core Graphics

**Problem:** The existing system captures the frontmost app's PID before the overlay shows. PID alone does not identify a specific terminal window -- iTerm2 can have 10 windows under one PID.

**Solution:** Add `core-graphics` crate to call `CGWindowListCopyWindowInfo` and extract the CGWindowID (window number) for the frontmost window of the captured PID. This pairs with PID to create a stable per-window key.

| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `core-graphics` | 0.25.0 | `CGWindowListCopyWindowInfo` to get `kCGWindowNumber` filtered by `kCGWindowOwnerPID` | Already partially available in the Core Graphics ecosystem; provides `CGWindowListCopyWindowInfo`, `kCGWindowNumber`, `kCGWindowOwnerPID` constants verified present at 0.25.0 (HIGH confidence) |
| `core-foundation` | 0.10.1 | CFDictionary/CFArray helpers to extract typed values from the window info result | Required companion for safe value extraction from CFDictionary; already a transitive dep via accessibility-sys |

**Why not `_AXUIElementGetWindow` (private API):** That function is undocumented and may break on any macOS update. `CGWindowListCopyWindowInfo` is public, documented by Apple, and verified working on macOS 13-15.

**Why not PID only:** Multiple iTerm2/WezTerm windows share one PID. The window ID from Core Graphics is the unique per-window identifier that macOS uses internally.

**Window key format:** Combine PID and window number as a string key: `"{pid}:{window_number}"`. If `CGWindowListCopyWindowInfo` returns no windows for the PID (e.g., the app is GPU-rendered and has no registered windows), fall back to PID-only key `"{pid}:0"` -- the history is still better than nothing.

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `core-graphics` | 0.25.0 | CGWindowListCopyWindowInfo for window ID | Add to Cargo.toml; used only in a new `window_id.rs` helper in `src-tauri/src/terminal/` |
| `core-foundation` | 0.10.1 | CFDictionary value extraction | Already transitively available; may need explicit dep for type-safe extraction |

No new npm packages are needed. Arrow key navigation and Zustand Map-keyed state are implemented in userland code.

### Development Tools

No changes. Existing tools (Vite, TypeScript, cargo) remain unchanged.

---

## Installation

No new npm packages. One new Rust crate:

```bash
# In src-tauri/
cargo add core-graphics
# core-foundation is likely already a transitive dep; add explicitly if build errors occur
cargo add core-foundation
```

Add feature flags if needed:

```toml
# Cargo.toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.25"
core-foundation = "0.10"
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `CGWindowListCopyWindowInfo` (public API) | `_AXUIElementGetWindow` (private API) | Never -- private APIs break silently on macOS updates |
| PID + CGWindowID composite key | PID-only key | Only if the app has a single-window guarantee (not true for iTerm2/WezTerm) |
| Zustand `Map<string, WindowHistory>` with `new Map(...)` shallow copy | Zustand slice per window | Map is simpler for unknown-count keyed state; slices require known keys at definition time |
| In-memory Zustand state (session-only) | `tauri-plugin-store` or `localStorage` | Never for this feature -- requirement is session-scoped, no persistence wanted |
| Zustand `Map` with `new Map(state.histories)` update pattern | Immer middleware | No need to add immer dependency; Map spread is two lines, immer adds 12KB |
| Native `onKeyDown` in existing `CommandInput.tsx` | External library (react-arrow-key-navigation-hook) | External libs handle DOM focus navigation; history cycling within a single textarea has no suitable off-the-shelf library and is 20 lines of logic |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `tauri-plugin-store` for history | Requirement explicitly says session-only; persisting history adds complexity and user surprise | In-memory Zustand `Map` |
| `localStorage` or `sessionStorage` | Tauri webview storage is persistent across sessions by default; violates session-only requirement | Zustand in-memory state (cleared on app restart automatically) |
| `react-arrow-key-navigation-hook` or similar | These libraries navigate between DOM elements (focus management); history cycling is index manipulation within state, not focus navigation | Custom `useRef<number>` history index in `CommandInput.tsx` |
| `immer` middleware in Zustand | Adds 12KB for a pattern that two lines solve (`new Map(state.histories)`) | Plain Zustand `set` with `new Map(...)` copy |
| Database (SQLite via `tauri-plugin-sql`) | Complete overkill; 7 entries per window, session-scoped | Zustand in-memory `Map` |
| `_AXUIElementGetWindow` private API | Not documented, no guarantee of stability across macOS releases | `CGWindowListCopyWindowInfo` public CG API |

---

## Stack Patterns by Variant

**If the frontmost app has no windows in CGWindowList (rare: some GPU-rendered terminals or non-standard apps):**
- Fall back to `"{pid}:0"` as the window key
- History is still tracked per-app-launch; not per-window, but acceptable degradation

**If Zustand Map update triggers extra re-renders:**
- Use `useShallow` selector when subscribing to the history Map in components
- `useOverlayStore(useShallow(state => state.windowHistories.get(windowKey)))` prevents re-render when other windows' histories change

**If arrow key navigation conflicts with multiline input:**
- ArrowUp should only cycle history when the cursor is on the first line (selectionStart within the first line)
- ArrowDown should only cycle when the cursor is on the last line
- This is the same semantic behavior as zsh/bash arrow key history in terminals
- Detect with: `textarea.value.lastIndexOf('\n', selectionStart - 1) === -1` for first line

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `core-graphics@0.25` | macOS 13+ (Ventura, Sonoma, Sequoia) | `CGWindowListCopyWindowInfo` deprecated in macOS 15 for ScreenCaptureKit but still functional for window enumeration (not screen capture) |
| `core-foundation@0.10.1` | `core-graphics@0.25` | Both from the `servo/core-foundation-rs` monorepo; compatible versions |
| `zustand@5.0.11` | React 19 | Already validated in production; `new Map(...)` update pattern works with React 19 concurrent mode |

**Note on CGWindowListCopyWindowInfo deprecation:** Apple marked this API deprecated in macOS 15 in the context of screen capture (replaced by ScreenCaptureKit). For window enumeration (no screen content, just window metadata), it remains fully functional. This is the same pattern used by apps like Raycast, Alfred, and Hammerspoon. Monitor Apple developer release notes for future removal (LOW near-term risk).

---

## Integration Points with Existing Codebase

### Rust Side

**New file:** `src-tauri/src/terminal/window_id.rs`
- Export function `fn get_window_id_for_pid(pid: i32) -> String`
- Calls `CGWindowListCopyWindowInfo` with `kCGWindowListOptionAll`
- Filters by `kCGWindowOwnerPID == pid`
- Returns `"{pid}:{kCGWindowNumber}"` of the first match, or `"{pid}:0"` if none

**Modified:** `src-tauri/src/commands/hotkey.rs` (in `register_hotkey` handler)
- After capturing `previous_app_pid`, also call `get_window_id_for_pid(pid)` and store in `AppState`

**Modified:** `src-tauri/src/state.rs`
- Add `pub window_key: Mutex<String>` field alongside `previous_app_pid`

**Modified:** `src-tauri/src/commands/terminal.rs` (`get_app_context`)
- Return `window_key` alongside `AppContext` (add field to `AppContext` struct or return as separate value via a new command `get_window_key`)
- Simplest: add `pub window_key: Option<String>` to `AppContext` struct in `src-tauri/src/terminal/mod.rs`

### TypeScript/React Side

**Modified:** `src/store/index.ts`
- Add `windowKey: string | null` to state (populated from `AppContext.window_key`)
- Add `windowHistories: Map<string, string[]>` for per-window query history (max 7 entries)
- Add `historyIndex: number` for current navigation position (-1 = not navigating)
- Modify `show()` action: reset `historyIndex` to -1, keep `windowHistories` (session-scoped, don't clear on overlay open)
- Modify `submitQuery()`: after successful query, call `pushToHistory(windowKey, query)` action
- Add `pushToHistory(key: string, query: string)` action: uses `new Map(state.windowHistories)` pattern, caps at 7

**Modified:** `src/components/CommandInput.tsx`
- Add `historyIndex` ref (local React `useRef<number>`, starts at -1)
- In `handleKeyDown`: when `ArrowUp` and cursor is at start of first line (or input is single-line), cycle history backward
- When `ArrowDown` and cursor is at end of last line, cycle history forward or return to current input (index -1)
- Save the in-progress input to a local ref when starting navigation so ArrowDown can restore it

**No change needed:** `useKeyboard.ts`, `ResultsArea.tsx`, AI streaming logic -- conversation `turnHistory` already exists in the store and is already per-overlay-session; the only change is scoping it per window key when the overlay opens for a specific window

---

## Sources

- [core-graphics 0.25.0 on docs.rs](https://docs.rs/core-graphics/0.25.0/core_graphics/window/index.html) -- verified `CGWindowListCopyWindowInfo`, `kCGWindowNumber`, `kCGWindowOwnerPID` present (HIGH confidence)
- [crates.io/crates/core-graphics](https://crates.io/crates/core-graphics) -- version 0.25.0 confirmed current (HIGH confidence)
- [crates.io/crates/core-foundation](https://crates.io/crates/core-foundation) -- version 0.10.1 confirmed current (HIGH confidence)
- [Apple Developer Documentation: CGWindowListCopyWindowInfo](https://developer.apple.com/documentation/coregraphics/1455137-cgwindowlistcopywindowinfo) -- public API, window enumeration use case not deprecated (MEDIUM confidence on longevity)
- [Obtaining Window ID on macOS - symdon.info](https://www.symdon.info/posts/1729078231/) -- confirmed Swift pattern translates to Rust FFI (MEDIUM confidence)
- [Zustand Maps and Sets usage guide](https://github.com/pmndrs/zustand/blob/main/docs/guides/maps-and-sets-usage.md) -- `new Map(state.foo).set(key, value)` pattern for triggering re-renders (HIGH confidence, official docs)
- [Rust Forum: CGWindowListCopyWindowInfo with PID filtering](https://users.rust-lang.org/t/how-to-get-window-owner-names-via-cgwindowlistcopywindowinfo/37958) -- confirmed approach works, requires `unsafe` CFDictionary extraction (MEDIUM confidence)

---

*Stack research for: v0.1.1 per-terminal-window command history and AI follow-up context*
*Researched: 2026-02-28*
