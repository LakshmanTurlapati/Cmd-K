---
status: diagnosed
phase: 08-window-identification-history-storage
source: 08-01-SUMMARY.md, 08-02-SUMMARY.md
started: 2026-03-01T07:10:00Z
updated: 2026-03-01T07:25:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Different Window Keys Per Terminal Tab
expected: Open two different iTerm2 tabs. Press Cmd+K in each. Check Rust stderr log for "[hotkey] computed window_key:" -- each tab shows a different key (different shell PIDs).
result: issue
reported: "Cursor IDE with 3 terminal tabs always picks shell PID 83090 regardless of which tab is active. find_shell_pid ancestry search returns all 3 descendant shells but always picks the same one. Different apps (Cursor vs Terminal.app) get different keys correctly (com.todesktop.230313mzl4w4u92:83090 vs com.apple.Terminal:2551)."
severity: major

### 2. Non-Terminal App Fallback
expected: Press Cmd+K while Finder (or another non-terminal app) is frontmost. Overlay opens without errors. Rust stderr shows a window key like "com.apple.finder:<pid>" -- no crash, no missing-key error.
result: pass

### 3. Window Key Available Before Typing
expected: Press Cmd+K from a terminal. Check browser console (or Rust stderr) for "[store] window key:" log appearing BEFORE "[store] window history entries:" and before you can type. The key is fetched instantly because the hotkey handler already computed it.
result: pass

### 4. History Survives Overlay Close/Reopen
expected: Press Cmd+K, submit a query, wait for response, then press Escape to close overlay. Press Cmd+K again from the same terminal tab. Check browser console for "[store] window history entries: 1" (or more) -- the previous query was persisted and retrieved.
result: pass

### 5. History Capped at 7 Entries Per Window
expected: Submit 8 queries from the same terminal tab (open overlay, type query, wait for response, close, repeat). After the 8th, press Cmd+K and check console -- "[store] window history entries: 7". The oldest entry was evicted.
result: skipped
reason: Requires 8 manual query submissions. Code-level eviction logic (VecDeque pop_front when len >= 7) is verified. Incremental history count confirmed working (1, 2, 3 entries observed).

## Summary

total: 5
passed: 3
issues: 1
pending: 0
skipped: 1

## Gaps

- truth: "Terminal tabs produce different window keys (bundle_id:shell_pid format)"
  status: failed
  reason: "Cursor IDE with 3 terminal tabs always picks shell PID 83090 via ancestry search. find_shell_pid returns all descendant shells but doesn't identify which tab is currently active/focused."
  severity: major
  test: 1
  root_cause: "find_shell_by_ancestry in process.rs uses highest-PID heuristic (max_by_key) to select among multiple descendant shells. PIDs reflect creation order, not tab focus. Electron IDEs bury shells 4-5 levels deep, so the fast recursive walk fails and falls through to the ancestry search which finds all shells but picks arbitrarily."
  artifacts:
    - path: "src-tauri/src/terminal/process.rs"
      issue: "find_shell_by_ancestry line 435 uses max_by_key(pid) -- highest PID is not the focused tab"
    - path: "src-tauri/src/commands/hotkey.rs"
      issue: "compute_window_key passes result from find_shell_pid directly as window key discriminator"
  missing:
    - "Need focused-tab detection for Electron IDEs -- AX-based CWD matching (compare each shell PID CWD against AXFocusedUIElement text) is recommended approach"
  debug_session: ".planning/debug/cursor-multi-tab-same-pid.md"
