---
phase: 28-uia-terminal-text-scoping
verified: 2026-03-11T18:00:00Z
status: human_needed
score: 6/7 must-haves verified
re_verification: false
human_verification:
  - test: "Open VS Code with a Dockerfile or config file containing Linux paths (/home/, /etc/) in the editor AND a PowerShell terminal open. Invoke CMD+K and check that the captured terminal text does not contain editor content."
    expected: "Terminal text should only contain PowerShell prompt/output, not Dockerfile or config file content."
    why_human: "UIA tree scoping requires a live VS Code window with specific panel layout to verify ControlType::List filtering works."
  - test: "Open VS Code with two terminal tabs (one active, one inactive). Invoke CMD+K and check that only the active terminal tab text is captured."
    expected: "Only the active terminal tab text appears in visible_output, not text from inactive tabs."
    why_human: "IsOffscreen property filtering for inactive tabs requires a live multi-tab terminal setup."
  - test: "Open Windows Terminal (standalone, not in IDE). Invoke CMD+K and verify text reading still works via Strategy 1 (TextPattern)."
    expected: "Windows Terminal text captured normally, Strategy 1 succeeds, scoped walk is never reached."
    why_human: "Verifying Strategy 1 precedence requires a live Windows Terminal instance."
---

# Phase 28: UIA Terminal Text Scoping Verification Report

**Phase Goal:** Terminal text reading captures only terminal panel content, not editor or sidebar text from the IDE window
**Verified:** 2026-03-11T18:00:00Z
**Status:** human_needed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Single Linux path in text does NOT trigger WSL detection | VERIFIED | `test_single_linux_path_no_prompt_is_false` and `test_editor_content_etc_path_is_false` pass; scoring requires >= 2, single path gives score 1 |
| 2 | Linux path plus prompt pattern DOES trigger WSL detection | VERIFIED | `test_linux_path_plus_prompt_is_true` and `test_prompt_with_path_is_true` pass |
| 3 | WSL mount path (/mnt/c/) alone triggers WSL detection (strong signal) | VERIFIED | `test_wsl_mount_mnt_c_is_true` and `test_mnt_c_alone_is_true` pass; WSL mount gets score 2 |
| 4 | Editor-like content (Dockerfiles, READMEs with Linux paths) does not false-positive | VERIFIED | `test_dockerfile_content_is_false` and `test_powershell_viewing_linux_path_is_false` pass |
| 5 | UIA text reading from VS Code captures only terminal panel text, not editor content | UNCERTAIN | `try_scoped_terminal_walk` implemented with ControlType::List filtering and IsOffscreen checks; requires live VS Code testing |
| 6 | Fallback to full tree walk occurs gracefully when scoped search finds no terminal panel | VERIFIED | `read_terminal_text_inner` lines 69-88 show Strategy 2 -> Strategy 3 fallback chain; Err from scoped walk triggers `try_walk_children` |
| 7 | Windows Terminal and conhost text reading is unchanged (TextPattern strategy unaffected) | VERIFIED | `try_text_pattern` is unchanged and still runs first as Strategy 1 at line 62; scoped walk only runs if TextPattern fails |

**Score:** 6/7 truths verified (1 needs human verification)

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src-tauri/src/terminal/mod.rs` | Multi-signal detect_wsl_from_text with scoring threshold | VERIFIED | Lines 611-690: scoring system with threshold >= 2, signals for WSL mount (+2), Linux paths (+1), prompt pattern (+1), prompt ending (+1) |
| `src-tauri/src/terminal/uia_reader.rs` | Scoped terminal tree walk using ControlType::List filtering | VERIFIED | Lines 169-241: `try_scoped_terminal_walk` finds List elements, filters IsOffscreen, reads children Names, validates via `looks_like_terminal_text` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `detect_wsl_from_text` | `detect_full_with_hwnd` | Called at line 170 for WSL text detection | WIRED | Line 170: `if detect_wsl_from_text(text)` inside the `#[cfg(target_os = "windows")]` block |
| `try_scoped_terminal_walk` | `read_terminal_text_inner` | Called as Strategy 2 | WIRED | Line 73: `if let Ok(text) = try_scoped_terminal_walk(&automation, &element)` |
| `try_walk_children` | `read_terminal_text_inner` | Fallback Strategy 3 | WIRED | Line 83: `if let Ok(text) = try_walk_children(&automation, &element)` after scoped walk fails |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| UIAS-01 | 28-02 | UIA text reading scoped to terminal panel elements only -- editor content, sidebars, menus excluded from text capture | NEEDS HUMAN | `try_scoped_terminal_walk` implemented with ControlType::List filtering; logic is sound but needs live VS Code verification |
| UIAS-02 | 28-01 | WSL text detection requires multiple corroborating signals before declaring WSL -- single Linux path in text insufficient | SATISFIED | Multi-signal scoring system (threshold >= 2) in `detect_wsl_from_text`; 12 unit tests all pass covering false-positive and true-positive scenarios |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | - | - | - | No anti-patterns found in modified files |

### Human Verification Required

### 1. Scoped UIA Walk in VS Code

**Test:** Open VS Code with a Dockerfile or config file containing Linux paths (/home/, /etc/) in the editor AND a PowerShell terminal open. Invoke CMD+K and check that the captured terminal text does not contain editor content.
**Expected:** Terminal text should only contain PowerShell prompt/output, not Dockerfile or config file content.
**Why human:** UIA tree scoping requires a live VS Code window with specific panel layout to verify ControlType::List filtering works.

### 2. Inactive Terminal Tab Exclusion

**Test:** Open VS Code with two terminal tabs (one active, one inactive). Invoke CMD+K and check that only the active terminal tab text is captured.
**Expected:** Only the active terminal tab text appears in visible_output, not text from inactive tabs.
**Why human:** IsOffscreen property filtering for inactive tabs requires a live multi-tab terminal setup.

### 3. Windows Terminal Unaffected

**Test:** Open Windows Terminal (standalone, not in IDE). Invoke CMD+K and verify text reading still works via Strategy 1 (TextPattern).
**Expected:** Windows Terminal text captured normally, Strategy 1 succeeds, scoped walk is never reached.
**Why human:** Verifying Strategy 1 precedence requires a live Windows Terminal instance.

### Gaps Summary

No automated gaps found. All code artifacts exist, are substantive (not stubs), and are properly wired. The multi-signal WSL detection (UIAS-02) is fully verified with 12 passing unit tests.

The scoped UIA tree walk (UIAS-01) is implemented correctly at the code level -- ControlType::List filtering, IsOffscreen checks, `looks_like_terminal_text` heuristic, and fallback to full tree walk are all in place. However, since UIA tree walking requires a live Windows UI Automation context with an actual VS Code window, the scoped walk cannot be verified purely through automated tests. Three human verification tests are required to confirm the implementation works against real IDE windows.

---

_Verified: 2026-03-11T18:00:00Z_
_Verifier: Claude (gsd-verifier)_
