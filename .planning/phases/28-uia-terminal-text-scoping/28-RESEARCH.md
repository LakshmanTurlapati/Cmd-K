# Phase 28: UIA Terminal Text Scoping - Research

**Researched:** 2026-03-11
**Domain:** Windows UI Automation element scoping for VS Code/Cursor terminal panels
**Confidence:** MEDIUM

## Summary

Phase 28 addresses two related problems in the current UIA text reading pipeline: (1) `try_walk_children` in `uia_reader.rs` uses `TreeScope::Descendants` with `create_true_condition()`, capturing ALL text from every element in the VS Code/Cursor window -- editor content, sidebar items, menus, and terminal text are indiscriminately concatenated; (2) `detect_wsl_from_text` in `mod.rs` triggers WSL detection on a single Linux path like `/home/` or `/etc/` appearing anywhere in the captured text, which false-positives when those paths appear in editor content (e.g., a Dockerfile or config file open in the editor).

The fix has two independent parts. For UIAS-01, the UIA tree walk must be scoped to only read text from the terminal panel's subtree, filtering by ControlType, ClassName, or structural position. For UIAS-02, `detect_wsl_from_text` must require multiple corroborating signals (e.g., Linux path + user@host prompt pattern) rather than triggering on a single indicator.

**Primary recommendation:** Scope `try_walk_children` by filtering the UIA tree to elements under the terminal panel (using the `UIMatcher` API with `classname()` and `control_type()` filters), and harden `detect_wsl_from_text` to require 2+ independent WSL indicators before returning true.

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| UIAS-01 | UIA text reading scoped to terminal panel elements only -- editor, sidebars, menus excluded | UIMatcher/property condition filtering by ControlType and ClassName; empirical UIA tree inspection needed |
| UIAS-02 | WSL text detection requires multiple corroborating signals -- single Linux path insufficient | Scoring/threshold system for WSL indicators; existing `detect_wsl_from_text` refactored to count signals |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| uiautomation | 0.24 | Windows UIA tree walking, element property queries, condition-based filtering | Already in Cargo.toml; wraps COM IUIAutomation safely |
| windows-sys | 0.59 | Low-level Win32 API calls (HWND, PID) | Already in Cargo.toml |

### Supporting
No new crates needed. All work is refactoring existing `uia_reader.rs` and `mod.rs`.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| UIMatcher API | Raw create_property_condition | UIMatcher has fluent builder, simpler code; raw conditions more flexible but verbose |
| ClassName filtering | AutomationId filtering | ClassName is more reliable for Chromium/Electron apps; AutomationId often empty |

## Architecture Patterns

### Current Architecture (Problem)

```
read_terminal_text_inner(hwnd)
  -> try_text_pattern(element)       // Strategy 1: TextPattern on root/children
  -> try_walk_children(element)      // Strategy 2: Walk ALL descendants, get Name
     -> find_all(TreeScope::Descendants, true_condition)  // CAPTURES EVERYTHING
     -> collect all Name properties into one string
```

For VS Code/Cursor (Electron apps), Strategy 1 (TextPattern) typically returns window chrome text ("Minimize", "Close") which is rejected by `is_window_chrome()`. Strategy 2 then runs and captures ALL text from every UI element in the window.

### Recommended Architecture (Fix)

```
read_terminal_text_inner(hwnd)
  -> try_text_pattern(element)       // Strategy 1: unchanged (works for Windows Terminal)
  -> try_scoped_terminal_walk(element)  // Strategy 2: SCOPED to terminal panel
     -> find terminal panel subtree via ClassName/ControlType filtering
     -> walk only terminal panel descendants
     -> collect text only from terminal elements
```

### Pattern 1: Scoped UIA Tree Walk with UIMatcher

**What:** Use UIMatcher to locate the terminal panel element, then walk only its descendants.
**When to use:** When reading text from IDE windows (VS Code, Cursor) where terminal is one panel among many.

**How VS Code/Cursor UIA tree is structured (Chromium/Electron):**

VS Code is an Electron app. Its UIA tree follows Chromium's accessibility mapping:
- Root window: ControlType::Pane, ClassName "Chrome_WidgetWin_1"
  - Chrome render host: ClassName "Chrome_RenderWidgetHostHWND"
    - Document element (web content root): ControlType::Document or ControlType::Pane
      - Editor panels, sidebar, terminal panel etc. as children

The xterm.js terminal creates an accessibility tree with:
- Container div: role="list" -> UIA ControlType::List
- Line items: role="listitem" -> UIA ControlType::ListItem
- Each listitem has a Name property containing the terminal line text

**Strategy for finding the terminal panel:**
1. Look for ControlType::List elements whose Name or ancestor context indicates "Terminal"
2. Filter by the characteristic that terminal list items contain prompt/command text patterns
3. Fall back to current full-tree walk if scoped search fails (graceful degradation)

```rust
// Pseudocode - actual implementation needs empirical verification
fn try_scoped_terminal_walk(
    automation: &UIAutomation,
    element: &UIElement,
) -> Result<String, String> {
    // Strategy A: Find List elements (xterm accessibility tree = role="list")
    let list_condition = automation.create_property_condition(
        UIProperty::ControlType,
        ControlType::List.into(),
        None,
    )?;

    let lists = element.find_all(TreeScope::Descendants, &list_condition)?;

    // If multiple lists found, pick the one with terminal-like content
    // (prompt patterns, command output, etc.)
    for list in &lists {
        let items = list.find_all(TreeScope::Children, &listitem_condition)?;
        let text = items.iter()
            .filter_map(|item| item.get_name().ok())
            .collect::<Vec<_>>()
            .join("\n");
        if looks_like_terminal_text(&text) {
            return Ok(text);
        }
    }

    // Strategy B: Fall back to walking by Name matching
    // ... (graceful degradation)

    Err("No terminal panel found".to_string())
}
```

### Pattern 2: Multi-Signal WSL Detection

**What:** Replace single-indicator WSL detection with a scoring system.
**When to use:** When UIA text may contain mixed content (terminal + non-terminal).

```rust
fn detect_wsl_from_text(text: &str) -> bool {
    let mut score: u32 = 0;
    let threshold: u32 = 2; // Require 2+ independent signals

    // Signal 1: Linux paths in text (weak - could be editor content)
    let linux_paths = ["/home/", "/root/", "/var/", "/etc/", "/usr/", "/tmp/", "/opt/"];
    if linux_paths.iter().any(|lp| text.contains(lp)) {
        score += 1;
    }

    // Signal 2: user@host:/path prompt (strong - terminal-specific)
    if has_linux_prompt_pattern(text) {
        score += 1;
    }

    // Signal 3: user@host...$ or # prompt ending (strong)
    if has_prompt_ending_pattern(text) {
        score += 1;
    }

    // Signal 4: WSL-specific markers (very strong)
    if text.contains("/mnt/c/") || text.contains("/mnt/d/") {
        score += 2; // WSL mount point is almost certainly WSL
    }

    score >= threshold
}
```

### Anti-Patterns to Avoid

- **Full-window text capture for IDEs:** Never use `find_all(TreeScope::Descendants, true_condition)` on an IDE window. This captures thousands of elements including editor text, file explorer, breadcrumbs, status bar.
- **Single-signal WSL detection:** A Linux path like `/home/user` appearing in editor content (e.g., a README, Dockerfile, shell script) must NOT trigger WSL mode.
- **Hardcoded tree depth assumptions:** The UIA tree depth for VS Code's terminal varies by version and configuration. Use condition-based search, not fixed depth walking.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UIA element filtering | Manual tree traversal with depth counting | `UIMatcher` with `control_type()` and `classname()` | Handles depth, timeout, and condition composition |
| Property conditions | Raw COM calls to IUIAutomation::CreatePropertyCondition | `automation.create_property_condition()` | Already wrapped safely by uiautomation crate |
| Text pattern extraction | Manual Name property collection with string building | `UITextPattern::get_document_range().get_text()` | Handles encoding, range selection properly |

## Common Pitfalls

### Pitfall 1: VS Code UIA Tree Structure Varies by Version
**What goes wrong:** Hardcoding tree positions (e.g., "terminal is 3rd child of 2nd pane") breaks on VS Code updates.
**Why it happens:** VS Code/Electron updates frequently change the DOM structure.
**How to avoid:** Filter by ControlType and element properties (Name, ClassName), not by tree position. Use find_all with conditions.
**Warning signs:** Tests pass on one VS Code version but fail on another.

### Pitfall 2: Chromium Accessibility Tree Activation
**What goes wrong:** UIA tree for Chromium-based apps may be sparse or empty until accessibility is "activated" by a WM_GETOBJECT message to Chrome_RenderWidgetHostHWND.
**Why it happens:** Chromium lazily initializes its accessibility tree for performance.
**How to avoid:** The existing `element_from_handle` call should trigger accessibility activation. If the tree is empty, the current fallback to full text walk handles this. Ensure the new scoped walk also has a fallback.
**Warning signs:** First read returns empty, subsequent reads work.

### Pitfall 3: Multiple Terminal Tabs in VS Code
**What goes wrong:** VS Code can have multiple terminal tabs. The scoped walk might capture text from an inactive terminal tab.
**Why it happens:** All terminal tabs may exist in the UIA tree, even if only one is visible.
**How to avoid:** If multiple List elements are found, prefer the one that is visible (check bounding rectangle or IsOffscreen property) or the one with the most recent/relevant content.
**Warning signs:** Terminal text from a different tab leaks into detection.

### Pitfall 4: Screen Reader Mode Changes UIA Behavior
**What goes wrong:** When VS Code screen reader mode is enabled, the terminal accessibility tree structure may change (more verbose, different roles).
**Why it happens:** xterm.js has a separate screen reader mode that exposes additional accessibility elements.
**How to avoid:** Out of scope per REQUIREMENTS.md ("Screen reader mode injection for VS Code UIA text" is explicitly out of scope). Just ensure the scoped walk handles both cases without crashing.
**Warning signs:** Different element structure when accessibility settings change.

### Pitfall 5: False WSL Detection from Editor Content
**What goes wrong:** A file open in VS Code containing `/home/user` or `/etc/nginx/nginx.conf` triggers WSL detection even though the terminal is running PowerShell on Windows.
**Why it happens:** Current `detect_wsl_from_text` checks for ANY Linux path in ALL captured text.
**How to avoid:** Require multiple corroborating signals. A Linux path alone (score=1) is insufficient; need a prompt pattern (score=1) or WSL mount path (score=2) to reach threshold.
**Warning signs:** WSL mode activates when editing Linux-related files in a Windows terminal session.

## Code Examples

### Example 1: Creating a ControlType Condition
```rust
// Source: uiautomation crate docs (docs.rs/uiautomation)
use uiautomation::UIAutomation;
use uiautomation::types::{UIProperty, ControlType, TreeScope};

let automation = UIAutomation::new()?;
let element = automation.element_from_handle(Handle::from(hwnd))?;

// Find all List elements (xterm terminal accessibility tree uses role="list")
let list_condition = automation.create_property_condition(
    UIProperty::ControlType,
    ControlType::List.into(),
    None,
)?;

let lists = element.find_all(TreeScope::Descendants, &list_condition)?;
```

### Example 2: Using UIMatcher for Scoped Search
```rust
// Source: uiautomation crate README (github.com/leexgone/uiautomation-rs)
let automation = UIAutomation::new()?;
let root = automation.element_from_handle(Handle::from(hwnd))?;

let matcher = automation.create_matcher()
    .from(root)
    .control_type(ControlType::List)
    .timeout(500)  // 500ms timeout to avoid blocking
    .depth(10);     // Search up to 10 levels deep

if let Ok(terminal_list) = matcher.find_first() {
    // Found the terminal panel's list element
    let children = terminal_list.find_all(
        TreeScope::Children,
        &automation.create_true_condition()?,
    )?;
    // Read Name from each child (list items = terminal lines)
}
```

### Example 3: Multi-Signal WSL Detection
```rust
/// Detect WSL from visible terminal text using multiple corroborating signals.
/// Requires at least 2 independent indicators to avoid false positives from
/// editor content containing Linux paths.
fn detect_wsl_from_text(text: &str) -> bool {
    let mut score: u32 = 0;

    // Weak signal: Linux paths (could appear in editor content)
    let linux_paths = ["/home/", "/root/", "/var/", "/etc/", "/usr/", "/tmp/", "/opt/"];
    if linux_paths.iter().any(|lp| text.contains(lp)) {
        score += 1;
    }

    // Strong signal: WSL mount points (almost certainly WSL)
    if text.contains("/mnt/c/") || text.contains("/mnt/d/") {
        score += 2;
    }

    // Strong signal: user@host:/path or user@host:~ prompt
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let before = &trimmed[..colon_pos];
            if before.contains('@') {
                let after = &trimmed[colon_pos + 1..];
                let path = after.trim_end_matches(|c: char| c == '$' || c == '#' || c == ' ');
                if path.starts_with('/') || path.starts_with('~') {
                    score += 1;
                    break;
                }
            }
        }
    }

    // Strong signal: prompt ending with $ or # after user@host
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.contains('@') && !trimmed.contains('\\') {
            if trimmed.ends_with('$') || trimmed.ends_with('#') {
                score += 1;
                break;
            }
        }
    }

    score >= 2
}
```

### Example 4: Checking Element Visibility
```rust
// Filter out offscreen terminal tabs
use uiautomation::types::UIProperty;

fn is_visible(element: &UIElement) -> bool {
    element.get_property_value(UIProperty::IsOffscreen)
        .and_then(|v| v.try_into().ok())
        .map(|offscreen: bool| !offscreen)
        .unwrap_or(true)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Full tree walk with true_condition | Scoped walk with ControlType/ClassName conditions | This phase | Eliminates editor/sidebar text leaking into terminal capture |
| Single Linux path triggers WSL | Multi-signal scoring (threshold >= 2) | This phase | Eliminates false WSL detection from editor content |

**Important note:** The exact UIA tree structure of VS Code's terminal panel needs empirical verification using Inspect.exe or Accessibility Insights on a running VS Code instance. The xterm.js accessibility tree (role="list" -> ControlType::List) mapping is based on the W3C Core Accessibility API Mappings spec and VS Code issue discussions, but the actual element hierarchy may vary.

## Open Questions

1. **Exact VS Code Terminal UIA Tree Structure**
   - What we know: xterm.js uses role="list" with role="listitem" children. Chromium maps these to ControlType::List and ControlType::ListItem per W3C Core-AAM spec.
   - What's unclear: The exact nesting depth, whether there are intermediary Pane/Group elements, what the Name property contains for the List element itself, and whether inactive terminal tabs are visible in the tree.
   - Recommendation: First task in the plan should be an empirical investigation using `Inspect.exe` or `Accessibility Insights for Windows` on a running VS Code instance with a terminal open. Log the tree structure to guide implementation. Alternatively, add debug logging to dump the UIA tree.

2. **Cursor IDE Differences**
   - What we know: Cursor is a VS Code fork, so its UIA tree should be very similar.
   - What's unclear: Whether Cursor has any modifications to the terminal accessibility tree.
   - Recommendation: Test on both VS Code and Cursor after implementation. Likely identical.

3. **Windows Terminal UIA Behavior**
   - What we know: Strategy 1 (TextPattern) works for Windows Terminal. Strategy 2 (tree walk) is the fallback for IDEs.
   - What's unclear: Whether the scoping changes affect Windows Terminal behavior.
   - Recommendation: Ensure the new scoped walk only activates for IDE windows, not standalone terminals. Use the existing `is_ide_with_terminal_exe` check.

4. **Performance Impact of Condition-Based Search**
   - What we know: `find_all` with conditions should be faster than full-tree walk because UIA can filter server-side.
   - What's unclear: Whether the condition-based search is significantly faster or about the same.
   - Recommendation: Should be fine within existing 750ms timeout. No special optimization needed.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test (#[test] + #[cfg(test)]) |
| Config file | None (Cargo.toml test section) |
| Quick run command | `cargo test --lib -p cmd-k -- --nocapture` |
| Full suite command | `cargo test --lib -p cmd-k` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UIAS-01 | Scoped UIA walk returns only terminal text | manual-only | Manual UAT with VS Code | N/A (UIA requires live window) |
| UIAS-01 | Fallback to full walk when scoped walk fails | unit | `cargo test --lib -p cmd-k -- uia_reader --nocapture` | Wave 0 |
| UIAS-02 | Single Linux path does NOT trigger WSL | unit | `cargo test --lib -p cmd-k -- detect_wsl --nocapture` | Wave 0 |
| UIAS-02 | Multiple signals DO trigger WSL | unit | `cargo test --lib -p cmd-k -- detect_wsl --nocapture` | Wave 0 |
| UIAS-02 | WSL mount path alone triggers WSL (strong signal) | unit | `cargo test --lib -p cmd-k -- detect_wsl --nocapture` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib -p cmd-k`
- **Per wave merge:** `cargo test --lib -p cmd-k` + manual UAT
- **Phase gate:** Full suite green + manual UAT before verify

### Wave 0 Gaps
- [ ] `src-tauri/src/terminal/mod.rs` tests section -- add `detect_wsl_from_text` unit tests (currently NO tests for this function)
- [ ] `src-tauri/src/terminal/uia_reader.rs` -- add unit tests for `is_window_chrome` and `truncate_text` (minor, but useful)

## Sources

### Primary (HIGH confidence)
- [uiautomation crate docs](https://docs.rs/uiautomation/latest/uiautomation/) - UIElement methods, UIMatcher API, condition creation
- [uiautomation-rs GitHub](https://github.com/leexgone/uiautomation-rs) - Usage examples, UIMatcher builder pattern
- [W3C Core Accessibility API Mappings 1.2](https://w3c.github.io/core-aam/) - ARIA role to UIA ControlType mappings (list->List, listitem->ListItem, document->Document)
- Project source: `src-tauri/src/terminal/uia_reader.rs`, `mod.rs`, `detect_windows.rs` - Current implementation

### Secondary (MEDIUM confidence)
- [VS Code Issue #98918](https://github.com/microsoft/vscode/issues/98918) - xterm.js terminal uses role="list" with role="listitem" children, class="xterm-accessibility-tree"
- [Chromium UIA documentation](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/accessibility/browser/uiautomation.md) - Chromium implements UIA provider APIs, maps to standard control types

### Tertiary (LOW confidence)
- Exact VS Code UIA tree structure -- based on spec mappings, needs empirical verification with Inspect.exe
- Cursor IDE UIA behavior -- assumed identical to VS Code, needs testing

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - using existing crate (uiautomation 0.24), no new dependencies
- Architecture (UIAS-01 scoping): MEDIUM - W3C spec says list/listitem mapping, but actual VS Code UIA tree needs empirical verification
- Architecture (UIAS-02 multi-signal): HIGH - pure logic refactoring with clear test cases
- Pitfalls: MEDIUM - based on Chromium accessibility behavior knowledge, some verified via official docs

**Research date:** 2026-03-11
**Valid until:** 2026-04-11 (stable domain, VS Code UIA tree may change with updates)
