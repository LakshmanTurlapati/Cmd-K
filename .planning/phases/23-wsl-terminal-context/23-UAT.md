---
status: diagnosed
phase: 23-wsl-terminal-context
source: [23-01-SUMMARY.md, 23-02-SUMMARY.md]
started: 2026-03-09T11:10:00Z
updated: 2026-03-09T11:20:00Z
---

## Current Test

[testing complete]

## Tests

### 1. WSL Session Detection
expected: Open a WSL terminal (Windows Terminal with WSL tab, VS Code Remote-WSL, Cursor Remote-WSL, or standalone wsl.exe). Trigger CMD+K. The app should detect this as a WSL session — visible through the badge and context behavior in subsequent tests.
result: issue
reported: "In VS Code with WSL terminal, detected as cmd instead of WSL. Process ancestry search finds cmd.exe (VS Code's own shell spawns) but never encounters wsl.exe. detect_wsl_in_ancestry either not running or wsl.exe not in process tree for VS Code WSL. Also fails in Windows Terminal with WSL tab — no shell found as descendant, defaults to powershell. UIA text shows Linux prompt 'parzival@Parzival: ~' but gets inferred as powershell. WSL 2 processes run in VM and are invisible in Windows process tree."
severity: blocker

### 2. WSL Badge Display
expected: When CMD+K is triggered from a WSL session, the context badge shows "WSL" (not the shell name, not "PowerShell", not "CMD").
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 3. Linux CWD in Context
expected: In a WSL session, navigate to a Linux directory (e.g., `cd /home/user/projects`). Trigger CMD+K. The context should show the Linux path, not a Windows-style path like `\\wsl$\...`.
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 4. Shell Type Detection
expected: The detected shell type should reflect your WSL shell (bash, zsh, or fish), not PowerShell or cmd.
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 5. Linux Command Generation
expected: In a WSL session, ask CMD+K for a command (e.g., "list all running processes"). The AI should generate a Linux command (e.g., `ps aux`) rather than a Windows command (e.g., `tasklist`).
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 6. Destructive Command Safety
expected: In a WSL session, ask for something that would generate a destructive Linux command (e.g., "delete everything in this directory recursively"). The safety confirmation should trigger, recognizing the destructive pattern.
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 7. Terminal Output Reading
expected: Run a command in the WSL terminal that produces visible output (e.g., `ls -la`). Trigger CMD+K. The app should be able to read/reference the visible terminal output for context.
result: skipped
reason: Blocked by Test 1 — WSL not detected

### 8. Secret Filtering
expected: If terminal output contains sensitive patterns (e.g., a line resembling a shadow hash `$6$rounds=...` or an API key `sk-ant-...`), those patterns should be filtered/redacted from the context sent to AI.
result: skipped
reason: Backend-only; cannot verify without WSL detection working first

## Summary

total: 8
passed: 0
issues: 1
pending: 0
skipped: 7

## Gaps

- truth: "WSL session detection works when triggering CMD+K from VS Code or Windows Terminal with WSL"
  status: failed
  reason: "User reported: WSL detection fails in both VS Code (finds cmd.exe instead of wsl.exe) and Windows Terminal (no shell found, defaults to powershell). detect_wsl_in_ancestry never called. WSL 2 processes run in VM and are invisible in Windows process tree. UIA text shows Linux prompt 'parzival@Parzival: ~' but is inferred as powershell instead of bash/WSL."
  severity: blocker
  test: 1
  root_cause: "Three compounding failures: (1) detect_wsl_in_ancestry walks Windows process tree but WSL 2 processes are in Hyper-V VM, invisible to CreateToolhelp32Snapshot. (2) UIA-based WSL inference in detect_full_with_hwnd is gated by if terminal.is_wsl which is already false — circular dependency. (3) infer_shell_from_text has no pattern for user@host:path Linux prompts, defaults to powershell."
  artifacts:
    - path: "src-tauri/src/terminal/process.rs"
      issue: "detect_wsl_in_ancestry (line 913) structurally cannot find WSL 2 sessions"
    - path: "src-tauri/src/terminal/mod.rs"
      issue: "Circular is_wsl guard at line 129; missing Linux prompt pattern in infer_shell_from_text (line 479); hardcoded powershell fallback at line 456"
  missing:
    - "UIA text-based WSL detection as primary mechanism (detect_wsl_from_text recognizing user@host: patterns)"
    - "Linux prompt pattern (user@host:path) in infer_shell_from_text"
    - "Remove circular is_wsl guard — run UIA WSL detection unconditionally"
  debug_session: ".planning/debug/wsl-detection-failure.md"
