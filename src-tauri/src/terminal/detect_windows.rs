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
use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

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
    "wsl.exe",
];

/// Known IDE executables with integrated terminals.
pub const KNOWN_IDE_EXES: &[&str] = &[
    "Code.exe",
    "Code - Insiders.exe",
    "Cursor.exe",
];

/// Known shell binary names (used for process tree walking).
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
        "wsl.exe" => "WSL".to_string(),
        _ => exe.trim_end_matches(".exe").to_string(),
    }
}
