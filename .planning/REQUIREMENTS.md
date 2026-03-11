# Requirements: CMD+K

**Defined:** 2026-03-11
**Core Value:** The overlay must appear on top of the active application and feel instant

## v0.2.8 Requirements

Requirements for Windows terminal detection fix milestone. Each maps to roadmap phases.

### Process Detection

- [ ] **PROC-01**: User's active shell correctly identified via ConPTY parentage (OpenConsole.exe/conhost.exe) instead of highest-PID heuristic
- [x] **PROC-02**: Internal IDE cmd.exe processes (git, extensions, tasks) filtered out — only interactive console-attached cmd.exe selected
- [x] **PROC-03**: Process snapshot consolidated into single CreateToolhelp32Snapshot call shared across shell discovery, WSL detection, and diagnostics

### WSL Detection

- [ ] **WSLD-01**: WSL terminals detected via wsl.exe sibling relationship (shares parent with detected shell under ConPTY)
- [ ] **WSLD-02**: WSL detected via environment block reading — WSL_DISTRO_NAME present in process environment confirms WSL
- [ ] **WSLD-03**: Correct WSL distro matched to active tab via process args or window title in multi-distro scenarios

### Active Tab Matching

- [ ] **TABM-01**: Active terminal tab's shell identified via CWD-based disambiguation using focused_cwd parameter (Windows parity with macOS)
- [ ] **TABM-02**: Windows Terminal focused pane identified via UIA tree walking to correlate focused TermControl with its shell process

### UIA Scoping

- [ ] **UIAS-01**: UIA text reading scoped to terminal panel elements only — editor content, sidebars, menus excluded from text capture
- [ ] **UIAS-02**: WSL text detection requires multiple corroborating signals before declaring WSL — single Linux path in text insufficient

## Future Requirements

### Deferred

- **Console API shell identification** — Use GetConsoleProcessList to enumerate processes on a console session (may be unnecessary if ConPTY filtering is sufficient)
- **OSC sequence reading** — Read OSC 7/133 sequences from ConPTY output for definitive shell/CWD info (requires architecture change)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Screen reader mode injection for VS Code UIA text | Degrades performance, changes UI — removed in prior work |
| VS Code extension for terminal detection | Violates zero-setup constraint |
| WMI/CIM queries for process info | Too slow (100-500ms), wmic.exe deprecated |
| Windows Terminal settings.json parsing | File location varies, doesn't tell which tab is focused |
| Named pipe / IPC to terminal emulators | Not feasible for standalone tool |
| Polling/watching terminal state | CPU waste, race conditions — snapshot at hotkey time is correct |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROC-01 | Phase 27 | Pending |
| PROC-02 | Phase 27 | Complete |
| PROC-03 | Phase 27 | Complete |
| UIAS-01 | Phase 28 | Pending |
| UIAS-02 | Phase 28 | Pending |
| TABM-01 | Phase 29 | Pending |
| TABM-02 | Phase 29 | Pending |
| WSLD-01 | Phase 30 | Pending |
| WSLD-02 | Phase 30 | Pending |
| WSLD-03 | Phase 30 | Pending |

**Coverage:**
- v0.2.8 requirements: 10 total
- Mapped to phases: 10
- Unmapped: 0

---
*Requirements defined: 2026-03-11*
*Last updated: 2026-03-11 after roadmap creation*
