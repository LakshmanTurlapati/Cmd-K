---
status: complete
phase: 27-conpty-discovery-process-snapshot
source: [27-01-SUMMARY.md, 27-02-SUMMARY.md]
started: 2026-03-11T14:10:00Z
updated: 2026-03-11T15:50:00Z
---

## Current Test

[testing complete]

## Tests

### 1. ConPTY shell selection in VS Code
expected: Open VS Code with 2+ terminal tabs (e.g., PowerShell and bash/zsh). Focus the most recently opened tab. Press the Cmd-K hotkey. The app should detect the shell in the focused terminal tab — not the first tab or a random one.
result: issue
reported: "Both for powershell and cmd it just detects as cmd. After batch filtering, 3 candidates remain (cmd.exe 32632, powershell.exe 28904, cmd.exe 31464) and pick_most_recent always selects cmd.exe 31464 regardless of which tab is focused. WSL detection works correctly via UIA text fallback."
severity: major

### 2. cmd.exe interactive tab support
expected: Open VS Code (or Windows Terminal) with a cmd.exe tab open interactively (no /C flag — just a plain cmd.exe prompt). Press the Cmd-K hotkey. The app should detect cmd.exe as a valid shell and work with it, rather than skipping it.
result: pass

### 3. cmd.exe background process filtering
expected: While a background cmd.exe process is running (e.g., a build tool spawning "cmd.exe /C npm run build"), press the hotkey. The app should NOT pick up the background cmd.exe — it should find your actual interactive shell instead.
result: issue
reported: "In Windows Terminal with cmd and powershell tabs, focused tab is not detected. WT only exposes one shell (powershell 32644) as direct child — find_shell_recursive picks it immediately without considering other tabs. UIA text correctly shows 'Command Prompt' vs 'Windows PowerShell' for focused tab but this signal is not used for non-WSL shell disambiguation."
severity: major

### 4. Standalone terminal detection
expected: Open a standalone terminal (e.g., Windows Terminal or plain conhost) with a single shell tab. Press the hotkey. Detection should still work via the direct descendant fallback.
result: pass

### 5. Unit tests pass
expected: Run `cargo test --manifest-path src-tauri/Cargo.toml` — all 25 tests pass with zero failures and no warnings.
result: pass

## Summary

total: 5
passed: 3
issues: 2
pending: 0
skipped: 0

## Gaps

- truth: "In VS Code with multiple terminal tabs, the correct interactive shell is selected via ConPTY parentage"
  status: failed
  reason: "User reported: Both for powershell and cmd it just detects as cmd. pick_most_recent selects most recently created cmd.exe regardless of focused tab. WSL works via UIA text fallback."
  severity: major
  test: 1
  artifacts: []
  missing: []

- truth: "Focused tab is correctly identified when multiple shells are open in Windows Terminal"
  status: failed
  reason: "User reported: WT only exposes one shell as direct child. find_shell_recursive picks it immediately. UIA text contains focused tab name ('Command Prompt' / 'Windows PowerShell') but is not used for non-WSL shell type disambiguation."
  severity: major
  test: 3
  artifacts: []
  missing: []
