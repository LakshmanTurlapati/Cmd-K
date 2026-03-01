# Project Research Summary

**Project:** CMD+K v0.1.1
**Domain:** macOS Tauri overlay app -- per-terminal-window command history and AI follow-up context
**Researched:** 2026-02-28
**Confidence:** HIGH

## Executive Summary

CMD+K is a macOS system overlay (Tauri v2 + Rust + React 19) that provides AI-powered command generation accessible via a global hotkey from any terminal. The v0.1.1 milestone adds two tightly coupled capabilities on top of the shipped v0.1.0 foundation: per-terminal-window command history with arrow key navigation (up to 7 entries, session-scoped) and persistent AI conversation context that survives overlay dismiss/reopen cycles within the same terminal window session. Both features require solving a single core problem -- stably identifying which terminal window is active -- which unlocks everything downstream.

The recommended implementation approach is layered in three phases. First, window identification and history storage architecture must be locked in before any UI work begins, because all other features depend on the window key. The key should be a composite of bundle ID and either the shell PID (primary) or TTY device path (most robust), computed synchronously in the hotkey handler before the overlay shows and stored in Rust `AppState`. Per-window history should be stored in Rust (`HashMap<String, WindowHistory>` in `AppState`), not in Zustand, to survive the `show()` reset that clears all ephemeral React state on every overlay open. Second, arrow key navigation is a self-contained frontend change to `CommandInput.tsx` once the store actions exist. Third, AI follow-up context is a wiring change in `submitQuery()` and `show()` to persist and restore `turnHistory` per window key.

The primary risks are: (1) using app PID alone as the window key, which silently mixes history across multiple iTerm2 windows; (2) storing history in Zustand only, where it is wiped on every overlay open by the existing `show()` reset; (3) intercepting ArrowUp unconditionally in the textarea, breaking multi-line cursor movement; and (4) including full terminal context in every AI history turn, causing token bloat by follow-up turn 3-4. All four risks have well-defined, low-cost preventions documented in the research.

## Key Findings

### Recommended Stack

The existing Tauri v2, Rust, React 19, TypeScript 5.8, and Zustand 5 stack requires no new npm packages and only one new Rust crate. The `core-graphics` crate (0.25.0) provides `CGWindowListCopyWindowInfo`, the public Apple API for resolving a terminal app PID to a specific CGWindowID. `core-foundation` (0.10.1) is required as a companion for safe CFDictionary extraction and is likely already a transitive dependency. The `CGWindowListCopyWindowInfo` API is marked deprecated in macOS 15 only in the context of screen capture; for window enumeration (metadata only, no screen content), it remains fully functional and is used by Raycast, Alfred, and Hammerspoon with no near-term removal risk.

**Core technologies:**
- `Tauri v2 (2.x)`: IPC bridge between Rust and React -- already integrated; new `get_previous_window_key` command follows existing patterns
- `Rust 1.88+ / core-graphics 0.25`: CGWindowListCopyWindowInfo for window ID derivation -- public API, stable on macOS 13-15
- `React 19 / TypeScript 5.8`: Arrow key navigation UI in `CommandInput.tsx` -- extends existing `onKeyDown` handler
- `Zustand 5.0.11`: Per-window state coordination -- `Record<string, WindowHistory>` pattern; spread-copy update pattern avoids immer dependency
- `core-foundation 0.10.1`: CFDictionary/CFArray value extraction -- required companion for safe CoreGraphics FFI

### Expected Features

The full v0.1.1 feature set is well-defined in PROJECT.md and validated against competitor behavior (bash/zsh, Raycast, Warp AI). All P1 features must ship together -- they share the window key dependency and deliver no value in isolation.

**Must have (table stakes -- P1 for v0.1.1):**
- Arrow-up navigates to previous query -- universal CLI muscle memory; users press this on first encounter
- Arrow-down navigates forward, restoring draft -- paired with arrow-up; missing this makes navigation feel broken
- Current draft preserved during navigation -- bash/zsh both do this; losing the draft is a UX regression
- History scoped to the active terminal window -- global history defeats the purpose of per-window context
- Session-scoped only (no disk persistence) -- zero-footprint expectation; disk writes add privacy risk
- AI sees prior turns across overlay open/close cycles -- the differentiating capability of this milestone
- History capped at 7 entries per window -- aligned with the existing 14-message (7 turn) AI context cap

**Should have (competitive -- P2):**
- History navigation visual indicator (e.g., subtle position counter) -- adds discoverability without requiring new UI surfaces

**Defer (v2+):**
- Persistent cross-session history with encryption and explicit user opt-in
- History search (Ctrl+R style fuzzy find) -- requires a separate UI surface
- CGWindowID-based window key (marginal accuracy improvement over shell PID, adds screen recording permission risk)
- Export conversation thread per window

### Architecture Approach

The architecture extends the existing Capture-Before-Show pattern: all pre-overlay data (PID, window key, AX text) must be captured synchronously in the hotkey handler before `show_and_make_key()` steals focus. A new `previous_window_key: Mutex<Option<String>>` field in `AppState` stores the computed key. A new `window_id.rs` module handles CoreGraphics FFI. On the frontend, `windowHistories: Record<string, WindowHistory>` in the Zustand store accumulates per-window state across overlay invocations. The `show()` action invokes `get_previous_window_key` in parallel with existing IPC calls and restores `turnHistory` from the map via `setActiveWindowKey`. Navigation state (`historyNavIndex`, `historyNavSnapshot`) lives in the store (not component state) so it can be reset by the `hide()` action.

**Major components:**
1. `src-tauri/src/commands/window_id.rs` (new) -- `get_window_key(pid)` via CGWindowListCopyWindowInfo; `get_previous_window_key` Tauri command
2. `src-tauri/src/state.rs` (modified) -- adds `previous_window_key: Mutex<Option<String>>`
3. `src-tauri/src/commands/hotkey.rs` (modified) -- captures window key before `toggle_overlay`; slots between PID capture and AX text capture
4. `src/store/index.ts` (modified) -- adds `windowHistories`, `activeWindowKey`, `historyNavIndex/Snapshot`; modifies `show()` and `submitQuery()`; adds `setActiveWindowKey`, `navigateHistoryUp`, `navigateHistoryDown`, `resetHistoryNav` actions
5. `src/components/CommandInput.tsx` (modified) -- adds ArrowUp/ArrowDown handlers gated on cursor line position

**Unchanged:** `ai.rs` (stream_ai_response receives history array unchanged), `paste.rs`, `terminal.rs`, `Overlay.tsx`, `ResultsArea.tsx`, all settings and onboarding components.

### Critical Pitfalls

1. **App PID alone as window key** -- silently mixes all iTerm2 windows under one history bucket. Use composite `bundle_id:window_id` (CGWindowID) or `bundle_id:shell_pid` as the key. Lock this in before any history storage or navigation UI is built -- everything depends on it.

2. **History stored in Zustand only (wiped on every show())** -- `show()` resets all ephemeral state. Per-window history that lives only in Zustand is erased every overlay open. Store `WindowHistory` in Rust `AppState` as a `HashMap<String, WindowHistory>` and fetch on each overlay open via a dedicated Tauri command.

3. **ArrowUp intercepted unconditionally in textarea** -- breaks multi-line input cursor movement (Shift+Enter prompts). Gate ArrowUp on `isOnFirstLine` (no newline before cursor) and ArrowDown on `isOnLastLine` (no newline after cursor). This is 5 lines of guard code; skipping it causes a hard-to-debug regression.

4. **Window key computed after overlay shows (TOCTOU race)** -- computing the key inside `get_app_context` (which runs async after the overlay steals focus) means a fast Cmd+K switch between windows assigns the wrong key. Compute the key synchronously in `hotkey.rs` alongside PID capture, before `show_and_make_key()`.

5. **Full terminal context in every AI history turn (token bloat)** -- terminal context (CWD, shell, 25 lines of output) included in all replayed history turns causes API payloads to grow 2-5x by follow-up turn 3-4, triggering timeouts and 429 rate limits. Include terminal context only in the first user message of a session; follow-up turns send bare query text only. Add a character-count secondary cap (6,000 chars) alongside the existing turn count cap.

## Implications for Roadmap

Based on research, the feature dependency graph mandates a strict three-phase build order. Window identification and history storage must be fully working in Rust before any frontend work begins. Arrow navigation is a self-contained frontend phase. AI follow-up context wiring is the final phase.

### Phase 1: Window Identification and History Storage

**Rationale:** All downstream features (arrow navigation, AI context) depend on the window key. The Pitfalls research identifies the choice of key format and storage location as the two highest-recovery-cost decisions in the milestone. Locking these in first eliminates the risk of needing to refactor the key type after the UI is built. The Rust AppState `HashMap` storage approach must be validated before the frontend store is modified.

**Delivers:** A stable `window_key` string available to the frontend on every overlay open; a `WindowHistory` map in Rust that survives across overlay open/close cycles; `get_previous_window_key` Tauri command; `get_window_history` and `update_window_history` commands (or equivalent) for frontend read/write of per-window history.

**Addresses:** Shell PID exposure in `TerminalContext`; CGWindowID composite key via `window_id.rs`; `AppState` extension; LRU eviction policy (cap at 50 windows, 4-hour TTL) to prevent unbounded memory growth in long daemon sessions.

**Avoids:** PID-only key pitfall, Zustand-wipe pitfall, TOCTOU race, unbounded map growth.

### Phase 2: Arrow Key History Navigation

**Rationale:** Arrow navigation is entirely a frontend concern (store actions + `CommandInput.tsx`) and can proceed immediately after Phase 1 since it depends only on the store actions and `windowHistories` map being available. It is the lowest-complexity phase and delivers the most immediately tangible user value.

**Delivers:** ArrowUp/ArrowDown history navigation in `CommandInput.tsx`; draft preservation (`historyNavSnapshot`) on first ArrowUp; draft restoration on ArrowDown past end of history; `navigateHistoryUp`, `navigateHistoryDown`, `resetHistoryNav` store actions; history index reset on `hide()` and `submitQuery()`.

**Addresses:** Draft loss pitfall; multi-line cursor conflict pitfall; `historyNavIndex` in store (not component state) for lifecycle-safe reset.

**Avoids:** ArrowUp unconditional intercept, draft loss on accidental navigation.

### Phase 3: AI Follow-up Context Per Window

**Rationale:** Restoring `turnHistory` from the per-window map requires Phase 1 (window key and history map) to be complete. The `show()` async sequence must call `setActiveWindowKey` after `get_previous_window_key` resolves, which restores `turnHistory` before any query can be submitted. The `submitQuery()` change to persist `turnHistory` back to the map is the final wiring step.

**Delivers:** `turnHistory` restored from `windowHistories[activeWindowKey]` on each overlay open; `submitQuery()` persists updated `turnHistory` and `promptHistory` to the per-window map after each AI response; context-only-in-first-turn optimization in `ai.rs` to prevent token bloat; graceful fallback for non-terminal apps (`windowKey = "global"`).

**Addresses:** `show()` async timing (window key resolves before user can submit); AI context token bloat; stale CWD in follow-up context (always re-run `get_app_context` on open).

**Avoids:** Token bloat at follow-up turn 3+, stale context from history.

### Phase Ordering Rationale

- **Phase 1 must come first** because `window_id.rs`, `AppState` extension, and Rust `HashMap` storage are compile-time dependencies for the Tauri IPC commands that Phases 2 and 3 invoke. `cargo build` must succeed before any frontend work proceeds.
- **Phase 2 before Phase 3** because arrow navigation only needs `promptHistory` (read-only), while AI context needs `turnHistory` (read and write with async timing). Starting with the simpler read-only use case validates the store patterns before the more complex async restore flow is added.
- **Phases 2 and 3 could be parallelized** by two developers, but the `show()` modification in Phase 3 touches the same area as navigation wiring in Phase 2. Sequential is safer for a single developer.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Window identification):** The CGWindowListCopyWindowInfo FFI requires careful CFDictionary traversal in unsafe Rust. The `get_bundle_id` function referenced in `window_id.rs` must be verified as exported from `terminal/detect.rs`. Confirm `pbi_tdev` device number resolution via `devname_r()` if TTY-based key is chosen over CGWindowID. GPU terminal fallback (Alacritty, kitty, WezTerm) must be explicitly tested.
- **Phase 3 (AI context token management):** The `build_user_message` refactor in `ai.rs` to support first-turn-only terminal context needs an explicit token/character budget test. Validate the 6,000-character secondary cap against real multi-turn sessions before shipping.

Phases with standard patterns (skip deeper research):
- **Phase 2 (Arrow navigation):** Pure frontend. Cursor line position guard is a well-documented pattern. Draft cache is 10 lines. Implement directly from PITFALLS.md guidance without additional research.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | `core-graphics@0.25.0` API surface verified on docs.rs; Zustand update pattern verified in official docs; no new npm packages required |
| Features | HIGH | Feature set specified in PROJECT.md; cross-referenced against bash/zsh/Warp/Raycast behavior; dependency graph verified against live codebase |
| Architecture | HIGH | Build order derived from actual code dependency graph; component boundaries verified against live codebase files; FFI patterns match existing `detect.rs` and `permissions.rs` |
| Pitfalls | HIGH | Pitfalls derived from live codebase code review and macOS process API documentation; all prevention strategies include concrete code snippets |

**Overall confidence:** HIGH

### Gaps to Address

- **CGWindowListCopyWindowInfo deprecation longevity:** Apple marked this API deprecated in macOS 15 in the screen capture context. Research confirms it remains functional for window enumeration, but a future macOS release could remove it. A fallback to shell PID key is already designed and requires no architecture change. Monitor Apple developer release notes.

- **Shell PID vs. CGWindowID key choice:** FEATURES.md recommends shell PID (simpler, no extra permission risk); STACK.md and ARCHITECTURE.md recommend CGWindowID. Both are valid. The decision must be made at Phase 1 implementation start. Recommendation: use shell PID as primary, CGWindowID as a potential enhancement in a later milestone, since shell PID avoids any potential screen recording permission prompt and is sufficient for the majority of use cases.

- **GPU terminal fallback coverage:** Alacritty, kitty, and WezTerm may not expose AX or shell PID through the existing detection path. The fallback to `bundle_id:pid` (app-scoped) is designed and documented in the research, but must be explicitly tested during Phase 1 acceptance before shipping.

## Sources

### Primary (HIGH confidence)
- `core-graphics@0.25.0` docs.rs -- verified `CGWindowListCopyWindowInfo`, `kCGWindowNumber`, `kCGWindowOwnerPID` API presence
- `crates.io/crates/core-graphics` -- version 0.25.0 confirmed current
- `crates.io/crates/core-foundation` -- version 0.10.1 confirmed current
- Zustand official docs: Maps and Sets usage guide -- `new Map(state.foo).set(key, value)` pattern for re-render triggering
- Live codebase: `state.rs`, `hotkey.rs`, `terminal/process.rs`, `store/index.ts`, `CommandInput.tsx`, `ai.rs` -- all integration points verified against actual file contents

### Secondary (MEDIUM confidence)
- Apple Developer Documentation: CGWindowListCopyWindowInfo -- public API, window enumeration use case not deprecated in macOS 15
- pdubs and GetWindowID utilities -- confirmed CGWindowListCopyWindowInfo approach works for PID-to-window mapping in Rust/Swift
- Shell history UX patterns (devlog, DEV Community) -- confirmed arrow navigation and draft preservation behavior matches bash/zsh expectations
- macOS PID reuse documentation -- confirmed TTY-based keys are more robust than raw PIDs for long-running daemon sessions
- LLM context management strategies (getmaxim.ai) -- confirmed first-turn-only context approach for token budget management

### Tertiary (LOW confidence, needs validation)
- CGWindowListCopyWindowInfo Rust FFI forum discussion -- approach confirmed working but requires `unsafe` CFDictionary extraction; implementation details need validation during Phase 1

---
*Research completed: 2026-02-28*
*Ready for roadmap: yes*
