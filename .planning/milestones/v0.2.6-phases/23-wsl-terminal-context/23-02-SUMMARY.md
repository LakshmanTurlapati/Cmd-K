---
phase: 23-wsl-terminal-context
plan: 02
subsystem: ai, ui
tags: [wsl, system-prompt, context-badge, terminal, linux]

requires:
  - phase: 23-wsl-terminal-context (plan 01)
    provides: "TerminalContext.is_wsl detection from WSL process ancestry"
provides:
  - "WSL-specific system prompt template for Linux command generation"
  - "is_wsl-aware AI prompt branching (system prompt + user message)"
  - "Frontend WSL badge display in resolveBadge"
  - "TerminalContext TypeScript interface with is_wsl field"
affects: [24-auto-updater]

tech-stack:
  added: []
  patterns: ["cfg-guarded WSL prompt constant (Windows-only)", "serde(default) for backward-compatible deserialization"]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/ai.rs
    - src/store/index.ts

key-decisions:
  - "WSL prompt is Windows-only via cfg guard; non-Windows falls back to standard template"
  - "Default shell is 'bash' for WSL sessions instead of 'zsh'"
  - "WSL badge takes priority 0 in resolveBadge, overriding shell type display"
  - "No safety.rs changes needed -- existing DESTRUCTIVE_PATTERNS already covers Linux commands"

patterns-established:
  - "cfg-guarded constants: platform-specific prompt variants compiled only where relevant"
  - "serde(default) on new bool fields for backward-compatible JSON deserialization"

requirements-completed: [WSLT-08, WSLT-09, WSLT-10]

duration: 3min
completed: 2026-03-09
---

# Phase 23 Plan 02: AI Prompt Wiring and WSL Badge Summary

**WSL system prompt for Linux commands, is_wsl-aware AI context branching, and "WSL" badge display**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-09T10:02:23Z
- **Completed:** 2026-03-09T10:04:57Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- WSL-specific system prompt template generates Linux commands for WSL sessions
- AI prompt selection branches on is_wsl with "OS: WSL on Windows" context in user message
- Frontend badge shows "WSL" for WSL sessions, overriding shell type display
- Full pipeline wired: detection -> context -> AI prompt -> badge

## Task Commits

Each task was committed atomically:

1. **Task 1: WSL system prompt, AI context branching, and safety awareness** - `e49f9d9` (feat)
2. **Task 2: Frontend WSL badge and interface update** - `b477ac0` (feat)

## Files Created/Modified
- `src-tauri/src/commands/ai.rs` - WSL system prompt template, is_wsl in TerminalContextView, prompt branching, OS context line
- `src/store/index.ts` - is_wsl in TerminalContext interface, WSL badge in resolveBadge

## Decisions Made
- WSL prompt constant is Windows-only (cfg guard) -- on non-Windows, is_wsl=true falls back to standard template
- Default shell changed to "bash" for WSL sessions (was "zsh" for all)
- WSL badge at priority 0 in resolveBadge overrides shell type -- per CONTEXT.md "just WSL" decision
- No changes to safety.rs -- existing DESTRUCTIVE_PATTERNS already matches Linux destructive commands (rm -rf, mkfs, dd, etc.)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full WSL pipeline complete: detection (plan 01) -> AI prompt + badge (plan 02)
- Phase 24 (Auto-Updater) can proceed independently

---
*Phase: 23-wsl-terminal-context*
*Completed: 2026-03-09*
