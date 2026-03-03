# Architecture Research: Windows Platform Support

**Domain:** Tauri v2 cross-platform overlay -- porting macOS CMD+K to Windows
**Researched:** 2026-03-01
**Confidence:** HIGH (core patterns), MEDIUM (specific Windows API edge cases)

---

## Context: This is a Platform Port Architecture Doc (v0.2.1)

This document analyzes how every Rust module and frontend component maps from macOS to Windows. It identifies what is already cross-platform, what needs a Windows-specific implementation, and what the abstraction strategy should be. The build order is dependency-driven.

---

## Module Classification: What Changes, What Stays

### Already Cross-Platform (Zero Changes)

These modules have no platform-specific code and work identically on Windows:

| Module | Why Cross-Platform |
|--------|--------------------|
| `commands/ai.rs` | Pure HTTP/SSE streaming via reqwest + eventsource-stream. No OS APIs. |
| `commands/xai.rs` | HTTP API validation. No OS APIs. |
| `commands/safety.rs` | Regex pattern matching (once_cell + regex). No OS APIs. |
| `commands/history.rs` | Pure Zustand/Rust HashMap operations. No OS APIs. |
| `state.rs` | Rust structs with Mutex fields. No OS APIs. |
| `terminal/filter.rs` | Regex-based sensitive data redaction. No OS APIs. |
| `commands/keychain.rs` | Uses `keyring` crate which already supports Windows Credential Manager (feature: `windows-native`). **Already cross-platform.** |
| All React components | React 19, Zustand, Tailwind CSS, Radix UI. Platform-agnostic web layer. |
| `src/store/index.ts` | Zustand store logic. All platform behavior enters via Tauri IPC, which is abstracted. |

**Key finding: `keychain.rs` requires NO code changes.** The `keyring` crate v3 with `windows-native` feature uses Windows Credential Manager on Windows and macOS Keychain on macOS transparently. The current Cargo.toml has `features = ["apple-native"]` -- this needs to become platform-conditional (see Cargo.toml section).

### Needs Windows Implementation (Platform-Specific Modules)

| Module | macOS API | Windows Equivalent | Complexity |
|--------|-----------|-------------------|------------|
| `commands/hotkey.rs` (`get_frontmost_pid`) | `NSWorkspace.frontmostApplication.processIdentifier` via ObjC FFI | `GetForegroundWindow()` + `GetWindowThreadProcessId()` via Win32 API | Low |
| `commands/paste.rs` | AppleScript dispatch + CGEventPost (Cmd+V, Ctrl+U) | `clipboard-win` + `SendInput` (Ctrl+V) via Win32 API | Medium |
| `commands/permissions.rs` | `AXIsProcessTrusted` + AX probe | No equivalent needed -- Windows has no global accessibility permission gate | Low (mostly removal) |
| `commands/window.rs` | `tauri_nspanel::ManagerExt` for NSPanel show/hide/positioning | Tauri native window with `always_on_top` + `WS_EX_NOACTIVATE` via raw HWND | High |
| `commands/tray.rs` | macOS menu bar tray with `icon_as_template` | Windows system tray (Tauri's tray API is cross-platform, minor adjustments) | Low |
| `terminal/ax_reader.rs` | macOS Accessibility API (`AXUIElement*` FFI) | Windows UI Automation API (`uiautomation` crate or direct COM) | High |
| `terminal/process.rs` | `libproc` FFI (`proc_pidinfo`, `proc_pidpath`, `proc_listchildpids`) | `NtQueryInformationProcess` + `sysinfo` crate for process tree | High |
| `terminal/detect.rs` | `NSRunningApplication` ObjC FFI for bundle ID + display name | `GetModuleFileNameExW` + `GetWindowTextW` via Win32 | Medium |
| `terminal/browser.rs` | macOS AX API for DevTools detection | Windows UI Automation for DevTools window detection | Medium |
| `lib.rs` (setup) | NSPanel creation, vibrancy, activation policy, panel level | HWND manipulation, Acrylic/Mica vibrancy, skip-taskbar | High |

---

## Platform Abstraction Strategy

### Recommendation: `cfg(target_os)` Conditional Compilation with Platform Submodules

Use the **same pattern already established in the codebase**. The existing code already uses `#[cfg(target_os = "macos")]` extensively with paired `#[cfg(not(target_os = "macos"))]` stubs. This is the correct approach. Do NOT introduce trait-based dispatch -- it would add unnecessary abstraction layers for a two-platform app.

**Pattern (already in use):**

```rust
// ax_reader.rs -- existing pattern
#[cfg(target_os = "macos")]
mod macos {
    // macOS implementation using AX FFI
}

#[cfg(target_os = "windows")]  // NEW: add Windows module alongside
mod windows {
    // Windows implementation using UI Automation
}

// Public API delegates to platform module
#[cfg(target_os = "macos")]
pub fn read_terminal_text(app_pid: i32, bundle_id: &str) -> Option<String> {
    macos::read_terminal_text(app_pid, bundle_id)
}

#[cfg(target_os = "windows")]
pub fn read_terminal_text(app_pid: i32, bundle_id: &str) -> Option<String> {
    windows::read_terminal_text(app_pid, bundle_id)
}
```

**Why this pattern:**
- The codebase already has `#[cfg(not(target_os = "macos"))]` stubs returning `None` for every platform-specific function -- those stubs become real Windows implementations
- No runtime dispatch overhead
- Each platform's code is isolated in its own module
- The public function signatures remain identical -- callers never change
- The Tauri command layer and frontend are completely unaware of the platform

**What changes from existing stubs:**
- Replace `#[cfg(not(target_os = "macos"))]` (catch-all stub) with `#[cfg(target_os = "windows")]` (real implementation) + keep `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` as the stub for future Linux

---

## Component-by-Component Windows Architecture

### 1. Overlay Window (lib.rs + commands/window.rs)

**macOS current approach:**
- `tauri_nspanel` converts Tauri window to NSPanel
- NSPanel with `can_become_key_window = true` + `NonactivatingPanel` style
- `PanelLevel::Status` to float above fullscreen apps
- `CollectionBehavior::full_screen_auxiliary().can_join_all_spaces()`
- `window_vibrancy::apply_vibrancy()` with HudWindow material

**Windows approach:**

`tauri_nspanel` is macOS-only. On Windows, use Tauri's native window with post-creation HWND manipulation:

```rust
#[cfg(target_os = "windows")]
fn setup_overlay_window(window: &tauri::WebviewWindow) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Foundation::HWND;

    // Get raw HWND from Tauri window
    let hwnd = window.hwnd().map_err(|e| e.to_string())?;

    unsafe {
        // Set always-on-top (equivalent to PanelLevel::Status)
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        );

        // Set WS_EX_NOACTIVATE + WS_EX_TOOLWINDOW:
        // - WS_EX_NOACTIVATE: prevents the window from stealing focus
        //   from the terminal (equivalent to NSPanel NonactivatingPanel)
        // - WS_EX_TOOLWINDOW: hides from taskbar and Alt+Tab
        //   (equivalent to skipTaskbar + ActivationPolicy::Accessory)
        let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
        SetWindowLongW(
            hwnd,
            GWL_EXSTYLE,
            (ex_style | WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0) as i32,
        );
    }

    // Apply Mica (Win11) or Acrylic (Win10) vibrancy
    // window-vibrancy crate already supports this
    if let Err(_) = window_vibrancy::apply_mica(&window, None) {
        // Fallback to Acrylic on Windows 10
        let _ = window_vibrancy::apply_acrylic(&window, Some((0, 0, 0, 128)));
    }

    Ok(())
}
```

**Critical consideration: WS_EX_NOACTIVATE + keyboard input.**
The macOS NSPanel approach allows `can_become_key_window = true` while being non-activating. On Windows, `WS_EX_NOACTIVATE` prevents the window from receiving keyboard focus entirely. The workaround is:

1. Set `WS_EX_NOACTIVATE` initially when the overlay is hidden
2. On show: temporarily remove `WS_EX_NOACTIVATE`, call `SetForegroundWindow` to take focus, then use `SetFocus` on the webview
3. On hide: restore `WS_EX_NOACTIVATE` and call `SetForegroundWindow` back to the previous terminal window

This is more complex than macOS but achievable. The key insight: we do NOT need the overlay to remain non-activating while visible (the user needs to type). We need it to:
- Not appear in taskbar or Alt+Tab (WS_EX_TOOLWINDOW handles this)
- Stay on top (HWND_TOPMOST handles this)
- Return focus to the terminal after dismiss (explicit `SetForegroundWindow(previous_hwnd)` handles this)

**The overlay show/hide flow on Windows:**

```
Show:
  1. Store previous HWND via GetForegroundWindow()
  2. Position window on current monitor (same logic as macOS, Tauri API is cross-platform)
  3. Remove WS_EX_NOACTIVATE temporarily
  4. window.show() + window.set_focus()
  5. Emit "overlay-shown" to frontend

Hide:
  1. window.hide()
  2. Restore WS_EX_NOACTIVATE
  3. SetForegroundWindow(previous_hwnd) to return focus to terminal
```

### 2. Frontmost App Detection (hotkey.rs)

**macOS:** `NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier` via ObjC FFI

**Windows:**

```rust
#[cfg(target_os = "windows")]
fn get_frontmost_pid() -> Option<i32> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    use windows::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == 0 {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid > 0 {
            Some(pid as i32)
        } else {
            None
        }
    }
}
```

Additionally, store the HWND (not just PID) in AppState for focus restoration:

```rust
// New AppState field for Windows
#[cfg(target_os = "windows")]
pub previous_app_hwnd: Mutex<Option<isize>>,  // HWND as isize
```

**Confidence: HIGH** -- `GetForegroundWindow` + `GetWindowThreadProcessId` is the standard Windows approach, well-documented, stable API.

### 3. Terminal Context: Process Inspection (terminal/process.rs)

**macOS:** `libproc` FFI for CWD (`proc_pidinfo` PROC_PIDVNODEPATHINFO), process name (`proc_pidpath`), child PIDs (`proc_listchildpids`), plus `pgrep`/`ps` fallbacks.

**Windows:**

| Function | Windows API | Crate |
|----------|------------|-------|
| Get CWD of remote process | `NtQueryInformationProcess` + `ReadProcessMemory` to read PEB -> RTL_USER_PROCESS_PARAMS -> CurrentDirectory | `windows` crate (`Wdk::System::Threading`) |
| Get process name | `GetModuleFileNameExW` or `QueryFullProcessImageNameW` | `windows` crate |
| Get child PIDs | `CreateToolhelp32Snapshot` + `Process32Next` filtering by `th32ParentProcessID` | `windows` crate |
| Get parent PID | `CreateToolhelp32Snapshot` + `Process32First/Next` | `windows` crate |

**CWD reading is the hardest part.** On Windows, reading a remote process's CWD requires:
1. `OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, pid)`
2. `NtQueryInformationProcess(ProcessBasicInformation)` to get PEB address
3. `ReadProcessMemory` to read PEB structure
4. `ReadProcessMemory` to read `RTL_USER_PROCESS_PARAMS`
5. `ReadProcessMemory` to read the `CurrentDirectory.DosPath` UNICODE_STRING

**Alternative (simpler but slower):** Use the `sysinfo` crate which provides cross-platform process inspection. However, `sysinfo` does NOT expose CWD on Windows (the field is always empty). So the PEB approach is required.

**Alternative fallback for CWD:** Shell-specific approaches:
- PowerShell: `(Get-Process -Id $pid).Path` does not give CWD, but `$PWD` environment variable can be read from the process environment block
- cmd.exe: The PEB approach works
- For WSL processes: CWD must be translated from Linux paths to Windows paths via `/mnt/c/...`

**Process tree walking on Windows:**
The `CreateToolhelp32Snapshot` API is equivalent to macOS's `proc_listchildpids`. The approach is:
1. Take a snapshot of all processes
2. Iterate and filter by `th32ParentProcessID == target_pid`
3. Walk the same shell detection logic (KNOWN_SHELLS check, MULTIPLEXERS, SHELL_WRAPPERS)

Windows shell names differ:
```rust
const KNOWN_SHELLS_WINDOWS: &[&str] = &[
    "powershell.exe", "pwsh.exe",  // PowerShell 5.1 and 7+
    "cmd.exe",                      // Command Prompt
    "bash.exe",                     // Git Bash / WSL
    "wsl.exe",                      // WSL launcher
    "zsh.exe", "fish.exe",          // Rare on Windows but possible via MSYS2
    "nu.exe",                       // Nushell
];
```

**Confidence: MEDIUM** -- The core APIs are well-documented, but reading remote process CWD via PEB is complex and may fail for elevated/protected processes. Needs thorough testing.

### 4. Terminal Text Reading (terminal/ax_reader.rs)

**macOS:** Accessibility API (`AXUIElement*`) to walk the AX tree and read `AXValue` from text areas.

**Windows:** Microsoft UI Automation (UIA) API.

**Approach:** Use the `uiautomation` crate (wrapper around Windows UIA COM API) or use the `windows` crate directly for `IUIAutomation` COM interface.

```rust
#[cfg(target_os = "windows")]
mod windows_impl {
    use uiautomation::UIAutomation;

    pub fn read_terminal_text(app_pid: i32, _app_name: &str) -> Option<String> {
        let automation = UIAutomation::new().ok()?;
        let root = automation.get_root_element().ok()?;

        // Find the terminal window by PID
        let condition = automation.create_property_condition(
            uiautomation::types::UIProperty::ProcessId,
            app_pid.into(),
        ).ok()?;

        let element = root.find_first(
            uiautomation::types::TreeScope::Children,
            &condition,
        ).ok()?;

        // Walk to find text content
        // Windows Terminal exposes UIA TextPattern
        // PowerShell/cmd expose legacy console text via UIA
        let text_pattern = element.get_text_pattern().ok()?;
        let text = text_pattern.get_document_range().ok()?
            .get_text(-1).ok()?;

        if text.is_empty() { None } else { Some(text) }
    }
}
```

**Terminal-specific UIA patterns:**
- **Windows Terminal:** Exposes `ITextProvider` (UIA Text Pattern) for reading terminal content
- **PowerShell (conhost):** The console host exposes text via UIA, accessible through `ITextProvider`
- **cmd.exe (conhost):** Same as PowerShell -- both use conhost which supports UIA
- **Git Bash (mintty):** May have limited UIA support -- needs testing
- **WSL terminals:** Depends on the host terminal (Windows Terminal provides UIA)

**NVDA (screen reader) has confirmed that Windows Terminal and conhost support UIA text reading**, which validates this approach.

**Confidence: MEDIUM** -- UIA text reading from Windows Terminal is proven (NVDA does it), but the exact Rust implementation path via `uiautomation` crate needs validation. The crate is actively maintained (latest release 2024) but less battle-tested than macOS AX API usage.

### 5. Paste to Terminal (commands/paste.rs)

**macOS:** AppleScript dispatch (iTerm2 `write text`, Terminal.app `keystroke`) + CGEventPost fallback (Cmd+V, Ctrl+U).

**Windows:**

```rust
#[cfg(target_os = "windows")]
pub fn paste_to_terminal(app: AppHandle, command: String) -> Result<(), String> {
    // 1. Write to Windows clipboard
    clipboard_win::set_clipboard_string(&command)
        .map_err(|e| format!("Clipboard error: {}", e))?;

    // 2. Restore focus to the terminal
    let state = app.try_state::<AppState>()
        .ok_or("AppState not found")?;
    let hwnd = state.previous_app_hwnd.lock()
        .map_err(|_| "mutex poisoned")?
        .ok_or("no previous HWND")?;

    unsafe {
        SetForegroundWindow(HWND(hwnd));
        std::thread::sleep(Duration::from_millis(100));

        // 3. Clear current line (Ctrl+U equivalent for PowerShell/cmd)
        //    PowerShell: Escape key clears line
        //    cmd.exe: Escape key clears line
        //    Git Bash: Ctrl+U works (same as macOS)
        send_key(VK_ESCAPE, &[])?;
        std::thread::sleep(Duration::from_millis(50));

        // 4. Paste via Ctrl+V (universal on Windows)
        send_key(VK_V, &[VK_CONTROL])?;
    }

    Ok(())
}

// SendInput helper for simulating keystrokes
unsafe fn send_key(vk: VIRTUAL_KEY, modifiers: &[VIRTUAL_KEY]) -> Result<(), String> {
    let mut inputs: Vec<INPUT> = Vec::new();

    // Press modifiers
    for &m in modifiers {
        inputs.push(make_key_input(m, true));
    }
    // Press key
    inputs.push(make_key_input(vk, true));
    // Release key
    inputs.push(make_key_input(vk, false));
    // Release modifiers (reverse order)
    for &m in modifiers.iter().rev() {
        inputs.push(make_key_input(m, false));
    }

    let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    if sent as usize != inputs.len() {
        return Err("SendInput failed".to_string());
    }
    Ok(())
}
```

**Terminal-specific paste behavior:**
- **Windows Terminal:** Ctrl+V pastes (with bracketed paste mode in modern shells)
- **PowerShell (conhost):** Right-click or Ctrl+V pastes
- **cmd.exe:** Ctrl+V pastes (modern Windows), right-click pastes (legacy)
- **Git Bash (mintty):** Shift+Insert or right-click pastes; Ctrl+V may not work. Need to detect mintty and use Shift+Insert instead
- **WSL:** Depends on host terminal

**Confirm command (Enter keystroke):** Same `SendInput` pattern with `VK_RETURN`.

**Confidence: HIGH** -- `SendInput` is the standard Windows approach for simulating keystrokes, well-documented. The clipboard API is straightforward.

### 6. Permissions (commands/permissions.rs)

**macOS:** Accessibility permission check via `AXIsProcessTrusted()` + probe fallback. Opens System Settings.

**Windows:** Windows does NOT have an equivalent global accessibility permission gate. UI Automation and `SendInput` work without special permissions for standard (non-elevated) processes.

```rust
#[cfg(target_os = "windows")]
pub fn check_accessibility_permission() -> bool {
    true  // No equivalent permission gate on Windows
}

#[cfg(target_os = "windows")]
pub fn request_accessibility_permission(_prompt: bool) -> bool {
    true  // No-op on Windows
}

#[cfg(target_os = "windows")]
pub fn open_accessibility_settings() {
    // No-op on Windows, or could open Windows Settings as a convenience
}
```

**However:** If CMD+K needs to read/write to elevated (admin) processes, the app itself may need to be elevated. This is an edge case -- most terminal sessions are user-level.

**Confidence: HIGH** -- Windows UI Automation and SendInput do not require special permissions for user-level processes.

### 7. Terminal Detection (terminal/detect.rs)

**macOS:** `NSRunningApplication` ObjC FFI for bundle ID and display name.

**Windows:** No bundle IDs on Windows. Use executable path/name instead.

```rust
#[cfg(target_os = "windows")]
pub fn get_bundle_id(pid: i32) -> Option<String> {
    // On Windows, return the exe name as the "bundle ID equivalent"
    // e.g., "WindowsTerminal.exe", "powershell.exe", "cmd.exe"
    get_process_exe_name(pid)
}

#[cfg(target_os = "windows")]
fn get_process_exe_name(pid: i32) -> Option<String> {
    use windows::Win32::System::Threading::*;
    use windows::Win32::System::ProcessStatus::*;

    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid as u32,
        ).ok()?;

        let mut buf = [0u16; 260];
        let len = GetModuleFileNameExW(handle, None, &mut buf);
        CloseHandle(handle);

        if len == 0 { return None; }

        let path = String::from_utf16_lossy(&buf[..len as usize]);
        std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }
}
```

**Windows terminal identifiers (equivalent to macOS TERMINAL_BUNDLE_IDS):**

```rust
#[cfg(target_os = "windows")]
pub const TERMINAL_EXE_NAMES: &[&str] = &[
    "WindowsTerminal.exe",      // Windows Terminal
    "powershell.exe",            // PowerShell 5.1
    "pwsh.exe",                  // PowerShell 7+
    "cmd.exe",                   // Command Prompt
    "mintty.exe",                // Git Bash (MSYS2)
    "ConEmu.exe",                // ConEmu
    "ConEmu64.exe",
    "Hyper.exe",                 // Hyper terminal
    "Alacritty.exe",             // Alacritty
    "wezterm-gui.exe",           // WezTerm
    "kitty.exe",                 // kitty
];

#[cfg(target_os = "windows")]
pub const IDE_EXE_NAMES: &[&str] = &[
    "Code.exe",                  // VS Code
    "Code - Insiders.exe",       // VS Code Insiders
    "Cursor.exe",                // Cursor IDE
];
```

**Window key format on Windows:** `"exe_name:shell_pid"` instead of `"bundle_id:shell_pid"`. Same concept, different identifier. The frontend does not parse window keys -- they are opaque strings used as HashMap keys.

**Confidence: HIGH** -- Standard Win32 process inspection APIs.

### 8. Browser Console Detection (terminal/browser.rs)

**macOS:** AX API to walk windows and find DevTools title.

**Windows:** UI Automation to enumerate windows and check titles.

```rust
#[cfg(target_os = "windows")]
pub fn detect_console(app_pid: i32, _app_name: &str) -> (bool, Option<String>) {
    // Use UI Automation to enumerate windows of the browser process
    // Check window titles for "DevTools", "Developer Tools", etc.
    // Same title-matching logic as macOS (is_devtools_title)
    // If found, attempt to read text from UIA TextPattern
    (false, None)  // Stub initially, implement after core features
}
```

**This is lower priority than terminal context reading.** Browser console detection is a nice-to-have feature. The macOS implementation returns `(false, None)` for many cases already. Safe to stub initially and implement later.

### 9. System Tray (commands/tray.rs)

**Tauri's tray API is already cross-platform.** The existing `TrayIconBuilder` code works on both macOS and Windows. Adjustments needed:

- **Icon format:** macOS uses template images (white icons on transparent). Windows uses colored `.ico` files. The `tauri.conf.json` already includes `icons/icon.ico`.
- **Left-click behavior:** On macOS, `show_menu_on_left_click(false)` follows convention. On Windows, left-click typically shows the menu. Add a platform check:

```rust
let show_on_left = cfg!(target_os = "windows");
builder = builder.show_menu_on_left_click(show_on_left);
```

- **`icon_as_template(true)`:** This is macOS-only (template rendering for menu bar). On Windows it is ignored.

**Confidence: HIGH** -- Tauri's tray API abstracts platform differences well.

### 10. Vibrancy / Visual Effects

**macOS:** `window_vibrancy::apply_vibrancy()` with `NSVisualEffectMaterial::HudWindow`

**Windows:** `window_vibrancy::apply_mica()` (Win11) or `window_vibrancy::apply_acrylic()` (Win10)

```rust
#[cfg(target_os = "windows")]
{
    // Try Mica first (Windows 11, cleaner look)
    if let Err(_) = window_vibrancy::apply_mica(&window, None) {
        // Fall back to Acrylic (Windows 10, slightly noisier)
        let _ = window_vibrancy::apply_acrylic(&window, Some((18, 18, 18, 200)));
    }
}
```

The `window-vibrancy` crate (v0.5+) already supports both platforms. This is a matter of calling the right function per platform.

**Confidence: HIGH** -- `window-vibrancy` is maintained by the Tauri team, cross-platform by design.

---

## Cargo.toml Changes

The keyring crate needs platform-conditional features:

```toml
[dependencies]
keyring = { version = "3" }

[target.'cfg(target_os = "macos")'.dependencies]
keyring = { version = "3", features = ["apple-native"] }
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
accessibility-sys = "0.2"
core-foundation-sys = "0.8"

[target.'cfg(target_os = "windows")'.dependencies]
keyring = { version = "3", features = ["windows-native"] }
windows = { version = "0.58", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_System_Threading",
    "Win32_System_ProcessStatus",
    "Win32_System_Diagnostics_ToolHelp",
    "Win32_Foundation",
] }
clipboard-win = "5"
uiautomation = "0.7"
```

**Note:** `tauri-nspanel`, `accessibility-sys`, and `core-foundation-sys` are macOS-only dependencies. They must be moved to a `[target.'cfg(target_os = "macos")'.dependencies]` section to avoid compilation errors on Windows.

---

## lib.rs Setup: Platform-Conditional Initialization

```rust
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_http::init())
        .manage(AppState::default());

    // macOS-only: NSPanel plugin
    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .setup(|app| {
            let window = app.get_webview_window("main")
                .expect("Window 'main' should exist");

            #[cfg(target_os = "macos")]
            setup_macos_overlay(app, &window)?;

            #[cfg(target_os = "windows")]
            setup_windows_overlay(app, &window)?;

            // Tray setup is cross-platform
            setup_tray(app)?;

            // Hotkey registration is cross-platform (shortcut string differs)
            let default_hotkey = if cfg!(target_os = "macos") {
                "Super+K"
            } else {
                "Ctrl+K"
            };
            let app_handle = app.handle().clone();
            if let Err(e) = register_hotkey(app_handle, default_hotkey.to_string()) {
                eprintln!("Warning: Failed to register default hotkey: {}", e);
            }

            window.hide().ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // ... same command list, all commands use cfg internally
        ])
        .run(tauri::generate_context!())
        .expect("error while running CMD+K application");
}
```

---

## Window Key Changes

The `compute_window_key` function in `hotkey.rs` works the same on both platforms conceptually. The key format shifts:

- **macOS:** `"com.apple.Terminal:12345"` (bundle_id:shell_pid)
- **Windows:** `"WindowsTerminal.exe:12345"` (exe_name:shell_pid)

The frontend treats window keys as opaque strings. No frontend changes needed for this.

---

## Frontend Changes

The React/Zustand frontend is **almost entirely platform-agnostic**. Changes needed:

### 1. Default Hotkey Display

The settings UI shows the hotkey string. Default changes from "Cmd+K" to "Ctrl+K" on Windows. This is driven by the stored hotkey string from Rust, not hardcoded in React.

### 2. Onboarding Flow

The macOS onboarding includes an "Accessibility Permission" step. On Windows, this step is not needed. The onboarding component should check via IPC:

```typescript
const hasAccessibilityStep = await invoke<boolean>("check_accessibility_permission");
// On Windows, this returns true immediately, so the step is skipped
```

The existing onboarding logic likely already handles `check_accessibility_permission() === true` as "permission granted, skip step."

### 3. System Prompt

`ai.rs` currently says "You are a terminal command generator for macOS." This should be platform-aware:

```rust
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macOS";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "Windows";

let system_prompt = format!(
    "You are a terminal command generator for {}. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run...",
    PLATFORM
);
```

### 4. Destructive Command Patterns

`safety.rs` patterns include macOS-specific commands (`diskutil erase`). Windows equivalents should be added:

```rust
// Windows-specific destructive patterns (add to existing set)
r"\bformat\s+[A-Za-z]:",          // format C:  (already present)
r"\bRemove-Item\s+.*-Recurse",    // PowerShell rm -rf equivalent
r"\bdel\s+/[sS]",                 // cmd.exe recursive delete
r"\brd\s+/[sS]",                  // cmd.exe recursive rmdir
r"\bReg\s+Delete\b",              // Registry deletion
r"\bStop-Process\s+.*-Force",     // PowerShell kill -9 equivalent
r"\bnet\s+stop\b",                // Stop Windows service
r"\bdism\b",                       // DISM system image manipulation
r"\bbcdedit\b",                   // Boot configuration
```

---

## Data Flow: Overlay Show/Hide on Windows

```
Ctrl+K pressed
  -> hotkey handler fires (cross-platform, tauri_plugin_global_shortcut)
  -> get_frontmost_pid():
       [macOS] NSWorkspace -> processIdentifier
       [Windows] GetForegroundWindow -> GetWindowThreadProcessId
  -> store PID in AppState.previous_app_pid
  -> [Windows only] store HWND in AppState.previous_app_hwnd
  -> pre-capture AX/UIA text:
       [macOS] ax_reader::read_focused_text_fast(pid)
       [Windows] uia_reader::read_focused_text_fast(pid)
  -> compute_window_key(pid, focused_cwd)
  -> toggle_overlay():
       [macOS] panel.show_and_make_key()
       [Windows] remove WS_EX_NOACTIVATE, window.show(), window.set_focus()
  -> frontend receives "overlay-shown" event
  -> store.show() executes (identical on both platforms)
  -> IPC calls: get_window_key, get_app_context (platform-specific Rust, same IPC interface)
  -> user types query, submits
  -> stream_ai_response (identical on both platforms)
  -> paste_to_terminal:
       [macOS] AppleScript/CGEventPost
       [Windows] clipboard-win + SendInput(Ctrl+V)
  -> focus restoration:
       [macOS] panel.resign_key_window + panel.make_key_window
       [Windows] SetForegroundWindow(previous_hwnd)
```

---

## Build System

### Single Codebase, Platform-Conditional Compilation

The existing project structure supports this. Rust's `cfg(target_os)` at compile time means:
- **On macOS:** Only macOS code is compiled. `tauri-nspanel`, `accessibility-sys` dependencies are used.
- **On Windows:** Only Windows code is compiled. `windows`, `clipboard-win`, `uiautomation` dependencies are used.

### CI/CD: GitHub Actions Matrix

```yaml
strategy:
  matrix:
    include:
      - os: macos-latest
        target: aarch64-apple-darwin
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: windows-latest
        target: x86_64-pc-windows-msvc
```

Tauri provides `tauri-action` for building platform-specific installers:
- macOS: `.dmg` (existing)
- Windows: `.msi` or `.exe` installer via NSIS or WiX

### Development

Cross-compilation from macOS to Windows is possible but fragile. **Recommendation: develop and test Windows features on a Windows machine or VM.** Use CI for cross-platform build validation.

---

## Suggested Build Order (Dependency-Driven)

### Phase 1: Build Infrastructure (no user-visible features)

1. **Cargo.toml restructure** -- Move macOS-only dependencies to `[target.'cfg(target_os = "macos")']`, add Windows dependencies
2. **lib.rs conditional setup** -- Split `setup()` into `setup_macos_overlay()` and `setup_windows_overlay()`
3. **Compile gate:** `cargo build --target x86_64-pc-windows-msvc` must pass with stubs

### Phase 2: Overlay Window on Windows

4. **Window HWND manipulation** -- `WS_EX_TOOLWINDOW`, `HWND_TOPMOST`, vibrancy (Mica/Acrylic)
5. **Show/hide cycle** -- Focus management, `WS_EX_NOACTIVATE` toggle
6. **Hotkey: `get_frontmost_pid` Windows implementation** -- `GetForegroundWindow` + `GetWindowThreadProcessId`
7. **Store HWND for focus restoration**

### Phase 3: Terminal Context on Windows

8. **`terminal/detect.rs` Windows implementation** -- Process exe name detection, terminal/IDE classification
9. **`terminal/process.rs` Windows implementation** -- Process tree walking via `CreateToolhelp32Snapshot`, CWD via PEB reading
10. **`terminal/ax_reader.rs` -> `terminal/uia_reader.rs`** -- UI Automation text reading from Windows Terminal/conhost

### Phase 4: Paste and Input Simulation

11. **`commands/paste.rs` Windows implementation** -- Clipboard + `SendInput(Ctrl+V)`
12. **Line clearing** -- `SendInput(VK_ESCAPE)` for PowerShell/cmd, `SendInput(Ctrl+U)` for bash

### Phase 5: Polish and Platform-Specific UI

13. **Permissions: stub Windows implementation** (always returns true)
14. **Onboarding flow adaptation** (skip accessibility step)
15. **AI system prompt: platform awareness** ("You are a terminal command generator for Windows...")
16. **Safety patterns: add Windows-specific destructive commands**
17. **Tray icon adjustments** (left-click behavior, icon format)
18. **Default hotkey: Ctrl+K on Windows**

### Phase 6: Integration Testing

19. **End-to-end testing** on Windows: Windows Terminal, PowerShell, cmd.exe, Git Bash
20. **WSL terminal support** (stretch goal)
21. **Windows installer** (NSIS or WiX via Tauri)

### Dependency Graph

```
Phase 1 (Cargo.toml + lib.rs)
    |
    +---> Phase 2 (Overlay window)
    |         |
    |         +---> Phase 4 (Paste, needs HWND from Phase 2)
    |
    +---> Phase 3 (Terminal context, independent of overlay)
    |         |
    |         +---> Phase 5 (Polish, depends on Phase 3 for terminal detection)
    |
    +---> Phase 6 (Integration, depends on all above)
```

Phases 2 and 3 can be developed in parallel since they are independent.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: Trait-Based Platform Abstraction

**What:** Define a `PlatformBackend` trait with methods like `get_frontmost_pid()`, `read_terminal_text()`, etc., and implement it for macOS and Windows.

**Why wrong:** Adds an abstraction layer that the codebase does not need. There are only two platforms, the public function signatures are already identical, and `cfg(target_os)` gives zero runtime overhead. Trait objects add vtable dispatch. The existing codebase successfully uses `cfg` -- changing patterns mid-project creates inconsistency.

**Do this instead:** Continue using `cfg(target_os)` with platform submodules, exactly as already done in `ax_reader.rs`, `process.rs`, etc.

### Anti-Pattern 2: Windows-First Development on macOS

**What:** Write all Windows code on a macOS machine, relying on `cargo check --target x86_64-pc-windows-msvc` to validate.

**Why wrong:** `cargo check` verifies syntax and type checking but cannot run the code. Windows API calls have subtle behavioral differences (HWND focus semantics, SendInput timing, UIA tree structure) that only manifest at runtime. GPU terminal UIA support varies between terminals.

**Do this instead:** Use a Windows machine or VM for development and testing. Use macOS CI to verify macOS builds are not broken by the changes.

### Anti-Pattern 3: Shimming NSPanel Behavior with a Tauri Plugin

**What:** Write a Windows Tauri plugin that replicates NSPanel's non-activating panel behavior.

**Why wrong:** The macOS NSPanel behavior (non-activating + keyboard input) has no direct Win32 equivalent. Attempting to replicate it creates a maintenance burden. The Windows UX for overlay apps (PowerToys Run, Windows PowerToys, etc.) uses a different pattern: take focus, then return focus on dismiss.

**Do this instead:** Accept that Windows overlay behavior differs slightly. Take focus on show, return focus on hide. This matches user expectations on Windows.

### Anti-Pattern 4: Using `sysinfo` Crate for CWD

**What:** Use the `sysinfo` crate's `Process::cwd()` method for reading remote process CWD on Windows.

**Why wrong:** `sysinfo` returns an empty string for CWD on Windows. The CWD field is not implemented on Windows in sysinfo.

**Do this instead:** Read CWD from the Process Environment Block (PEB) using `NtQueryInformationProcess` + `ReadProcessMemory`, or shell-specific fallback methods.

---

## New Components Summary

| Component | Platform | Purpose |
|-----------|----------|---------|
| `terminal/uia_reader.rs` | Windows | UI Automation text reading (replaces AX reader) |
| `terminal/process_win.rs` or inline `windows` mod | Windows | Process tree walking + CWD via PEB |
| `terminal/detect_win.rs` or inline `windows` mod | Windows | Exe name detection, terminal classification |
| `commands/paste_win.rs` or inline `windows` mod | Windows | Clipboard + SendInput paste |
| `commands/hotkey_win.rs` or inline `windows` mod | Windows | GetForegroundWindow + HWND storage |

All of these can be either separate files or `mod windows { }` blocks inside existing files, following the `ax_reader.rs` pattern.

---

## Sources

- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy) -- Windows Acrylic/Mica support (HIGH confidence)
- [uiautomation-rs crate](https://github.com/leexgone/uiautomation-rs) -- Windows UI Automation wrapper (MEDIUM confidence)
- [keyring crate](https://github.com/open-source-cooperative/keyring-rs) -- Cross-platform credential storage including Windows Credential Manager (HIGH confidence)
- [Windows SendInput documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput) -- Keyboard simulation (HIGH confidence)
- [GetForegroundWindow documentation](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow) -- Active window detection (HIGH confidence)
- [NtQueryInformationProcess documentation](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess) -- Remote process PEB/CWD reading (MEDIUM confidence)
- [WS_EX_NOACTIVATE behavior](https://devblogs.microsoft.com/oldnewthing/20240919-00/?p=110283) -- Non-activating window caveats (HIGH confidence)
- [NVDA UIA console support](https://github.com/nvaccess/nvda/pull/9614) -- Validates UIA text reading from Windows Terminal (MEDIUM confidence)
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) -- Tauri window configuration (HIGH confidence)
- [clipboard-win crate](https://crates.io/crates/clipboard-win) -- Windows clipboard operations (HIGH confidence)
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/) -- Cross-platform tray API (HIGH confidence)
- [Windows Extended Window Styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles) -- WS_EX flags reference (HIGH confidence)
- Codebase analysis: `src-tauri/src/` -- all existing modules reviewed for platform coupling

---

*Architecture research for: CMD+K v0.2.1 Windows platform support*
*Researched: 2026-03-01*
