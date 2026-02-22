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
///
/// NOTE on ARM64 calling convention: objc_msgSend is declared variadic but is
/// actually a trampoline that forwards to the method implementation using the
/// standard (non-variadic) calling convention. On ARM64, variadic arguments go
/// to the stack while non-variadic go to registers. For calls WITH extra arguments
/// (like pid), we must cast objc_msgSend to a properly typed non-variadic function
/// pointer so the argument lands in the correct register (x2) instead of the stack.
/// Calls WITHOUT extra arguments (just receiver + selector) are unaffected.
#[cfg(target_os = "macos")]
pub fn get_bundle_id(pid: i32) -> Option<String> {
    use std::ffi::c_void;
    extern "C" {
        fn objc_getClass(name: *const u8) -> *mut c_void;
        fn sel_registerName(name: *const u8) -> *mut c_void;
        fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }

    // Typed function pointer for objc_msgSend calls that pass an i32 argument.
    // This ensures the pid lands in register x2 (not on the stack) on ARM64.
    type MsgSendI32 = unsafe extern "C" fn(*mut c_void, *mut c_void, i32) -> *mut c_void;

    // SAFETY: These are stable ObjC runtime functions. All return values are
    // null-checked before use. NSRunningApplication and NSString are standard AppKit classes.
    // The transmute converts variadic objc_msgSend to a non-variadic function pointer
    // with the correct signature for ARM64 register-based argument passing.
    unsafe {
        // [NSRunningApplication runningApplicationWithProcessIdentifier:pid]
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr());
        if cls.is_null() {
            return None;
        }

        let sel = sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr());
        // Use typed function pointer for the call with pid argument (ARM64 fix)
        let msg_send_i32: MsgSendI32 = std::mem::transmute(objc_msgSend as *mut c_void);
        let app = msg_send_i32(cls, sel, pid);
        if app.is_null() {
            return None;
        }

        // [app bundleIdentifier] -- no extra args, variadic call is fine
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
