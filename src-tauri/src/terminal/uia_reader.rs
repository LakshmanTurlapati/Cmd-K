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

    // Try to get text via TextPattern first (most reliable for terminals)
    if let Ok(text) = try_text_pattern(&element) {
        if !text.is_empty() {
            let truncated = truncate_text(&text, TEXT_BUF_SIZE);
            return Ok(Some(truncated));
        }
    }

    // Fallback: walk child elements looking for text content
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
