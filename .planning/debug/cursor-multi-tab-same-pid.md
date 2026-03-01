---
status: diagnosed
trigger: "Cursor IDE with multiple terminal tabs always resolves to the same shell PID (83090)"
created: 2026-03-01T00:00:00Z
updated: 2026-03-01T00:00:00Z
---

## Root Cause Analysis

### Summary

The bug is confirmed. `find_shell_pid` correctly finds ALL shell descendants of
the Cursor IDE process, but has no mechanism to determine which shell corresponds
to the currently focused terminal tab. It falls back to a heuristic -- "highest
PID" -- which always picks the same one regardless of tab focus.

### The Selection Logic (Confirmed)

In `src-tauri/src/terminal/process.rs`, the `find_shell_by_ancestry` function
(line 340-437) implements a three-step selection after collecting all descendant
shells of the IDE process:

1. **Filter sub-shells** (lines 382-401): Removes shells that are children of
   other shells (e.g., Claude Code's bash spawned from a user's zsh). This step
   is correct.

2. **Prefer $SHELL type** (lines 405-432): If mixed shell types remain (e.g.,
   extension-spawned bash + user zsh), prefer shells matching $SHELL. This step
   is a reasonable heuristic for filtering out extension-spawned shells.

3. **Pick highest PID** (lines 434-436): Among remaining candidates, selects
   `max_by_key(|(pid, _)| *pid)`. This is the flawed heuristic.

The comment on line 338 states: "picks the most recently spawned one (highest PID),
which is most likely the active/focused terminal tab." This assumption is false.

**Why highest PID fails:** PIDs on macOS are assigned sequentially by the kernel
and wrap around. The highest PID is the most recently *created* shell, not the
most recently *focused* tab. If a user opens tabs A, B, C in order, then
switches focus to tab A, the code still returns tab C's shell PID (highest).
PIDs do not change when a tab gains or loses focus.

### Why The Fast Path Fails for IDEs

The fast path `find_shell_recursive` (line 523-557) walks direct children of the
terminal PID, looking for shells up to 3 levels deep. This works for Terminal.app
because the tree is shallow: `Terminal.app -> login -> zsh`.

For Cursor/VS Code, the tree is deep and wide:
```
Cursor (39593)
  +-- Cursor Helper (GPU)
  +-- Cursor Helper (Renderer)
  +-- ...
  +-- Cursor Helper (Plugin Host)
       +-- node
            +-- pty-helper
                 +-- zsh (498)    [tab 1]
                 +-- zsh (15095)  [tab 2]
                 +-- zsh (83090)  [tab 3]
```

The recursive walk at depth 3 cannot reach shells buried 4-5 levels deep in the
Electron process tree. It falls through to `find_shell_by_ancestry` which does
the broad `pgrep` search.

### Is This Fixable?

**Process tree alone: NO.** The macOS process tree contains no information about
which terminal tab is focused in an Electron app. All shells are peer descendants
of the same pty-helper process. There is no OS-level "foreground process group"
equivalent for IDE terminal tabs like there is for real TTYs.

**Potential approaches (ranked by feasibility):**

1. **macOS Accessibility API (AXFocusedUIElement)** -- MOST PROMISING
   - The code already uses AX APIs in `ax_reader.rs` for text reading.
   - For Cursor/VS Code, `AXFocusedUIElement` on the app returns the focused
     terminal pane's AX element.
   - The AX tree exposes terminal tab title, which typically contains the CWD
     or shell PID info (e.g., "zsh - /Users/foo/project").
   - Could also walk from the focused element to find `AXValue` text containing
     the shell prompt, then match that CWD against the shell PIDs' CWDs.
   - **Approach:** For each candidate shell PID, get its CWD via `get_process_cwd`.
     Get the focused terminal tab's title/content via AX. Match the CWD that
     appears in the focused tab to identify the correct shell PID.
   - **Limitation:** Requires Accessibility permissions (already required).
     AX tree activation for Electron is already handled by `ensure_ax_tree_active`.

2. **TTY device matching** -- FEASIBLE BUT FRAGILE
   - Each IDE terminal tab has its own PTY (pseudo-terminal).
   - Could read `/dev/ttys*` device for each shell via `ps -o tty= -p <pid>`.
   - The focused tab's PTY might be identifiable via the AX tree or some
     Electron-specific IPC, but there is no standard API for this.
   - More fragile than option 1.

3. **IDE extension/plugin** -- RELIABLE BUT HEAVY
   - A VS Code / Cursor extension can use the `vscode.window.activeTerminal` API
     to get the focused terminal's `processId`.
   - Would require IPC between the extension and this Tauri app (e.g., local
     HTTP, Unix socket, or filesystem).
   - Most reliable but adds a significant dependency.

4. **Accept the limitation** -- PRAGMATIC
   - Document that multi-tab IDE terminal detection picks an arbitrary tab.
   - Use the window key `bundle_id:app_pid` for IDEs instead of
     `bundle_id:shell_pid`, giving one history bucket per IDE window.
   - Simple, no false precision, but loses per-tab history separation.

### Recommendation

**Option 1 (AX-based CWD matching)** is the best balance of reliability and
implementation complexity. The infrastructure already exists:
- AX tree activation for Electron is already in `ax_reader.rs`
- CWD reading per shell PID is already in `process.rs`
- The focused terminal tab's AX element is accessible via the existing framework

The algorithm would be:
1. Collect all candidate shell PIDs (existing logic).
2. For each candidate, read its CWD via `get_process_cwd`.
3. Read the AXFocusedUIElement from the IDE (existing `get_ax_attribute` function).
4. Extract the tab title or visible text from the focused element.
5. Match the CWD from step 2 against the text from step 4.
6. If a match is found, return that shell PID. Otherwise, fall back to highest PID.

**Edge case:** Multiple tabs open to the same directory would still be ambiguous.
This could be refined by also matching the running process name or shell prompt
content. But even a CWD-based match would be correct for the majority of
real-world usage (users typically have different project directories per tab).

### Files Involved

- `src-tauri/src/terminal/process.rs` (lines 340-437): `find_shell_by_ancestry`
  contains the flawed "highest PID" heuristic at line 435.
- `src-tauri/src/commands/hotkey.rs` (lines 67-84): `compute_window_key` calls
  `find_shell_pid` and uses the result as the window key discriminator.
- `src-tauri/src/terminal/ax_reader.rs`: Already has the AX infrastructure
  needed for the fix (AXFocusedUIElement, text extraction, Electron tree
  activation).

### Evidence

- Log shows `find_shell_pid` ancestry search finds 3 shells: [(498, "zsh"),
  (15095, "zsh"), (83090, "zsh")]. All are the same shell type (zsh), so the
  $SHELL preference filter has no effect. Sub-shell filtering also has no effect
  (they are peer processes, not parent-child). The code falls through to
  `max_by_key` which deterministically picks 83090.
- The comment at line 338 explicitly acknowledges this is a "most likely" guess,
  not a reliable determination.
- For Terminal.app, the fast path works because each window/tab IS a direct child
  of the Terminal process, so the recursive walk finds the right shell without
  needing the ancestry fallback.
