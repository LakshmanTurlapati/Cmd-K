# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-01)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** v0.2.1 Windows Support -- Phase 11 complete, ready for Phase 12

## Current Position

Phase: 11 of 16 (Build Infrastructure and Overlay Foundation)
Plan: 3 of 3
Status: Phase 11 complete
Last activity: 2026-03-02 -- Completed 11-03-PLAN.md (Windows focus management)

Progress: [=====                         ] 17%

## Performance Metrics

**v0.1.0 Summary:**
- Total phases: 8
- Total plans: 21
- Timeline: 8 days (2026-02-21 to 2026-02-28)
- Codebase: 4,042 LOC Rust + 2,868 LOC TypeScript

**v0.1.1 Summary:**
- Total phases: 3 (Phases 8-10)
- Total plans: 6
- Timeline: 2 days (2026-02-28 to 2026-03-01)
- Git: 32 commits, 47 files changed, 4,637 insertions

**v0.2.1:**

| Phase | Plan | Duration | Tasks | Files |
|-------|------|----------|-------|-------|
| 11    | 01   | 6min     | 2     | 9     |
| 11    | 02   | 2min     | 2     | 3     |
| 11    | 03   | 2min     | 2     | 2     |

## Accumulated Context

### Decisions

All prior decisions archived in PROJECT.md Key Decisions table.

v0.2.1 decisions:
- Default hotkey: Ctrl+Shift+K on Windows (not Ctrl+K -- too many conflicts)
- Acrylic for Win10, Mica for Win11 vibrancy
- WS_EX_TOOLWINDOW to hide from Alt+Tab
- Phases 12 and 13 can be developed in parallel (architecturally independent)
- Extract macOS paste/confirm into dedicated gated helper functions for readability (11-01)
- Standard Tauri window.show()/hide() as cross-platform overlay stubs (11-01)
- open_url cross-platform: open on macOS, cmd /c start on Windows, xdg-open on Linux (11-01)
- Tray platform conventions: macOS right-click+template icon, Windows left-click+normal icon (11-01)
- Acrylic-only vibrancy on Windows, no Mica fallback, per locked CONTEXT.md decision (11-02)
- WS_EX_TOOLWINDOW via direct Win32 API instead of Tauri skipTaskbar (buggy per #10422) (11-02)
- raw-window-handle 0.6 as Windows-only dep for HWND access in window style manipulation (11-02)
- Foreground window comparison for click-outside vs Escape/hotkey dismiss detection (11-03)
- AttachThreadInput + SetForegroundWindow with AllowSetForegroundWindow fallback for focus restoration (11-03)
- IsWindow validation before focus restoration to handle stale HWNDs gracefully (11-03)

### Pending Todos

None.

### Blockers/Concerns

- WS_EX_NOACTIVATE + WebView keyboard input interaction needs prototyping on Windows hardware
- PEB CWD reading requires unsafe Rust with cross-process memory access
- uiautomation crate maturity (less battle-tested than macOS AX APIs)
- SmartScreen reputation for unsigned binaries takes weeks to build
- 13 non-critical tech debt items from v0.1.0 and v0.1.1

## Session Continuity

Last session: 2026-03-02 (Phase 11 Plan 03 execution)
Stopped at: Completed 11-03-PLAN.md -- Phase 11 fully complete
Resume file: None
