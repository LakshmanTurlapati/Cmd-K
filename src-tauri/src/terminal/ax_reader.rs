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

    use std::ffi::{c_char, c_void, CStr};
    use std::os::raw::c_long;

    // Raw CF type aliases. These match the definitions in core-foundation-sys 0.8.x.
    // We declare them here to avoid taking a direct dependency on core-foundation-sys.
    type CFTypeRef = *const c_void;
    type CFStringRef = *const c_void;
    type CFArrayRef = *const c_void;
    type CFIndex = c_long;
    type CFStringEncoding = u32;

    const K_CF_STRING_ENCODING_UTF8: CFStringEncoding = 0x08000100;

    // AX attribute name constants -- these match accessibility_sys string constants exactly.
    const AX_FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
    const AX_FOCUSED_WINDOW: &str = "AXFocusedWindow";
    const AX_CHILDREN: &str = "AXChildren";
    const AX_ROLE: &str = "AXRole";
    const AX_VALUE: &str = "AXValue";
    const AX_SCROLL_AREA_ROLE: &str = "AXScrollArea";
    const AX_TEXT_AREA_ROLE: &str = "AXTextArea";

    // K_AX_ERROR_SUCCESS is imported from accessibility_sys as AX_SUCCESS above.

    // Maximum recursion depth when walking the AX children tree.
    const MAX_DEPTH: u32 = 5;

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
