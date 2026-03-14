//! Linux terminal and IDE detection.
//!
//! Provides terminal emulator / IDE classification and process name resolution
//! for Linux platforms. Mirrors detect_windows.rs structure.

// Constants and helpers are compiled on all platforms but only used on Linux.
// The #[allow(dead_code)] is applied at the module declaration in mod.rs.

/// Known standalone terminal emulator executables on Linux.
pub const KNOWN_TERMINAL_EXES_LINUX: &[&str] = &[
    "gnome-terminal-server", // GNOME Terminal (server process, not the client)
    "kitty",
    "alacritty",
    "konsole",
    "wezterm-gui",
    "xfce4-terminal",
    "tilix",
    "terminator",
    "xterm",
    "urxvt",
    "st",
    "foot",
    "sakura",
    "terminology",
    "lxterminal",
    "mate-terminal",
    "guake",
    "tilda",
];

/// Known IDE executables with integrated terminals on Linux.
pub const KNOWN_IDE_EXES_LINUX: &[&str] = &[
    "code",     // VS Code
    "cursor",   // Cursor
    "codium",   // VSCodium
    "idea",     // IntelliJ IDEA
    "pycharm",  // PyCharm
    "webstorm", // WebStorm
    "clion",    // CLion
    "goland",   // GoLand
    "rustrover", // RustRover
];

/// Returns true if the given exe name is a known terminal emulator on Linux.
/// Case-sensitive match (Linux is case-sensitive).
pub fn is_known_terminal_exe(name: &str) -> bool {
    KNOWN_TERMINAL_EXES_LINUX.iter().any(|&t| t == name)
}

/// Returns true if the given exe name is a known IDE with an integrated terminal on Linux.
/// Case-sensitive match (Linux is case-sensitive).
pub fn is_ide_with_terminal_exe(name: &str) -> bool {
    KNOWN_IDE_EXES_LINUX.iter().any(|&t| t == name)
}

/// Get the executable name for a given PID on Linux.
/// Delegates to process::get_process_name which reads /proc/PID/exe.
#[cfg(target_os = "linux")]
pub fn get_exe_name_for_pid(pid: i32) -> Option<String> {
    super::process::get_process_name(pid)
}

#[cfg(not(target_os = "linux"))]
pub fn get_exe_name_for_pid(_pid: i32) -> Option<String> {
    None
}

/// Clean a Linux exe name into a display-friendly app name.
///
/// Handles common terminal emulator naming conventions:
/// - "gnome-terminal-server" -> "GNOME Terminal"
/// - "xfce4-terminal" -> "XFCE Terminal"
/// - "wezterm-gui" -> "WezTerm"
/// - Unknown names are returned as-is.
pub fn clean_linux_app_name(exe_name: &str) -> String {
    match exe_name {
        "gnome-terminal-server" => "GNOME Terminal".to_string(),
        "kitty" => "kitty".to_string(),
        "alacritty" => "Alacritty".to_string(),
        "konsole" => "Konsole".to_string(),
        "wezterm-gui" => "WezTerm".to_string(),
        "xfce4-terminal" => "XFCE Terminal".to_string(),
        "tilix" => "Tilix".to_string(),
        "terminator" => "Terminator".to_string(),
        "xterm" => "XTerm".to_string(),
        "urxvt" => "URxvt".to_string(),
        "st" => "st".to_string(),
        "foot" => "foot".to_string(),
        "sakura" => "Sakura".to_string(),
        "terminology" => "Terminology".to_string(),
        "lxterminal" => "LXTerminal".to_string(),
        "mate-terminal" => "MATE Terminal".to_string(),
        "guake" => "Guake".to_string(),
        "tilda" => "Tilda".to_string(),
        "code" => "Code".to_string(),
        "cursor" => "Cursor".to_string(),
        "codium" => "VSCodium".to_string(),
        "idea" => "IntelliJ IDEA".to_string(),
        "pycharm" => "PyCharm".to_string(),
        "webstorm" => "WebStorm".to_string(),
        "clion" => "CLion".to_string(),
        "goland" => "GoLand".to_string(),
        "rustrover" => "RustRover".to_string(),
        _ => exe_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_terminals() {
        assert!(is_known_terminal_exe("kitty"));
        assert!(is_known_terminal_exe("alacritty"));
        assert!(is_known_terminal_exe("gnome-terminal-server"));
        assert!(!is_known_terminal_exe("code"));
        assert!(!is_known_terminal_exe("unknown"));
    }

    #[test]
    fn test_case_sensitive() {
        // Linux is case-sensitive -- "Kitty" should not match "kitty"
        assert!(!is_known_terminal_exe("Kitty"));
        assert!(!is_known_terminal_exe("ALACRITTY"));
    }

    #[test]
    fn test_known_ides() {
        assert!(is_ide_with_terminal_exe("code"));
        assert!(is_ide_with_terminal_exe("cursor"));
        assert!(is_ide_with_terminal_exe("idea"));
        assert!(!is_ide_with_terminal_exe("kitty"));
    }

    #[test]
    fn test_clean_app_name() {
        assert_eq!(clean_linux_app_name("gnome-terminal-server"), "GNOME Terminal");
        assert_eq!(clean_linux_app_name("wezterm-gui"), "WezTerm");
        assert_eq!(clean_linux_app_name("xfce4-terminal"), "XFCE Terminal");
        assert_eq!(clean_linux_app_name("code"), "Code");
        assert_eq!(clean_linux_app_name("unknown-app"), "unknown-app");
    }
}
