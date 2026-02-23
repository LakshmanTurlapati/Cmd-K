/// Known terminal emulator bundle identifiers on macOS.
/// This list covers all major GPU-accelerated and native terminal apps per the research doc.
pub const TERMINAL_BUNDLE_IDS: &[&str] = &[
    "com.apple.Terminal",
    "com.googlecode.iterm2",
    "io.alacritty",
    "net.kovidgoyal.kitty",
    "com.github.wez.wezterm",
];

/// Known browser bundle identifiers mapped to their canonical display names.
/// Chromium-based browsers (Arc, Edge, Brave) show their own name, not "Chrome".
pub const BROWSER_BUNDLE_IDS: &[(&str, &str)] = &[
    ("com.google.Chrome", "Chrome"),
    ("com.apple.Safari", "Safari"),
    ("org.mozilla.firefox", "Firefox"),
    ("company.thebrowser.Browser", "Arc"),
    ("com.microsoft.edgemac", "Edge"),
    ("com.brave.Browser", "Brave"),
];

/// Returns true if the given bundle ID corresponds to a known browser.
pub fn is_known_browser(bundle_id: &str) -> bool {
    BROWSER_BUNDLE_IDS.iter().any(|(id, _)| *id == bundle_id)
}

/// Static mapping of verbose macOS app names to shortened display names.
const APP_NAME_MAP: &[(&str, &str)] = &[
    ("Google Chrome", "Chrome"),
    ("Visual Studio Code", "Code"),
    ("Visual Studio Code - Insiders", "Code"),
    ("Mozilla Firefox", "Firefox"),
    ("Microsoft Edge", "Edge"),
    ("Brave Browser", "Brave"),
    // Arc, Safari, Finder already have clean names.
];

/// Clean and shorten a raw localized app name for badge display.
///
/// Applies the static APP_NAME_MAP first. If not matched, strips common
/// suffixes like " - Insiders" or " Beta" before returning the name.
pub fn clean_app_name(raw_name: &str) -> String {
    // Check explicit mapping first.
    for (long, short) in APP_NAME_MAP {
        if *long == raw_name {
            return (*short).to_string();
        }
    }

    // Strip common verbose suffixes not covered by the map.
    let suffixes = [" - Insiders", " Beta", " Nightly", " Dev", " Canary"];
    let mut cleaned = raw_name.to_string();
    for suffix in &suffixes {
        if let Some(stripped) = cleaned.strip_suffix(suffix) {
            cleaned = stripped.to_string();
            break;
        }
    }
    cleaned
}

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
            eprintln!("[detect] NSRunningApplication class not found");
            return None;
        }

        let sel = sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr());
        // Use typed function pointer for the call with pid argument (ARM64 fix)
        let msg_send_i32: MsgSendI32 = std::mem::transmute(objc_msgSend as *mut c_void);
        let app = msg_send_i32(cls, sel, pid);
        if app.is_null() {
            eprintln!("[detect] runningApplicationWithProcessIdentifier:{} returned nil", pid);
            return None;
        }

        // [app bundleIdentifier] -- no extra args, variadic call is fine
        let bundle_sel = sel_registerName(b"bundleIdentifier\0".as_ptr());
        let ns_string = objc_msgSend(app, bundle_sel);
        if ns_string.is_null() {
            eprintln!("[detect] bundleIdentifier returned nil for pid {}", pid);
            return None;
        }

        // Convert NSString to Rust &str via UTF8String (returns a C string pointer)
        let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr());
        let c_str = objc_msgSend(ns_string, utf8_sel) as *const i8;
        if c_str.is_null() {
            eprintln!("[detect] UTF8String returned nil for pid {}", pid);
            return None;
        }

        let bundle_id = std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned();
        eprintln!("[detect] pid {} -> bundle_id: {}", pid, bundle_id);
        Some(bundle_id)
    }
}

/// Non-macOS stub -- always returns None.
#[cfg(not(target_os = "macos"))]
pub fn get_bundle_id(_pid: i32) -> Option<String> {
    None
}

/// Get the localized (user-visible) display name of a running application by PID.
///
/// Uses NSRunningApplication.localizedName via ObjC FFI, following the same
/// ARM64-safe pattern as get_bundle_id. Returns the raw localized name before
/// any cleaning -- callers should pass the result through clean_app_name().
///
/// Returns None if the PID is not found or the app has no localized name.
#[cfg(target_os = "macos")]
pub fn get_app_display_name(pid: i32) -> Option<String> {
    use std::ffi::c_void;
    extern "C" {
        fn objc_getClass(name: *const u8) -> *mut c_void;
        fn sel_registerName(name: *const u8) -> *mut c_void;
        fn objc_msgSend(receiver: *mut c_void, sel: *mut c_void, ...) -> *mut c_void;
    }

    // Typed function pointer for objc_msgSend calls that pass an i32 argument (ARM64 fix).
    type MsgSendI32 = unsafe extern "C" fn(*mut c_void, *mut c_void, i32) -> *mut c_void;

    // SAFETY: Stable ObjC runtime functions. All returns are null-checked before use.
    // Transmute converts variadic objc_msgSend to typed function pointer for ARM64.
    unsafe {
        // [NSRunningApplication runningApplicationWithProcessIdentifier:pid]
        let cls = objc_getClass(b"NSRunningApplication\0".as_ptr());
        if cls.is_null() {
            return None;
        }

        let sel = sel_registerName(b"runningApplicationWithProcessIdentifier:\0".as_ptr());
        let msg_send_i32: MsgSendI32 = std::mem::transmute(objc_msgSend as *mut c_void);
        let app = msg_send_i32(cls, sel, pid);
        if app.is_null() {
            return None;
        }

        // [app localizedName] -- no extra args, variadic call is fine
        let name_sel = sel_registerName(b"localizedName\0".as_ptr());
        let ns_string = objc_msgSend(app, name_sel);
        if ns_string.is_null() {
            return None;
        }

        // Convert NSString -> Rust String via UTF8String
        let utf8_sel = sel_registerName(b"UTF8String\0".as_ptr());
        let c_str = objc_msgSend(ns_string, utf8_sel) as *const i8;
        if c_str.is_null() {
            return None;
        }

        let name = std::ffi::CStr::from_ptr(c_str).to_string_lossy().into_owned();
        eprintln!("[detect] pid {} -> localized_name: {}", pid, name);
        Some(name)
    }
}

/// Non-macOS stub -- always returns None.
#[cfg(not(target_os = "macos"))]
pub fn get_app_display_name(_pid: i32) -> Option<String> {
    None
}
