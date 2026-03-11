//! Windows UI Automation (UIA) terminal text reading.
//!
//! Uses the `uiautomation` crate (safe Rust wrapper around Windows UI Automation COM)
//! to read visible terminal text from Windows Terminal, PowerShell/CMD conhost, and
//! other UIA-capable terminal emulators.
//!
//! Returns None (graceful fallback) for mintty, GPU terminals, or any UIA error.

#[cfg(target_os = "windows")]
use uiautomation::UIAutomation;
#[cfg(target_os = "windows")]
use uiautomation::types::Handle;
#[cfg(target_os = "windows")]
use uiautomation::types::TreeScope;
#[cfg(target_os = "windows")]
use uiautomation::types::{UIProperty, ControlType};
use uiautomation::variants::Variant;
#[cfg(target_os = "windows")]
use uiautomation::patterns::UITextPattern;

/// Maximum text size to capture (matches macOS TEXT_BUF_SIZE).
#[cfg(target_os = "windows")]
const TEXT_BUF_SIZE: usize = 65_536;

/// Read visible terminal text from a window identified by HWND.
///
/// Uses UI Automation TextPattern to extract the visible text content.
/// Supports:
/// - Windows Terminal (UIA TextPattern on terminal control)
/// - PowerShell/CMD conhost (native UIA support)
///
/// Returns None for:
/// - mintty (no UIA support)
/// - GPU terminals (Alacritty, kitty, WezTerm) — may not expose UIA text
/// - Any UIA initialization or text extraction error
///
/// Text is truncated to TEXT_BUF_SIZE (65KB) to match macOS behavior.
#[cfg(target_os = "windows")]
pub fn read_terminal_text_windows(hwnd: isize) -> Option<String> {
    read_terminal_text_inner(hwnd).unwrap_or_else(|e| {
        eprintln!("[uia_reader] UIA text reading failed (graceful fallback): {}", e);
        None
    })
}

#[cfg(not(target_os = "windows"))]
pub fn read_terminal_text_windows(_hwnd: isize) -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
fn read_terminal_text_inner(hwnd: isize) -> Result<Option<String>, String> {
    let automation = UIAutomation::new().map_err(|e| format!("UIAutomation::new failed: {}", e))?;

    // Get the UIA element from the HWND
    let element = automation
        .element_from_handle(Handle::from(hwnd))
        .map_err(|e| format!("element_from_handle failed: {}", e))?;

    // Try to get text via TextPattern first (most reliable for terminals).
    // Reject if it looks like VS Code window chrome (short text with "Minimize"/"Close").
    if let Ok(text) = try_text_pattern(&element) {
        if !text.is_empty() && !is_window_chrome(&text) {
            let truncated = truncate_text(&text, TEXT_BUF_SIZE);
            return Ok(Some(truncated));
        }
    }

    // Strategy 2: Scoped terminal walk (targets ControlType::List elements from xterm.js)
    // For VS Code/Cursor, this finds the terminal panel's accessibility tree (role="list")
    // and reads only terminal line text, excluding editor, sidebar, and menu content.
    eprintln!("[uia_reader] trying scoped terminal walk...");
    if let Ok(text) = try_scoped_terminal_walk(&automation, &element) {
        if !text.is_empty() {
            eprintln!("[uia_reader] scoped walk returned {} chars", text.len());
            let truncated = truncate_text(&text, TEXT_BUF_SIZE);
            return Ok(Some(truncated));
        }
    }

    // Strategy 3: Full tree walk (fallback for non-IDE terminals and edge cases)
    eprintln!("[uia_reader] scoped walk failed, falling back to full tree walk");
    if let Ok(text) = try_walk_children(&automation, &element) {
        if !text.is_empty() {
            let truncated = truncate_text(&text, TEXT_BUF_SIZE);
            return Ok(Some(truncated));
        }
    }

    eprintln!("[uia_reader] no text found via UIA for HWND {}", hwnd);
    Ok(None)
}

/// Try to read text using UIA TextPattern on the element or its children.
#[cfg(target_os = "windows")]
fn try_text_pattern(element: &uiautomation::UIElement) -> Result<String, String> {
    // Try TextPattern on the element itself
    if let Ok(text_pattern) = element.get_pattern::<UITextPattern>() {
        if let Ok(range) = text_pattern.get_document_range() {
            if let Ok(text) = range.get_text(-1) {
                if !text.is_empty() {
                    eprintln!("[uia_reader] TextPattern found {} chars on root element", text.len());
                    return Ok(text);
                }
            }
        }
    }

    // Try TextPattern on direct children (Windows Terminal has the text in a child control)
    // Use walker to iterate children safely
    if let Ok(walker) = UIAutomation::new().and_then(|a| a.create_tree_walker()) {
        let mut child = walker.get_first_child(&element).ok();
        while let Some(ref elem) = child {
            if let Ok(text_pattern) = elem.get_pattern::<UITextPattern>() {
                if let Ok(range) = text_pattern.get_document_range() {
                    if let Ok(text) = range.get_text(-1) {
                        if !text.is_empty() {
                            eprintln!("[uia_reader] TextPattern found {} chars on child element", text.len());
                            return Ok(text);
                        }
                    }
                }
            }
            child = walker.get_next_sibling(elem).ok();
        }
    }

    Err("No TextPattern found".to_string())
}

/// Walk children looking for elements with Name text content.
#[cfg(target_os = "windows")]
fn try_walk_children(
    automation: &UIAutomation,
    element: &uiautomation::UIElement,
) -> Result<String, String> {
    let mut text_parts: Vec<String> = Vec::new();
    let mut total_len: usize = 0;

    // Walk descendants using tree walker
    let true_condition = automation
        .create_true_condition()
        .map_err(|e| format!("create_true_condition failed: {}", e))?;

    if let Ok(children) = element.find_all(TreeScope::Descendants, &true_condition) {
        for child in children {
            if total_len >= TEXT_BUF_SIZE {
                break;
            }

            // Try Name property
            if let Ok(name) = child.get_name() {
                if !name.is_empty() && name.len() > 1 {
                    total_len += name.len();
                    text_parts.push(name);
                }
            }
        }
    }

    if text_parts.is_empty() {
        return Err("No text found in children".to_string());
    }

    Ok(text_parts.join("\n"))
}


/// Scoped terminal walk: find ControlType::List elements (xterm.js accessibility tree)
/// and read only their children's Name properties. This targets the terminal panel
/// in VS Code/Cursor, excluding editor content, sidebars, and menus.
#[cfg(target_os = "windows")]
fn try_scoped_terminal_walk(
    automation: &UIAutomation,
    element: &uiautomation::UIElement,
) -> Result<String, String> {
    // Find all List elements (xterm.js terminal uses role="list" -> ControlType::List)
    let list_condition = automation
        .create_property_condition(UIProperty::ControlType, Variant::from(ControlType::List as i32), None)
        .map_err(|e| format!("create list condition failed: {}", e))?;

    let lists = element
        .find_all(TreeScope::Descendants, &list_condition)
        .map_err(|e| format!("find_all List elements failed: {}", e))?;

    let true_condition = automation
        .create_true_condition()
        .map_err(|e| format!("create_true_condition failed: {}", e))?;

    // Track visible lists and their text
    let total_lists = lists.len();
    let mut visible_count = 0usize;
    let mut best_text = String::new();
    let mut best_len = 0usize;

    for list in &lists {
        // Skip offscreen elements (inactive terminal tabs)
        let is_offscreen = list
            .get_property_value(UIProperty::IsOffscreen)
            .ok()
            .and_then(|v| v.try_into().ok())
            .unwrap_or(false);

        if is_offscreen {
            continue;
        }
        visible_count += 1;

        // Get children (list items = terminal lines)
        let children = match list.find_all(TreeScope::Children, &true_condition) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut text_parts: Vec<String> = Vec::new();
        for child in &children {
            if let Ok(name) = child.get_name() {
                if !name.is_empty() {
                    text_parts.push(name);
                }
            }
        }

        let text = text_parts.join("\n");
        if looks_like_terminal_text(&text) && text.len() > best_len {
            best_len = text.len();
            best_text = text;
        }
    }

    eprintln!(
        "[uia_reader] found {} List elements, {} visible",
        total_lists, visible_count
    );

    if best_text.is_empty() {
        return Err("No terminal-like text found in List elements".to_string());
    }

    Ok(best_text)
}

/// Best-effort heuristic to determine if text looks like terminal output.
/// Checks for prompt patterns, command output indicators, and sufficient content.
/// False negatives are acceptable -- the full tree walk fallback handles misses.
#[cfg(target_os = "windows")]
fn looks_like_terminal_text(text: &str) -> bool {
    let non_empty_lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();

    // Need at least 3 non-empty lines for terminal output
    if non_empty_lines.len() < 3 {
        return false;
    }

    // Check for common terminal indicators
    for line in &non_empty_lines {
        let trimmed = line.trim();

        // PowerShell prompt: "PS C:\..." or "PS>"
        if trimmed.starts_with("PS ") || trimmed.starts_with("PS>") {
            return true;
        }

        // CMD prompt: single drive letter followed by ":\"
        if trimmed.len() >= 3 {
            let bytes = trimmed.as_bytes();
            if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'\\' {
                return true;
            }
        }

        // Linux prompt: contains "@" with "$" or "#" ending
        if trimmed.contains('@') && (trimmed.ends_with('$') || trimmed.ends_with('#')) {
            return true;
        }

        // Linux prompt: user@host:/path pattern
        if trimmed.contains('@') {
            if let Some(colon_pos) = trimmed.find(':') {
                let before = &trimmed[..colon_pos];
                if before.contains('@') {
                    let after = &trimmed[colon_pos + 1..];
                    let path = after.trim_end_matches(|c: char| c == '$' || c == '#' || c == ' ');
                    if path.starts_with('/') || path.starts_with('~') {
                        return true;
                    }
                }
            }
        }
    }

    // Fallback: substantial text (20+ non-empty lines) is likely terminal output
    // even without recognizable prompts (e.g., command output, logs)
    non_empty_lines.len() >= 20
}

/// Reject UIA text that's clearly window chrome (title bar buttons), not terminal content.
/// VS Code sometimes returns "Title\nMinimize\nRestore\nClose" from TextPattern.
#[cfg(target_os = "windows")]
fn is_window_chrome(text: &str) -> bool {
    text.len() < 200 && text.contains("Minimize") && text.contains("Close")
}

/// Truncate text to a maximum byte size, splitting on a newline boundary.
#[cfg(target_os = "windows")]
fn truncate_text(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    // Find the last newline within the limit
    let slice = &text[..max_bytes];
    match slice.rfind('\n') {
        Some(pos) => slice[..pos].to_string(),
        None => slice.to_string(),
    }
}
