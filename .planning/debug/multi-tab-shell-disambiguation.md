---
status: diagnosed
slug: multi-tab-shell-disambiguation
created: 2026-03-11T15:50:00Z
updated: 2026-03-11T15:50:00Z
---

# Multi-Tab Shell Disambiguation

## Symptoms
- VS Code with cmd.exe + powershell tabs: always detects cmd.exe regardless of focused tab
- Windows Terminal with cmd + powershell tabs: always detects powershell (first direct child) regardless of focused tab
- WSL tabs work correctly (UIA text fallback detects `user@host:path` pattern)

## Root Cause

UIA text already contains the focused tab's shell type but is only consumed by `detect_wsl_from_text`:

- When cmd tab focused: UIA contains `"Command Prompt"`
- When PowerShell tab focused: UIA contains `"Windows PowerShell"`
- When WSL tab focused: UIA contains `parzival@Parzival:/path` (already detected)

The process tree approach (`find_shell_by_ancestry` / `find_shell_recursive`) cannot determine which tab is focused:
- VS Code: all shells are descendants, `pick_most_recent` picks by creation time (not focus)
- Windows Terminal: only one shell visible as direct child, recursive walk returns immediately

## Evidence

From logs — UIA text per focused tab:
```
# PowerShell tab focused:
"Windows PowerShell\nWindows PowerShell\nClose Tab\nNew Tab\n..."

# Command Prompt tab focused:
"Command Prompt\nCommand Prompt\nClose Tab\nNew Tab\n..."

# WSL tab focused (already working):
"parzival@Parzival:/mnt/c/Users/laksh/..."
```

## Files Involved
- `src-tauri/src/terminal/mod.rs`: `detect_full_with_hwnd` captures UIA text, calls `detect_wsl_from_text`, but doesn't extract shell type for non-WSL
- `src-tauri/src/terminal/process.rs`: `find_shell_by_ancestry` / `pick_most_recent` — no UIA signal available
- `src-tauri/src/terminal/detect_windows.rs`: `detect_wsl_from_text` — only WSL pattern matching

## Suggested Fix Direction
Parse shell type from UIA text (e.g., "Command Prompt" → cmd, "Windows PowerShell" → powershell, "pwsh" → pwsh, "bash" / "zsh" → corresponding shell). Use this as a filter/preference signal when multiple process tree candidates exist. This would extend the existing WSL UIA text pattern to all shell types.
