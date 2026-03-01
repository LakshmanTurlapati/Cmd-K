# Domain Pitfalls: Tauri macOS Overlay App with Terminal Integration

**Domain:** macOS overlay application -- per-terminal-window command history and AI follow-up context
**Researched:** 2026-02-28 (v0.1.1 milestone update; v0.1.0 pitfalls preserved below)
**Confidence:** HIGH (code reviewed against live codebase + verified with macOS process API docs)

---

## v0.1.1 Critical Pitfalls

Mistakes specific to adding per-terminal-window history and AI follow-up context to the existing system.

### Pitfall 1: Using App PID Alone as the Terminal Window Key

**What goes wrong:**
A `HashMap<i32, WindowHistory>` keyed on the terminal app PID (e.g., `com.googlecode.iterm2` at PID 12345) maps ALL windows and tabs of that terminal application to a single history entry. Opening a second iTerm2 window, or switching to a different tab, silently overwrites or mixes the history of unrelated sessions. The app already captures `previous_app_pid` in `AppState` -- this PID is app-level, not window-level.

**Why it happens:**
`NSRunningApplication.runningApplicationWithProcessIdentifier` returns the app process, not the window or tab. One iTerm2 process owns dozens of windows and hundreds of tabs at the same PID. Developers see PID already captured in `AppState` and use it directly as the map key because it's convenient.

**How to avoid:**
Use a composite window key: `bundle_id + ":" + tty_device_path`. The TTY path (`/dev/ttys003`) is unique per pseudo-terminal session.

The TTY is accessible without subprocesses: `proc_pidinfo(shell_pid, PROC_PIDTBSDINFO, ...)` returns a `proc_bsdinfo` struct containing `pbi_tdev` (the controlling terminal device number). Resolve that device number to a path via `devname_r()` or by matching `/dev/ttys*` entries. The shell PID is already found by `find_shell_pid()` in `process.rs` -- pass it to this call.

For GPU terminals (Alacritty, kitty, WezTerm) where AX is unavailable and shell discovery may fail, fall back to `bundle_id + ":" + terminal_app_pid` (app-scoped, coarse) and document the limitation.

Compute the key synchronously in the hotkey handler at the same point `previous_app_pid` is captured -- before `show_and_make_key()` is called. Store it in a new `previous_window_key: Mutex<Option<String>>` field in `AppState`.

**Warning signs:**
- History from one iTerm2 window appears in a freshly-opened second window
- Arrow-up in the overlay cycles through commands from a completely different project directory
- Shell type in history (zsh) mismatches the current window's shell (bash)

**Phase to address:**
Phase 1 (window identification). The key design must be locked in before history storage and arrow navigation are built -- everything downstream depends on it.

---

### Pitfall 2: PID Reuse Corrupting Stale History Entries

**What goes wrong:**
macOS recycles PIDs after processes exit. A shell that exits (user closes terminal tab) frees its PID. A new, unrelated shell may later receive the same PID. The history map then serves the new shell stale history from a completely different context -- wrong project, wrong directory, wrong commands.

**Why it happens:**
The history map has no mechanism to detect that a key's process has exited and its identity was reassigned. Maps grow over the app's daemon lifetime and are never cleaned up.

**How to avoid:**
- Use TTY-based keys rather than raw PIDs. TTY device paths (`/dev/ttys003`) are not reused while the pseudo-terminal is assigned; the kernel recycles them only after the window fully closes.
- Add a liveness check before serving cached history: call `get_process_name(shell_pid)` -- if it returns `None` or returns a non-shell name, the entry is stale, evict it.
- Include a `last_seen: Instant` in each history entry; evict entries not accessed within 4 hours.

**Warning signs:**
- Overlay shows history with a CWD that does not match the current shell
- Sporadic "wrong history" reports that are hard to reproduce (timing-dependent, only after PID recycle)
- Shell type in history does not match the current terminal session

**Phase to address:**
Phase 1 (window identification). Eviction logic belongs in the same data structure as the history map.

---

### Pitfall 3: Arrow Key Conflict with Textarea Cursor in Multi-line Input

**What goes wrong:**
Adding ArrowUp/ArrowDown history navigation to `CommandInput` works for single-line input but breaks when the user types a multi-line prompt (Shift+Enter). ArrowUp while the cursor is on line 2 of a 3-line input should move the cursor up one line within the textarea -- not navigate to a history entry. Intercepting all ArrowUp events unconditionally destroys the textarea's native cursor movement.

**Why it happens:**
The natural implementation intercepts `e.key === "ArrowUp"` at the top of `handleKeyDown` and calls `e.preventDefault()`. This works for single-line inputs (the only existing use case) but ignores the cursor's vertical position within multi-line content.

**How to avoid:**
Gate history navigation on cursor line position:

```typescript
if (e.key === "ArrowUp") {
  const el = textareaRef.current;
  if (!el) return;
  const textBefore = el.value.substring(0, el.selectionStart);
  const isOnFirstLine = !textBefore.includes("\n");
  if (isOnFirstLine) {
    e.preventDefault();
    navigateHistory(-1);
  }
  // else: let browser handle cursor movement within the textarea
}
if (e.key === "ArrowDown") {
  const el = textareaRef.current;
  if (!el) return;
  const textAfter = el.value.substring(el.selectionEnd);
  const isOnLastLine = !textAfter.includes("\n");
  if (isOnLastLine) {
    e.preventDefault();
    navigateHistory(+1);
  }
}
```

**Warning signs:**
- ArrowUp in a 2-line prompt jumps to a history entry instead of moving the cursor to line 1
- User cannot edit multi-line prompts with arrow keys after history feature is added
- Reports that the overlay "swallowed" cursor movement

**Phase to address:**
Phase 2 (arrow key navigation). Test explicitly with multi-line Shift+Enter prompts as a required acceptance criterion.

---

### Pitfall 4: History Stored in Zustand State Is Wiped on Every Overlay Open

**What goes wrong:**
`turnHistory` in the Zustand store is cleared on every `show()` call (line 214 in `store/index.ts`: `turnHistory: []`). If per-window history is stored as a plain Zustand array, it is wiped every time the user dismisses and re-opens the overlay -- defeating the purpose of persistent session history entirely.

**Why it happens:**
The existing `show()` intentionally resets all ephemeral state on each overlay open. This is correct for `streamingText`, `displayMode`, and destructive state, but wrong for per-window history, which must survive across multiple overlay invocations.

**How to avoid:**
Store per-window history in the Rust `AppState` as `Mutex<HashMap<String, WindowHistory>>`. The frontend fetches the current window's history via a Tauri command on each overlay open, and posts updates back after each completed turn. This decouples history persistence from the React component lifecycle entirely.

Do NOT attempt to split the Zustand store into "ephemeral" vs. "persistent" slices -- the `show()` reset is a single `set({...})` call that would require careful surgery and is easy to break. Backend storage is the cleaner approach.

**Warning signs:**
- History works within a single overlay session but is empty when the overlay is reopened
- Arrow-up navigation always starts from an empty state regardless of prior usage
- `console.log` shows `turnHistory: []` immediately after `show()` triggers

**Phase to address:**
Phase 1 (history storage architecture). This is an architecture decision that must precede any UI wiring.

---

### Pitfall 5: Unbounded History Map Growth in Long-Running Daemon Sessions

**What goes wrong:**
The Rust history map accumulates an entry for every terminal window and tab opened during the app's lifetime. A developer with 50 terminal tabs over a work week builds a map that never shrinks. Each entry holds up to 14 conversation turns; with verbose terminal output in context, each turn can be several kilobytes. Total memory footprint becomes measurable after heavy use. The app runs as a background daemon and is never restarted in normal use.

**Why it happens:**
HashMap entries are added on first overlay trigger from a new window but never removed. No eviction policy, no max-entries cap, no TTL.

**How to avoid:**
Implement LRU-style eviction:
- Cap total entries at a fixed limit (50 windows is generous).
- Track `last_accessed: SystemTime` per entry.
- On each insert or update: if `len() > cap`, evict the entry with the oldest `last_accessed`.
- Also evict entries not accessed within 4 hours regardless of cap.

```rust
fn evict_stale(map: &mut HashMap<String, WindowHistory>, cap: usize) {
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(4 * 3600))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    map.retain(|_, v| v.last_accessed > cutoff);
    while map.len() > cap {
        let oldest = map.iter()
            .min_by_key(|(_, v)| v.last_accessed)
            .map(|(k, _)| k.clone());
        if let Some(k) = oldest { map.remove(&k); } else { break; }
    }
}
```

Call `evict_stale` before each insert.

**Warning signs:**
- App memory usage climbs steadily over a multi-day session (visible in Activity Monitor)
- HashMap size visible via debug logging grows beyond expected terminal window count
- RSS grows proportionally with number of terminal tabs opened since last restart

**Phase to address:**
Phase 1 (history storage). Cap and eviction must be part of the initial data structure design.

---

### Pitfall 6: Conversation Context Tokens Grow Linearly with Follow-Up Depth

**What goes wrong:**
Each follow-up query to grok-3 includes the full history (up to 14 turns) PLUS terminal context (shell, CWD, up to 25 lines of terminal output) in the first user message. In a long session with verbose terminal output, the effective payload per API call can exceed 6,000-8,000 tokens by turn 4. The existing 10-second SSE timeout can trigger on context-heavy requests. xAI rate limits at the token level may start blocking rapid follow-up sequences.

**Why it happens:**
`build_user_message()` in `ai.rs` includes terminal context in every first user turn. When the history is replayed across follow-ups, this context is embedded in the history payload that gets sent to the API repeatedly. The current 14-message hard cap was designed for turn count, not total token budget.

**How to avoid:**
- Include full terminal context (CWD, shell output) only in the FIRST user message of a session. For follow-up turns, the user message is the bare query only (context is already established).
- Add a character-count secondary cap alongside the turn cap: if the sum of all history message content exceeds 6,000 characters, drop from the oldest end until under budget.
- Track whether the current query is a follow-up (turnHistory length > 0) and call a stripped `build_follow_up_message(query)` instead of the full `build_user_message(query, ctx)`.

**Warning signs:**
- grok-3 API calls start timing out after 2-3 follow-up turns (context size growing)
- xAI 429 rate limit errors appearing in rapid succession during follow-up sequences
- Response latency degrades noticeably at follow-up turn 3 or beyond

**Phase to address:**
Phase 3 (AI follow-up context). The `build_user_message` and history-passing logic in `ai.rs` needs deliberate refactoring alongside the history feature.

---

### Pitfall 7: Window Key Computed After Overlay Shows Causes TOCTOU Race

**What goes wrong:**
If the window key (bundle_id + TTY) is computed lazily inside `get_app_context` after the overlay has already shown and stolen focus, there is a race between the overlay appearing and the TTY lookup completing. If the user triggers Cmd+K from Window A, quickly switches to Window B before context detection finishes, the TTY resolved is Window B's but the event was triggered by Window A. The history assigned will belong to the wrong window.

**Why it happens:**
The existing "capture-before-show" pattern correctly captures `previous_app_pid` synchronously in the hotkey handler before `show_and_make_key()`. The window key must be captured with the same synchronicity. Any TTY lookup that forks a subprocess (`lsof`, `pgrep`) can take 50-200ms -- long enough for a context switch to occur.

**How to avoid:**
Compute the complete window key synchronously in the hotkey handler at the same point `previous_app_pid` is captured:

```rust
// In hotkey handler (before show_and_make_key):
let previous_pid = get_frontmost_pid();
let shell_pid = find_shell_pid_fast(previous_pid); // direct proc walk, no subprocesses
let window_key = compute_tty_key(previous_pid, shell_pid); // proc_pidinfo only
state.previous_window_key = Mutex::new(Some(window_key));
// THEN call show_and_make_key()
```

Use only `proc_pidinfo` with `PROC_PIDTBSDINFO` for TTY extraction -- this is a kernel syscall bounded to microseconds. Do not use `lsof` or `pgrep` in the key computation path.

**Warning signs:**
- On fast Cmd+K sequences (open from Window A, close, quickly open again from Window B), history from A appears in B
- History map shows keys that don't match the currently active terminal's TTY
- Race is intermittent and only reproducible under speed stress

**Phase to address:**
Phase 1 (window identification). Test with rapid alternating Cmd+K from two different terminal windows.

---

### Pitfall 8: History Navigation Draft Loss -- No Draft Cache on ArrowUp

**What goes wrong:**
User types a multi-word query ("list all docker containers sorted by size"), then accidentally presses ArrowUp to navigate history. The draft is replaced by a history entry. Pressing ArrowDown to return does not restore the original draft -- it was discarded.

**Why it happens:**
A naive history navigation implementation sets `inputValue` directly to the history entry without saving the current draft. Standard shell history navigation (bash/zsh) caches the current buffer and restores it on ArrowDown past the end of history -- but this behavior is not automatic.

**How to avoid:**
Cache the unsaved draft when navigation begins:

```typescript
const [historyDraft, setHistoryDraft] = useState<string | null>(null);
const [historyIndex, setHistoryIndex] = useState<number>(-1);

function navigateHistory(direction: -1 | 1) {
  if (direction === -1 && historyIndex === -1) {
    // Save draft before navigating away
    setHistoryDraft(inputValue);
  }
  const newIndex = historyIndex + direction;
  if (newIndex < -1) return; // Already at end
  if (newIndex >= history.length) return; // Already at oldest
  if (newIndex === -1) {
    // Restore draft
    setInputValue(historyDraft ?? "");
    setHistoryDraft(null);
  } else {
    setInputValue(history[newIndex]);
  }
  setHistoryIndex(newIndex);
}
```

**Warning signs:**
- Users report losing typed queries when accidentally pressing ArrowUp
- ArrowDown at the end of history shows empty input instead of the original draft

**Phase to address:**
Phase 2 (arrow key navigation). This is part of the correct history UX, not an edge case.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Key on PID alone (no TTY) | No change to AppState shape, works for single-window usage | History bleeds across iTerm2 tabs; cross-window contamination on PID reuse | Never -- TTY-based key is cheap to implement from the start |
| Store history in Zustand only | No Rust changes needed | History wiped on every overlay open; invisible to Rust-side context building | Never -- backend storage is needed for durability |
| No eviction policy on history map | Simpler code | Memory leak for long-running daemon in heavy-use environments | Acceptable in v0.1.1 only if a hard max_entries cap (20) is enforced |
| Include full terminal context in every history turn | Simpler `build_user_message` | Token bloat at follow-up 3+; rate limit risk | Never -- first-turn-only context is trivially implementable |
| Arrow-up always navigates history (no line check) | Simple 5-line implementation | Breaks multi-line input cursor movement | Never -- line-position check is 5 lines of code |
| Prefix-search history on arrow-up (bash-style) | More polish | Conflicts with simple index-based navigation; adds complexity | Defer to v0.2 -- plain up/down is sufficient for v0.1.1 |

---

## Integration Gotchas

Common mistakes when connecting history to existing components.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| xAI history replay | Including terminal context fields in every historical user message | Context goes only in the first user message of the session; subsequent turns send bare query text |
| `proc_pidinfo` PROC_PIDTBSDINFO | Treating `pbi_tdev` directly as a path string | `pbi_tdev` is a device number (major/minor encoded); resolve via `devname_r()` or match against `/dev/ttys*` entries |
| Tauri `AppState` Mutex | Holding the lock across `await` points | Hold `Mutex` guard only for map read/write; drop before any `await`; or use `tokio::sync::Mutex` for async-native locking |
| Zustand `show()` reset | Adding history fields to AppState and forgetting to exclude them from the show() reset | Keep window history in Rust backend; only fetch the relevant window's history into a local variable on show() |
| Arrow key `e.preventDefault()` | Calling it unconditionally in textarea `onKeyDown` | Gate on cursor line position (see Pitfall 3); only prevent default when actually navigating history |
| History turn trimming | Trimming from the newest end | Always trim from the oldest end (index 0); preserve the most recent N turns for coherent follow-up |
| Liveness check in history fetch | Calling `get_process_name` with a stale PID during context detection race | Capture shell PID in the hotkey handler and store it alongside the window key |

---

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| TTY device lookup using `lsof` in hotkey handler | 50-200ms added to overlay open latency; perceptible delay before overlay appears | Use `proc_pidinfo(PROC_PIDTBSDINFO)` only -- kernel syscall, no subprocess | Every hotkey trigger if `lsof` is in the synchronous path |
| History map Mutex held across `await` points | Deadlock or very long lock hold times; other Tauri commands block waiting for the mutex | Hold Mutex guard only for map read/write; drop before any async operation | First time a second Tauri command fires while history update is in-flight |
| Large `visible_output` in all history turns | API call latency grows 2-5x by follow-up 3-4 | Strip terminal context from history replay; context is first-turn-only | Follow-up query 3 or beyond in a verbose terminal session |
| `HashMap::clear()` not releasing memory | App RSS stays high after many terminal tabs are opened and closed | Call `map.shrink_to_fit()` after bulk removal, or use entry-by-entry `remove()` | After a session where 30+ terminal tabs were opened then closed |

---

## Security Mistakes

Domain-specific security issues for the history feature.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Persisting conversation history to disk (e.g., JSON file) | Terminal output may contain secrets (API keys, passwords from `cat .env`); disk persistence makes them recoverable | Keep history in-process only (Rust AppState); never persist to disk in v0.1.1 |
| Including full shell history from `~/.zsh_history` in AI context | Sends sensitive command sequences to xAI API without user awareness | Use only the current session's overlay-generated history, never the shell's own history file |
| Logging full history turns via `eprintln!` in production builds | Conversation content with potential secrets appears in system logs | Gate verbose history logging behind `#[cfg(debug_assertions)]` |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| No visual indicator that history navigation is available | User does not know ArrowUp history exists; feature is invisible | Show a subtle indicator (e.g., small arrow hint or turn count) in the overlay footer when history length > 0 |
| History navigation changes input immediately with no way to undo | User accidentally navigates to a long previous command and cannot get back to their draft | Cache the unsaved draft on first ArrowUp press; restore it on ArrowDown past end (Pitfall 8) |
| Follow-up context shows stale CWD from a previous session's history | User opens overlay from a new directory; AI responds as if in the old directory | Always refresh context on overlay open by re-running `get_app_context`; never use cached context from history turns as the current context |
| Arrow navigation wraps around (oldest entry goes back to newest) | User presses ArrowUp past oldest entry and ends up at newest entry; disorienting | Do not wrap; clamp at oldest entry (ArrowUp has no effect when at oldest) and at draft (ArrowDown has no effect when at draft) |

---

## "Looks Done But Isn't" Checklist

- [ ] **Window identification:** Works with single iTerm2 window -- verify it also isolates history between TWO simultaneously open iTerm2 windows with different CWDs
- [ ] **Arrow navigation:** Works at a single-line prompt -- verify ArrowUp does NOT intercept cursor movement when on line 2 of a Shift+Enter multi-line input
- [ ] **History persistence:** History survives overlay dismiss/reopen -- verify it is EMPTY for a brand-new terminal tab in the same app
- [ ] **Follow-up context:** AI receives correct history -- verify Rust-side API payload size does NOT grow linearly with terminal output size after 3 turns
- [ ] **GPU terminal fallback:** History isolated correctly for iTerm2 -- verify Alacritty (no AX/TTY) falls back to app-scoped key gracefully, not a crash
- [ ] **Reset-on-show integrity:** History persists -- verify ALL other ephemeral state (streamingText, isDestructive, displayMode) still resets correctly on show()
- [ ] **Rapid Cmd+K:** Window key correct under speed stress -- verify no TOCTOU race by alternating Cmd+K from two different terminal windows in quick succession
- [ ] **Draft preservation:** ArrowUp and back down restores the original unsaved draft, not an empty string

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| PID-only key discovered after history UI is built | HIGH | Refactor AppState key type from `i32` to `String`; update all callers of history map; re-test all history scenarios |
| History stored in Zustand (wiped on show) | HIGH | Migrate to backend HashMap; add `get_window_history` and `update_window_history` Tauri commands; update frontend to call them on show() and after each turn |
| No eviction causing memory growth | LOW | Add `evict_stale()` call on each history update; no API surface change required |
| Terminal context in all history turns (token bloat) | MEDIUM | Refactor `build_user_message` to detect first vs. follow-up turn; no state change required, only logic in `ai.rs` |
| Arrow key eating multi-line cursor movement | LOW | Add `isOnFirstLine`/`isOnLastLine` guards to existing `handleKeyDown`; localized to `CommandInput.tsx` |
| Draft loss on ArrowUp | LOW | Add `historyDraft` and `historyIndex` state; cache draft on first navigation; restore on ArrowDown past end |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| PID alone as window key | Phase 1: Window identification | Test two simultaneously open iTerm2 windows; verify separate histories |
| PID reuse corrupting history | Phase 1: Window identification | Close a terminal tab; verify liveness check evicts the stale entry |
| History in Zustand wiped on show | Phase 1: History storage architecture | Verify history survives 5 overlay open/close cycles |
| Unbounded map growth | Phase 1: History storage architecture | Simulate 100 window events; verify map size stays at or below cap |
| Window key computed after overlay shows | Phase 1: Window identification | Rapid alternating Cmd+K test between two terminal windows |
| Arrow key vs. multi-line cursor | Phase 2: Arrow key navigation | ArrowUp on line 2 of Shift+Enter multi-line prompt does not navigate history |
| Draft loss on ArrowUp | Phase 2: Arrow key navigation | ArrowUp then ArrowDown restores the original draft |
| Token bloat in follow-up context | Phase 3: AI follow-up context | Log API payload size for turns 1-5; verify no linear growth with terminal output |

---

## v0.1.0 Foundation Pitfalls

These remain applicable and are preserved from the original research (February 21, 2026). They are already resolved in the shipped v0.1.0 code.

### Pitfall 9: Sandboxing Incompatible with Accessibility API

**What goes wrong:** You cannot use the macOS Accessibility API from a sandboxed app. If sandboxing is enabled (required for App Store distribution), terminal state reading and paste into other apps both fail.

**Prevention:** DO NOT enable `com.apple.security.app-sandbox`. Use Developer ID distribution (notarization only). Already resolved in v0.1.0.

**Phase to address:** Resolved. Foundation phase.

---

### Pitfall 10: AppleScript Command Injection via Unsanitized AI Responses

**What goes wrong:** AI responses containing backticks or semicolons interpolated directly into AppleScript `do script` commands allow arbitrary terminal command execution.

**Prevention:** Never directly interpolate AI responses into AppleScript strings. Escape properly or use temp file intermediary. Already resolved in v0.1.0 with the paste-not-execute approach.

**Phase to address:** Resolved. Terminal integration phase.

---

### Pitfall 11: Transparent Window Rendering Glitches on macOS Sonoma

**What goes wrong:** Transparent Tauri windows exhibit visual artifacts after focus changes on Sonoma with Stage Manager.

**Prevention:** Set `ActivationPolicy::Accessory` before any window operations. Already resolved in v0.1.0 (`lib.rs` line 55).

**Phase to address:** Resolved. Foundation phase.

---

### Pitfall 12: Global Hotkey Cmd+K Fires Twice on macOS

**What goes wrong:** Tauri global-shortcut plugin bug (#10025) causes double-fire of hotkey events.

**Prevention:** 200ms debounce via `last_hotkey_trigger` timestamp in AppState. Already resolved in v0.1.0.

**Phase to address:** Resolved. Foundation phase.

---

### Pitfall 13: Accessibility Permissions Silently Fail Without Prompting User

**What goes wrong:** macOS Accessibility requires manual user action in System Settings; no automatic prompt. Failures are silent.

**Prevention:** AX probe fallback for permission detection; onboarding wizard guides user. Already resolved in v0.1.0.

**Phase to address:** Resolved. Foundation phase.

---

### Pitfall 14: Non-Focusable Windows Still Steal Focus on macOS

**What goes wrong:** `focusable: false` does not work on macOS in Tauri v2.

**Prevention:** Use NSPanel (tauri-nspanel) with `nonactivating_panel` style mask and `can_become_key_window: true`. Already resolved in v0.1.0.

**Phase to address:** Resolved. Foundation phase.

---

### Pitfall 15: Streaming xAI Responses Cause IPC Performance Bottleneck

**What goes wrong:** Streaming large AI responses through Tauri commands causes UI lag and high CPU if not chunked properly.

**Prevention:** Use Tauri IPC Channel for streaming; forward individual tokens as they arrive. Already resolved in v0.1.0 (`stream_ai_response` in `ai.rs`).

**Phase to address:** Resolved. AI integration phase.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 1: Window identification | Using app PID as map key | Compute bundle_id + TTY composite key synchronously in hotkey handler |
| Phase 1: History storage | Zustand state wiped on show | Store history in Rust AppState HashMap, fetch on overlay open |
| Phase 1: History storage | Unbounded map growth | Enforce cap + LRU eviction from the start |
| Phase 2: Arrow key navigation | Multi-line input cursor conflict | Gate ArrowUp/Down on cursor line position |
| Phase 2: Arrow key navigation | Draft loss on accidental navigation | Cache draft before first ArrowUp; restore on ArrowDown past end |
| Phase 3: AI follow-up context | Token bloat at follow-up 3+ | Context in first user message only; character-count secondary cap on history |
| Phase 3: AI follow-up context | Stale context in follow-up | Re-run `get_app_context` on every overlay open; never use history-embedded context as current context |

---

## Sources

- Codebase review: `src-tauri/src/state.rs`, `src-tauri/src/commands/terminal.rs`, `src-tauri/src/terminal/process.rs`, `src-tauri/src/terminal/ax_reader.rs`, `src/store/index.ts`, `src/hooks/useKeyboard.ts`, `src/components/CommandInput.tsx`, `src-tauri/src/commands/ai.rs`
- macOS process API: [proc_pidinfo library (mmastrac)](https://github.com/mmastrac/proc_pidinfo), [macOS PID reuse (HackTricks)](https://book.hacktricks.wiki/en/macos-hardening/macos-security-and-privilege-escalation/macos-proces-abuse/macos-ipc-inter-process-communication/macos-xpc/macos-xpc-connecting-process-check/macos-pid-reuse.html)
- macOS window identification: [GetWindowID utility (smokris)](https://github.com/smokris/GetWindowID), [iTerm2 Variables documentation](https://iterm2.com/documentation-variables.html)
- TTY session identification: [The TTY demystified (Linus Akesson)](https://www.linusakesson.net/programming/tty/)
- LLM context management: [Context Window Management Strategies (getmaxim.ai)](https://www.getmaxim.ai/articles/context-window-management-strategies-for-long-context-ai-agents-and-chatbots/), [LLM performance degradation at context limits](https://demiliani.com/2025/11/02/understanding-llm-performance-degradation-a-deep-dive-into-context-window-limits/)
- xAI Grok token limits: [xAI Models and Pricing](https://docs.x.ai/developers/models), [Grok 3 context window analysis](https://www.byteplus.com/en/topic/568342)
- Tauri state management: [Tauri v2 State Management docs](https://v2.tauri.app/develop/state-management/), [Tauri HashMap state discussion](https://github.com/tauri-apps/tauri/discussions/7279)
- Rust HashMap memory: [Handling memory leaks in Rust (LogRocket)](https://blog.logrocket.com/handling-memory-leaks-rust/)
- Arrow key in React textarea: [stopPropagation for input navigation (DEV Community)](https://dev.to/rajeshroyal/practical-use-of-the-eventstoppropagation-react-mui-data-grid-allowing-arrow-key-navigation-for-571a)

---
*Pitfalls research for: per-terminal-window command history and AI follow-up context (Tauri v2 macOS overlay, v0.1.1 milestone)*
*Researched: 2026-02-28*
