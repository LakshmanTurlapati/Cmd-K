//! Accessibility tree text reader for Terminal.app and iTerm2.
//!
//! Uses the macOS Accessibility API (ApplicationServices framework) via the
//! accessibility-sys crate to read visible terminal text.
//!
//! AX tree paths:
//! - Terminal.app: AXApplication -> AXFocusedWindow -> AXScrollArea -> AXTextArea -> AXValue
//! - iTerm2:       AXApplication -> AXFocusedUIElement (active text area) -> AXValue
//!
//! GPU-rendered terminals (Alacritty, kitty, WezTerm) do not expose text via AX.
//! Callers should check is_gpu_terminal() before calling read_terminal_text().
//! Any AX error is treated as a silent None return (per the research pitfalls).

#[cfg(target_os = "macos")]
mod macos {
    // Use accessibility_sys for the AX error success constant.
    // This makes the accessibility-sys crate dependency explicit and verifiable.
    use accessibility_sys::kAXErrorSuccess as AX_SUCCESS;

    use std::collections::HashSet;
    use std::ffi::{c_char, c_void, CStr};
    use std::os::raw::c_long;
    use std::sync::Mutex as StdMutex;

    // Raw CF type aliases. These match the definitions in core-foundation-sys 0.8.x.
    // We declare them here to avoid taking a direct dependency on core-foundation-sys.
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFIndex = c_long;
    type CFStringEncoding = u32;

    const K_CF_STRING_ENCODING_UTF8: CFStringEncoding = 0x08000100;

    // PID activation cache -- ensures the ~150ms AX tree activation sleep
    // only happens once per app launch (per PID).
    static ACTIVATED_PIDS: std::sync::LazyLock<StdMutex<HashSet<i32>>> =
        std::sync::LazyLock::new(|| StdMutex::new(HashSet::new()));

    // AX attribute name constants -- these match accessibility_sys string constants exactly.
    const AX_FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
    const AX_FOCUSED_WINDOW: &str = "AXFocusedWindow";
    const AX_CHILDREN: &str = "AXChildren";
    const AX_ROLE: &str = "AXRole";
    const AX_VALUE: &str = "AXValue";
    const AX_TITLE: &str = "AXTitle";
    const AX_SCROLL_AREA_ROLE: &str = "AXScrollArea";
    const AX_TEXT_AREA_ROLE: &str = "AXTextArea";
    const AX_STATIC_TEXT_ROLE: &str = "AXStaticText";
    const AX_TEXT_FIELD_ROLE: &str = "AXTextField";
    const AX_WEB_AREA_ROLE: &str = "AXWebArea";

    // K_AX_ERROR_SUCCESS is imported from accessibility_sys as AX_SUCCESS above.

    // Maximum recursion depth when walking the AX children tree.
    const MAX_DEPTH: u32 = 5;

    // Maximum recursion depth for generic app text collection.
    // Electron apps (Notion, Slack, etc.) nest text inside AXGroups at depth 10-15.
    const GENERIC_MAX_DEPTH: u32 = 15;

    // Maximum number of AX elements to visit during generic text collection.
    // Prevents slow traversal on apps with very large AX trees.
    const GENERIC_MAX_ELEMENTS: u32 = 500;

    // Maximum text to collect from generic apps (~4KB).
    const GENERIC_TEXT_LIMIT: usize = 4096;

    // Buffer size for reading terminal text (64KB).
    const TEXT_BUF_SIZE: CFIndex = 65536;

    // FFI declarations for ApplicationServices / CoreFoundation functions.
    // All of these are stable public macOS APIs.
    extern "C" {
        // AX API (ApplicationServices.framework)
        fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
        fn AXUIElementSetMessagingTimeout(element: CFTypeRef, timeout_seconds: f32) -> i32;
        fn AXUIElementCopyAttributeValue(
            element: CFTypeRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementSetAttributeValue(
            element: CFTypeRef,
            attribute: CFStringRef,
            value: CFTypeRef,
        ) -> i32;

        // CoreFoundation string API (CoreFoundation.framework)
        fn CFStringCreateWithBytes(
            alloc: CFTypeRef,
            bytes: *const u8,
            num_bytes: CFIndex,
            encoding: CFStringEncoding,
            is_external_representation: u8,
        ) -> CFStringRef;
        fn CFStringGetCString(
            the_string: CFStringRef,
            buffer: *mut c_char,
            buffer_size: CFIndex,
            encoding: CFStringEncoding,
        ) -> u8;

        // CoreFoundation array API
        fn CFArrayGetCount(the_array: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(the_array: CFArrayRef, idx: CFIndex) -> CFTypeRef;

        // CoreFoundation memory management
        fn CFRelease(cf: CFTypeRef);
        fn CFRetain(cf: CFTypeRef) -> CFTypeRef;

        // CoreFoundation boolean constant
        static kCFBooleanTrue: CFTypeRef;
    }

    /// Read visible terminal text for Terminal.app or iTerm2 via the Accessibility tree.
    ///
    /// - `app_pid`:   PID of the terminal application process.
    /// - `bundle_id`: Bundle identifier used to choose the correct AX traversal path.
    ///
    /// Returns `Some(text)` on success, `None` on any AX error or unsupported terminal.
    pub(super) fn read_terminal_text(app_pid: i32, bundle_id: &str) -> Option<String> {
        // SAFETY: AXUIElementCreateApplication is a stable public macOS API.
        // The returned AXUIElementRef (opaque CFTypeRef) is owned by this function
        // and released at the end of the scope.
        unsafe {
            let app_elem = AXUIElementCreateApplication(app_pid);
            if app_elem.is_null() {
                return None;
            }

            // Set a 1-second per-element messaging timeout to prevent hangs on
            // unresponsive or GPU-accelerated terminals (Pitfall 4 from research).
            AXUIElementSetMessagingTimeout(app_elem, 1.0);

            let result = if bundle_id == "com.googlecode.iterm2" {
                read_iterm2(app_elem)
            } else {
                // Default path for Terminal.app and any future AX-capable terminal.
                read_terminal_app(app_elem)
            };

            CFRelease(app_elem);
            result
        }
    }

    /// iTerm2 path: AXApplication -> kAXFocusedUIElementAttribute -> kAXValueAttribute.
    ///
    /// iTerm2 exposes the active text area directly via the focused UI element attribute,
    /// so no tree walk is needed.
    unsafe fn read_iterm2(app_elem: CFTypeRef) -> Option<String> {
        let focused = get_ax_attribute(app_elem, AX_FOCUSED_UI_ELEMENT)?;
        let text = get_ax_string_value(focused);
        CFRelease(focused);
        text
    }

    /// Terminal.app path: AXApplication -> AXFocusedWindow -> children walk -> AXTextArea -> AXValue.
    unsafe fn read_terminal_app(app_elem: CFTypeRef) -> Option<String> {
        let window = get_ax_attribute(app_elem, AX_FOCUSED_WINDOW)?;
        let text_area = find_text_area_in_children(window, MAX_DEPTH);
        let result = if let Some(ta) = text_area {
            let text = get_ax_string_value(ta);
            CFRelease(ta);
            text
        } else {
            None
        };
        CFRelease(window);
        result
    }

    /// Get an AX attribute value from an element.
    ///
    /// Wraps AXUIElementCopyAttributeValue. Returns None on any AX error (silent fallback).
    /// The caller is responsible for CFReleasing the returned value.
    unsafe fn get_ax_attribute(element: CFTypeRef, attribute: &str) -> Option<CFTypeRef> {
        let attr_cf = cf_string_from_str(attribute)?;
        let mut value: CFTypeRef = std::ptr::null();

        let err = AXUIElementCopyAttributeValue(element, attr_cf, &mut value);
        CFRelease(attr_cf);

        if err != AX_SUCCESS || value.is_null() {
            // Silent fallback -- kAXErrorCannotComplete, kAXErrorNotImplemented, etc.
            // are all treated as "this terminal doesn't support this attribute."
            return None;
        }

        Some(value)
    }

    /// Extract the kAXValueAttribute as a Rust String from an AX element.
    ///
    /// Returns None if the attribute is missing or not a CFString.
    unsafe fn get_ax_string_value(element: CFTypeRef) -> Option<String> {
        let value = get_ax_attribute(element, AX_VALUE)?;
        let text = cf_type_to_string(value);
        CFRelease(value);
        text
    }

    /// Extract the kAXRoleAttribute as a Rust String from an AX element.
    unsafe fn get_ax_role(element: CFTypeRef) -> Option<String> {
        let value = get_ax_attribute(element, AX_ROLE)?;
        let role = cf_type_to_string(value);
        CFRelease(value);
        role
    }

    /// Walk the children of an AX element to find an AXTextArea.
    ///
    /// - Recurses into AXScrollArea elements (which contain AXTextArea in Terminal.app).
    /// - Stops at `max_depth` to prevent runaway walks.
    /// - The caller is responsible for CFReleasing the returned CFTypeRef.
    unsafe fn find_text_area_in_children(
        element: CFTypeRef,
        max_depth: u32,
    ) -> Option<CFTypeRef> {
        if max_depth == 0 {
            return None;
        }

        let children_val = get_ax_attribute(element, AX_CHILDREN)?;
        let children_array = children_val as CFArrayRef;
        let count = CFArrayGetCount(children_array);

        let mut result: Option<CFTypeRef> = None;

        for i in 0..count {
            let child = CFArrayGetValueAtIndex(children_array, i);
            if child.is_null() {
                continue;
            }

            let role = get_ax_role(child);
            match role.as_deref() {
                Some(r) if r == AX_TEXT_AREA_ROLE => {
                    // CFArrayGetValueAtIndex does NOT retain the item.
                    // We must retain it before releasing children_val (the array),
                    // so the item stays alive for the caller.
                    CFRetain(child);
                    result = Some(child);
                    break;
                }
                Some(r) if r == AX_SCROLL_AREA_ROLE => {
                    // Recurse into scroll area to find text area inside.
                    if let Some(ta) = find_text_area_in_children(child, max_depth - 1) {
                        result = Some(ta);
                        break;
                    }
                }
                _ => {}
            }
        }

        CFRelease(children_val);
        result
    }

    /// Convert a CFTypeRef that holds a CFString to a Rust String.
    ///
    /// Uses CFStringGetCString to copy the string content into a local buffer.
    /// Returns None if the ref is null or if content cannot be converted.
    unsafe fn cf_type_to_string(cf: CFTypeRef) -> Option<String> {
        if cf.is_null() {
            return None;
        }

        let cf_str = cf as CFStringRef;
        let mut buf: Vec<u8> = vec![0u8; TEXT_BUF_SIZE as usize];

        let ok = CFStringGetCString(
            cf_str,
            buf.as_mut_ptr() as *mut c_char,
            TEXT_BUF_SIZE,
            K_CF_STRING_ENCODING_UTF8,
        );

        if ok == 0 {
            return None;
        }

        let c_str = CStr::from_ptr(buf.as_ptr() as *const c_char);
        Some(c_str.to_string_lossy().into_owned())
    }

    /// Create a CFStringRef from a Rust &str (UTF-8).
    ///
    /// The caller is responsible for CFReleasing the returned CFStringRef.
    unsafe fn cf_string_from_str(s: &str) -> Option<CFStringRef> {
        let bytes = s.as_bytes();
        let cf_str = CFStringCreateWithBytes(
            std::ptr::null(), // use default allocator
            bytes.as_ptr(),
            bytes.len() as CFIndex,
            K_CF_STRING_ENCODING_UTF8,
            0, // not external (BOM) representation
        );

        if cf_str.is_null() {
            None
        } else {
            Some(cf_str)
        }
    }

    /// Ensure the AX tree is fully built for the given application.
    ///
    /// Electron/Chromium apps (Notion, Slack, Discord, VS Code) do not construct
    /// their accessibility tree until explicitly told to. This function sets
    /// BOTH `AXEnhancedUserInterface` (what Chromium actually responds to) and
    /// `AXManualAccessibility` (covers other apps) unconditionally.
    ///
    /// Creates a dedicated AXUIElement with a generous 2.0s timeout for the
    /// attribute-setting IPC round-trip, decoupled from the caller's reading
    /// timeout (the fast pre-capture path uses only 200ms, which is too short
    /// for AXEnhancedUserInterface IPC to Electron apps).
    ///
    /// The first activation for a PID incurs a 150ms sleep to let the tree build.
    /// Subsequent calls for the same PID return immediately (cached).
    /// Known AX error codes for diagnostics.
    const AX_ERROR_NOT_IMPLEMENTED: i32 = -25208;

    unsafe fn ensure_ax_tree_active(pid: i32) {
        // Fast path: already activated this PID
        {
            let cache = ACTIVATED_PIDS.lock().unwrap();
            if cache.contains(&pid) {
                return;
            }
        }

        // Create a dedicated AX element with a generous timeout for activation.
        // This decouples the activation round-trip from the caller's reading timeout.
        let activation_elem = AXUIElementCreateApplication(pid);
        if activation_elem.is_null() {
            let mut cache = ACTIVATED_PIDS.lock().unwrap();
            cache.insert(pid);
            return;
        }
        AXUIElementSetMessagingTimeout(activation_elem, 2.0);

        let mut any_succeeded = false;
        // Track if we got -25208 (kAXErrorNotImplemented) -- a known Electron bug
        // where the attribute IS set internally but the error code is wrong because
        // it falls through to NSApplication's superclass (Electron PR #38102).
        let mut maybe_succeeded = false;

        // Set AXEnhancedUserInterface -- this is what Chromium/Electron responds to
        if let Some(attr) = cf_string_from_str("AXEnhancedUserInterface") {
            let err = AXUIElementSetAttributeValue(activation_elem, attr, kCFBooleanTrue);
            CFRelease(attr);
            if err == AX_SUCCESS {
                eprintln!("[ax_reader] set AXEnhancedUserInterface for pid {}", pid);
                any_succeeded = true;
            } else if err == AX_ERROR_NOT_IMPLEMENTED {
                // Known Electron bug: attribute may have been set internally despite
                // the error (falls through to NSApplication superclass). Treat as
                // potential success -- we'll still sleep to give the tree time to build.
                eprintln!("[ax_reader] AXEnhancedUserInterface returned -25208 (NotImplemented) for pid {} -- may have succeeded internally", pid);
                maybe_succeeded = true;
            } else {
                eprintln!("[ax_reader] AXEnhancedUserInterface failed for pid {} (err={})", pid, err);
            }
        }

        // Also set AXManualAccessibility -- covers apps that use this instead
        if let Some(attr) = cf_string_from_str("AXManualAccessibility") {
            let err = AXUIElementSetAttributeValue(activation_elem, attr, kCFBooleanTrue);
            CFRelease(attr);
            if err == AX_SUCCESS {
                eprintln!("[ax_reader] set AXManualAccessibility for pid {}", pid);
                any_succeeded = true;
            } else if err == AX_ERROR_NOT_IMPLEMENTED {
                eprintln!("[ax_reader] AXManualAccessibility returned -25208 (NotImplemented) for pid {} -- may have succeeded internally", pid);
                maybe_succeeded = true;
            } else {
                eprintln!("[ax_reader] AXManualAccessibility failed for pid {} (err={})", pid, err);
            }
        }

        // Fallback: try setting AXEnhancedUserInterface on the focused window element.
        // Some Chromium versions handle this at the window level rather than application level.
        if !any_succeeded {
            if let Some(window) = get_ax_attribute(activation_elem, AX_FOCUSED_WINDOW) {
                if let Some(attr) = cf_string_from_str("AXEnhancedUserInterface") {
                    let err = AXUIElementSetAttributeValue(window, attr, kCFBooleanTrue);
                    CFRelease(attr);
                    if err == AX_SUCCESS {
                        eprintln!("[ax_reader] set AXEnhancedUserInterface on window for pid {}", pid);
                        any_succeeded = true;
                    } else if err == AX_ERROR_NOT_IMPLEMENTED {
                        eprintln!("[ax_reader] AXEnhancedUserInterface on window returned -25208 for pid {} -- may have succeeded", pid);
                        maybe_succeeded = true;
                    }
                }
                CFRelease(window);
            }
        }

        CFRelease(activation_elem);

        if any_succeeded || maybe_succeeded {
            // Give the app time to build the tree on first activation.
            // We sleep even for "maybe succeeded" (-25208) because the attribute
            // may have taken effect internally despite the error code.
            std::thread::sleep(std::time::Duration::from_millis(150));
        } else {
            eprintln!(
                "[ax_reader] AX tree activation not needed for pid {} (native app)",
                pid
            );
        }

        // Cache this PID regardless
        let mut cache = ACTIVATED_PIDS.lock().unwrap();
        cache.insert(pid);
    }

    /// Read visible text from ANY app via the Accessibility tree with a configurable timeout.
    ///
    /// Uses multiple strategies to extract text:
    /// 1. Walk the focused window's AX children tree (works for native apps)
    /// 2. Use AXFocusedUIElement to jump directly into Electron/web content
    /// 3. Read AXTitle/AXDescription from elements (not just AXValue)
    ///
    /// `timeout_secs` controls the per-element AX messaging timeout.
    /// Returns `Some(text)` with concatenated visible text, `None` on any AX error
    /// or if no text elements are found.
    pub(super) fn read_focused_text_with_timeout(app_pid: i32, timeout_secs: f32) -> Option<String> {
        unsafe {
            let app_elem = AXUIElementCreateApplication(app_pid);
            if app_elem.is_null() {
                eprintln!("[ax_reader:generic] AXUIElementCreateApplication returned null for pid {}", app_pid);
                return None;
            }

            AXUIElementSetMessagingTimeout(app_elem, timeout_secs);

            // Activate the AX tree for Electron/Chromium apps before walking it.
            // This is a no-op for native apps and cached after first call per PID.
            ensure_ax_tree_active(app_pid);

            let mut collected = String::new();
            let mut element_count: u32 = 0;

            // Strategy 1: Walk from focused window (works for native apps)
            if let Some(window) = get_ax_attribute(app_elem, AX_FOCUSED_WINDOW) {
                collect_text_from_children(window, GENERIC_MAX_DEPTH, &mut collected, &mut element_count);
                CFRelease(window);
                eprintln!(
                    "[ax_reader:generic] strategy 1: {} elements, {} bytes",
                    element_count, collected.len()
                );
            }

            // Strategy 2: Use AXFocusedUIElement to jump into content
            // (critical for Electron apps where window children are shallow AXGroups)
            // Also runs when strategy 1 only captured a small amount (e.g., just a title)
            if collected.len() < 200 {
                if let Some(focused) = get_ax_attribute(app_elem, AX_FOCUSED_UI_ELEMENT) {
                    // Read text directly from focused element
                    append_element_text(focused, &mut collected);

                    // Walk children of the focused element to get surrounding content
                    element_count = 0;
                    collect_text_from_children(focused, GENERIC_MAX_DEPTH, &mut collected, &mut element_count);

                    // Also walk the parent to get sibling content (other blocks on the page)
                    if let Some(parent) = get_ax_attribute(focused, "AXParent") {
                        collect_text_from_children(parent, GENERIC_MAX_DEPTH.min(10), &mut collected, &mut element_count);
                        CFRelease(parent);
                    }

                    eprintln!(
                        "[ax_reader:generic] strategy 2: {} elements, {} bytes",
                        element_count, collected.len()
                    );
                    CFRelease(focused);
                }
            }

            CFRelease(app_elem);

            if collected.is_empty() {
                None
            } else {
                Some(collected)
            }
        }
    }

    /// Read visible text from any app with the default 1.0s AX timeout.
    pub(super) fn read_focused_text(app_pid: i32) -> Option<String> {
        read_focused_text_with_timeout(app_pid, 1.0)
    }

    /// Read visible text from any app with a fast 0.2s AX timeout.
    ///
    /// Intended for use in the hotkey handler BEFORE the overlay steals focus.
    /// Bounds the hotkey handler delay to ~200ms max.
    pub(super) fn read_focused_text_fast(app_pid: i32) -> Option<String> {
        read_focused_text_with_timeout(app_pid, 0.2)
    }

    /// Extract the CWD from the focused terminal tab's AX title or value text.
    ///
    /// Intended for Electron IDEs (Cursor, VS Code) with multiple terminal tabs.
    /// The focused tab's AXTitle often contains the CWD (e.g., "zsh - /Users/foo/project").
    /// Falls back to extracting a path from the last line of AXValue (shell prompt).
    ///
    /// Returns `Some(path)` if a valid directory path was found, `None` otherwise.
    /// Uses a short 0.3s messaging timeout since this runs in the hotkey handler path.
    pub(super) fn get_focused_terminal_cwd(app_pid: i32) -> Option<String> {
        unsafe {
            let app_elem = AXUIElementCreateApplication(app_pid);
            if app_elem.is_null() {
                return None;
            }

            // Short timeout -- this runs in the hotkey handler path and must be fast.
            AXUIElementSetMessagingTimeout(app_elem, 0.3);

            // Wake up Electron AX trees if needed (cached after first call per PID).
            ensure_ax_tree_active(app_pid);

            // Get the focused UI element (the active terminal tab's text area).
            let focused = match get_ax_attribute(app_elem, AX_FOCUSED_UI_ELEMENT) {
                Some(f) => f,
                None => {
                    eprintln!("[ax_reader] get_focused_terminal_cwd: no AXFocusedUIElement for pid {}", app_pid);
                    CFRelease(app_elem);
                    return None;
                }
            };

            // Strategy 1: Try AXTitle (more reliable -- tab titles often contain the CWD).
            // Common formats:
            //   "zsh - /Users/foo/project"
            //   "1: zsh - /Users/foo/project"
            //   "/Users/foo/project"
            //   "node server.js" (no path)
            let mut result: Option<String> = None;

            if let Some(title_ref) = get_ax_attribute(focused, AX_TITLE) {
                if let Some(title) = cf_type_to_string(title_ref) {
                    eprintln!("[ax_reader] focused element AXTitle: {:?}", &title);
                    result = extract_dir_path_from_text(&title);
                }
                CFRelease(title_ref);
            }

            // Strategy 2: If no path from title, try extracting from the last line of
            // AXValue text (the shell prompt often contains the CWD).
            if result.is_none() {
                if let Some(value) = get_ax_string_value(focused) {
                    if let Some(last_line) = value.lines().rev().find(|l| !l.trim().is_empty()) {
                        eprintln!("[ax_reader] focused element AXValue last line: {:?}", last_line);
                        result = extract_dir_path_from_text(last_line);
                    }
                }
            }

            if let Some(ref path) = result {
                eprintln!("[ax_reader] get_focused_terminal_cwd for pid {}: {}", app_pid, path);
            } else {
                eprintln!("[ax_reader] get_focused_terminal_cwd for pid {}: no path found", app_pid);
            }

            CFRelease(focused);
            CFRelease(app_elem);
            result
        }
    }

    /// Try to extract a valid directory path from a text string.
    ///
    /// Handles common terminal tab title formats:
    /// - "zsh - /Users/foo/project" (split on " - ")
    /// - "1: zsh - /Users/foo/project" (tab number prefix)
    /// - "/Users/foo/project" (just the path)
    /// - "~ " or "~/project" (tilde expansion)
    ///
    /// Returns None for strings without a valid directory path (e.g., "node server.js").
    fn extract_dir_path_from_text(text: &str) -> Option<String> {
        // Try splitting on " - " and checking each part for a path.
        // This handles "zsh - /Users/foo/project" and "1: zsh - /Users/foo/project".
        for part in text.split(" - ") {
            let trimmed = part.trim();
            if let Some(path) = try_as_dir_path(trimmed) {
                return Some(path);
            }
        }

        // Try the whole string as a path (handles "/Users/foo/project" directly).
        if let Some(path) = try_as_dir_path(text.trim()) {
            return Some(path);
        }

        None
    }

    /// Check if a string is a valid directory path, expanding ~ if needed.
    /// Returns the canonical path string if it is a directory, None otherwise.
    fn try_as_dir_path(s: &str) -> Option<String> {
        if s.is_empty() {
            return None;
        }

        // Expand tilde to home directory.
        let expanded = if s.starts_with('~') {
            if let Ok(home) = std::env::var("HOME") {
                if s == "~" {
                    home
                } else if let Some(rest) = s.strip_prefix("~/") {
                    format!("{}/{}", home, rest)
                } else {
                    return None; // ~otheruser -- not supported
                }
            } else {
                return None;
            }
        } else if s.starts_with('/') {
            s.to_string()
        } else {
            return None; // Not an absolute path or tilde path
        };

        if std::path::Path::new(&expanded).is_dir() {
            Some(expanded)
        } else {
            None
        }
    }

    /// Append text from an element's AXValue, AXTitle, or AXDescription to the output.
    unsafe fn append_element_text(element: CFTypeRef, out: &mut String) {
        // Try AXValue first (primary text content)
        if let Some(value) = get_ax_string_value(element) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                if !out.is_empty() {
                    out.push('\n');
                }
                let remaining = GENERIC_TEXT_LIMIT.saturating_sub(out.len());
                let to_add = if trimmed.len() <= remaining { trimmed } else { &trimmed[..remaining] };
                out.push_str(to_add);
                return;
            }
        }

        // Try AXTitle (used by headings, buttons, etc.)
        if let Some(title_ref) = get_ax_attribute(element, AX_TITLE) {
            if let Some(title) = cf_type_to_string(title_ref) {
                let trimmed = title.trim();
                if !trimmed.is_empty() {
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    let remaining = GENERIC_TEXT_LIMIT.saturating_sub(out.len());
                    let to_add = if trimmed.len() <= remaining { trimmed } else { &trimmed[..remaining] };
                    out.push_str(to_add);
                }
            }
            CFRelease(title_ref);
        }
    }

    /// Recursively walk AX children and collect text from text-bearing elements.
    ///
    /// Appends to `out` and stops when GENERIC_TEXT_LIMIT or GENERIC_MAX_ELEMENTS is reached.
    unsafe fn collect_text_from_children(
        element: CFTypeRef,
        depth: u32,
        out: &mut String,
        count: &mut u32,
    ) {
        if depth == 0 || out.len() >= GENERIC_TEXT_LIMIT || *count >= GENERIC_MAX_ELEMENTS {
            return;
        }

        *count += 1;

        let role = get_ax_role(element);

        let is_text_element = matches!(
            role.as_deref(),
            Some(r) if r == AX_STATIC_TEXT_ROLE
                || r == AX_TEXT_FIELD_ROLE
                || r == AX_TEXT_AREA_ROLE
                || r == AX_WEB_AREA_ROLE
        );

        if is_text_element {
            append_element_text(element, out);
        } else {
            // For non-text roles, try AXTitle (catches headings, labels, buttons)
            if let Some(title_ref) = get_ax_attribute(element, AX_TITLE) {
                if let Some(title) = cf_type_to_string(title_ref) {
                    let trimmed = title.trim();
                    if !trimmed.is_empty() && out.len() < GENERIC_TEXT_LIMIT {
                        if !out.is_empty() {
                            out.push('\n');
                        }
                        let remaining = GENERIC_TEXT_LIMIT.saturating_sub(out.len());
                        let to_add = if trimmed.len() <= remaining { trimmed } else { &trimmed[..remaining] };
                        out.push_str(to_add);
                    }
                }
                CFRelease(title_ref);
            }
        }

        // Recurse into children
        if out.len() >= GENERIC_TEXT_LIMIT || *count >= GENERIC_MAX_ELEMENTS {
            return;
        }

        if let Some(children_val) = get_ax_attribute(element, AX_CHILDREN) {
            let children_array = children_val as CFArrayRef;
            let child_count = CFArrayGetCount(children_array);

            for i in 0..child_count {
                if out.len() >= GENERIC_TEXT_LIMIT || *count >= GENERIC_MAX_ELEMENTS {
                    break;
                }
                let child = CFArrayGetValueAtIndex(children_array, i);
                if !child.is_null() {
                    collect_text_from_children(child, depth - 1, out, count);
                }
            }

            CFRelease(children_val);
        }
    }
}

/// Read visible terminal text via the Accessibility API.
///
/// On non-macOS targets this always returns None (stub for cross-compilation).
#[cfg(not(target_os = "macos"))]
pub fn read_terminal_text(_app_pid: i32, _bundle_id: &str) -> Option<String> {
    None
}

/// Read visible terminal text via the Accessibility API.
///
/// Delegates to the macOS implementation in the `macos` submodule.
/// On macOS this calls into the ApplicationServices AX tree walker.
#[cfg(target_os = "macos")]
pub fn read_terminal_text(app_pid: i32, bundle_id: &str) -> Option<String> {
    macos::read_terminal_text(app_pid, bundle_id)
}

/// Read visible text from any app via the Accessibility API.
///
/// On non-macOS targets this always returns None (stub for cross-compilation).
#[cfg(not(target_os = "macos"))]
pub fn read_focused_text(_app_pid: i32) -> Option<String> {
    None
}

/// Read visible text from any app via the Accessibility API.
///
/// Walks the focused window's AX children tree collecting text from
/// AXStaticText, AXTextField, and AXTextArea elements.
/// Returns up to ~4KB of concatenated text.
#[cfg(target_os = "macos")]
pub fn read_focused_text(app_pid: i32) -> Option<String> {
    macos::read_focused_text(app_pid)
}

/// Fast variant of read_focused_text with a 200ms AX timeout.
///
/// Designed for use in the hotkey handler before the overlay steals focus.
/// On non-macOS targets this always returns None.
#[cfg(not(target_os = "macos"))]
pub fn read_focused_text_fast(_app_pid: i32) -> Option<String> {
    None
}

/// Fast variant of read_focused_text with a 200ms AX timeout.
///
/// Designed for use in the hotkey handler before the overlay steals focus.
#[cfg(target_os = "macos")]
pub fn read_focused_text_fast(app_pid: i32) -> Option<String> {
    macos::read_focused_text_fast(app_pid)
}

/// Extract CWD from the focused terminal tab via AX title/value text.
///
/// On non-macOS targets this always returns None.
#[cfg(not(target_os = "macos"))]
pub fn get_focused_terminal_cwd(_app_pid: i32) -> Option<String> {
    None
}

/// Extract CWD from the focused terminal tab via AX title/value text.
///
/// Uses AXFocusedUIElement to read the title of the active terminal tab in
/// Electron IDEs (Cursor, VS Code). The title typically contains the CWD.
/// Falls back to parsing the last line of the terminal text content.
#[cfg(target_os = "macos")]
pub fn get_focused_terminal_cwd(app_pid: i32) -> Option<String> {
    macos::get_focused_terminal_cwd(app_pid)
}
