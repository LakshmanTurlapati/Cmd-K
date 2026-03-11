---
phase: 28-uia-terminal-text-scoping
plan: 01
subsystem: terminal-detection
tags: [wsl, uia, false-positive, scoring]
dependency_graph:
  requires: []
  provides: [multi-signal-wsl-text-detection]
  affects: [detect_full_with_hwnd, detect_wsl_from_text]
tech_stack:
  added: []
  patterns: [multi-signal-scoring-threshold]
key_files:
  created: []
  modified:
    - src-tauri/src/terminal/mod.rs
decisions:
  - Scoring threshold of 2 eliminates single-path false positives while preserving WSL mount detection
  - WSL mount paths (/mnt/c/) get score 2 as unambiguous signal; Linux paths get score 1 as weak signal
  - Removed cfg(target_os = "windows") from detect_wsl_from_text to enable cross-platform testing
metrics:
  duration: 4m
  completed: "2026-03-11T17:19:23Z"
---

# Phase 28 Plan 01: Multi-Signal WSL Text Detection Summary

Multi-signal scoring system (threshold >= 2) for detect_wsl_from_text eliminates false positives from editor content containing Linux paths while preserving strong WSL signals like /mnt/c/ mount paths.

## What Was Done

### Task 1: TDD RED - Failing tests for multi-signal detection
- Added 12 unit tests covering both false-positive and true-positive scenarios
- False-positive tests: single Linux path, Dockerfile content, PowerShell with Linux arg, bare user@host
- True-positive tests: WSL mount paths, prompt+path combos, full terminal output
- Confirmed 7 tests failed against old single-match implementation

### Task 2: TDD GREEN - Implement scoring system
- Replaced single-match `detect_wsl_from_text` with multi-signal scoring:
  - score += 2 for WSL mount paths (/mnt/[a-z]/) -- strong, unambiguous
  - score += 1 for Linux paths (/home/, /etc/, /var/, etc.) -- weak, could be editor content
  - score += 1 for user@host:/path or user@host:~ prompt pattern
  - score += 1 for user@host...$ or user@host...# prompt ending
  - Returns `score >= 2`
- Removed `#[cfg(target_os = "windows")]` from function (only does string matching), added `#[allow(dead_code)]` for non-Windows
- All 46 tests pass (34 existing + 12 new)

## Deviations from Plan

None - plan executed exactly as written.

## Commits

| Commit | Type | Description |
|--------|------|-------------|
| b15e6b7 | test | Add failing tests for multi-signal WSL text detection |
| 9c8888c | feat | Implement multi-signal scoring for WSL text detection |

## Verification

- `cargo test --lib -p cmd-k -- terminal::tests`: 12/12 pass
- `cargo test --lib -p cmd-k`: 46/46 pass (no regressions)
- `cargo check`: clean compile on Linux

## Self-Check: PASSED
