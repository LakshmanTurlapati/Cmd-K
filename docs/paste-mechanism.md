# Paste Mechanism -- Architecture & Debugging Record

## Overview

CMD+K pastes commands into the user's terminal by synthesizing keystrokes after the overlay is dismissed. The mechanism varies by terminal application due to macOS-specific behaviors around CGEvent posting, Accessibility (TCC), and Automation permissions.

---

## Paste Strategies by Terminal

| Terminal | Strategy | Why |
|---|---|---|
| **iTerm2** | AppleScript `write text` (direct AppleEvent) | Native API, most reliable. Chunks text in groups of 7 chars with 16ms delays to avoid dropped characters. Falls back to Cmd+V on failure. |
| **Terminal.app** | System Events `keystroke` via osascript | Both CGEvent unicode and CGEvent Cmd+V are unreliable after TCC resets. System Events is the only method that works consistently. |
| **All others** (Cursor, VS Code, Warp, etc.) | System Events `keystroke` via osascript | Provides a streaming/typewriter character effect. Falls back to Cmd+V if keystroke fails. |

---

## Paste Flow (Non-iTerm)

```
1. Write command to system clipboard via pbcopy (backup, also used by Cmd+V fallback)
2. Resign key window on the overlay NSPanel
3. Activate target terminal via AppleScript: tell application id "<bundle_id>" activate
4. Wait 150ms for the terminal to become frontmost
5. Clear current shell line:
   - Terminal.app: System Events keystroke "u" using control down
   - Others: CGEvent Ctrl+U
6. Wait 50ms
7. Type text via System Events keystroke (all non-iTerm terminals):
   osascript -e 'tell application "System Events" to keystroke "<command>"'
   Falls back to CGEvent Cmd+V if keystroke fails (generic path only)
8. Wait 150ms for keystrokes to be processed
9. Re-acquire key window on overlay panel (for blur/dismiss detection)
```

### Why System Events for Everything (Not CGEvent)

After investigating CGEvent failures on 2026-02-28, the entire non-iTerm paste
pipeline was migrated from CGEvent-based input to System Events `keystroke`.
CGEvent unicode (`CGEventKeyboardSetUnicodeString`) and even CGEvent Cmd+V
(`CGEventPost` with modifier flags) both proved unreliable -- they report
success but silently fail to deliver keystrokes to the target app. This
affects both Terminal.app and Electron-based terminals (Cursor, VS Code).

System Events `keystroke` operates at a higher level through the Automation
framework. It requires `kTCCServiceAppleEvents` (Automation permission) but
is significantly more reliable because:

- It goes through the accessibility/automation stack, not the raw HID event tap.
- It returns meaningful errors when permissions are missing.
- It is not affected by TCC state corruption that breaks CGEventPost.

The tradeoff is a small latency overhead from spawning `osascript`, but this
is imperceptible for typical command lengths.

---

## macOS Permissions Required

### 1. Accessibility (kTCCServiceAccessibility)

- **Purpose**: Required for `CGEventPost` to inject synthetic keyboard events (Ctrl+U, Cmd+V, Return).
- **Checked via**: `AXIsProcessTrusted()` in `ensure_accessibility()`.
- **TCC identifier**: Varies by build type.
  - Production (signed): `com.lakshmanturlapati.cmd-k`
  - Dev (debug): `cmd_k-<hash>` (e.g., `cmd_k-2f4e4b7172319f1b`)
- **Important**: `tccutil reset Accessibility <id>` must target the correct identifier. Resetting the wrong one has no effect on the running binary.

### 2. Automation (kTCCServiceAppleEvents)

- **Purpose**: Required for `osascript` to control other applications (activate, keystroke).
- **Prompt**: macOS shows "CMD+K wants to control [App]" on first use per target app.
- **Critical behavior**: If Automation permission is denied or reset, the `activate` AppleScript silently fails. The target app never becomes frontmost, and CGEvent keystrokes are delivered to the wrong app (or nowhere). The paste code reports "success" because `CGEventPost` does not return errors.

---

## Debugging Incident: 2026-02-28

### Symptom

Paste stopped working for all terminals (Terminal.app and Cursor). The app logs reported success at every step, but no text appeared in the terminal. Terminal.app produced a system beep.

### Root Cause (Multi-Factor)

1. **`tccutil reset Accessibility`** was run for the production bundle ID (`com.lakshmanturlapati.cmd-k`). This did NOT affect the dev build (different TCC identifier), but opened a debugging rabbit hole.

2. **`tccutil reset AppleEvents`** was run for the production bundle ID. This invalidated the Automation permission. After restarting the dev server, the osascript `activate` calls silently failed -- the target terminal never became frontmost, so CGEvent keystrokes went nowhere.

3. **CGEvent unicode (`CGEventKeyboardSetUnicodeString`) stopped working** even after Automation was re-granted. The exact cause is unclear, but after the TCC resets, this API silently dropped all characters for both Terminal.app and Cursor. `AXIsProcessTrusted()` returned `true`, `CGEventPost` returned no errors, but the target apps never received the keystrokes.

### Resolution

| Terminal | Before (broken) | After (working) |
|---|---|---|
| Terminal.app | CGEvent unicode keystrokes | System Events `keystroke` via osascript |
| Cursor / generic | CGEvent unicode keystrokes | System Events `keystroke` via osascript |
| iTerm2 | AppleScript `write text` | (unchanged, was never broken) |

**Additional finding:** Even CGEvent Cmd+V (not just CGEvent unicode) was unreliable for Terminal.app. The Cmd+V keystroke is also posted via `CGEventPost`, which suffers from the same silent failure. Terminal.app required a full migration to System Events for both the Ctrl+U clear and the text input.

### Key Diagnostic That Helped

Adding a System Events query after the `activate` call served two purposes:
- Confirmed the target app was actually frontmost (ruling out activation failure).
- Revealed that the activate was succeeding but CGEvent unicode was the broken link.

```rust
// This osascript call confirmed frontmost app = Terminal/Cursor
if let Ok(front) = std::process::Command::new("osascript")
    .arg("-e")
    .arg(r#"tell application "System Events" to get name of first application process whose frontmost is true"#)
    .output()
{
    let name = String::from_utf8_lossy(&front.stdout);
    eprintln!("[paste] frontmost app after activate: {}", name.trim());
}
```

### Lessons Learned

1. **Never run `tccutil reset` casually.** It can break permissions in ways that are invisible to the app (APIs report success but silently fail). Re-granting the permission may not fully restore the previous state.

2. **`CGEventPost` does not return errors.** It posts events fire-and-forget. If the target app doesn't receive them, the caller has no way to know. Always verify the frontmost app independently.

3. **`AXIsProcessTrusted()` can return `true` while `CGEventPost` is effectively broken.** The two checks are not equivalent. TCC accessibility covers AX API access; CGEvent posting may require additional internal state that is lost after a reset.

4. **Dev and production builds have different TCC identifiers.** Resetting permissions for the production bundle ID (`com.lakshmanturlapati.cmd-k`) does not affect the dev binary (`cmd_k-<hash>`), and vice versa.

5. **System Events `keystroke` is more reliable than CGEvent unicode** for typing text into arbitrary apps. It goes through the Automation framework rather than the lower-level HID event tap, and provides visible errors when permissions are missing.

---

## File Reference

- **Paste logic**: `src-tauri/src/commands/paste.rs`
- **Accessibility check**: `src-tauri/src/commands/permissions.rs` (`check_accessibility_permission`, `ensure_accessibility`)
- **App activation**: `build_activate_script()` in `paste.rs`
- **CGEvent FFI**: `cg_keys` module in `paste.rs`
