---
status: diagnosed
trigger: "WSL detection fails in both VS Code and Windows Terminal - is_wsl always false"
created: 2026-03-09T00:00:00Z
updated: 2026-03-09T00:00:00Z
---

## Current Focus

hypothesis: WSL 2 runs Linux processes in a Hyper-V VM so wsl.exe is never a process-tree ancestor of the detected shell PID. Additionally, infer_shell_from_text has no WSL-specific patterns and defaults to "powershell".
test: Code trace through both VS Code and Windows Terminal paths
expecting: Confirm that detect_wsl_in_ancestry is either never called or structurally unable to find wsl.exe, and that UIA text inference misclassifies WSL prompts
next_action: Write findings

## Symptoms

expected: When user activates CMD+K from a WSL terminal, is_wsl=true, shell_type="bash"/"zsh", cwd="/home/user/..."
actual: is_wsl=false, shell_type="cmd" (VS Code) or "powershell" (Windows Terminal), CWD is Windows path or None
errors: No errors - silent misdetection
reproduction: Trigger CMD+K hotkey from any WSL terminal session in VS Code or Windows Terminal
started: Always broken (WSL detection never worked in practice)

## Eliminated

(none - first investigation pass)

## Evidence

- timestamp: 2026-03-09T00:01:00Z
  checked: process.rs detect_wsl_in_ancestry (line 913-964)
  found: |
    The function walks UP the parent chain from shell_pid looking for wsl.exe.
    WSL 2 architecture: wsl.exe spawns a Hyper-V VM. Linux processes (bash, zsh)
    run INSIDE the VM and are invisible to Windows process APIs. The Windows-side
    process tree looks like: Terminal -> ConPTY/OpenConsole -> cmd.exe (or nothing).
    wsl.exe is a separate launcher process, NOT an ancestor of the shell PID that
    gets detected. The detected shell PID is always a Windows-native process (cmd.exe)
    or nothing at all.
  implication: detect_wsl_in_ancestry is structurally incapable of finding WSL sessions

- timestamp: 2026-03-09T00:02:00Z
  checked: process.rs get_foreground_info (line 89-138) - where detect_wsl_in_ancestry is called
  found: |
    Line 128: `let is_wsl = detect_wsl_in_ancestry(shell_pid);`
    This is called ONLY when find_shell_pid returns Some(pid). The shell_pid it
    receives is whatever Windows-native process was found (e.g., cmd.exe PID 27828
    in VS Code case). Walking up from cmd.exe -> Code.exe -> explorer.exe - no wsl.exe
    in that chain. For Windows Terminal, the ConPTY fallback finds cmd.exe shells
    parented by OpenConsole.exe - also no wsl.exe in ancestry.
  implication: Even when detect_wsl_in_ancestry IS called, it gets a non-WSL shell PID

- timestamp: 2026-03-09T00:03:00Z
  checked: VS Code case - why cmd.exe is found instead of WSL
  found: |
    Code.exe PID 16700 has cmd.exe children (PID 27828). These are VS Code's
    internal terminal host processes or extension host shells. The actual WSL session
    communicates through a different mechanism (wsl.exe -> VM pipe). The process tree
    walk finds cmd.exe as a "shell" because cmd.exe IS in KNOWN_SHELLS (process.rs:17).
    find_shell_recursive picks cmd.exe as the shell at depth 3.
  implication: VS Code's process tree contains real cmd.exe children that mask the WSL session

- timestamp: 2026-03-09T00:04:00Z
  checked: Windows Terminal case - why no shell found as descendant
  found: |
    WindowsTerminal.exe PID 20552 has NO direct children (ConPTY architecture).
    find_shell_by_ancestry walks all shell candidates but none have WT as ancestor.
    ConPTY fallback (line 818-833) looks for shells parented by OpenConsole.exe,
    finds only cmd.exe shells. WSL's bash/zsh are Linux processes invisible to
    CreateToolhelp32Snapshot. The fallback picks cmd.exe, or if no OpenConsole
    children are found, returns None.
  implication: Windows Terminal falls through to default "powershell" (mod.rs:456)

- timestamp: 2026-03-09T00:05:00Z
  checked: mod.rs detect_app_context_windows (line 452-463)
  found: |
    When has_shell is false but _is_terminal is true (Windows Terminal case),
    the code defaults to: shell_type=Some("powershell"), is_wsl=false.
    This is the "no shell in process tree" fallback at line 456.
    There is zero WSL awareness in this fallback path.
  implication: Known terminal without detectable shell always becomes "powershell"

- timestamp: 2026-03-09T00:06:00Z
  checked: mod.rs infer_shell_from_text (line 479-504) - why "parzival@Parzival: ~" becomes "powershell"
  found: |
    The function checks last 10 lines for prompt patterns:
    1. "PS " or "PS>" -> powershell (NO)
    2. C:\path> without PS -> cmd (NO)
    3. user@host ending with > and no backslash -> fish (NO - "parzival@Parzival: ~" does NOT end with >)
    4. ends with $ or contains "$ " -> bash (NO - "parzival@Parzival: ~" has no $)
    5. contains @ and ends with # or has "# " -> bash (NO - no #)
    6. DEFAULT: "powershell" (line 503)

    The prompt "parzival@Parzival: ~" matches NONE of the patterns because:
    - It has no $ suffix (the prompt line likely ends with just "~" or ": ~")
    - The actual prompt character $ may be on the same line but after a space: "parzival@Parzival:~$"
      vs the UIA may capture it as "parzival@Parzival: ~" without the trailing $
    - Even if the full prompt is "parzival@Parzival:~$ ", the pattern check is
      trimmed.ends_with('$') which would match "parzival@Parzival:~$" BUT
      trimmed.contains("$ ") would match "parzival@Parzival:~$ command"
    - The UIA capture likely shows just the static text without the cursor/prompt char
  implication: infer_shell_from_text defaults to "powershell" for Linux-style prompts without $/#

- timestamp: 2026-03-09T00:07:00Z
  checked: mod.rs detect_full_with_hwnd (line 108-161) - WSL UIA text inference path
  found: |
    Lines 129-149: WSL-specific UIA text inference runs ONLY when `terminal.is_wsl` is
    already true (line 129: `if terminal.is_wsl { ... }`). But is_wsl is false because
    detect_wsl_in_ancestry failed. So the WSL CWD inference and shell override code
    at lines 131-140 NEVER EXECUTES.

    The non-WSL path (lines 141-148) runs when `terminal.cwd.is_none()`, which infers
    shell from text but has no WSL awareness - it just calls infer_shell_from_text
    which defaults to "powershell".
  implication: WSL UIA inference is gated on is_wsl which is already false - circular dependency

- timestamp: 2026-03-09T00:08:00Z
  checked: mod.rs infer_shell_from_text - missing WSL/Linux prompt pattern
  found: |
    The function has no pattern for "user@host:" or "user@host:path" which is the
    canonical Linux/WSL bash prompt format (PS1 default: \u@\h:\w\$).
    The "user@host" check at line 491 (fish detection) requires ending with '>'.
    The "user@host" check at line 499 (root bash) requires ending with '#'.
    Standard user bash prompt "user@host:~$" would match line 495 (ends_with('$'))
    BUT only if the $ is captured in UIA text, which it often isn't.
  implication: Need a "user@host:" pattern that detects Linux/WSL regardless of trailing $

## Resolution

root_cause: |
  THREE compounding failures cause WSL to never be detected:

  1. ARCHITECTURAL: detect_wsl_in_ancestry walks the Windows process tree from the
     detected shell PID upward looking for wsl.exe. But WSL 2 runs Linux processes
     in a Hyper-V VM - they are invisible to Windows process APIs. The shell PID
     found by the process tree walk is always a Windows-native process (cmd.exe or
     powershell.exe), never a WSL process. wsl.exe is a launcher, not an ancestor
     of the terminal's ConPTY shell.

  2. CIRCULAR DEPENDENCY: The UIA text-based WSL inference in detect_full_with_hwnd
     (lines 129-140) only runs when is_wsl is already true. But is_wsl can only be
     set by the process tree walk (cause #1), which always fails. So the text-based
     detection that COULD identify WSL is gated behind the broken process detection.

  3. MISSING PATTERN: infer_shell_from_text has no pattern for Linux-style prompts
     like "user@host:path" or "user@host: ~". The function defaults to "powershell"
     when no pattern matches, so even the non-WSL text inference path misidentifies
     WSL prompts.

fix: (not applied - diagnosis only)
verification: (not applied - diagnosis only)
files_changed: []

## Suggested Fix Strategy

The minimal fix should:

1. **Add WSL detection to infer_shell_from_text**: Add a pattern that recognizes
   "user@host:" format (with or without trailing $) as a Linux/WSL prompt. Return
   "bash" (or better, a new indicator).

2. **Add a new infer_wsl_from_text function**: Check UIA text for Linux prompt
   patterns (user@host:, Linux paths like /home/..., ~ as CWD indicator). This
   should set is_wsl=true when process tree detection fails.

3. **Break the circular dependency in detect_full_with_hwnd**: The UIA text WSL
   detection (lines 129-140) should run regardless of the current is_wsl value.
   Change the guard from `if terminal.is_wsl` to always check UIA text for WSL
   indicators. Specifically:
   - After reading UIA text, call infer_wsl_from_text(text)
   - If it returns true, set terminal.is_wsl = true, then run the existing
     WSL CWD/shell inference code

4. **Update the Windows Terminal fallback** (detect_app_context_windows line 452-463):
   Don't hardcode "powershell" - leave shell_type as None so the UIA text inference
   in detect_full_with_hwnd has a chance to override it.

Key patterns for WSL detection from UIA text:
- `user@host:` (canonical bash PS1)
- `user@host /path>` could be fish-on-WSL
- Paths starting with `/home/`, `/root/`, `~`
- Absence of Windows path separators (`\`, `C:`)
