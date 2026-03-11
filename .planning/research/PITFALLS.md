# Domain Pitfalls: Windows Terminal Detection Fix (v0.2.8)

**Domain:** Windows terminal/shell detection in IDE environments (VS Code, Cursor) and standalone terminals
**Researched:** 2026-03-11
**Overall confidence:** HIGH (based on project's own failure history, code analysis, and prior pitfalls research)

> This document supersedes the v0.2.6 PITFALLS.md for terminal detection topics. Prior pitfalls for multi-provider AI (Pitfalls 1-2, 6-8, 11-12, 14) and auto-updater (Pitfalls 3, 5, 10) remain valid but are out of scope for v0.2.8. This document covers ONLY pitfalls specific to fixing Windows terminal detection: WSL detection, shell differentiation, IDE process filtering, and UIA/Win32 API usage with Electron apps.

---

## Critical Pitfalls

Mistakes that cause reverts, regressions, or fundamental architectural failures. Each has already bitten this project at least once.

---

### Pitfall 1: Treating wsl.exe as a Shell Process

**What goes wrong:** Adding wsl.exe to KNOWN_SHELL_EXES or KNOWN_TERMINAL_EXES causes ALL VS Code terminals to misdetect as WSL. VS Code spawns 10-16+ internal wsl.exe processes for file watching, Remote-WSL extension host, and other background features. None of these correspond to user terminal tabs.

**Why it happens:** The intuition "WSL terminals run under wsl.exe, so wsl.exe is a shell" seems logical. But wsl.exe is a launcher/bridge process, not a shell. VS Code uses wsl.exe for many non-terminal purposes. Any approach that counts wsl.exe instances or includes wsl.exe in shell candidate lists will be flooded with false positives.

**Consequences:** Every VS Code terminal (PowerShell, CMD, Git Bash) gets detected as WSL. Already happened: fix in commit e0ef4be, reverted in commit bcea9cd. This burned a full debug-and-revert cycle.

**Prevention:**
- Never add wsl.exe to any "known shells" or "known terminals" list
- WSL detection must use output-based signals (UIA text patterns, window title) not process-based signals
- Any new WSL detection mechanism must be tested with VS Code open but with NO WSL terminal tabs active -- if it still detects WSL, it is broken
- The existing code correctly removed wsl.exe from KNOWN_TERMINAL_EXES in commit 1e4cd7a -- any refactoring must preserve this removal

**Detection:** If `is_wsl` returns true when the user has only a PowerShell tab open in VS Code, this pitfall has been hit.

**Phase relevance:** Must be respected in every phase. Gate: any code change touching WSL detection must include a "no WSL terminal active" negative test scenario.

---

### Pitfall 2: Circular Dependency Between Process-Based and Text-Based Detection

**What goes wrong:** Text-based WSL detection (UIA terminal output analysis) is gated behind `if terminal.is_wsl`, which can only be set by process-tree detection, which structurally fails for WSL 2. The text detection that COULD fix the problem never runs because it waits for the broken detector to succeed first.

**Why it happens:** The original code structure assumed process-tree WSL detection would work and UIA text was a refinement. When the process-tree approach was found to be architecturally broken (WSL 2 Hyper-V VM makes Linux processes invisible to Windows process APIs), the guard condition was not updated. Diagnosed in `.planning/debug/wsl-detection-failure.md` as "root cause #2: CIRCULAR DEPENDENCY."

**Consequences:** WSL is never detected. UIA text with clear Linux prompts (user@host:~$) is available but the code skips it because is_wsl is already false.

**Prevention:**
- UIA text WSL detection must run UNCONDITIONALLY, not gated behind any prior WSL flag
- Structure detection as a waterfall: each subsequent check can SET is_wsl=true but should never require it to already be true
- The current code (post-fix in mod.rs lines 148-157) already runs `detect_wsl_from_text` unconditionally -- any refactoring MUST preserve this property
- Write the detection pipeline as: `process_tree -> title_check -> uia_text_check -> cwd_path_check`, where each can independently set is_wsl=true
- Code review check: grep for `if.*is_wsl.*detect_wsl` or `if.*is_wsl.*infer.*wsl` -- any such pattern is a potential circular dependency

**Detection:** Add a log line "WSL text detection skipped because is_wsl already false" -- if this ever appears in production, the circular dependency is back.

**Phase relevance:** Any refactoring of detect_full_with_hwnd or the WSL detection pipeline. Must be explicitly verified in code review.

---

### Pitfall 3: "Highest PID = Active Terminal" Heuristic Picks Internal IDE Processes

**What goes wrong:** When multiple shell processes are descendants of an IDE (VS Code/Cursor), the code picks the one with the highest PID assuming it was most recently spawned and therefore most likely the active terminal tab. This is wrong -- internal IDE processes (extension hosts, task runners, git helper shells) are often spawned AFTER user terminal tabs and have higher PIDs.

**Why it happens:** PID assignment order correlates with spawn time on most OSes, so "highest PID" seems like a reasonable proxy for "most recent." But IDE internals continuously spawn and despawn processes. A git fetch, extension activation, or background task can spawn a cmd.exe or powershell.exe with a PID higher than the user's terminal shell.

**Consequences:** The wrong shell is detected. User opens a bash terminal, but CMD+K reads context from an internal cmd.exe spawned by a VS Code extension. This is currently the ACTIVE bug that v0.2.8 must fix.

**Prevention:**
- Filter out non-interactive shell processes BEFORE applying any PID-based selection
- The existing IDE-mode cmd.exe deprioritization (process.rs `find_shell_by_ancestry`) is a partial fix but insufficient -- internal powershell.exe processes also exist
- Better approach: use CWD-based matching (compare shell CWD against focused tab CWD from UIA) as primary disambiguation, PID as last resort only
- On Windows, CWD-based matching is already implemented for macOS (`focused_cwd` parameter) but the Windows path ignores it (`_focused_cwd` with underscore prefix)
- Consider additional signals: does the shell have a ConPTY/OpenConsole parent? Does it have a visible console allocation? Is its parent a known pty-helper process?

**Detection:** Compare the detected shell_type and cwd against what the user actually sees in their focused terminal tab. If they don't match, this pitfall is active.

**Phase relevance:** Shell disambiguation phase. The Windows `find_shell_by_ancestry` currently discards the `_focused_cwd` parameter -- implementing CWD matching requires finding the focused tab's identity through UIA or another mechanism first.

---

### Pitfall 4: WSL 2 Hyper-V VM Makes Linux Processes Invisible to Windows APIs

**What goes wrong:** Process tree walking (CreateToolhelp32Snapshot) finds only Windows-native processes. WSL 2 runs Linux processes (bash, zsh, etc.) inside a lightweight Hyper-V VM. These processes do not appear in the Windows process table. The detected "shell" is always a Windows-side process like cmd.exe or conhost.exe.

**Why it happens:** WSL 1 ran Linux binaries as pico processes visible to Windows APIs. WSL 2 switched to a real Linux kernel in a VM. wsl.exe is a bridge that connects a Windows ConPTY to the VM's /dev/pts, but the actual shell process lives in the Linux namespace, invisible to Win32 APIs. Diagnosed in `.planning/debug/wsl-detection-failure.md` as "root cause #1: ARCHITECTURAL."

**Consequences:** Process tree detection finds cmd.exe (the ConPTY host) instead of bash/zsh. `detect_wsl_in_ancestry` walks from cmd.exe up to Code.exe/explorer.exe -- no wsl.exe ancestor exists. WSL detection based purely on process tree will ALWAYS fail for WSL 2.

**Prevention:**
- Process tree is useful for: finding Windows-native shells (PowerShell, cmd.exe, Git Bash)
- Process tree is useless for: detecting WSL sessions or finding Linux shells
- WSL detection MUST use output-based signals: UIA terminal text patterns, window title "[WSL: distro]", CWD path style (\\wsl$\..., /home/...)
- Never add a "detect WSL from process tree" approach for WSL 2 -- it is architecturally impossible
- The wsl.exe subprocess calls (get_wsl_cwd, get_wsl_shell) are the correct approach for reading Linux-side context AFTER WSL is detected through other means

**Detection:** If WSL detection works for WSL 1 but not WSL 2, this is the cause.

**Phase relevance:** Foundational constraint that must be documented and understood before any WSL detection code is written. All WSL detection code must be output-based, not process-based.

---

### Pitfall 5: UIA Focused-Element Strategy Fails for Electron/VS Code

**What goes wrong:** Using `UIAutomation::focused_element()` to find the active terminal area returns the CMD+K overlay element (or nothing useful) because focus has already moved away from VS Code by the time UIA reading runs. Additionally, VS Code does not reliably expose xterm.js accessibility tree elements without screen reader mode enabled.

**Why it happens:** The hotkey handler captures PID and HWND synchronously before showing the overlay, but the actual UIA tree traversal happens on a background thread after the overlay is visible. By then, the focused element is the overlay, not the VS Code terminal. Chromium-based apps (VS Code, Cursor) only fully populate the xterm.js accessibility tree when screen reader mode is detected. This strategy was researched in Phase 23.1 and found to be unreliable.

**Consequences:** UIA focused-element returns overlay UI elements. The xterm.js accessibility subtree may be empty or minimal. Phase 23.1 research concluded: "UIA focused element approach: LOW confidence -- theoretical, needs testing to verify viability."

**Prevention:**
- Do NOT rely on `UIAutomation::focused_element()` for determining which terminal tab is active
- Use `element_from_handle(hwnd)` with the pre-captured HWND to root UIA traversal at the correct window
- Accept that VS Code UIA tree walking will return mixed content (menus, sidebar, editor, terminal) and make detection patterns specific enough to handle noise
- For active tab identification, use: window title (contains file/project info), UIA tab list with selection state, or process-tree CWD matching
- Do NOT assume xterm.js accessibility tree is always populated -- it requires VS Code's `terminal.integrated.accessibilitySupport` setting or system screen reader detection

**Detection:** If UIA text is empty for VS Code windows, or if the text contains only overlay content, this pitfall is active.

**Phase relevance:** Any phase implementing UIA-based terminal text reading or active-tab detection for Electron IDEs.

---

## Moderate Pitfalls

These cause incorrect behavior but are recoverable without rewrites.

---

### Pitfall 6: VS Code Internal cmd.exe Processes Polluting Shell Candidates

**What goes wrong:** VS Code spawns multiple cmd.exe processes for git operations, task runners, extension host startup, and other internal features. These appear as descendants of Code.exe in the process tree and are indistinguishable from user-created cmd.exe terminal tabs.

**Why it happens:** VS Code uses cmd.exe as a general-purpose process launcher on Windows. The `find_shell_by_ancestry` function correctly finds these as shell descendants of Code.exe, but cannot tell which (if any) is the user's active terminal tab.

**Prevention:**
- The existing IDE-mode deprioritization (prefer powershell/pwsh/bash/zsh over cmd.exe) is the right direction
- To improve: check if cmd.exe has an associated ConPTY/OpenConsole parent, which would indicate it is a terminal session rather than an internal process
- Consider checking if the cmd.exe process has child processes (interactive cmd.exe terminals often have children; internal VS Code cmd.exe calls are fire-and-forget)
- Long-term: CWD matching against the focused tab is the reliable disambiguation

**Detection:** If shell_type returns "cmd" when the user has a PowerShell tab focused in VS Code, internal cmd.exe is being selected.

**Phase relevance:** IDE process filtering phase.

---

### Pitfall 7: ConPTY Architecture Breaks Parent-Child Process Tree Assumptions

**What goes wrong:** Windows Terminal uses ConPTY, where shell processes are NOT descendants of WindowsTerminal.exe. The process tree looks like: WindowsTerminal.exe (no children) and separately OpenConsole.exe -> shell.exe. The parent-child relationship between the terminal app and its shells does not exist.

**Why it happens:** ConPTY is a pseudo-terminal implementation where the hosting process (Windows Terminal) communicates with shells through pipes, not process hierarchy. OpenConsole.exe is the PTY master that spawns the shell, but OpenConsole.exe is not a child of WindowsTerminal.exe.

**Prevention:**
- The existing ConPTY fallback in `find_shell_by_ancestry` (searching for shells parented by OpenConsole.exe) is the correct approach for Windows Terminal
- When no descendant shells are found and the app is a known terminal (WindowsTerminal.exe), always try the ConPTY fallback
- The default "powershell" fallback in `detect_app_context_windows` (line 505) is a reasonable last resort but should be documented as a guess
- Any new shell-finding logic must be tested with Windows Terminal specifically because its process architecture is unlike other terminal hosts

**Detection:** If Windows Terminal detection consistently returns the default shell_type="powershell" regardless of what shell is actually running, ConPTY disambiguation needs improvement.

**Phase relevance:** Any phase modifying process tree walking or shell detection.

---

### Pitfall 8: UIA Text Is Noisy for Electron Apps (VS Code/Cursor)

**What goes wrong:** UIA tree walking for VS Code returns text from the entire window: menus ("File", "Edit", "View"), sidebar file names, editor content, status bar items, AND terminal text. WSL detection patterns must work despite this noise.

**Why it happens:** VS Code is a single-HWND Electron app. Unlike Windows Terminal which has a dedicated UIA TextPattern provider for terminal content, VS Code's UIA tree is the full Chromium accessibility tree. The `try_walk_children` function walks all descendants and collects Name properties indiscriminately.

**Prevention:**
- Detection patterns (detect_wsl_from_text, infer_shell_from_text) must be specific enough to avoid false positives from editor/menu content
- The user@host:path prompt pattern is highly specific and unlikely to appear in editor content -- prioritize this over bare path detection
- Consider filtering UIA elements by control type: List elements correspond to xterm.js accessibility tree
- The `is_window_chrome` filter already rejects title bar button text -- extend this approach to filter known non-terminal patterns
- For VS Code, attempt to find the terminal-specific UIA subtree (List control type with xterm.js characteristics) before falling back to full-tree walking

**Detection:** If detect_wsl_from_text returns true when the user has a Python file open containing Linux paths but NO WSL terminal, this noise is causing false positives.

**Phase relevance:** UIA text reading and WSL detection phases.

---

### Pitfall 9: Window Title WSL Detection Only Works for Remote-WSL Mode

**What goes wrong:** `detect_wsl_from_title` checks for "[WSL: Ubuntu]" in the VS Code window title. This pattern only appears when VS Code is connected via Remote-WSL extension (full WSL mode). When a user opens a local WSL terminal profile (Terminal > New Terminal > Ubuntu), the window title does NOT contain "[WSL:]".

**Why it happens:** Remote-WSL is a full VS Code operating mode where the extension host runs inside WSL. The window title reflects this mode. A local WSL terminal profile is just another terminal tab -- VS Code does not change the window title for individual terminal profiles.

**Consequences:** Remote-WSL terminals are detected but local WSL terminal tabs in VS Code are not. This means WSL detection depends entirely on UIA text patterns for the local-WSL-terminal-profile case.

**Prevention:**
- Window title detection is a fast, reliable signal but only for Remote-WSL mode
- For local WSL terminal profiles, fall through to UIA text-based detection (detect_wsl_from_text)
- The multi-signal approach (title -> UIA text -> CWD path) handles both scenarios -- do not short-circuit after title check fails
- Document this limitation: window title is a fast-path optimization, not the sole detection mechanism

**Detection:** If Remote-WSL terminals are detected but local WSL terminal tabs in VS Code are not, this is the cause.

**Phase relevance:** WSL detection in non-Remote-WSL mode -- the primary v0.2.8 target.

---

### Pitfall 10: wsl.exe Subprocess Returns Home Directory, Not Active Terminal CWD

**What goes wrong:** `get_wsl_cwd()` spawns `wsl.exe -e sh -c "pwd"` which returns the default user's home directory (/home/user), not the CWD of the user's active WSL terminal session. Similarly, `get_wsl_shell()` reads $SHELL which is the configured default, not necessarily the running shell.

**Why it happens:** `wsl.exe -e` starts a new shell process in the default WSL distro. This new process has its own CWD (home directory) and reads the system $SHELL variable. It has no connection to the user's existing terminal session.

**Prevention:**
- Use wsl.exe subprocess values as FALLBACK only, after UIA text inference fails
- UIA text inference (`infer_linux_cwd_from_text`) provides the actual terminal CWD by parsing prompt patterns like `user@host:/actual/path$` -- this is more accurate
- The current code uses `.or()` pattern: `process::get_wsl_cwd().or(terminal.cwd.take())` -- when UIA CWD is available, it should take precedence
- The comment in process.rs (line 1080) already documents this: "returns the HOME directory... UIA text inference provides better CWD when available"

**Detection:** If WSL CWD always shows as /home/username regardless of the actual directory, subprocess values are overriding UIA-inferred values.

**Phase relevance:** WSL context accuracy. Ensure UIA-inferred CWD overrides subprocess CWD in the pipeline ordering.

---

### Pitfall 11: detect_wsl_from_text False Positives from Editor Content

**What goes wrong:** The function checks for Linux paths (/home/, /root/, /var/, etc.) anywhere in the UIA text. If a user is editing a file that contains these paths (a Dockerfile, a shell script, Python code with path strings), WSL will be falsely detected even when no WSL terminal is open.

**Why it happens:** The current `detect_wsl_from_text` scans ALL UIA text for Linux path strings. In VS Code, UIA text includes editor content alongside terminal output. A Python file containing `open("/home/user/data.csv")` would trigger the `/home/` pattern.

**Consequences:** Non-WSL sessions get WSL context applied. The system prompt switches to Linux mode. Generated commands are Linux commands pasted into a PowerShell terminal.

**Prevention:**
- Require multiple signals: Linux path + Linux prompt pattern (user@host:) together, not just a bare path
- Weight prompt patterns (user@host:path$ or user@host:~$) more heavily than bare path mentions
- Consider checking only the last N lines of text for prompt patterns (prompts appear at the bottom of terminal output)
- For VS Code specifically: if both terminal text and editor text are mixed, a single "/home/" in editor code should not trigger WSL detection without a corroborating prompt pattern
- Add negative signals: if the text also contains "PS C:\>" or "C:\>" patterns, it is NOT WSL despite any Linux paths in editor content

**Detection:** Trigger CMD+K from VS Code with a Python/Shell/Docker file open that contains Linux paths, but with a PowerShell terminal tab. If is_wsl=true, false positive is occurring.

**Phase relevance:** WSL detection accuracy. Should be addressed when refining detect_wsl_from_text.

---

## Minor Pitfalls

These cause suboptimal behavior or developer confusion but are not user-facing regressions.

---

### Pitfall 12: Multiple CreateToolhelp32Snapshot Calls Per Detection Cycle

**What goes wrong:** The current code takes 3-4 separate process snapshots per detection cycle: one in `find_shell_by_ancestry`, one in `detect_wsl_in_ancestry`, one in `scan_wsl_processes_diagnostic`, and one in `get_child_pids_windows`. Each snapshot is ~1ms but they add up, and the process table can change between snapshots leading to inconsistent parent-child views.

**Prevention:**
- Take a single snapshot at the start of detection, build the parent map and exe map once, pass them through all functions
- `find_shell_by_ancestry` already builds a local parent_map but `detect_wsl_in_ancestry` takes its own separate snapshot
- Refactoring to a shared snapshot would improve both consistency and performance
- Consider adding a `ProcessSnapshot` struct that encapsulates parent_map + exe_map, created once per detection cycle

**Phase relevance:** Performance optimization phase. Not a correctness issue but improves detection consistency.

---

### Pitfall 13: Shell Type Normalization Inconsistency

**What goes wrong:** Shell names are normalized differently in different code paths. `get_process_name` strips ".exe" but not lowercasing. `exe_to_shell_type` lowercases and strips ".exe". `infer_shell_from_text` returns lowercase strings. The `KNOWN_SHELLS` list uses lowercase without ".exe" but `KNOWN_SHELL_EXES` uses original case with ".exe".

**Prevention:**
- Establish a single canonical form: lowercase, no ".exe" suffix (e.g., "powershell", "cmd", "bash")
- Normalize at the boundary (when reading process name or inferring from text) and use canonical form everywhere internally
- The existing shell_type stripping in `detect_app_context_windows` (lines 479-483) is a symptom of this inconsistency -- it should happen once in the detection layer, not at consumption time

**Phase relevance:** Any phase touching shell type values. Low priority but prevents confusion.

---

### Pitfall 14: 750ms Timeout Budget Pressure

**What goes wrong:** The `detect_full_with_hwnd` function has a 750ms hard timeout. Adding more detection steps (additional UIA traversals, wsl.exe subprocess calls, multiple snapshot walks) risks exceeding this budget, causing the function to return None and losing all context.

**Prevention:**
- Budget (approximate): window title < 1ms, process tree walk ~5-10ms, single UIA text read 50-200ms, wsl.exe subprocess 100-300ms
- Total sequential: title + process + UIA + WSL subprocess = ~250-500ms, within budget
- Danger zone: multiple UIA traversals (e.g., tree walk + focused element + tab list) or serial wsl.exe calls
- Combine wsl.exe CWD and shell queries into a single subprocess call if both are needed
- Make wsl.exe subprocess calls lazy -- only invoke AFTER WSL is detected through faster means (title or UIA text)
- The current code already follows this pattern: wsl.exe subprocess only runs when is_wsl is already true

**Detection:** If detect_full_with_hwnd intermittently returns None (visible as missing context in the overlay), timeout pressure is the likely cause.

**Phase relevance:** Any phase adding detection steps. Must measure timing on real Windows hardware before merging.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation | Severity |
|-------------|---------------|------------|----------|
| IDE process filtering | Pitfall 3, 6 (highest PID picks internal process, cmd.exe pollution) | Filter by ConPTY parent, CWD match, deprioritize known-internal patterns | Critical |
| WSL detection in non-Remote-WSL | Pitfall 1, 2, 4, 9 (wsl.exe as shell, circular deps, Hyper-V invisibility, title-only) | Multi-signal output-based detection, unconditional UIA text check | Critical |
| cmd vs PowerShell differentiation | Pitfall 3, 6, 13 (wrong process selected, cmd.exe internal, normalization) | IDE-mode filtering, consistent normalization, CWD-based disambiguation | Moderate |
| UIA/Win32 with Electron | Pitfall 5, 8, 14 (focused element fails, noisy text, timeout) | Use HWND-based element, specific patterns, budget management | Moderate |
| CWD reading for WSL | Pitfall 10, 11 (subprocess returns home dir, false positives from editor) | Prefer UIA-inferred CWD, require multiple WSL signals | Moderate |
| Refactoring detection pipeline | Pitfall 2, 12 (circular dependency reintroduction, multiple snapshots) | Waterfall structure, shared snapshot | Critical during refactor |

---

## Integration Pitfalls (Adding Fixes to Existing System)

These are specific to the v0.2.8 scenario: modifying a working detection pipeline to fix specific bugs without breaking other paths.

---

### Integration Risk 1: Fixing VS Code Detection Breaks Windows Terminal or Standalone Shells

**What goes wrong:** The detection pipeline has 4 terminal host paths: Windows Terminal (ConPTY), VS Code, Cursor, and standalone terminals (powershell.exe, cmd.exe directly). Changes to IDE-specific logic inadvertently affect non-IDE paths. For example, adding process filtering that deprioritizes cmd.exe globally would break standalone cmd.exe terminal detection.

**Prevention:**
- ALL IDE-specific logic MUST be gated behind `is_ide_with_terminal_exe` checks -- never apply IDE heuristics globally
- Test matrix after every change: {Windows Terminal, VS Code, Cursor, standalone PowerShell, standalone cmd} x {PowerShell tab, CMD tab, WSL bash tab, Git Bash} = 20 scenarios
- The current code correctly gates IDE cmd.exe deprioritization behind `if is_ide && descendant_shells.len() > 1` -- preserve this pattern
- Add regression tests (even manual) for Windows Terminal ConPTY fallback -- it is the most fragile path

---

### Integration Risk 2: Ordering Changes in Multi-Signal Detection Pipeline

**What goes wrong:** The detection pipeline runs signals in order: process tree -> window title -> UIA text -> CWD path. Changing the order, adding early returns, or restructuring can skip signals that would have caught a case the earlier signal missed.

**Prevention:**
- Each signal should be able to SET is_wsl=true but should NEVER CLEAR it (no `is_wsl = false` after initial detection)
- Later signals should REFINE context (better CWD, better shell_type) not OVERRIDE earlier detection
- If window title says WSL, UIA text should not contradict it -- but UIA text CAN provide CWD/shell details that title lacks
- The `.or()` chaining pattern (e.g., `process::get_wsl_cwd().or(terminal.cwd.take())`) determines priority -- ensure UIA-inferred values are preferred when available

---

### Integration Risk 3: Breaking the Pre-Capture-Before-Show Pattern

**What goes wrong:** Terminal context detection requires the frontmost app's PID and HWND captured BEFORE the overlay steals focus. Any refactoring that moves detection logic to a different point in the pipeline may not have access to the pre-captured values.

**Prevention:**
- The hotkey handler in `commands/hotkey.rs` captures PID and HWND synchronously before calling toggle_overlay
- These values must flow through to `detect_full_with_hwnd` unchanged
- Never call UIA or process APIs with the overlay's own PID/HWND -- this returns CMD+K's own process info
- If adding new detection logic, verify it receives the ORIGINAL HWND/PID, not current-foreground values

---

### Integration Risk 4: Losing Existing WSL Detection That Works

**What goes wrong:** The current multi-signal pipeline (title + UIA text + CWD path) already works for some WSL scenarios: Remote-WSL in VS Code (via title), WSL in Windows Terminal (via UIA text), and WSL with UNC paths (via CWD). Refactoring to fix other scenarios (local WSL terminal profiles) risks breaking these working paths.

**Prevention:**
- Document which scenarios currently work BEFORE making changes
- Currently working: Remote-WSL (title "[WSL: distro]"), Windows Terminal WSL (UIA text Linux prompts), CWD \\wsl$\ paths
- Currently broken: local WSL terminal profiles in VS Code without Remote-WSL, cmd vs PowerShell misdetection in IDE
- Changes must preserve the working paths while fixing the broken ones
- Consider additive changes (new signals, new patterns) over restructuring changes (reordering, replacing existing logic)

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical verification for v0.2.8.

- [ ] **WSL detection in VS Code:** is_wsl=true when WSL tab is active -- but verify is_wsl=false when ONLY PowerShell tabs are open (Pitfall 1 negative test)
- [ ] **Shell type differentiation:** shell_type shows "powershell" for PowerShell tab -- but verify it is the USER's PowerShell terminal, not an internal VS Code powershell.exe process (Pitfall 3)
- [ ] **cmd.exe filtering:** cmd.exe is deprioritized in IDE mode -- but verify standalone cmd.exe (not in IDE) still detects correctly (Integration Risk 1)
- [ ] **UIA text WSL detection:** detect_wsl_from_text finds Linux prompts -- but verify it does NOT false-positive on editor content containing Linux paths (Pitfall 11)
- [ ] **CWD for WSL sessions:** CWD shows Linux path -- but verify it is the ACTUAL terminal CWD, not /home/username (Pitfall 10)
- [ ] **Windows Terminal still works:** All changes pass Windows Terminal test -- verify ConPTY fallback path is untouched (Integration Risk 1)
- [ ] **Detection pipeline ordering:** All WSL signals run -- but verify no signal can CLEAR is_wsl after it is set to true (Integration Risk 2)
- [ ] **Timeout budget:** Detection completes within 750ms -- but verify on actual Windows hardware, not just local dev (Pitfall 14)

---

## Sources

### Primary (HIGH confidence -- project's own failure history)
- Commit bcea9cd: Revert of wsl.exe in shell list, commit message documents VS Code spawning 16+ internal wsl.exe processes
- Commit e0ef4be: Original WSL fix that was reverted
- Commit 1e4cd7a: Removal of wsl.exe from KNOWN_TERMINAL_EXES (correct fix)
- Commit d63f909: UIA fix removing broken focused subtree approach
- `.planning/debug/wsl-detection-failure.md`: Detailed diagnosis of 3 compounding root causes (architectural, circular dependency, missing patterns)
- `.planning/milestones/v0.2.6-phases/23.1-vs-code-wsl-terminal-tab-detection-via-uia/23.1-RESEARCH.md`: UIA strategy research with VS Code accessibility tree analysis
- Source code: `src-tauri/src/terminal/mod.rs` (detection pipeline), `process.rs` (process tree walking), `detect_windows.rs` (shell/terminal lists), `uia_reader.rs` (UIA text reading)

### Secondary (MEDIUM confidence -- domain knowledge from prior research)
- WSL 2 architecture: Linux processes in Hyper-V VM, invisible to CreateToolhelp32Snapshot
- VS Code Remote-WSL window title format "[WSL: distro]" is stable across versions
- ConPTY architecture: shells not descendants of WindowsTerminal.exe, connected via OpenConsole.exe
- Chromium UIA bridge: Electron apps expose DOM accessibility tree via UIA, xterm.js accessibility tree requires screen reader mode

---
*Pitfalls research for: CMD+K v0.2.8 Windows Terminal Detection Fix*
*Researched: 2026-03-11*
