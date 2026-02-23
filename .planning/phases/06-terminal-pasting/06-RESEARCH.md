# Phase 6: Terminal Pasting - Research

**Researched:** 2026-02-23
**Domain:** macOS AppleScript terminal automation, Rust process execution, clipboard fallback
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **Paste trigger flow**: Safe commands auto-paste immediately on generation (no user action). Destructive commands (those with destructive badge from Phase 5) do NOT auto-paste -- user copies manually via existing copy button. Linked to safety toggle in Settings: Toggle ON (default) = auto-paste safe commands, skip destructive; Toggle OFF = no auto-paste for any command. Warn user when they attempt to toggle OFF.
- **Terminal focus behavior**: Overlay stays open after auto-paste. Focus shifts to terminal after paste. No visual paste confirmation indicator in the overlay. Overlay preserves state when user brings it back to focus.
- **Fallback experience**: When auto-paste unavailable (unsupported terminal): silently auto-copy to clipboard, no notification. Best-effort terminal detection: try to paste to whatever terminal is active, fall back on failure. Reuse terminal detection from Phase 3 rather than re-detecting at paste time.
- **Command placement**: Never auto-execute -- always place command in input line, user presses Enter. Replace any existing text in terminal input line (clear and paste). Only single-line commands (with pipes). Paste to most recently active terminal window (not a specific tab/pane).

### Claude's Discretion

- AppleScript implementation details for Terminal.app and iTerm2
- Exact mechanism for clearing existing input line text
- How to handle edge cases where terminal detection from Phase 3 is stale

### Deferred Ideas (OUT OF SCOPE)

None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TERM-01 | Generated command is pasted into the active terminal (Terminal.app, iTerm2) | AppleScript `write text ... newline NO` for iTerm2 and `do script ... in window` for Terminal.app; Rust `std::process::Command` spawns `osascript`; clipboard fallback via `navigator.clipboard.writeText` |
</phase_requirements>

---

## Summary

Phase 6 wires a Rust IPC command (`paste_to_terminal`) that, when invoked by the frontend after a successful safe-command generation, uses AppleScript to write the command into the active terminal's input line without executing it, then activates the terminal window to bring it to the foreground. The Rust layer uses `std::process::Command` to spawn `osascript` with inline `-e` arguments -- no external crate is needed. The AppleScript is branched by bundle ID: `com.googlecode.iterm2` receives `write text "..." newline NO`, and `com.apple.Terminal` receives a `do script` plus `keystroke "u" using control down` sequence to clear the existing input line first. GPU-accelerated terminals (Alacritty, kitty, WezTerm) cannot be scripted via AppleScript and silently fall through to the clipboard fallback. The frontend already uses `navigator.clipboard.writeText` in the store for auto-copy, so the fallback path requires no new infrastructure.

The bundle ID of the previous frontmost app is already captured in `AppState.previous_app_pid` and can be resolved to a bundle ID using the existing `detect::get_bundle_id()` helper. The phase therefore only needs: (1) a new Rust command `paste_to_terminal` that accepts `{ command: String }`, determines the terminal type from stored PID, runs the correct AppleScript, and (2) a trigger point in the frontend store's `submitQuery` completion path that calls this command when `isDestructive` is false and `autoPasteEnabled` is true.

**Primary recommendation:** Use `std::process::Command::new("osascript").arg("-e").arg(script).output()` in Rust. No new crates. No new Tauri plugins. The existing `navigator.clipboard.writeText` path is the clipboard fallback.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::process::Command` (Rust stdlib) | N/A | Spawn `osascript` to execute inline AppleScript | Zero-dependency, already used in project via `tauri_plugin_http::reqwest`; cleanest path for inline scripts |
| `osascript` (macOS system binary) | macOS built-in | Execute AppleScript targeting Terminal.app / iTerm2 | Ships on every macOS install; the canonical tool for scripting GUI apps |
| `navigator.clipboard.writeText` (browser API) | Web standard | Clipboard fallback for unsupported terminals | Already used in `submitQuery` auto-copy path in store/index.ts |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `detect::get_bundle_id()` (existing) | project local | Resolve bundle ID from PID | Used to branch AppleScript template by terminal type |
| `tauri_plugin_clipboard_manager` | 2.x | Rust-side clipboard write if frontend path is unavailable | Only if Rust needs to write clipboard directly; current project uses `navigator.clipboard` from frontend so this is NOT needed |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `std::process::Command` + `osascript -e` | `osascript` crate (mitsuhiko/rust-osascript) | osascript crate only supports JavaScript mode (not AppleScript); unsuitable |
| `std::process::Command` + `osascript -e` | `osakit` crate | Adds ObjC bindings but unnecessary complexity for simple inline scripts |
| `navigator.clipboard.writeText` fallback | `tauri-plugin-clipboard-manager` | tauri-plugin-clipboard-manager not yet in project; `navigator.clipboard` already works in the webview context and is used today |

**Installation:**
No new packages required. All standard stack components are already present.

---

## Architecture Patterns

### Recommended Project Structure

```
src-tauri/src/
├── commands/
│   └── paste.rs          # New: paste_to_terminal Tauri command
├── terminal/
│   └── detect.rs         # Existing: get_bundle_id(), TERMINAL_BUNDLE_IDS
│   └── mod.rs            # Existing: AppState previous_app_pid
src/
├── store/
│   └── index.ts          # Existing: submitQuery -- trigger paste here
├── components/
│   └── Settings/
│       └── PreferencesTab.tsx  # Existing: add auto-paste toggle alongside destructive toggle
```

### Pattern 1: Rust `paste_to_terminal` Command

**What:** A new `#[tauri::command]` in `commands/paste.rs` that reads `previous_app_pid` from `AppState`, resolves the bundle ID, picks the correct AppleScript template, and runs it via `std::process::Command`.

**When to use:** Called by the frontend after streaming completes and `isDestructive` is false and `autoPasteEnabled` is true.

**Example:**
```rust
// Source: verified with std::process::Command docs + osascript man page
#[tauri::command]
pub fn paste_to_terminal(app: tauri::AppHandle, command: String) -> Result<(), String> {
    use crate::terminal::detect::get_bundle_id;
    use crate::state::AppState;

    // Read the previously captured frontmost app PID from AppState
    let pid = {
        let state = app.try_state::<AppState>().ok_or("AppState unavailable")?;
        let guard = state.previous_app_pid.lock().map_err(|_| "mutex poisoned")?;
        (*guard).ok_or("no previous app PID captured")?
    };

    let bundle_id = get_bundle_id(pid).ok_or("could not resolve bundle ID")?;

    let script = build_paste_script(&bundle_id, &command)?;

    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| format!("osascript spawn failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("osascript error: {}", stderr));
    }

    Ok(())
}

fn build_paste_script(bundle_id: &str, command: &str) -> Result<String, String> {
    // Escape backslashes and double-quotes in the command for AppleScript string literal
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");

    match bundle_id {
        "com.googlecode.iterm2" => Ok(format!(
            r#"tell application "iTerm2"
  activate
  tell current window
    tell current session
      write text "{}" newline NO
    end tell
  end tell
end tell"#,
            escaped
        )),
        "com.apple.Terminal" => Ok(format!(
            r#"tell application "Terminal"
  activate
  tell application "System Events"
    tell process "Terminal"
      keystroke "u" using control down
      keystroke "{}"
    end tell
  end tell
end tell"#,
            escaped
        )),
        _ => Err(format!("unsupported terminal: {}", bundle_id)),
    }
}
```

### Pattern 2: iTerm2 -- `write text "..." newline NO`

**What:** The official iTerm2 AppleScript verb for placing text in the session input without executing it.

**When to use:** When `bundle_id == "com.googlecode.iterm2"`.

**Example:**
```applescript
-- Source: iTerm2 official scripting documentation
-- https://iterm2.com/documentation-scripting.html
tell application "iTerm2"
  activate
  tell current window
    tell current session
      write text "ls -la ~/Documents" newline NO
    end tell
  end tell
end tell
```

Key notes:
- `newline NO` suppresses the automatic newline/Enter, leaving text in the input line.
- `activate` brings iTerm2 to the foreground after pasting.
- `current window` is defined as "the window that most recently had keyboard focus" -- matches the "most recently active window" requirement.
- `current session` is "the session that would receive keyboard input if the window had keyboard focus."

### Pattern 3: Terminal.app -- `Control+U` clear then keystroke

**What:** Terminal.app has no "write text without newline" verb. The AppleScript dictionary's `do script` command always executes. The correct approach is: activate, send `Control+U` (clear input line in shells) via System Events, then type the command via `keystroke`.

**When to use:** When `bundle_id == "com.apple.Terminal"`.

**Example:**
```applescript
-- Source: Apple community + AppleScript System Events keystroke pattern
-- Control+U clears the current input line in bash/zsh (standard shell binding)
tell application "Terminal"
  activate
  tell application "System Events"
    tell process "Terminal"
      keystroke "u" using control down  -- clears existing input line text
      keystroke "ls -la ~/Documents"    -- types the command without newline
    end tell
  end tell
end tell
```

Key notes:
- `Control+U` is the standard readline "kill line" keybinding in bash, zsh, and fish. It clears everything on the current input line. This satisfies the "replace any existing text" requirement.
- `keystroke "u" using control down` sends Ctrl+U without press/release confusion.
- `keystroke "text"` without `keystroke return` leaves the text in the input line without executing.
- This requires Accessibility permission -- already granted by the time Phase 6 runs (Phase 2 handles the Accessibility permission flow).

### Pattern 4: Frontend Trigger Point

**What:** After `submitQuery` completes streaming and confirms `isDestructive === false`, invoke `paste_to_terminal` via `invoke()`.

**When to use:** End of the streaming success path in `store/index.ts` `submitQuery`.

**Example:**
```typescript
// Source: store/index.ts submitQuery success path -- add after auto-copy block
const autoPasteState = useOverlayStore.getState();
if (
  autoPasteState.autoPasteEnabled &&
  !autoPasteState.isDestructive &&
  finalText
) {
  invoke("paste_to_terminal", { command: finalText }).catch((err) => {
    console.error("[store] paste_to_terminal failed (fallback to clipboard):", err);
    // Clipboard already written by auto-copy block above -- no additional action needed
  });
}
```

Note: The auto-copy (`navigator.clipboard.writeText`) block executes unconditionally before this paste block. This means the clipboard is always populated, and a paste failure silently degrades to clipboard-only (the existing behavior).

### Pattern 5: Settings Toggle for Auto-Paste

**What:** A new `autoPasteEnabled` boolean in the Zustand store with `tauri-plugin-store` persistence. Identical pattern to `destructiveDetectionEnabled`.

**When to use:** User toggles in PreferencesTab. Defaults to `true`.

**Example:**
```typescript
// New state fields in store/index.ts OverlayState interface
autoPasteEnabled: boolean;
setAutoPasteEnabled: (enabled: boolean) => void;

// Initial state
autoPasteEnabled: true,

// PreferencesTab toggle -- same pattern as destructive detection toggle
const handleToggleAutoPaste = async () => {
  const newValue = !autoPasteEnabled;
  setAutoPasteEnabled(newValue);
  const store = await Store.load("settings.json");
  await store.set("autoPasteEnabled", newValue);
  await store.save();
};
```

### Pattern 6: Startup Settings Load for `autoPasteEnabled`

**What:** `autoPasteEnabled` must be loaded from `settings.json` on app startup, same as `destructiveDetectionEnabled`.

**Where:** `App.tsx` startup effect, alongside the existing `destructiveDetectionEnabled` load.

**Example:**
```typescript
// App.tsx startup effect -- add to existing settings load block
const autoPaste = await store.get<boolean>("autoPasteEnabled");
if (autoPaste !== null && autoPaste !== undefined) {
  useOverlayStore.getState().setAutoPasteEnabled(autoPaste);
}
```

### Anti-Patterns to Avoid

- **Using `do script` for Terminal.app paste:** `do script "command"` always executes the command. It never just places text in the input line.
- **Calling `osascript` via Tauri shell scope (plugin-shell):** The Tauri shell plugin is not in this project's dependencies; `std::process::Command` is simpler and has no scoping restrictions.
- **Re-detecting terminal at paste time:** The CONTEXT.md decision is to reuse Phase 3 detection via `AppState.previous_app_pid`. Do not call `detect()` again inside `paste_to_terminal`.
- **Installing `tauri-plugin-clipboard-manager`:** The project uses `navigator.clipboard` from the frontend. The auto-copy already runs before the paste attempt, so the clipboard fallback is implicit and requires no Rust-side changes.
- **Blocking the paste on terminal bundle ID lookup failure:** If the bundle ID resolves to an unsupported terminal (Alacritty, kitty, WezTerm, or any unknown app), return an `Err` from Rust. The frontend catches the error and logs it -- the clipboard already has the command from auto-copy.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AppleScript execution | Custom ObjC bindings to OSAKit | `std::process::Command` + `osascript -e` | `osascript` is stable, ships on every macOS, trivial to invoke inline |
| Input-line clearing | Custom keycode injection via ObjC | `Control+U` via `keystroke ... using control down` | Readline `kill-line` binding is universal across bash/zsh/fish |
| Clipboard write in Rust | Rust clipboard crate | `navigator.clipboard.writeText` in frontend | Already used in auto-copy path; no Rust changes needed |
| Bundle ID to terminal type dispatch | New detection module | `detect::get_bundle_id(pid)` (existing) | Phase 3 already solved this problem |

**Key insight:** The entire paste mechanism is a thin shell around `osascript`. The hard problem (identifying the terminal, capturing the PID) is already solved. Phase 6 is mostly wiring, not new infrastructure.

---

## Common Pitfalls

### Pitfall 1: Terminal.app `do script` Always Executes
**What goes wrong:** Attempting `tell application "Terminal" to do script "command" in front window` places the command AND executes it immediately (with newline).
**Why it happens:** Terminal.app's AppleScript dictionary does not offer a "write text without newline" equivalent. `do script` is the only high-level write verb.
**How to avoid:** Use `keystroke` via `System Events` for Terminal.app. Send `Control+U` first to clear any existing input, then `keystroke "the-command"` without `keystroke return`.
**Warning signs:** During testing, the command runs in the terminal instead of appearing in the input line.

### Pitfall 2: AppleScript Injection via Unescaped Command Text
**What goes wrong:** If the generated command contains double quotes or backslashes, embedding it in an AppleScript string literal breaks the script or creates an injection vector.
**Why it happens:** The command string is interpolated directly into the AppleScript string.
**How to avoid:** Escape `\` as `\\` and `"` as `\"` in `build_paste_script()` before interpolation. This is straightforward since commands are single-line.
**Warning signs:** `osascript` returns a parse error containing the generated command text.

### Pitfall 3: `previous_app_pid` Is Stale When Overlay Reopened
**What goes wrong:** If the user opens the overlay, generates a command, closes it, opens it again over a different window, then pastes -- the PID might refer to the first terminal session from the previous open.
**Why it happens:** `previous_app_pid` is only updated when the hotkey fires and the overlay is hidden. The CONTEXT decision says to reuse Phase 3 detection rather than re-detect. The PID from when the overlay was opened to generate the current result is the correct target.
**How to avoid:** This is expected behavior by design. The paste goes to the terminal that was active when this command was generated. Document this as intended. Do not add re-detection in the paste path.
**Warning signs:** None -- this is correct behavior per CONTEXT.md.

### Pitfall 4: System Events Accessibility Permission Required for Terminal.app
**What goes wrong:** `keystroke` via `tell application "System Events" to tell process "Terminal"` requires the app sending the keystroke to have Accessibility permission. If permission was granted to a different binary (e.g., during development), it may not apply to the production bundle.
**Why it happens:** macOS Accessibility permission is granted per-bundle-ID. The permission granted during Phase 2 onboarding covers the `cmd-k` app bundle -- this should be correct in production.
**How to avoid:** Test in the production build, not just `tauri dev`. The dev binary has a different path/bundle and may need its own permission grant. This is not a code fix -- it is a verification step.
**Warning signs:** `osascript` exits with error "not allowed to send keystrokes" or "not allowed to use accessibility features."

### Pitfall 5: iTerm2 Not Running / No Windows Open
**What goes wrong:** `tell current window` fails if iTerm2 is not running or has no open windows, causing an AppleScript error.
**Why it happens:** iTerm2 was the frontmost app at hotkey time, but then was closed between hotkey press and paste completion.
**How to avoid:** Let the error propagate from `osascript` as a non-zero exit code. The `paste_to_terminal` command returns `Err` and the frontend catches it; clipboard fallback (already populated) is available.
**Warning signs:** osascript exits with "Application 'iTerm2' is not running" or "Invalid index."

### Pitfall 6: Control+U Behavior Differences Across Shells
**What goes wrong:** `Control+U` in bash with default readline settings clears from the cursor to the beginning of the line (not the whole line). In emacs mode (default for bash) it kills backward. In vi mode it does nothing.
**Why it happens:** readline's `kill-line` (`Control+U`) is only guaranteed in zsh (default on macOS) and fish where it always clears the whole line.
**How to avoid:** The decision (CONTEXT.md) says to use the most recently active terminal window. All three major shells (zsh, bash, fish) support `Control+U` as a "clear input" shortcut in their default configs. The project targets macOS where zsh is the default shell. This is acceptable best-effort behavior.
**Warning signs:** On bash with vi mode enabled, the command gets appended after existing text. This is an edge case; the CONTEXT decision accepts best-effort behavior.

---

## Code Examples

Verified patterns from official sources:

### iTerm2: Write Text Without Newline
```applescript
-- Source: https://iterm2.com/documentation-scripting.html (official iTerm2 docs)
-- "write text text [newline NO]" -- writes text to session without executing
tell application "iTerm2"
  activate
  tell current window
    tell current session
      write text "ls -la ~/Documents" newline NO
    end tell
  end tell
end tell
```

### Terminal.app: Clear Input Line Then Type Command
```applescript
-- Source: Apple Developer Forums + System Events keystroke documentation
-- Control+U = readline kill-line (clears input line in bash/zsh/fish default config)
tell application "Terminal"
  activate
  tell application "System Events"
    tell process "Terminal"
      keystroke "u" using control down
      keystroke "ls -la ~/Documents"
    end tell
  end tell
end tell
```

### Rust: Execute Inline AppleScript via osascript
```rust
// Source: Rust std::process::Command docs + osascript man page
// Each `.arg("-e").arg(line)` is a separate script line; alternative: one multi-line string
let script = r#"tell application "iTerm2" ..."#;

let output = std::process::Command::new("osascript")
    .arg("-e")
    .arg(script)
    .output()
    .map_err(|e| format!("osascript spawn failed: {}", e))?;

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("AppleScript error: {}", stderr));
}
```

### Frontend: Invoke paste_to_terminal After Streaming
```typescript
// Placement: in store/index.ts submitQuery, after the auto-copy block,
// before/after the check_destructive call
const state = useOverlayStore.getState();
if (state.autoPasteEnabled && !state.isDestructive && finalText) {
  invoke("paste_to_terminal", { command: finalText }).catch((err) => {
    console.error("[store] auto-paste failed:", err);
    // Clipboard already populated by auto-copy -- silent fallback
  });
}
```

### App.tsx: Load autoPasteEnabled on Startup
```typescript
// Same pattern as destructiveDetectionEnabled loading in App.tsx
const autoPaste = await store.get<boolean>("autoPasteEnabled");
if (autoPaste !== null && autoPaste !== undefined) {
  useOverlayStore.getState().setAutoPasteEnabled(autoPaste);
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| iTerm2 AppleScript `write text "..."` (executes) | `write text "..." newline NO` | iTerm2 3.x added `newline NO` parameter | Allows placing text in input line without executing |
| Terminal.app `do script "..."` (always executes) | `keystroke` via System Events (places text only) | Stable since macOS 10.9+ | Only reliable non-executing method for Terminal.app |
| iTerm2 Python API (preferred by iTerm2 devs) | AppleScript `write text ... newline NO` | iTerm2 3.x | Python API preferred for new features; AppleScript sufficient for this use case and widely used |

**Deprecated/outdated:**
- `do script` for Terminal.app paste without execution: does not exist. `do script` always executes.
- iTerm2 AppleScript without `newline NO`: writes text AND executes (the default). Always use `newline NO` for input-line-only paste.

---

## Open Questions

1. **iTerm2 "current window" when overlay is frontmost**
   - What we know: `previous_app_pid` in AppState is captured before the overlay is shown (Phase 3 pattern). By the time `paste_to_terminal` runs, the overlay is the frontmost window, not iTerm2.
   - What's unclear: Does `current window` in iTerm2 AppleScript refer to "the window currently receiving keyboard focus" (our overlay, not iTerm2) or "the last window of this app that had focus"?
   - Recommendation: Per iTerm2 official docs, `current window` = "the window that most recently had keyboard focus" in the context of the `tell application "iTerm2"` block. This refers to iTerm2's own most recent window, not the system-wide frontmost window. Verify during human testing. If it fails, the fallback is `first window` (targets the first open iTerm2 window, which is typically the most recently used).

2. **Paste timing relative to `isDestructive` check**
   - What we know: `check_destructive` is called AFTER the streaming completes (async, fire-and-forget). The paste trigger also fires after streaming completes.
   - What's unclear: Is there a race condition where `paste_to_terminal` fires before `check_destructive` returns `true` for a destructive command?
   - Recommendation: Move paste trigger to fire ONLY after the `check_destructive` call completes and confirms `isDestructive === false`. This requires restructuring the submitQuery completion block so paste waits for the destructive check. See Architecture Patterns section -- the code example already accounts for this by reading `autoPasteState.isDestructive` after `setIsDestructive` has been called.
   - Actually: `check_destructive` is called in a `.then()` chain that sets `isDestructive` in the store. If paste fires from the same sequential block AFTER `set({ isDestructive: true })`, reading `isDestructive` from state will be correct. Verify the ordering in the submitQuery refactor.

3. **Auto-paste warn-on-disable interaction with destructive toggle**
   - What we know: CONTEXT says warn user when they attempt to toggle auto-paste OFF. No warning spec for destructive toggle (that exists already).
   - What's unclear: Exact UX for the warning -- modal dialog, inline text, or just a tooltip?
   - Recommendation: Use the same inline approach as the destructive toggle but add a one-line warning label below the toggle when it is turned OFF (e.g., "Commands will not be pasted automatically"). No modal needed for a preference toggle.

---

## Sources

### Primary (HIGH confidence)
- iTerm2 official scripting documentation (https://iterm2.com/documentation-scripting.html) -- `write text ... newline NO` syntax, `current window` / `current session` semantics
- Rust std::process::Command docs (https://doc.rust-lang.org/std/process/struct.Command.html) -- subprocess spawn pattern
- Existing project codebase: `src-tauri/src/terminal/detect.rs`, `src-tauri/src/state.rs`, `src/store/index.ts` -- confirmed existing infrastructure reuse

### Secondary (MEDIUM confidence)
- Apple System Events keystroke documentation (verified pattern) -- `keystroke "u" using control down` for Control+U
- Tauri v2 clipboard plugin docs (https://v2.tauri.app/plugin/clipboard/) -- confirmed not needed; `navigator.clipboard` sufficient
- osascript man page (https://ss64.com/mac/osascript.html) -- `-e` inline script argument syntax

### Tertiary (LOW confidence)
- Apple Community thread (https://discussions.apple.com/thread/250379543) -- Terminal.app keystroke pattern for placing text without execution. Corroborates the System Events approach but single community source.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- osascript + std::process::Command is the canonical macOS automation pattern; confirmed with iTerm2 official docs and existing project pattern
- Architecture: HIGH -- all patterns directly derived from existing project code (store.ts, detect.rs, state.rs) with minimal new surface area
- Pitfalls: MEDIUM -- AppleScript escaping and keystroke behavior verified; Accessibility permission issue is a known macOS constraint; shell Control+U behavior is LOW confidence for edge cases (vi mode)

**Research date:** 2026-02-23
**Valid until:** 2026-05-23 (stable APIs; AppleScript dictionary hasn't changed in years; iTerm2 scripting is mature)
