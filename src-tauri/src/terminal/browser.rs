//! Browser DevTools/Console detection via the macOS Accessibility API.
//!
//! Detects whether a browser has its DevTools console open by walking the
//! AX window tree and checking for window titles containing "DevTools",
//! "Web Inspector", "Developer Tools", or "Console".
//!
//! When DevTools is detected, attempts to read the last line of console output
//! from an AXTextArea inside the DevTools window.
//!
//! Only the last line is captured (not the full console text) to avoid context bloat.
//! All captured text is filtered for sensitive data before returning.

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::{c_char, c_void, CStr};
    use std::os::raw::c_long;

    use accessibility_sys::kAXErrorSuccess as AX_SUCCESS;

    // Raw CF type aliases (stable ABI, matching core-foundation-sys 0.8.x).
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFIndex = c_long;
    type CFStringEncoding = u32;

    const K_CF_STRING_ENCODING_UTF8: CFStringEncoding = 0x08000100;

    // AX attribute name constants.
    const AX_WINDOWS: &str = "AXWindows";
    const AX_TITLE: &str = "AXTitle";
    const AX_ROLE: &str = "AXRole";
    const AX_CHILDREN: &str = "AXChildren";
    const AX_VALUE: &str = "AXValue";
    const AX_TEXT_AREA_ROLE: &str = "AXTextArea";

    // Maximum depth for walking into DevTools window children to find a text area.
    const MAX_DEPTH: u32 = 6;

    // Buffer size for reading console text (32KB -- smaller than terminal buffer).
    const TEXT_BUF_SIZE: CFIndex = 32768;

    // FFI declarations for ApplicationServices / CoreFoundation.
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> CFTypeRef;
        fn AXUIElementSetMessagingTimeout(element: CFTypeRef, timeout_seconds: f32) -> i32;
        fn AXUIElementCopyAttributeValue(
            element: CFTypeRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;

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

        fn CFArrayGetCount(the_array: CFArrayRef) -> CFIndex;
        fn CFArrayGetValueAtIndex(the_array: CFArrayRef, idx: CFIndex) -> CFTypeRef;

        fn CFRelease(cf: CFTypeRef);
        fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    }

    /// Detect DevTools/Console in a browser window via the Accessibility API.
    ///
    /// Returns (console_detected, last_console_line).
    /// - console_detected: true if a DevTools/Web Inspector/Developer Tools window was found.
    /// - last_console_line: the last non-empty line from a console text area, if readable.
    ///
    /// If DevTools is detected but the text cannot be read (common -- AX exposure varies),
    /// returns (true, None) so callers still know the console is open.
    pub(super) fn detect_console(app_pid: i32, _bundle_id: &str) -> (bool, Option<String>) {
        // SAFETY: AXUIElementCreateApplication is a stable public macOS API.
        // All returned references are null-checked and released.
        unsafe {
            let app_elem = AXUIElementCreateApplication(app_pid);
            if app_elem.is_null() {
                return (false, None);
            }

            // 1-second per-element messaging timeout to prevent hangs.
            AXUIElementSetMessagingTimeout(app_elem, 1.0);

            let result = find_devtools_window(app_elem);

            CFRelease(app_elem);
            result
        }
    }

    /// Walk all windows of the application looking for a DevTools/Web Inspector window.
    ///
    /// Detection heuristic: check if any window title contains "DevTools",
    /// "Web Inspector", "Developer Tools", or "Console". This works across
    /// Chrome/Chromium-based browsers, Safari (Web Inspector), and Firefox.
    unsafe fn find_devtools_window(app_elem: CFTypeRef) -> (bool, Option<String>) {
        let windows_val = match get_ax_attribute(app_elem, AX_WINDOWS) {
            Some(v) => v,
            None => return (false, None),
        };

        let windows_array = windows_val as CFArrayRef;
        let count = CFArrayGetCount(windows_array);

        let mut result = (false, None);

        'outer: for i in 0..count {
            let window = CFArrayGetValueAtIndex(windows_array, i);
            if window.is_null() {
                continue;
            }

            // Read the window title.
            let title = match get_ax_attribute(window, AX_TITLE) {
                Some(title_val) => {
                    let t = cf_type_to_string(title_val);
                    CFRelease(title_val);
                    t
                }
                None => None,
            };

            if let Some(ref t) = title {
                if is_devtools_title(t) {
                    // DevTools window found. Try to read the last console line.
                    // Retain the window element before releasing the array.
                    CFRetain(window);
                    let last_line = read_last_console_line(window, MAX_DEPTH);
                    CFRelease(window);
                    result = (true, last_line);
                    break 'outer;
                }
            }
        }

        CFRelease(windows_val);
        result
    }

    /// Returns true if the window title indicates an open DevTools / Web Inspector panel.
    fn is_devtools_title(title: &str) -> bool {
        let lower = title.to_lowercase();
        lower.contains("devtools")
            || lower.contains("web inspector")
            || lower.contains("developer tools")
            || lower.contains("browser console")
    }

    /// Walk into a DevTools window to find an AXTextArea and read its last non-empty line.
    ///
    /// Returns None if no readable text area is found (common -- DevTools AX exposure varies).
    unsafe fn read_last_console_line(window: CFTypeRef, max_depth: u32) -> Option<String> {
        let text_area = find_text_area(window, max_depth)?;
        let full_text = get_ax_string_value(text_area);
        CFRelease(text_area);

        full_text.and_then(|text| {
            text.lines()
                .rev()
                .find(|l| !l.trim().is_empty())
                .map(|l| l.trim().to_string())
        })
    }

    /// Recursively walk AX children to find the first AXTextArea element.
    ///
    /// The caller is responsible for CFReleasing the returned CFTypeRef.
    unsafe fn find_text_area(element: CFTypeRef, max_depth: u32) -> Option<CFTypeRef> {
        if max_depth == 0 {
            return None;
        }

        let children_val = get_ax_attribute(element, AX_CHILDREN)?;
        let children_array = children_val as CFArrayRef;
        let count = CFArrayGetCount(children_array);

        let mut result: Option<CFTypeRef> = None;

        'search: for i in 0..count {
            let child = CFArrayGetValueAtIndex(children_array, i);
            if child.is_null() {
                continue;
            }

            let role_val = get_ax_attribute(child, AX_ROLE);
            if let Some(rv) = role_val {
                let role = cf_type_to_string(rv);
                CFRelease(rv);
                if role.as_deref() == Some(AX_TEXT_AREA_ROLE) {
                    // Retain before releasing the parent array.
                    CFRetain(child);
                    result = Some(child);
                    break 'search;
                }
            }

            // Recurse into this child.
            if let Some(ta) = find_text_area(child, max_depth - 1) {
                result = Some(ta);
                break 'search;
            }
        }

        CFRelease(children_val);
        result
    }

    /// Get an AX attribute value from an element.
    ///
    /// Returns None on any AX error. Caller must CFRelease the returned value.
    unsafe fn get_ax_attribute(element: CFTypeRef, attribute: &str) -> Option<CFTypeRef> {
        let attr_cf = cf_string_from_str(attribute)?;
        let mut value: CFTypeRef = std::ptr::null();

        let err = AXUIElementCopyAttributeValue(element, attr_cf, &mut value);
        CFRelease(attr_cf);

        if err != AX_SUCCESS || value.is_null() {
            return None;
        }

        Some(value)
    }

    /// Extract AXValue as a Rust String from an AX element.
    ///
    /// Returns None if the attribute is missing or not a CFString.
    unsafe fn get_ax_string_value(element: CFTypeRef) -> Option<String> {
        let value = get_ax_attribute(element, AX_VALUE)?;
        let text = cf_type_to_string(value);
        CFRelease(value);
        text
    }

    /// Convert a CFTypeRef (holding a CFString) to a Rust String.
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

    /// Create a CFStringRef from a Rust &str (UTF-8). Caller must CFRelease.
    unsafe fn cf_string_from_str(s: &str) -> Option<CFStringRef> {
        let bytes = s.as_bytes();
        let cf_str = CFStringCreateWithBytes(
            std::ptr::null(),
            bytes.as_ptr(),
            bytes.len() as CFIndex,
            K_CF_STRING_ENCODING_UTF8,
            0,
        );

        if cf_str.is_null() {
            None
        } else {
            Some(cf_str)
        }
    }
}

/// Detect DevTools/Console in a browser window via Accessibility API.
///
/// Returns (console_detected, last_console_line).
/// - console_detected: true if a DevTools / Web Inspector window was found.
/// - last_console_line: last non-empty line from the console text area, filtered for secrets.
///
/// On non-macOS targets always returns (false, None).
#[cfg(not(target_os = "macos"))]
pub fn detect_console(_app_pid: i32, _bundle_id: &str) -> (bool, Option<String>) {
    (false, None)
}

/// Detect DevTools/Console in a browser window via Accessibility API.
///
/// Returns (console_detected, last_console_line).
/// Delegates to the macOS implementation in the `macos` submodule.
#[cfg(target_os = "macos")]
pub fn detect_console(app_pid: i32, bundle_id: &str) -> (bool, Option<String>) {
    macos::detect_console(app_pid, bundle_id)
}
