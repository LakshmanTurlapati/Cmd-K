/// Known terminal emulator bundle identifiers on macOS.
/// This list covers all major GPU-accelerated and native terminal apps per the research doc.
pub const TERMINAL_BUNDLE_IDS: &[&str] = &[
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "io.alacritty",
    "net.kovidgoyal.kitty",
    "com.github.wez.wezterm",
];

/// GPU-accelerated terminal bundle IDs (Alacritty, Kitty, WezTerm).
/// These do not expose Accessibility API text (Plan 02 handles them differently).
pub fn is_gpu_terminal(bundle_id: &str) -> bool {
    matches!(
        bundle_id,
        "io.alacritty" | "net.kovidgoyal.kitty" | "com.github.wez.wezterm"
    )
}

/// Returns true if the given bundle ID corresponds to a known terminal emulator.
pub fn is_known_terminal(bundle_id: &str) -> bool {
    TERMINAL_BUNDLE_IDS.contains(&bundle_id)
}

/// Get the bundle identifier of a running application by PID.
///
/// Uses NSRunningApplication via ObjC FFI -- the same pattern as AXIsProcessTrusted
/// in commands/permissions.rs. This is a stable macOS public API.
///
/// Returns None if the PID is not found or has no bundle ID
/// (command-line-only processes have no bundle ID).
#[cfg(target_os = "macos")]
pub fn get_bundle_id(pid: i32) -> Option<String> {
    use std::ffi::c_void;
    extern "C" {
        fn objc_getClass(name: *const u8) -> *mut c_void;
        fn sel_registerName(name: *const u8) -> *mut c_void;
        fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }
    // SAFETY: These are stable ObjC runtime functions. All return values are
    // null-checked before use. NSRunningApplication and NSString are standard AppKit classes.
    // pid is passed as pid_t (i32) matching the Obj-C method signature.
    unsafe {
        // [NSRunningApplication runningApplicationWithProcessIdentifier:pid]
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr());
        if cls.is_null() {
            return None;
        }

        let sel = sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr());
        let app = objc_msgSend(cls, sel, pid);
        if app.is_null() {
            return None;
        }

        // [app bundleIdentifier]
        let bundle_sel = sel_registerName(b"bundleIdentifier\0".as_ptr());
        let ns_string = objc_msgSend(app, bundle_sel);
        if ns_string.is_null() {
            return None;
        }

        // Convert NSString to Rust &str via UTF8String (returns a C string pointer)
        let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr());
        let c_str = objc_msgSend(ns_string, utf8_sel) as *const i8;
        if c_str.is_null() {
            return None;
        }

        Some(std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned())
    }
}

/// Non-macOS stub -- always returns None.
#[cfg(not(target_os = "macos"))]
pub fn get_bundle_id(_pid: i32) -> Option<String> {
    None
}
