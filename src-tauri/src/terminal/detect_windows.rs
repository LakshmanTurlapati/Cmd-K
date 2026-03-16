//! Windows terminal and shell detection.
//!
//! Provides HWND → exe name resolution, known terminal/IDE classification,
//! and process tree walking for Windows platforms.

// Constants and helpers are compiled on all platforms but only used on Windows.
// The #[allow(dead_code)] is applied at the module declaration in mod.rs.

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

/// Known standalone terminal emulator executables on Windows.
pub const KNOWN_TERMINAL_EXES: &[&str] = &[
    "WindowsTerminal.exe",
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "bash.exe",
    "alacritty.exe",
    "wezterm-gui.exe",
    "kitty.exe",
    "mintty.exe",
    "hyper.exe",
    "conhost.exe",
];

/// Known IDE executables with integrated terminals.
pub const KNOWN_IDE_EXES: &[&str] = &[
    "Code.exe",
    "Code - Insiders.exe",
    "Cursor.exe",
];

/// Known shell binary names (used for process tree walking).
/// wsl.exe is included because ConPTY priority filtering in find_shell_by_ancestry
/// naturally excludes internal VS Code wsl.exe processes (which are NOT ConPTY-hosted)
/// while finding the terminal-originated wsl.exe (which IS ConPTY-hosted).
pub const KNOWN_SHELL_EXES: &[&str] = &[
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "bash.exe",
    "zsh.exe",
    "fish.exe",
    "nu.exe",
    "sh.exe",
    "wsl.exe",
];

/// Returns true if the given exe name is a known terminal emulator.
pub fn is_known_terminal_exe(name: &str) -> bool {
    KNOWN_TERMINAL_EXES.iter().any(|&t| t.eq_ignore_ascii_case(name))
}

/// Returns true if the given exe name is a known IDE with an integrated terminal.
pub fn is_ide_with_terminal_exe(name: &str) -> bool {
    KNOWN_IDE_EXES.iter().any(|&t| t.eq_ignore_ascii_case(name))
}

/// Returns true if the given exe name is a known shell.
pub fn is_known_shell_exe(name: &str) -> bool {
    KNOWN_SHELL_EXES.iter().any(|&t| t.eq_ignore_ascii_case(name))
}

/// Get the executable name of the process that owns a given HWND.
///
/// Uses GetWindowThreadProcessId → OpenProcess → QueryFullProcessImageNameW.
/// Returns just the filename (e.g., "WindowsTerminal.exe").
#[cfg(target_os = "windows")]
pub fn get_exe_name(hwnd: isize) -> Option<String> {
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd as windows_sys::Win32::Foundation::HWND, &mut pid);
        if pid == 0 {
            return None;
        }
        get_exe_name_for_pid(pid)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_exe_name(_hwnd: isize) -> Option<String> {
    None
}

/// Get the executable name for a given PID.
#[cfg(target_os = "windows")]
pub fn get_exe_name_for_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 260];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut size);
        CloseHandle(handle);
        if ok == 0 || size == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_exe_name_for_pid(_pid: u32) -> Option<String> {
    None
}

/// Get the PID of the process that owns a given HWND.
#[cfg(target_os = "windows")]
pub fn get_pid_from_hwnd(hwnd: isize) -> Option<u32> {
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd as windows_sys::Win32::Foundation::HWND, &mut pid);
        if pid == 0 { None } else { Some(pid) }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_pid_from_hwnd(_hwnd: isize) -> Option<u32> {
    None
}

/// Derive a short shell type name from an exe name.
/// e.g., "powershell.exe" → "powershell", "bash.exe" → "bash"
pub fn exe_to_shell_type(exe: &str) -> String {
    exe.trim_end_matches(".exe")
        .trim_end_matches(".EXE")
        .to_lowercase()
}

/// Clean a Windows exe name into a display-friendly app name.
/// e.g., "WindowsTerminal.exe" → "Windows Terminal"
pub fn clean_exe_name(exe: &str) -> String {
    match exe.to_lowercase().as_str() {
        "windowsterminal.exe" => "Windows Terminal".to_string(),
        "powershell.exe" | "pwsh.exe" => "PowerShell".to_string(),
        "cmd.exe" => "CMD".to_string(),
        "bash.exe" => "Git Bash".to_string(),
        "code.exe" => "Code".to_string(),
        "code - insiders.exe" => "Code Insiders".to_string(),
        "cursor.exe" => "Cursor".to_string(),
        "alacritty.exe" => "Alacritty".to_string(),
        "wezterm-gui.exe" => "WezTerm".to_string(),
        "kitty.exe" => "kitty".to_string(),
        "mintty.exe" => "mintty".to_string(),
        "hyper.exe" => "Hyper".to_string(),
        _ => exe.trim_end_matches(".exe").to_string(),
    }
}

/// Read the window title text via Win32 GetWindowTextW.
/// Fast (< 1ms), no UIA overhead.
#[cfg(target_os = "windows")]
pub fn get_window_title(hwnd: isize) -> Option<String> {
    unsafe {
        let len = GetWindowTextLengthW(hwnd as _);
        if len <= 0 {
            return None;
        }
        let mut buf = vec![0u16; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd as _, buf.as_mut_ptr(), buf.len() as i32);
        if copied <= 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..copied as usize]))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_window_title(_hwnd: isize) -> Option<String> {
    None
}

/// Detect WSL from VS Code/Cursor window title.
/// Remote-WSL mode shows "[WSL: Ubuntu]" or "[WSL: Debian]" etc. in the title bar.
/// Example title: "file.rs - project [WSL: Ubuntu] - Visual Studio Code"
pub fn detect_wsl_from_title(title: &str) -> bool {
    title.contains("[WSL:")
}

/// Extract WSL distro name from window title if present.
/// "[WSL: Ubuntu]" -> Some("Ubuntu")
pub fn extract_wsl_distro_from_title(title: &str) -> Option<String> {
    if let Some(start) = title.find("[WSL: ") {
        let after = &title[start + 6..];
        if let Some(end) = after.find(']') {
            let distro = after[..end].trim();
            if !distro.is_empty() {
                return Some(distro.to_string());
            }
        }
    }
    None
}

/// Detect shell type from UIA text content of the focused terminal tab.
///
/// UIA text for a focused tab starts with the tab title (e.g., "Windows PowerShell",
/// "Command Prompt"). This function scans the first few lines for shell type indicators.
///
/// Returns a static shell type string matching exe_to_shell_type output (e.g., "cmd",
/// "powershell", "pwsh", "bash") or None if no match found.
///
/// Does NOT match user@host patterns (WSL detection is handled separately).
pub fn detect_shell_type_from_uia_text(text: &str) -> Option<&'static str> {
    // Only check first 5 lines -- tab title appears at the top of UIA text
    for line in text.lines().take(5) {
        let lower = line.to_lowercase();
        let trimmed = lower.trim();

        if trimmed.contains("command prompt") {
            return Some("cmd");
        }
        if trimmed.contains("windows powershell") {
            return Some("powershell");
        }
        if trimmed.contains("powershell 7") || trimmed.starts_with("pwsh") {
            return Some("pwsh");
        }
        if trimmed.starts_with("mingw") || trimmed.contains("git bash") {
            return Some("bash");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_type_powershell() {
        let text = "Windows PowerShell\nWindows PowerShell\nClose Tab\nNew Tab\n";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("powershell"));
    }

    #[test]
    fn test_detect_shell_type_cmd() {
        let text = "Command Prompt\nCommand Prompt\nClose Tab\nNew Tab\n";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("cmd"));
    }

    #[test]
    fn test_detect_shell_type_admin_powershell() {
        let text = "Administrator: Windows PowerShell";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("powershell"));
    }

    #[test]
    fn test_detect_shell_type_wsl_not_matched() {
        // WSL prompt patterns should NOT be matched -- handled separately
        let text = "parzival@host:/mnt/c$";
        assert_eq!(detect_shell_type_from_uia_text(text), None);
    }

    #[test]
    fn test_detect_shell_type_random_text_no_match() {
        let text = "some random app text";
        assert_eq!(detect_shell_type_from_uia_text(text), None);
    }

    #[test]
    fn test_detect_shell_type_pwsh() {
        let text = "PowerShell 7\npwsh";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("pwsh"));
    }

    #[test]
    fn test_detect_shell_type_git_bash() {
        let text = "Git Bash\nMINGW64";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("bash"));
    }

    #[test]
    fn test_detect_shell_type_mingw() {
        let text = "MINGW64:/c/Users/test$";
        assert_eq!(detect_shell_type_from_uia_text(text), Some("bash"));
    }

    #[test]
    fn test_detect_shell_type_empty() {
        assert_eq!(detect_shell_type_from_uia_text(""), None);
    }
}
