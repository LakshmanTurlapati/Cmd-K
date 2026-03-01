---
phase: 03-terminal-context-reading
plan: "05"
subsystem: frontend-context
tags: [typescript, react, zustand, tauri-ipc, badge-system]
dependency_graph:
  requires: ["03-03", "03-04"]
  provides: ["badge priority display", "AppContext in Zustand"]
  affects: ["Phase 04 - AI context consumption"]
tech_stack:
  patterns:
    - "AppContext wrapping TerminalContext in Zustand store"
    - "resolveBadge() pure function for badge priority resolution"
    - "get_app_context IPC replacing get_terminal_context"
key_files:
  modified:
    - src/store/index.ts
    - src/components/Overlay.tsx
    - src-tauri/src/terminal/detect.rs
decisions:
  - "Removed unused browser_display_name() -- clean_app_name() handles display names via APP_NAME_MAP"
  - "resolveBadge() is a pure exported function, not a Zustand selector, for testability"
  - "Existing get_terminal_context IPC kept for backward compatibility but frontend now uses get_app_context"
metrics:
  duration: "3 min"
  completed: "2026-02-22"
  tasks_completed: 2
  files_changed: 3
---

# Phase 3 Plan 5: Frontend AppContext and Badge Priority Summary

Switched frontend from TerminalContext to AppContext with badge priority system: shell > console > app name.

## What Was Built

### Task 1: Zustand store + Overlay badge update

**src/store/index.ts:**
- Added `AppContext` interface: `app_name`, `terminal` (TerminalContext | null), `console_detected`, `console_last_line`
- Added `resolveBadge(ctx)` pure function implementing priority: shell type > "Console" > app name
- Replaced `terminalContext` state field with `appContext`
- `show()` action now calls `get_app_context` IPC instead of `get_terminal_context`
- All context data (CWD, visible output, console last line) stored for Phase 4 AI consumption

**src/components/Overlay.tsx:**
- Imports `resolveBadge` from store
- Computes `badgeText` from `appContext` using the resolver
- Badge area shows: spinner during detection, badge text after, nothing on failure
- Accessibility banner unchanged

**src-tauri/src/terminal/detect.rs:**
- Removed unused `browser_display_name()` function (eliminated the only cargo warning)

### Task 2: Human verification

Verified badge behavior across scenarios:
- Terminal apps show shell type (e.g., "zsh")
- Browsers without DevTools show browser name (e.g., "Chrome", "Arc")
- Browsers with DevTools open show "Console"
- Generic apps show cleaned app name (e.g., "Finder")
- Editors with integrated terminal show shell type, not editor name
- Spinner appears during detection, badge after completion
- Accessibility banner functional

## Deviations from Plan

### Removed browser_display_name

**Found during:** cargo check warning elimination

**Issue:** `browser_display_name()` was defined in detect.rs but never called. The `clean_app_name()` function with `APP_NAME_MAP` already handles display name resolution for all apps including browsers.

**Fix:** Removed the unused function. Zero warnings now.

## Self-Check: PASSED

- TypeScript compiles with zero errors
- Rust cargo check passes with zero warnings
- Badge priority: shell > console > app name verified
- All context data stored in Zustand for Phase 4
- Backward compatibility maintained (get_terminal_context still exists)
