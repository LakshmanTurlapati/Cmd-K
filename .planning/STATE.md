---
gsd_state_version: 1.0
milestone: v0.1
milestone_name: milestone
status: executing
stopped_at: Completed 36-02-PLAN.md
last_updated: "2026-03-15T19:17:22.229Z"
last_activity: 2026-03-15 — Completed 34-01 linux terminal text reading
progress:
  total_phases: 7
  completed_phases: 6
  total_plans: 10
  completed_plans: 9
  percent: 92
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-14)

**Core value:** The overlay must appear on top of the active application and feel instant
**Current focus:** Phase 31 - Linux Overlay & Hotkey

## Current Position

Phase: 36 (7 of 7 in v0.3.9) — Showcase Website Update
Plan: 2 of 3 in current phase
Status: Executing
Last activity: 2026-03-15 — Completed 36-02 privacy policy update

Progress: [█████████░] 90%

## Performance Metrics

**All Milestones:**
- v0.1.0: 8 phases, 21 plans, 8 days
- v0.1.1: 3 phases, 6 plans, 2 days
- v0.2.1: 7 phases, 11 plans, 3 days
- v0.2.4: 4 phases, 5 plans, 2 days
- v0.2.6: 5 phases, 10 plans, 1 day
- v0.2.7: 2 phases, 3 plans, 1 day
- v0.2.8: 3 phases, 6 plans, 1 day
- Cumulative: 32 phases, 62 plans, 18 days

## Accumulated Context

### Decisions

Recent decisions affecting current work:

- [v0.3.9]: X11-first with XWayland fallback for Wayland (industry standard)
- [v0.3.9]: /proc filesystem for process detection (zero deps, simpler than macOS/Windows)
- [v0.3.9]: find_shell_pid must maintain 3-arg arity for cross-platform compat
- [v0.3.9]: AppImage on Ubuntu 22.04 for glibc floor
- [v0.3.9]: Terminal text reading deferred to enhancement phase (AT-SPI2 complexity)
- [Phase 30]: Linux /proc leaf functions use target_os=linux cfg gates for explicit three-way split
- [Phase 30]: Case-sensitive exe matching for Linux terminal/IDE classification
- [Phase 30]: process::get_process_name made pub(crate) on Linux for detect_linux cross-module access
- [Phase 31]: CSS backdrop-blur-xl for Linux frosted glass (no native vibrancy support)
- [Phase 31]: Three-tier border radius: macOS rounded-xl, Linux rounded-lg, Windows rounded-md
- [Phase Phase 31]: x11rb for direct EWMH property queries (already transitive dep, no subprocess)
- [Phase Phase 31]: Fresh X11 connection per hotkey press (1ms overhead acceptable with 200ms debounce)
- [Phase 32]: Return-value hint communication (Result<String, String>) for paste/confirm fallback signaling
- [Phase 32]: arboard removed from Linux fallback (Windows-only dep); xclip/wl-copy sufficient
- [Phase 33]: Prefix-based context window lookup in context.rs rather than extending ModelWithMeta struct
- [Phase 33]: 12% budget fraction (midpoint of 10-15% range) with chars/4 token estimation
- [Phase 33]: Pipeline order: ANSI strip -> budget truncate -> sensitive filter
- [Phase 34]: zbus with default features (async-io required) for AT-SPI2 D-Bus blocking calls
- [Phase 34]: Strategy dispatch by exe_name: VTE/Qt -> AT-SPI2, kitty -> remote control, wezterm -> CLI
- [Phase 35]: Native ARM runner (ubuntu-22.04-arm) for aarch64 AppImage builds
- [Phase 35]: Tray warning and skip when AppImage location not writable (no error)
- [Phase 36]: Preserved full March 9 policy verbatim in collapsible details element for version history

### Roadmap Evolution

- Phase 36 added: Showcase Website Update — version numbers, platform-specific downloads, privacy policy with history

### Pending Todos

None.

### Blockers/Concerns

- Wayland has no global hotkey protocol -- Tauri global-shortcut silently fails on pure Wayland
- AT-SPI2 terminal text reliability untested across terminal emulators
- x11rb vs xdotool for PID capture needs decision during Phase 31 planning

## Session Continuity

Last session: 2026-03-15T19:17:22.180Z
Stopped at: Completed 36-02-PLAN.md
Next action: Execute next phase plan or advance to next phase
