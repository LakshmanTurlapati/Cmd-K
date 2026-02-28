/// Opens macOS System Settings to the Accessibility pane.
/// Fire-and-forget -- no return value needed.
#[tauri::command]
pub fn open_accessibility_settings() {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .ok();
}

/// Probes actual AX API access on an EXTERNAL process (Dock).
/// Probing our own PID can produce false positives -- an app can read its
/// own AX attributes without full Accessibility permission on some macOS
/// versions/signing contexts.  By probing Dock (always running, stable),
/// we force a genuine cross-process AX call that REQUIRES Accessibility
/// permission to succeed.
///
/// Returns false ONLY when error is kAXErrorNotTrusted (-25211).
/// All other codes (success, attribute-absent, etc.) mean permission is granted.
#[cfg(target_os = "macos")]
fn ax_probe_external() -> bool {
    use std::ffi::{c_void, CString};

    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> *const c_void;
        fn AXUIElementCopyAttributeValue(
            element: *const c_void,
            attribute: *const c_void,
            value: *mut *const c_void,
        ) -> i32;
        fn CFRelease(cf: *const c_void);
    }

    // Find Dock's PID -- it is always running on macOS.
    let dock_pid = match std::process::Command::new("pgrep")
        .arg("-x")
        .arg("Dock")
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            match stdout.trim().parse::<i32>() {
                Ok(pid) => pid,
                Err(_) => {
                    eprintln!("[permissions] ax_probe_external: failed to parse Dock PID from {:?}", stdout.trim());
                    return false;
                }
            }
        }
        _ => {
            eprintln!("[permissions] ax_probe_external: pgrep -x Dock failed");
            return false;
        }
    };

    eprintln!("[permissions] ax_probe_external: probing Dock (pid={})", dock_pid);

    unsafe {
        let app_elem = AXUIElementCreateApplication(dock_pid);
        if app_elem.is_null() {
            return false;
        }

        use core_foundation_sys::base::kCFAllocatorDefault;
        use core_foundation_sys::string::{CFStringCreateWithCString, kCFStringEncodingUTF8};

        let attr_name = CString::new("AXRole").expect("CString ok");
        let attr_cf =
            CFStringCreateWithCString(kCFAllocatorDefault, attr_name.as_ptr(), kCFStringEncodingUTF8);
        if attr_cf.is_null() {
            CFRelease(app_elem);
            return false;
        }

        let mut value: *const c_void = std::ptr::null();
        let err = AXUIElementCopyAttributeValue(app_elem, attr_cf as *const c_void, &mut value);
        if !value.is_null() {
            CFRelease(value);
        }
        CFRelease(attr_cf as *const c_void);
        CFRelease(app_elem);

        // kAXErrorNotTrusted (-25211) is the definitive "no permission" code.
        // All other codes (0 = success, -25205 = attribute unsupported,
        // -25204 = no value, -25212 = cannot complete) indicate the OS
        // allowed the call -- permission IS granted.
        const AX_ERROR_NOT_TRUSTED: i32 = -25211;
        eprintln!("[permissions] ax_probe_external: AXUIElementCopyAttributeValue returned {}", err);
        err != AX_ERROR_NOT_TRUSTED
    }
}

/// Returns whether the app currently has macOS Accessibility permission.
/// Uses a dual-check: AXIsProcessTrusted() first (fast path), then falls back
/// to a live AX probe on our own PID. The probe handles unsigned production builds
/// where TCC identity mismatch causes AXIsProcessTrusted to return false even when
/// permission is actually granted.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    let trusted = unsafe { accessibility_sys::AXIsProcessTrusted() };
    if trusted {
        eprintln!("[permissions] AXIsProcessTrusted() = true (fast path)");
        return true;
    }
    let probe_result = ax_probe_external();
    eprintln!(
        "[permissions] AXIsProcessTrusted() = false, ax_probe_external() = {probe_result}"
    );
    probe_result
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    false
}

/// Requests accessibility permission using AXIsProcessTrustedWithOptions.
/// When `prompt` is true, macOS shows its system-level accessibility prompt
/// for apps not yet in the list. For already-listed apps (stale signatures),
/// it returns the current trust status without prompting.
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn request_accessibility_permission(prompt: bool) -> bool {
    use core_foundation_sys::base::{CFRelease, kCFAllocatorDefault};
    use core_foundation_sys::dictionary::{
        CFDictionaryCreate, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
    };
    use core_foundation_sys::number::{kCFBooleanFalse, kCFBooleanTrue};
    use core_foundation_sys::string::CFStringCreateWithCString;
    use std::ffi::CString;

    unsafe {
        extern "C" {
            fn AXIsProcessTrustedWithOptions(
                options: core_foundation_sys::dictionary::CFDictionaryRef,
            ) -> bool;
        }

        let key_str =
            CString::new("AXTrustedCheckOptionPrompt").expect("CString creation failed");
        let key = CFStringCreateWithCString(
            kCFAllocatorDefault,
            key_str.as_ptr(),
            core_foundation_sys::string::kCFStringEncodingUTF8,
        );

        if key.is_null() {
            eprintln!("[permissions] CFStringCreateWithCString returned null, falling back");
            return accessibility_sys::AXIsProcessTrusted();
        }

        let value = if prompt {
            kCFBooleanTrue
        } else {
            kCFBooleanFalse
        };

        let keys = [key as *const std::ffi::c_void];
        let values = [value as *const std::ffi::c_void];

        let options = CFDictionaryCreate(
            kCFAllocatorDefault,
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks,
            &kCFTypeDictionaryValueCallBacks,
        );

        if options.is_null() {
            eprintln!("[permissions] CFDictionaryCreate returned null, falling back");
            CFRelease(key as *const std::ffi::c_void);
            return accessibility_sys::AXIsProcessTrusted();
        }

        let trusted = AXIsProcessTrustedWithOptions(options);

        CFRelease(options as *const std::ffi::c_void);
        CFRelease(key as *const std::ffi::c_void);

        eprintln!(
            "[permissions] AXIsProcessTrustedWithOptions(prompt={prompt}) = {trusted}"
        );
        trusted
    }
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn request_accessibility_permission(_prompt: bool) -> bool {
    false
}
