# Domain Pitfalls: Adding Windows Support to macOS Tauri Overlay App

**Domain:** Porting macOS Tauri overlay app (CMD+K) to Windows
**Researched:** 2026-03-01 (v0.2.1 Windows milestone)
**Confidence:** HIGH (verified against codebase, official Tauri docs, Windows API docs, community issues)

---

## Critical Pitfalls

Mistakes that cause rewrites, broken builds, or fundamental architectural failures.

### Pitfall 1: No NSPanel Equivalent on Windows -- Overlay Will Steal Focus

**What goes wrong:**
The entire macOS overlay UX depends on `tauri-nspanel` which converts a Tauri window into a macOS `NSPanel` with `NonactivatingPanel` style mask. This lets the overlay accept keyboard input without deactivating the underlying terminal. On Windows, `tauri-nspanel` simply does not exist. Using a standard Tauri `alwaysOnTop` window causes the terminal to lose focus, breaking the "capture-before-show" pattern where PID, AX text, and window key are captured from the frontmost app. When the overlay shows, `GetForegroundWindow()` returns the overlay's HWND, not the terminal.

**Why it happens:**
macOS NSPanel is a first-class OS concept with decades of API support (Spotlight, Raycast all use it). Windows has no direct equivalent. Developers assume `alwaysOnTop: true` + `skipTaskbar: true` approximates NSPanel behavior, but it still activates the window and steals focus from the terminal beneath.

**How to avoid:**
Use Win32 extended window styles via unsafe Rust FFI to the `windows` crate:
1. Set `WS_EX_NOACTIVATE` on the overlay HWND to prevent focus stealing
2. Set `WS_EX_TOOLWINDOW` to hide from Alt+Tab and taskbar
3. Use `ShowWindow(SW_SHOWNOACTIVATE)` instead of standard show
4. Handle `WM_MOUSEACTIVATE` returning `MA_NOACTIVATE` so clicks on the overlay do not steal focus from the terminal
5. For keyboard input, call `SetForegroundWindow` only when the user explicitly types into the input field (focused via mouse click), then immediately `SetForegroundWindow` back to the terminal after paste

The critical insight: the overlay must NOT activate when it appears. It should float above everything but the terminal remains the foreground window. Keyboard input to the overlay's webview requires careful `WM_MOUSEACTIVATE` handling -- the webview needs to receive input without the window becoming the "active" window in Win32 terms.

Note: Tauri has a known bug (#10422) where `skipTaskbar` does not work on Windows. The workaround is to use `SetWindowLongPtrW` to set `WS_EX_TOOLWINDOW` and clear `WS_EX_APPWINDOW` on the HWND after window creation.

**Warning signs:**
- Overlay appears but terminal cursor stops blinking
- Alt+Tab shows CMD+K in the task switcher
- `GetForegroundWindow()` in the hotkey handler returns the overlay PID after the first toggle
- Pressing Escape in the overlay does not return focus to the terminal

**Phase to address:**
Phase 1 (Windows Overlay Foundation) -- this must be solved first; every other feature depends on the overlay not stealing focus.

---

### Pitfall 2: SendInput Blocked by UIPI When Terminal Runs Elevated

**What goes wrong:**
The paste mechanism on macOS uses `CGEventPost` which works because macOS Accessibility API grants cross-process input rights globally. On Windows, `SendInput` is subject to User Interface Privilege Isolation (UIPI). If the user runs PowerShell or CMD as Administrator (elevated, high integrity), CMD+K (running non-elevated, medium integrity) cannot inject keystrokes into it. `SendInput` silently drops the input -- no error, no `GetLastError`, the function returns the count of events as if it succeeded but the keystrokes vanish. The user types a command, clicks paste, and nothing appears in their elevated terminal.

**Why it happens:**
UIPI is a Windows Vista+ security boundary. A medium-integrity process cannot send input to a high-integrity process. Most terminal users run non-elevated, but power users (sysadmins, developers running Docker) frequently use elevated terminals. The failure is completely silent.

**How to avoid:**
1. Detect elevation of the target process before attempting paste: use `OpenProcessToken` + `GetTokenInformation(TokenElevation)` on the target terminal's PID
2. If elevated, show a clear warning: "Cannot paste to elevated terminal. Either run CMD+K as Administrator or run your terminal without elevation."
3. Do NOT run CMD+K as Administrator by default -- this is a security anti-pattern and triggers SmartScreen/UAC prompts
4. Alternative: use clipboard-based paste (`SetClipboardData` + simulate Ctrl+V) which works in some elevated scenarios where the target terminal reads from clipboard directly rather than receiving SendInput events
5. Long-term: investigate `UIAccess` manifest flag (requires code signing + installation to a trusted location like Program Files), which allows a medium-integrity process to send input to elevated windows

**Warning signs:**
- Paste works to normal terminals but silently fails to "Run as Administrator" terminals
- Users report "paste does nothing" intermittently (they sometimes run elevated, sometimes not)
- `SendInput` returns success (non-zero) but no characters appear

**Phase to address:**
Phase 3 (Terminal Paste) -- must be addressed during paste implementation, not deferred.

---

### Pitfall 3: Windows Terminal Hides Shell PIDs Behind a Multi-Process Architecture

**What goes wrong:**
On macOS, the process tree is simple: Terminal.app -> login -> zsh, or iTerm2 -> zsh. The app walks children with `proc_listchildpids` or `pgrep -P`. On Windows, Windows Terminal (wt.exe) is a single UWP/Win32 process that hosts multiple tabs, each running a separate ConPTY-connected shell (powershell.exe, cmd.exe, bash.exe). The shell processes are NOT direct children of wt.exe. They are spawned by `OpenConsole.exe` (the ConPTY host), which itself is spawned by the Windows Terminal process tree. The parent-child chain is: `WindowsTerminal.exe -> OpenConsole.exe -> powershell.exe`. But `GetForegroundWindow()` returns the HWND of `WindowsTerminal.exe`, and there is no documented public API to determine WHICH TAB is active.

**Why it happens:**
Windows Terminal's architecture is fundamentally different from macOS terminal apps. A single window hosts multiple tabs, each with its own ConPTY session. Unlike macOS where each iTerm2 tab is visible in the AX tree, Windows Terminal does not expose per-tab information to external processes. The process tree enumeration finds ALL shell processes under the Windows Terminal tree, with no way to distinguish the active tab's shell from background tabs.

**How to avoid:**
1. Use `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` + `Process32First/Next` to build the full process tree -- this is the Windows equivalent of `pgrep -P` and `proc_listchildpids`. The `PROCESSENTRY32.th32ParentProcessID` field gives parent PID
2. For Windows Terminal specifically, find all shell processes descended from `WindowsTerminal.exe` and use a heuristic: the most recently focused shell (based on process creation time via `GetProcessTimes`) or the shell whose CWD matches the window title
3. Windows Terminal exposes the current tab's shell name and CWD in the window title (e.g., "powershell - C:\Users\foo\project"). Parse the window title via `GetWindowTextW` on the HWND from `GetForegroundWindow()`
4. Fall back to highest-PID heuristic (matching the macOS `find_shell_by_ancestry` pattern) when title parsing fails
5. For CMD.exe and PowerShell.exe launched standalone (not inside Windows Terminal), they ARE the foreground process directly -- no tree walk needed

**Warning signs:**
- `get_terminal_context` returns context from the wrong tab
- History entries appear under the wrong window key
- Multiple shell PIDs found with no way to pick the correct one
- CWD is wrong when user has multiple tabs open in Windows Terminal

**Phase to address:**
Phase 2 (Terminal Context Reading) -- process tree walking is the foundation for CWD, shell type, and visible output.

---

### Pitfall 4: Breaking macOS When Adding Windows cfg Blocks

**What goes wrong:**
The codebase has 15+ Rust files with `#[cfg(target_os = "macos")]` blocks and corresponding `#[cfg(not(target_os = "macos"))]` stubs. When adding Windows support, developers add `#[cfg(target_os = "windows")]` blocks but accidentally remove or modify the existing macOS code paths. Especially dangerous: changing a `#[cfg(not(target_os = "macos"))]` stub (which currently covers all non-macOS targets including Windows) to `#[cfg(target_os = "windows")]`, which now leaves Linux and other targets without an implementation -- future Linux support breaks. Or worse: editing the body of a `#[cfg(target_os = "macos")]` function while testing only on Windows, introducing a bug that is invisible until the next macOS build.

Specific files at risk in this codebase:
- `hotkey.rs`: `get_frontmost_pid()` -- macOS uses ObjC FFI, Windows needs `GetForegroundWindow`
- `paste.rs`: entire `cg_keys` module is `#[cfg(target_os = "macos")]`
- `process.rs`: `get_process_cwd`, `get_process_name`, `get_child_pids`, `find_shell_by_ancestry` all have macOS FFI + non-macOS stubs
- `detect.rs`: `get_bundle_id`, `get_app_display_name` use ObjC runtime
- `permissions.rs`: `check_accessibility_permission`, `request_accessibility_permission` use CoreFoundation
- `ax_reader.rs`: entire 800-line `macos` module + non-macOS stubs
- `lib.rs`: NSPanel setup, vibrancy, activation policy all in `#[cfg(target_os = "macos")]`

**Why it happens:**
Rust's `#[cfg]` is evaluated at compile time. When developing on Windows, the macOS code paths are not compiled or tested. A typo, missing import, or changed function signature in macOS code is invisible until someone builds on macOS. CI that only runs on one platform misses the other.

**How to avoid:**
1. Adopt a platform-module pattern: `terminal/process_macos.rs`, `terminal/process_windows.rs`, `terminal/process.rs` (facade with `cfg` re-exports). Each platform file is self-contained -- editing the Windows file cannot touch the macOS file
2. CI must build on BOTH macOS and Windows for every PR. Use `cargo check --target x86_64-pc-windows-msvc` on macOS (cross-check) and vice versa
3. Replace `#[cfg(not(target_os = "macos"))]` stubs with explicit `#[cfg(target_os = "windows")]` implementations + `#[cfg(not(any(target_os = "macos", target_os = "windows")))]` stubs for future platforms
4. Never edit a macOS `#[cfg]` block while developing on Windows -- changes to the macOS path require macOS compilation to verify
5. Use the `cfg_if!` macro for cleaner three-way platform branching

**Warning signs:**
- macOS build fails after a Windows PR merge
- `cargo check --target x86_64-apple-darwin` on CI fails with "function not found" errors
- Non-macOS stubs return `None` when they should have been replaced with Windows implementations
- Tests pass on Windows but fail on macOS

**Phase to address:**
Phase 1 (Project Setup) -- establish the platform-module pattern and cross-platform CI before writing any Windows code.

---

### Pitfall 5: Windows Defender SmartScreen Blocks Unsigned Binaries

**What goes wrong:**
On macOS, the app is already signed with a Developer ID certificate. On Windows, distributing an unsigned EXE triggers SmartScreen, which shows a full-screen blue warning: "Windows protected your PC -- Microsoft Defender SmartScreen prevented an unrecognized app from starting." Most users will not click "More info -> Run anyway." Even with an OV code signing certificate, SmartScreen still shows warnings until the certificate builds reputation (weeks to months of downloads). As of March 2024, even EV certificates no longer get instant SmartScreen bypass.

Additionally, the NSIS uninstaller (`uninstall.exe`) is NOT signed by Tauri's signing process (Issue #7348), so even if the main EXE is signed, uninstallation triggers a separate UAC warning.

**Why it happens:**
Microsoft's SmartScreen reputation system is opaque and based on download volume + certificate age. New certificates start with zero reputation regardless of type (OV vs EV). Unlike Apple's notarization (which is a one-time per-build process), SmartScreen reputation builds over time through user downloads.

**How to avoid:**
1. Purchase a code signing certificate BEFORE starting development -- reputation building takes time
2. Use an EV certificate if budget allows (still the fastest path to reputation, even post-March 2024)
3. Configure `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` environment variables for Tauri's built-in signing
4. Use the `signCommand` option in `tauri.conf.json` under `bundle > windows` if using an external signing tool (e.g., Azure SignTool for cloud-based EV certificates)
5. Sign BOTH the MSI/NSIS installer AND the main EXE
6. Start distributing signed beta builds early to begin building reputation
7. Consider MSI over NSIS -- MSI triggers fewer false positives with enterprise security tools and is easier to deploy via Group Policy in enterprise environments

**Warning signs:**
- Users report "Windows protected your PC" when downloading
- Download counts are low because users abandon at the SmartScreen warning
- Antivirus false positives on the unsigned EXE
- Enterprise customers cannot install due to security policy

**Phase to address:**
Phase 7 (Build and Distribution) -- but certificate purchase should happen in Phase 1 to start building reputation early.

---

### Pitfall 6: Getting CWD on Windows Requires Reading Another Process's PEB

**What goes wrong:**
On macOS, getting a process's CWD is trivial: `proc_pidinfo(pid, PROC_PIDVNODEPATHINFO)` returns the CWD directly, or `lsof -p pid -d cwd` as a fallback. On Windows, there is NO public, documented, stable API to get the current working directory of another process. The only reliable method is reading the Process Environment Block (PEB) of the target process via `NtQueryInformationProcess(ProcessBasicInformation)` + `ReadProcessMemory` to extract `RTL_USER_PROCESS_PARAMETERS.CurrentDirectory`. This is an undocumented NT API that works but is technically internal. Microsoft explicitly notes: "These functions may be altered or unavailable in future versions of Windows."

**Why it happens:**
Unix exposes process state via `/proc/<pid>/cwd` (symlink) and `proc_pidinfo` (macOS). Windows has no such abstraction. The PEB approach is what Process Explorer, PowerShell, and every other tool uses internally, but it requires `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ` access to the target process.

**How to avoid:**
1. Use `NtQueryInformationProcess` + `ReadProcessMemory` to read PEB -> `RTL_USER_PROCESS_PARAMETERS` -> `CurrentDirectory.DosPath` -- this is the de facto standard approach
2. The `windows` crate (`windows::Wdk::System::Threading::NtQueryInformationProcess`) exposes this function
3. Handle the case where `OpenProcess` fails due to insufficient privileges (elevated terminal processes may deny access from a non-elevated CMD+K)
4. Verify the approach works for 64-bit processes from a 64-bit CMD+K (WOW64 complications exist when reading 32-bit process PEB from 64-bit process, but in practice all modern shells are 64-bit)
5. Fall back to parsing the Windows Terminal window title for CWD if PEB reading fails
6. For PowerShell specifically, the CWD from PEB may lag behind the actual shell CWD (PowerShell changes directory with `Set-Location` which updates its internal state but the process CWD may not update immediately). Consider supplementing with `wmic process where processid=PID get commandline` or window title parsing

**Warning signs:**
- CWD returned is `C:\Windows\System32` for all terminals (default process CWD, PEB read failed)
- Access denied errors when targeting elevated processes
- CWD is correct for CMD but wrong for PowerShell (PEB CWD vs PowerShell's internal location)

**Phase to address:**
Phase 2 (Terminal Context Reading) -- CWD is the most critical terminal context field.

---

### Pitfall 7: Vibrancy Looks and Behaves Differently on Windows

**What goes wrong:**
On macOS, `apply_vibrancy(&window, NSVisualEffectMaterial::HudWindow, None, Some(12.0))` gives a consistent frosted glass look. On Windows, the `window-vibrancy` crate (now built into Tauri v2) provides `apply_acrylic` and `apply_mica`, but they behave fundamentally differently:
- Acrylic has significant performance issues when resizing/dragging on Windows 11 22621+
- Mica only works on Windows 11, not Windows 10
- Neither Acrylic nor Mica supports rounded corners natively -- the DWM (Desktop Window Manager) handles corner rounding automatically on Windows 11, but on Windows 10 windows are rectangular
- The blur radius and color tint are different from macOS vibrancy
- Transparent windows with `decorations: false` may show a white flash on creation before the vibrancy effect applies

**Why it happens:**
macOS vibrancy is a deeply integrated OS feature with fine-grained control (material, blending mode, corner radius). Windows Acrylic/Mica is a newer DWM composition effect with less flexibility. The Tauri `window-vibrancy` integration abstracts platform differences but cannot eliminate them.

**How to avoid:**
1. Use Mica on Windows 11 (looks best, least performance overhead), fall back to Acrylic on Windows 10
2. Detect Windows version at runtime: `windows::Win32::System::SystemInformation::GetVersionExW` or parse `winver`
3. Set `decorations: false` and `transparent: true` in the window config (already done for macOS)
4. Accept that rounded corners come from Windows 11 DWM automatically (do not try to implement them manually via clip-path on Windows 10)
5. Add a small delay (50-100ms) before showing the window to avoid the white flash during vibrancy initialization
6. Test on BOTH Windows 10 and Windows 11 -- the visual appearance is significantly different

**Warning signs:**
- White flash when overlay appears
- Rectangular corners on Windows 10 (expected, not a bug)
- Laggy overlay animation during drag/resize
- Vibrancy effect not visible (window appears solid black or white)

**Phase to address:**
Phase 1 (Windows Overlay Foundation) -- visual fidelity is part of the overlay UX.

---

### Pitfall 8: Ctrl+K Conflicts with Browser and Editor Shortcuts

**What goes wrong:**
On macOS, Cmd+K is relatively conflict-free (few apps use it). On Windows, the equivalent Ctrl+K is already used by:
- Chrome/Edge/Firefox: opens the address bar / search
- VS Code: chord prefix for dozens of keybindings (Ctrl+K, Ctrl+C = comment, etc.)
- Slack: focus the search bar
- Notion: create a link
- Microsoft Teams: open the search box
- Word/Outlook: insert hyperlink

When CMD+K registers Ctrl+K as a global hotkey, it steals the shortcut from every application on the system. Users lose critical shortcuts in their daily tools. Worse: some applications (VS Code) wait for a second key after Ctrl+K -- the global hotkey fires before VS Code sees the chord, causing VS Code to enter an inconsistent state.

**Why it happens:**
macOS has a cultural separation between Cmd (OS/app shortcuts) and Ctrl (terminal shortcuts). Windows conflates Ctrl for both purposes. Ctrl+K is an extremely popular shortcut across the Windows ecosystem, much more so than Cmd+K is on macOS.

**How to avoid:**
1. Default to a DIFFERENT hotkey on Windows: `Ctrl+Shift+K` or `Alt+Space` (which mirrors Spotlight/Raycast behavior)
2. Make the hotkey configurable from the FIRST RUN onboarding (already have this feature on macOS)
3. In the hotkey configuration UI, warn about known conflicts: "Ctrl+K conflicts with Chrome, VS Code, and Slack"
4. Consider using `Win+K` but NOTE: Windows 10/11 reserves `Win+K` for "Connect to wireless displays" -- this will NOT work as a global hotkey
5. Detect if the foreground application is a known conflicting app and show a brief notification if so
6. Document recommended alternatives in the onboarding flow

**Warning signs:**
- User reports "Chrome search bar stopped working"
- VS Code users report broken chord keybindings
- GitHub issues titled "hotkey conflicts with [popular app]"
- Low adoption on Windows despite macOS popularity

**Phase to address:**
Phase 1 (Windows Overlay Foundation) -- hotkey registration is the first user interaction.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using `wmic process` for process queries | Easy to implement, no FFI needed | wmic.exe is deprecated since Win10 21H1, spawns subprocess (100ms+ per call), breaks in future Windows | Never -- use `CreateToolhelp32Snapshot` from the start |
| Hardcoding `i32` for PIDs on Windows | Matches macOS `pid_t` type, no refactoring | Windows PIDs are `DWORD` (u32), can exceed i32 range, causes subtle truncation bugs | Never -- use a `PlatformPid` type alias: `i32` on macOS, `u32` on Windows |
| Using `clipboard + Ctrl+V SendInput` for paste | Simple, works for non-elevated terminals | Clobbers user clipboard, fails for elevated terminals, race condition between clipboard write and paste keystroke | MVP only -- replace with `SendInput` direct character injection or per-terminal API later |
| Running CMD+K as Administrator | Bypasses UIPI, paste works everywhere | UAC prompt on every launch, security risk, breaks auto-start, unsigned EXE + UAC = red flag for users | Never -- use UIAccess manifest or detect+warn instead |
| Using `GetWindowText` for terminal detection | Quick heuristic for identifying terminal apps | Window titles change with CWD, running process, user config; brittle pattern matching | Acceptable as supplementary signal, never as primary detection |
| Single `process.rs` file with `#[cfg]` blocks | Less files, faster initial development | 600+ line file becomes 1200+ with Windows code interleaved, impossible to review, easy to accidentally modify wrong platform block | Never -- split into `process_macos.rs` + `process_windows.rs` from day one |

## Integration Gotchas

Common mistakes when connecting platform-specific Windows services.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Windows Credential Manager (keyring crate) | Using `features = ["apple-native"]` on Windows, or not specifying `windows-native` feature | Use `features = ["apple-native"]` on macOS, `features = ["windows-native"]` on Windows. In Cargo.toml: `[target.'cfg(target_os = "macos")'.dependencies] keyring = { version = "3", features = ["apple-native"] }` and `[target.'cfg(target_os = "windows")'.dependencies] keyring = { version = "3", features = ["windows-native"] }` |
| Windows Credential Manager (threading) | Accessing keyring Entry from multiple threads simultaneously | keyring docs explicitly warn: "Operating on the same entry from different threads does not reliably sequence the operations." Serialize all keyring access through a single-threaded executor or use a mutex |
| Windows Credential Manager (enterprise) | Assuming Credential Manager is always accessible | Credential Guard (default-on in Windows 11 22H2+ enterprise) may restrict credential access. Some enterprise Group Policies disable Credential Manager entirely. Add a fallback to encrypted file storage or detect policy restrictions |
| System Tray on Windows | Assuming tray icon works identically to macOS menu bar | Windows system tray is in the bottom-right (not top-right). Tray icons can be hidden in the "overflow" area by default. Users must manually pin the icon. The app should prompt or document this |
| `open` command for URLs | Using `open` (macOS command) to launch URLs/settings | Windows uses `start` or `ShellExecuteW`. Replace `std::process::Command::new("open")` with platform-specific: `cmd /c start` on Windows |
| `pbcopy` for clipboard | Using `pbcopy` to write to clipboard | Windows has no `pbcopy`. Use `SetClipboardData` via Win32 API or the `clipboard-win` / `arboard` crate for cross-platform clipboard access |

## Performance Traps

Patterns that work at small scale but fail under real usage.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Spawning `wmic` or `tasklist` per hotkey press | 200-500ms delay before overlay appears | Use `CreateToolhelp32Snapshot` (single call, <5ms) | First hotkey press feels laggy |
| Building full process tree on every hotkey | Snapshot of ALL processes (hundreds) parsed on main thread | Cache process tree, invalidate on timer (e.g., 2s TTL) | When system has 500+ processes |
| `ReadProcessMemory` on every CWD query without caching | Cross-process memory read per overlay invocation | Cache CWD per shell PID with 1s TTL, invalidate on window key change | When user hammers hotkey rapidly |
| Windows DPI change event causing overlay resize loop | Overlay flickers/resizes when dragged between monitors with different DPI | Handle `WM_DPICHANGED` once, debounce with 100ms delay | When user has mixed-DPI multi-monitor setup (very common on Windows: laptop 150% + external 100%) |

## Security Mistakes

Domain-specific security issues for a Windows overlay app.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Running CMD+K as Administrator to bypass UIPI | Broad attack surface: any malicious command runs elevated | Never default to admin. Detect elevated terminals and warn. Use UIAccess manifest (requires code signing) for cross-integrity input |
| Storing API key in plaintext config file instead of Credential Manager | Key exposed to any process that can read the file | Use keyring crate with `windows-native` feature. Already have Keychain on macOS -- port properly |
| Using `clip.exe` or PowerShell clipboard for paste (spawns subprocess with key visible in process list) | API key or commands visible in `Get-Process` output | Use Win32 `SetClipboardData` API directly via the `windows` crate, or use `SendInput` character injection |
| Not signing the EXE | SmartScreen blocks installation; antivirus false positives; enterprise GPO blocks unsigned software | Sign with EV or OV certificate. Budget this early -- it is a distribution requirement, not optional |
| Ignoring Windows path traversal in CWD/command strings | Path injection if CWD contains special characters (e.g., `%APPDATA%` expansion) | Sanitize paths, use `\\?\` prefix for long paths, never pass CWD through shell expansion |

## UX Pitfalls

Common user experience mistakes when porting macOS overlay UX to Windows.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Using macOS-style keyboard shortcuts in UI text ("Cmd+V", "Cmd+K") | Confuses Windows users who see unfamiliar modifier keys | Detect platform, show "Ctrl+V" on Windows, "Cmd+V" on macOS. Use Tauri's `navigator.platform` or pass OS type from Rust |
| Onboarding asks for Accessibility permission (macOS concept) | Windows has no equivalent permission. Users see a confusing request with no action to take | Skip Accessibility step on Windows. Windows does not need explicit permission for input injection (UIPI is automatic based on integrity level) |
| Overlay appears at macOS-style 25% from top | On Windows with taskbar at bottom, 25% from top may feel too high. Windows convention is more centered (like Start menu search, PowerToys Run) | Consider 30-35% from top on Windows, or let the user choose. Test with taskbar at top/left/right/bottom |
| System tray icon not visible by default | Windows auto-hides new tray icons in the overflow area. Users think the app is not running | On first run, show a notification balloon from the tray icon explaining where to find it. Prompt user to pin the icon |
| No context menu on right-click of tray icon | macOS menu bar apps use left-click for menu. Windows convention is right-click for context menu, left-click for primary action | Register both left-click (show overlay) and right-click (show settings menu) handlers on the tray icon |
| Escape key dismissal assumes NSPanel resign behavior | On macOS, `panel.hide()` + `resign_key_window()` returns focus to the terminal. On Windows, hiding the overlay window does not automatically reactivate the previous foreground window | After hiding the overlay, explicitly call `SetForegroundWindow(previous_hwnd)` to return focus to the terminal. Store the previous HWND at hotkey capture time |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Overlay window:** Shows on top -- but verify it does NOT appear in Alt+Tab, does NOT steal focus, and tray icon is visible (not in overflow)
- [ ] **Hotkey registration:** Ctrl+K works -- but verify it does not break Chrome address bar, VS Code chords, or Slack search
- [ ] **Terminal paste:** Works for normal PowerShell -- but verify it works for CMD, Git Bash, WSL terminals, AND fails gracefully for elevated terminals
- [ ] **CWD detection:** Works for standalone CMD -- but verify it works for PowerShell (which may not update process CWD), Windows Terminal tabs, and WSL shells
- [ ] **Window key computation:** Returns a key -- but verify it returns DIFFERENT keys for different Windows Terminal tabs, not the same key for all tabs
- [ ] **API key storage:** Saves/loads via Credential Manager -- but verify it works on machines with Credential Guard enabled and under enterprise Group Policy restrictions
- [ ] **Vibrancy effect:** Looks good on Windows 11 -- but verify appearance on Windows 10 (no Mica, no rounded corners), and on high-DPI displays
- [ ] **Auto-start on login:** App starts at boot -- but verify it does NOT trigger UAC prompt and works via Task Scheduler or registry Run key
- [ ] **Cross-platform build:** Windows build passes -- but verify macOS build still passes after all Windows changes (CI on both platforms)
- [ ] **Process tree walk:** Finds shell PID -- but verify `DWORD` (u32) PIDs are handled correctly, not truncated to i32
- [ ] **WSL detection:** Detects WSL bash -- but verify CWD resolves correctly (Linux path vs Windows path), and that the window key is stable across WSL restarts

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Overlay steals focus (no WS_EX_NOACTIVATE) | MEDIUM | Add WS_EX_NOACTIVATE post-creation via SetWindowLongPtrW. Requires understanding Win32 window styles but does not require architectural changes |
| SendInput fails for elevated terminals | LOW | Add elevation detection + warning UI. Does not require code restructuring |
| macOS build broken by Windows changes | HIGH if discovered late, LOW if CI catches it | Add cross-platform CI immediately. Fix involves reverting or fixing cfg blocks |
| SmartScreen blocks unsigned EXE | HIGH (reputation takes weeks) | Purchase certificate, sign, distribute, wait for reputation. No quick fix |
| Wrong CWD for PowerShell | MEDIUM | Add window title parsing as supplementary CWD source. Requires additional code but not architectural change |
| Ctrl+K conflicts break user workflows | LOW | Change default hotkey, add conflict detection. UI change only |
| Clipboard clobbered during paste | MEDIUM | Implement save/restore clipboard pattern: read clipboard before writing command, restore after paste. Or switch to SendInput character injection |
| PID type mismatch (i32 vs DWORD) | HIGH if caught late | Introduce `PlatformPid` type alias across entire codebase. Touches many files but is mechanical |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| No NSPanel equivalent (focus stealing) | Phase 1: Overlay Foundation | Overlay appears without Alt+Tab entry; terminal cursor stays active; GetForegroundWindow still returns terminal HWND |
| SendInput blocked by UIPI | Phase 3: Terminal Paste | Paste to normal terminal succeeds; paste to elevated terminal shows warning; no silent failure |
| Windows Terminal process tree | Phase 2: Terminal Context | Correct CWD for active tab in multi-tab Windows Terminal; different window keys per tab |
| Breaking macOS with cfg blocks | Phase 1: Project Setup | CI passes on both macOS and Windows; cargo check --target on both platforms |
| SmartScreen blocks unsigned EXE | Phase 7: Distribution (certificate in Phase 1) | Signed EXE downloads without SmartScreen warning (after reputation period) |
| PEB-based CWD reading | Phase 2: Terminal Context | CWD returned for CMD, PowerShell, Git Bash; graceful failure for elevated processes |
| Vibrancy differences | Phase 1: Overlay Foundation | Acrylic/Mica applied on Win11; graceful fallback on Win10; no white flash |
| Ctrl+K hotkey conflicts | Phase 1: Overlay Foundation | Default hotkey does not conflict with Chrome/VS Code; configurable; conflict warning in UI |
| PID type mismatch (i32 vs u32) | Phase 1: Project Setup | PlatformPid type alias used everywhere; no truncation; CI verifies |
| WSL boundary crossing | Phase 2: Terminal Context | WSL CWD resolves to Windows path; window key stable; paste works in WSL shell |
| Clipboard clobber during paste | Phase 3: Terminal Paste | User clipboard preserved after paste; or use non-clipboard paste method |
| Enterprise Credential Guard | Phase 4: Credential Storage | API key saved/loaded on enterprise machines; fallback if Credential Manager restricted |

## Sources

### Official Documentation
- [Tauri v2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/)
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri v2 Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- [Microsoft SendInput API (UIPI)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [Microsoft NtQueryInformationProcess](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess)
- [Microsoft High DPI Desktop Development](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
- [Microsoft Extended Window Styles (WS_EX_NOACTIVATE)](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)
- [Microsoft Console Screen Buffers](https://learn.microsoft.com/en-us/windows/console/console-screen-buffers)
- [Microsoft WSL Filesystem Interop](https://learn.microsoft.com/en-us/windows/wsl/filesystems)
- [Microsoft CreateToolhelp32Snapshot](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot)
- [Rust windows crate GetForegroundWindow](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/WindowsAndMessaging/fn.GetForegroundWindow.html)
- [Rust windows crate PROCESS_BASIC_INFORMATION](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Threading/struct.PROCESS_BASIC_INFORMATION.html)

### Tauri GitHub Issues
- [#10422: skipTaskbar not working on Windows](https://github.com/tauri-apps/tauri/issues/10422)
- [#11566: focus config not working on Windows](https://github.com/tauri-apps/tauri/issues/11566)
- [#7348: NSIS uninstall.exe not code signed](https://github.com/tauri-apps/tauri/issues/7348)
- [#11673: NSIS plugins not signed](https://github.com/tauri-apps/tauri/issues/11673)
- [#3610: Window size increases across monitors with different DPI](https://github.com/tauri-apps/tauri/issues/3610)
- [#11754: Windows EV certificate custom signing command issues](https://github.com/tauri-apps/tauri/issues/11754)

### Windows Terminal Issues
- [#5694: Identify WindowsTerminal process ID](https://github.com/microsoft/terminal/issues/5694)
- [#14902: Find window process doesn't locate active tab](https://github.com/microsoft/terminal/issues/14902)
- [#262: ConPTY overlapped I/O not supported](https://github.com/microsoft/terminal/issues/262)

### Crate Documentation
- [keyring crate (windows-native backend)](https://docs.rs/keyring)
- [windows-native-keyring-store](https://docs.rs/windows-native-keyring-store/latest/windows_native_keyring_store/)
- [window-vibrancy crate (Acrylic/Mica)](https://github.com/tauri-apps/window-vibrancy)
- [Rust windows crate CreateToolhelp32Snapshot](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/System/Diagnostics/ToolHelp/fn.CreateToolhelp32Snapshot.html)

### Community Resources
- [Tracking active process in Windows with Rust](https://hellocode.co/blog/post/tracking-active-process-windows-rust/)
- [Building a Process Tree with ToolHelp and PID Reuse](https://trainsec.net/library/windows-internals/building-a-process-tree/)
- [SmartScreen reputation post-March 2024 changes](https://learn.microsoft.com/en-us/answers/questions/5584097/how-to-bypass-windows-defender-smartscreen-even-af)
- [Claude Code Ctrl+V paste issue in legacy conhost](https://github.com/anthropics/claude-code/issues/12298)

---
*Pitfalls research for: Adding Windows support to CMD+K macOS Tauri overlay app*
*Researched: 2026-03-01*
