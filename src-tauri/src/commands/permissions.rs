/// Opens macOS System Settings to the Accessibility pane.
/// Fire-and-forget -- no return value needed.
#[tauri::command]
pub fn open_accessibility_settings() {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        .spawn()
        .ok();
}

/// Returns whether the app currently has macOS Accessibility permission.
/// Uses AXIsProcessTrusted() from the ApplicationServices framework (stable public C API).
#[tauri::command]
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    // SAFETY: AXIsProcessTrusted is a well-known, stable macOS public C function.
    // It takes no arguments and returns a bool; there are no memory safety concerns.
    unsafe { AXIsProcessTrusted() }
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    false
}
