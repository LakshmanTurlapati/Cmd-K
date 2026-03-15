//! Linux terminal text reader with three strategies:
//! - AT-SPI2 via zbus (blocking D-Bus) for VTE-based terminals (GNOME Terminal, Tilix, etc.) and Qt terminals (Konsole)
//! - kitty remote control subprocess (`kitty @ get-text --extent screen`)
//! - WezTerm CLI subprocess (`wezterm cli get-text`)
//!
//! Unsupported terminals (Alacritty, st, foot, xterm, urxvt) return None gracefully.
//! All strategies use a 500ms timeout to keep hotkey response snappy.

#[cfg(target_os = "linux")]
mod linux {
    use std::process::Command;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    /// Read visible terminal text for the given terminal process.
    ///
    /// Dispatches to the appropriate strategy based on the terminal executable name.
    /// Returns None for unsupported terminals or on any failure.
    pub fn read_terminal_text_linux(pid: i32, exe_name: &str) -> Option<String> {
        match exe_name {
            // VTE-based terminals (GTK) and Qt terminals (AT-SPI2 via qt-atspi bridge)
            "gnome-terminal-server" | "tilix" | "terminator" | "mate-terminal"
            | "xfce4-terminal" | "guake" | "tilda" | "sakura" | "lxterminal"
            | "terminology" | "konsole" => read_via_atspi(pid),
            // Direct API terminals
            "kitty" => read_via_kitty(),
            "wezterm-gui" => read_via_wezterm(),
            // Everything else: no text reading support (alacritty, st, foot, xterm, urxvt, etc.)
            _ => None,
        }
    }

    /// Read terminal text via AT-SPI2 D-Bus using zbus blocking connection.
    ///
    /// Connects to the accessibility bus, finds the terminal application by PID,
    /// walks the accessible tree to find a widget with role Terminal (60),
    /// then reads visible text via the org.a11y.atspi.Text interface.
    ///
    /// Wrapped in a thread with 500ms timeout to prevent blocking the hotkey.
    fn read_via_atspi(pid: i32) -> Option<String> {
        eprintln!("[linux_reader] attempting AT-SPI2 for pid={}", pid);

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = atspi_inner(pid);
            let _ = tx.send(result);
        });

        rx.recv_timeout(Duration::from_millis(500)).ok()?
    }

    /// Inner AT-SPI2 logic using zbus blocking D-Bus calls.
    ///
    /// Flow:
    /// 1. Connect to session bus
    /// 2. Get AT-SPI2 bus address via org.a11y.Bus.GetAddress
    /// 3. Connect to the accessibility bus
    /// 4. Get desktop children (applications) from registry
    /// 5. Find application matching target PID
    /// 6. Walk children looking for role=Terminal (60)
    /// 7. Read text via org.a11y.atspi.Text.GetText
    fn atspi_inner(pid: i32) -> Option<String> {
        use zbus::blocking::Connection;
        use zbus::zvariant::OwnedValue;

        // Step 1: Connect to session bus
        let session = Connection::session().ok()?;

        // Step 2: Get AT-SPI2 bus address
        let reply = session
            .call_method(
                Some("org.a11y.Bus"),
                "/org/a11y/bus",
                Some("org.a11y.Bus"),
                "GetAddress",
                &(),
            )
            .ok()?;
        let addr: String = reply.body().deserialize().ok()?;

        if addr.is_empty() {
            return None;
        }

        // Step 3: Connect to accessibility bus
        let a11y_addr: zbus::Address = addr.as_str().try_into().ok()?;
        let a11y_conn = zbus::blocking::connection::Builder::address(a11y_addr)
            .ok()?
            .build()
            .ok()?;

        // Step 4: Get desktop children from registry
        // Desktop: bus=org.a11y.atspi.Registry, path=/org/a11y/atspi/accessible/root
        let children_reply = a11y_conn
            .call_method(
                Some("org.a11y.atspi.Registry"),
                "/org/a11y/atspi/accessible/root",
                Some("org.a11y.atspi.Accessible"),
                "GetChildren",
                &(),
            )
            .ok()?;

        // Children are Vec<(bus_name, object_path)>
        let children: Vec<(String, zbus::zvariant::OwnedObjectPath)> =
            children_reply.body().deserialize().ok()?;

        // Step 5: Find application matching target PID
        for (ref bus_name, ref obj_path) in children {
            // Get application PID via Properties.Get
            let bus_str: &str = bus_name.as_str();
            let path_str: &str = obj_path;
            let pid_reply = a11y_conn
                .call_method(
                    Some(bus_str),
                    path_str,
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.a11y.atspi.Application", "Id"),
                )
                .ok();

            if let Some(reply) = pid_reply {
                let variant: OwnedValue = reply.body().deserialize().ok()?;
                // Try to extract PID as i32 from the variant
                let app_pid: i32 = i32::try_from(variant).ok()?;

                if app_pid != pid {
                    continue;
                }

                // Found matching app -- walk its children for Terminal role
                if let Some(text) =
                    find_terminal_text(&a11y_conn, bus_str, path_str)
                {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        return Some(trimmed);
                    }
                }
            }
        }

        None
    }

    /// Recursively walk accessible children looking for role=Terminal (60),
    /// then read text from the org.a11y.atspi.Text interface.
    fn find_terminal_text(
        conn: &zbus::blocking::Connection,
        bus_name: &str,
        path: &str,
    ) -> Option<String> {
        use zbus::zvariant::OwnedValue;

        const ATSPI_ROLE_TERMINAL: u32 = 60;

        // Get role of this accessible
        let role_reply = conn
            .call_method(
                Some(bus_name),
                path,
                Some("org.a11y.atspi.Accessible"),
                "GetRole",
                &(),
            )
            .ok()?;
        let role: u32 = role_reply.body().deserialize().ok()?;

        if role == ATSPI_ROLE_TERMINAL {
            // Read CharacterCount property
            let count_reply = conn
                .call_method(
                    Some(bus_name),
                    path,
                    Some("org.freedesktop.DBus.Properties"),
                    "Get",
                    &("org.a11y.atspi.Text", "CharacterCount"),
                )
                .ok()?;
            let count_variant: OwnedValue = count_reply.body().deserialize().ok()?;
            let count: i32 = i32::try_from(count_variant).ok()?;

            if count <= 0 {
                return None;
            }

            // Read text via GetText(0, count)
            let text_reply = conn
                .call_method(
                    Some(bus_name),
                    path,
                    Some("org.a11y.atspi.Text"),
                    "GetText",
                    &(0i32, count),
                )
                .ok()?;
            let text: String = text_reply.body().deserialize().ok()?;
            return Some(text);
        }

        // Not a terminal -- recurse into children
        let children_reply = conn
            .call_method(
                Some(bus_name),
                path,
                Some("org.a11y.atspi.Accessible"),
                "GetChildren",
                &(),
            )
            .ok();

        if let Some(reply) = children_reply {
            let children: Vec<(String, zbus::zvariant::OwnedObjectPath)> =
                reply.body().deserialize().unwrap_or_default();
            for (ref child_bus, ref child_path) in children {
                let child_path_str: &str = child_path;
                if let Some(text) = find_terminal_text(conn, child_bus.as_str(), child_path_str) {
                    return Some(text);
                }
            }
        }

        None
    }

    /// Read terminal text via kitty remote control.
    ///
    /// Requires `allow_remote_control` to be enabled in kitty.conf.
    /// Returns None silently if remote control is disabled.
    fn read_via_kitty() -> Option<String> {
        eprintln!("[linux_reader] attempting kitty remote control");

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = Command::new("kitty")
                .args(["@", "get-text", "--extent", "screen"])
                .output();
            let _ = tx.send(result);
        });

        let output = rx.recv_timeout(Duration::from_millis(500)).ok()?.ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    /// Read terminal text via WezTerm CLI.
    fn read_via_wezterm() -> Option<String> {
        eprintln!("[linux_reader] attempting wezterm cli");

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let result = Command::new("wezterm")
                .args(["cli", "get-text"])
                .output();
            let _ = tx.send(result);
        });

        let output = rx.recv_timeout(Duration::from_millis(500)).ok()?.ok()?;

        if !output.status.success() {
            return None;
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux::read_terminal_text_linux;

/// Non-Linux stub: always returns None.
#[cfg(not(target_os = "linux"))]
pub fn read_terminal_text_linux(_pid: i32, _exe_name: &str) -> Option<String> {
    None
}
