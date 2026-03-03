# Feature Landscape: Windows Platform Support

**Domain:** Windows platform port of macOS overlay terminal command generator
**Milestone:** v0.2.1 Windows Support
**Researched:** 2026-03-01
**Confidence:** HIGH (official docs, Windows APIs, Tauri docs, codebase analysis verified)

---

## Scope

This research covers Windows-specific feature considerations for porting CMD+K from macOS to Windows. Existing features are already built on macOS. The question is: what does each feature require on Windows, what are the user expectations, and what is the complexity?

---

## 1. Overlay UX -- Frosted Glass Floating Panel

### macOS Current Implementation

NSPanel with `NSWindowStyleMaskNonActivatingPanel` via `tauri-nspanel`. Status window level floats above all windows. Frosted glass via NSVisualEffectView. Panel does not steal focus from terminal -- critical for maintaining the "which app was frontmost" context.

### Windows Equivalent

**Material System:**
- **Acrylic** (Windows 10 v1903+ and Windows 11): Semi-transparent frosted glass effect. Designed for transient, light-dismiss surfaces -- exactly what the overlay is. Use `window-vibrancy` crate's `apply_acrylic()`.
- **Mica** (Windows 11 only): Opaque material that samples the desktop wallpaper. Designed for long-lived app backgrounds, NOT transient overlays. Do not use Mica for the overlay -- it is semantically wrong and visually opaque.
- **Recommendation:** Use Acrylic. It is the direct equivalent of macOS vibrancy for floating panels. Fall back to solid semi-transparent background on Windows 10 builds older than 1903.

**Always-on-Top:**
- Tauri v2 provides `always_on_top: true` in window config. This maps to `HWND_TOPMOST` via `SetWindowPos`. Works on all Windows versions.
- Unlike macOS NSPanel which has "status" window level (above fullscreen), Windows `TOPMOST` does NOT float above fullscreen exclusive apps. This is acceptable -- Windows users do not expect overlay tools to work over fullscreen games or exclusive-mode apps.

**Non-Activating Behavior (Critical):**
- macOS uses `NSWindowStyleMaskNonActivatingPanel` so the overlay can appear without stealing focus. On Windows, the equivalent is `WS_EX_NOACTIVATE` extended window style.
- `tauri-nspanel` is macOS-only. There is NO Tauri plugin equivalent for Windows.
- **Solution:** After overlay hides, use `SetForegroundWindow(hwnd)` to restore focus to the previously active window. Capture the HWND before showing the overlay (equivalent to the current `get_frontmost_pid` pattern).
- **Alternative:** Set `WS_EX_NOACTIVATE` on the Tauri window via `windows` crate FFI after creation. This is the cleanest approach but requires careful handling -- buttons inside `WS_EX_NOACTIVATE` windows do not show pressed state by default, and keyboard events need explicit routing.
- **Recommendation:** Use standard Tauri `always_on_top` window + capture-and-restore focus pattern. The overlay WILL briefly steal focus, but hiding restores it. This matches how tools like PowerToys Run and Wox work on Windows. Users accept this behavior.

**Known Issue:** `window-vibrancy` Acrylic has performance issues when resizing/dragging on Windows 11 build 22621+. Since the overlay is fixed-size and non-draggable, this does not affect us.

### Feature Table

| Feature | Complexity | Windows API | Notes |
|---------|-----------|-------------|-------|
| Acrylic frosted glass | LOW | `window-vibrancy::apply_acrylic()` | Tauri v2 has built-in vibrancy support |
| Always-on-top | LOW | Tauri `always_on_top: true` | Standard Tauri config |
| Non-activating panel | HIGH | `WS_EX_NOACTIVATE` or focus-restore pattern | No Tauri plugin exists; needs custom Rust |
| Skip taskbar | LOW | `skip_taskbar: true` in Tauri config | Standard Tauri config |
| Position on current monitor | LOW | `current_monitor()` + logical position | Already cross-platform in `window.rs` |
| Transparent background | MEDIUM | Tauri `transparent: true` + CSS | Known v2 transparency bugs on Windows |

### Dependency on Existing Code

`window.rs` positioning logic (`position_overlay`) is cross-platform via Tauri APIs. The platform-specific part is:
- `tauri-nspanel` calls (`get_webview_panel`, `show_and_make_key`, `resign_key_window`) -- **must be replaced** with standard Tauri window show/hide + Windows focus management
- Panel ordering (`make_key_window`) -- replaced by `SetForegroundWindow` restore

---

## 2. Terminal Diversity -- CWD, Output, Pasting

### macOS Current Implementation

- **CWD detection:** `proc_pidinfo(PROC_PIDVNODEPATHINFO)` reads CWD from kernel. Fallback: `lsof`.
- **Shell detection:** `proc_pidpath` gets binary name. Process tree walk finds shell child of terminal.
- **Output reading:** Accessibility API (AX tree) reads visible text from Terminal.app and iTerm2.
- **Pasting:** AppleScript `write text` for iTerm2, `CGEventPost` synthetic keystrokes for others.

### Windows Terminal Landscape

| Terminal | Process Architecture | CWD Method | Output Access | Paste Method |
|----------|---------------------|------------|---------------|-------------|
| **Windows Terminal** | `WindowsTerminal.exe` -> `conhost/OpenConsole.exe` -> `powershell.exe`/`cmd.exe` | NtQueryInformationProcess on shell PID | UIA (supported), Console buffer API | SendInput Ctrl+V |
| **PowerShell (standalone)** | `conhost.exe` -> `powershell.exe` | NtQueryInformationProcess | UIA, Console buffer | SendInput Ctrl+V |
| **CMD** | `conhost.exe` -> `cmd.exe` | NtQueryInformationProcess | UIA, Console buffer | SendInput Ctrl+V (right-click paste on legacy mode) |
| **Git Bash** | `mintty.exe` -> `bash.exe` | NtQueryInformationProcess on bash PID | NO UIA, NO Console API (mintty is not a console app) | SendInput Shift+Insert or clipboard API |
| **WSL** | `WindowsTerminal.exe` -> `wsl.exe` -> Linux VM | Cannot read CWD (different PID namespace in VM) | Same as host terminal (WT provides UIA) | SendInput Ctrl+V in WT |
| **Hyper** | Electron -> `node.exe` -> `conpty` -> shell | NtQueryInformationProcess | UIA (limited, Electron Chromium) | SendInput Ctrl+V |
| **Alacritty** | `alacritty.exe` -> shell | NtQueryInformationProcess | NO UIA (GPU-rendered) | SendInput Ctrl+V |

### CWD Detection on Windows

**The core challenge:** There is NO public documented Windows API equivalent to macOS's `proc_pidinfo(PROC_PIDVNODEPATHINFO)`.

**Approach: NtQueryInformationProcess + PEB reading**
1. `OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, pid)` to get a process handle
2. `NtQueryInformationProcess(ProcessBasicInformation)` to get `PebBaseAddress`
3. `ReadProcessMemory` to read PEB -> `ProcessParameters` -> `CurrentDirectory.DosPath`
4. This is an undocumented (but stable and widely used) API path

The `windows` crate provides all necessary bindings: `NtQueryInformationProcess`, `PROCESS_BASIC_INFORMATION`, `PEB`, `RTL_USER_PROCESS_PARAMETERS`.

**Complexity: HIGH** -- this requires unsafe Rust, cross-process memory reading, and handling 32-bit vs 64-bit process mismatches. However, this approach is used by Process Explorer, Process Monitor, and many other tools.

**Fallback: PowerShell-specific `$PWD` escape sequence (OSC 9;9)**
Windows Terminal and modern PowerShell profiles emit `OSC 9;9;<CWD>` in the terminal escape stream. However, reading this requires either:
- Intercepting the terminal's output stream (not possible without ConPTY injection)
- Relying on users having shell integration configured (violates zero-setup constraint)

**Recommendation:** Use NtQueryInformationProcess for CWD. It works for all shell types without user configuration.

### Terminal Output Reading

**Windows UI Automation (UIA)** is the equivalent of macOS Accessibility API:
- Works for: Windows Terminal, PowerShell console, CMD console
- Does NOT work for: mintty (Git Bash), GPU terminals (Alacritty, kitty, WezTerm)
- NVDA (screen reader) confirms UIA works in Windows Terminal and PowerShell

**Console Screen Buffer API** (ReadConsoleOutput) is an alternative:
- Works only for classic console applications (cmd.exe, powershell.exe via conhost)
- Does NOT work for Windows Terminal (uses ConPTY, not classic console)
- Requires attaching to the target console

**Recommendation:** Use Windows UIA as primary (mirrors macOS AX reader pattern). Fall back to no-output for terminals that don't expose UIA text (mintty, GPU terminals). This exactly mirrors the macOS approach where GPU terminals return None.

### Pasting Into Terminal

**On macOS:** AppleScript `write text` for iTerm2, `CGEventPost(Cmd+V)` for others.
**On Windows:** `SendInput` API with Ctrl+V key events is the universal approach.

Steps:
1. Set clipboard text via `clipboard-win` crate or `SetClipboardData` Win32 API (replaces `pbcopy`)
2. Restore focus to terminal window via `SetForegroundWindow(hwnd)` (replaces AppleScript `activate`)
3. `SendInput` Ctrl+V keystrokes (replaces `CGEventPost`)
4. Small delay for keystrokes to process
5. Return focus handling to overlay

**Ctrl+U (clear line) equivalent:** PowerShell uses `Escape` key to clear the line. CMD uses `Escape`. Bash/zsh in Git Bash use `Ctrl+U`. The clear-line approach needs per-shell dispatch or can be omitted if we just overwrite clipboard and paste.

**Recommendation:** Clipboard write + SendInput Ctrl+V. No per-terminal dispatch needed on Windows (all terminals accept Ctrl+V). Clear line before paste: send `Home`, `Shift+End`, `Delete` sequence as a universal approach, or simply skip it (paste appends to existing text which is acceptable).

### Shell Process Tree Walking on Windows

| macOS Pattern | Windows Equivalent |
|---------------|--------------------|
| `proc_listchildpids` | `CreateToolhelp32Snapshot` + `Process32First/Next` to enumerate processes and filter by parent PID |
| `proc_pidpath` (binary name) | `QueryFullProcessImageNameW` or `GetModuleFileNameExW` |
| `pgrep -P <pid>` | Enumerate all processes, filter by `th32ParentProcessID` |
| `ps -eo pid=,ppid=` | `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` + iterate |

The `windows` crate provides all these APIs. The process tree walk logic in `process.rs` can be ported by replacing the macOS FFI calls with Windows equivalents. The algorithmic logic (find shell child, walk through multiplexers/wrappers, sub-shell filtering) stays identical.

**Windows-specific wrappers to walk through:** `conhost.exe`, `OpenConsole.exe`, `wsl.exe`, `wslhost.exe`

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| CWD detection via NtQueryInformationProcess | HIGH | Undocumented but stable; cross-process memory read |
| Shell type detection | MEDIUM | Process tree walk with Windows Toolhelp32 APIs |
| Terminal output via UIA | HIGH | Windows equivalent of AX reader; new UIA FFI module |
| Auto-paste via SendInput | MEDIUM | Clipboard + SendInput Ctrl+V; simpler than macOS multi-path |
| Process tree walking | MEDIUM | Same algorithm, different APIs |
| WSL CWD detection | NOT FEASIBLE | WSL2 runs in VM with separate PID namespace |
| Git Bash output reading | NOT FEASIBLE | mintty does not expose UIA or console API |

---

## 3. Hotkey Conventions

### macOS Current Implementation

Default: `Cmd+K`. Configurable via `tauri-plugin-global-shortcut`.

### Windows Considerations

**Ctrl+K conflicts:**
- VS Code: `Ctrl+K` is a chord prefix (Ctrl+K, Ctrl+C for comment, etc.)
- Slack: `Ctrl+K` opens the conversation switcher
- Most browsers: `Ctrl+K` focuses the address bar / search bar
- Notion: `Ctrl+K` opens link insertion

**Ctrl+K is a MUCH more contested shortcut on Windows than Cmd+K on macOS.** On macOS, Cmd+K only conflicts in VS Code (and the chord prefix still works because VS Code waits for the second key). On Windows, Ctrl+K conflicts with multiple daily-use applications.

**Recommendation:**
- Default hotkey: `Ctrl+Shift+K` (avoids all known conflicts)
- Alternative suggestion: `Ctrl+Space` (used by some launchers but less conflicted)
- Make it configurable on first-run onboarding (already supported -- `register_hotkey` command)
- The hotkey config code in `hotkey.rs` uses `tauri-plugin-global-shortcut` which is cross-platform -- no changes needed to registration logic

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| Default Ctrl+Shift+K | LOW | Change default string only |
| Hotkey configuration UI | NONE | Already exists in settings panel |
| Global shortcut registration | NONE | `tauri-plugin-global-shortcut` is cross-platform |
| Debounce for double-fire bug | NONE | Already implemented in `hotkey.rs` |

---

## 4. System Tray

### macOS Current Implementation

Menu bar icon via Tauri's tray API. Shows app name, hotkey, quit option.

### Windows Equivalent

**Windows Notification Area** (system tray) is the direct equivalent. Located in bottom-right corner of taskbar.

**User Expectations:**
- Single left-click: show/hide the app or show a quick menu
- Right-click: context menu with options (settings, hotkey info, quit)
- Minimize to tray on window close (common Windows pattern)
- Tooltip on hover showing app name

**Tauri v2 Tray API** is cross-platform. The existing `tray.rs` code should work on Windows without changes. Tauri handles the platform differences internally.

**Windows-specific consideration:** The notification area can overflow (icons hidden behind the `^` chevron). Users must manually pin the icon to the visible area. The installer or first-run should inform users about this.

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| System tray icon | NONE | Tauri tray API is cross-platform |
| Tray context menu | NONE | Already implemented in `tray.rs` |
| Minimize to tray | LOW | Add window close handler to hide instead of quit |
| Tooltip on hover | LOW | Tauri tray `with_tooltip()` |

---

## 5. Permissions Model

### macOS Current Implementation

Requires Accessibility permission for:
- CGEventPost (synthetic keystrokes for paste)
- AXUIElement reading (terminal output, window titles)
- Getting frontmost app info

Onboarding guides user through System Settings > Accessibility. Complex AX probe fallback for unsigned builds.

### Windows Permissions

**Windows does NOT have an equivalent to macOS Accessibility permission.** This is a major simplification.

- **SendInput** works without any special permission (any user-level process can send keystrokes)
- **UIA (UI Automation)** works without special permission for non-elevated processes
- **ReadProcessMemory** (for CWD reading) requires `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ` access rights. Works fine for user-owned processes. Fails for elevated (admin) processes.
- **GetForegroundWindow** works without any permission

**What DOES need attention:**
- Reading CWD from an elevated PowerShell (Admin) requires our process to also be elevated. Recommendation: do NOT run CMD+K as admin. Gracefully degrade when target process is elevated (return "unknown CWD").
- Windows Defender or antivirus may flag `ReadProcessMemory` calls as suspicious. This is a signing/reputation issue, not a permission issue.
- **Windows Firewall:** If using embedded WebView2, no firewall rules needed (it uses the system browser engine).

**Onboarding simplification:** No permission prompt needed on Windows. Remove the accessibility check step from onboarding. Replace with a simpler "welcome" screen that just handles API key setup and model selection.

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| Remove accessibility permission flow | LOW | Conditional compile: skip on Windows |
| Handle elevated process CWD failure | LOW | Graceful fallback, return None |
| Code signing for antivirus reputation | MEDIUM | EV certificate or SmartScreen approval |
| Onboarding flow adaptation | LOW | Remove permission step, keep API key setup |

---

## 6. Installer Experience

### macOS Current Implementation

DMG with drag-to-Applications. Tauri bundler handles this.

### Windows Options (via Tauri v2 Bundler)

**NSIS (.exe setup installer):**
- Per-user or per-machine installation
- Supports WebView2 bootstrapper embedding
- Cross-compilable from macOS/Linux
- Customizable install location
- Most familiar format for Windows users
- Supports silent install (`/S` flag)

**WiX (.msi installer):**
- Enterprise-friendly (Group Policy deployment)
- Can only be built on Windows (requires WiX Toolset v3)
- Requires VBSCRIPT Windows optional feature
- Better for managed IT environments

**Recommendation:** Use NSIS as primary installer format because:
1. Cross-compilable (can build Windows installer from macOS CI)
2. Per-user install by default (no admin needed)
3. Most familiar to consumer Windows users
4. Supports WebView2 bootstrapper embedding

**WebView2 Runtime:**
- Pre-installed on Windows 10 April 2018+ and Windows 11
- NSIS can embed the bootstrapper (+1.8MB) as fallback
- Use `downloadBootstrapper` mode (smallest, requires internet -- acceptable since the app requires internet for AI anyway)

**winget / Microsoft Store:** Optional future distribution. As of June 2025, individual developers can publish for free on Microsoft Store. winget manifest submission is straightforward.

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| NSIS installer config | LOW | Tauri bundler built-in |
| WebView2 bootstrapper | LOW | Tauri config: `downloadBootstrapper` |
| Per-user install (no admin) | LOW | NSIS `installerMode: "currentUser"` |
| Custom install location | NONE | NSIS default behavior |
| Silent install support | NONE | NSIS built-in |

---

## 7. Auto-Start on Login

### macOS Current Implementation

Not explicitly implemented in v0.1.x (users manually open the app). Launch Agent is the macOS standard.

### Windows Options

**Tauri provides `tauri-plugin-autostart`** which handles this cross-platform:
- On Windows: writes to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` registry key
- On macOS: uses Launch Agent or AppleScript
- Provides `enable()`, `disable()`, `isEnabled()` JavaScript API

**User expectation on Windows:** Background daemon apps (like Discord, Slack, Steam) auto-start by default with an option to disable in settings. Users expect a toggle in the settings panel.

**Task Manager visibility:** Windows Task Manager shows startup apps with their "Startup impact" rating. Apps using the registry `Run` key appear here. Users can disable autostart from Task Manager.

**Recommendation:** Use `tauri-plugin-autostart`. Add a "Start on login" toggle to the Settings panel. Default to OFF on first install (let user opt in). This avoids user annoyance and aligns with best practices.

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| Auto-start via registry | LOW | `tauri-plugin-autostart` handles it |
| Settings toggle | LOW | Add checkbox to PreferencesTab |
| Default off, user opt-in | LOW | Plugin default behavior |

---

## 8. API Key Storage

### macOS Current Implementation

`keyring` crate -> macOS Keychain via `security-framework`.

### Windows Equivalent

The `keyring` crate already supports Windows via **Windows Credential Manager** (`wincred`). The existing `keychain.rs` code uses `keyring::Entry` which is cross-platform -- it will use Windows Credential Manager automatically on Windows.

**No code changes needed in `keychain.rs`.** The `keyring` crate abstracts the platform difference.

The only change: rename user-facing strings from "Keychain" to "Credential Manager" or use a generic term like "secure storage" in UI text and error messages.

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| Credential storage | NONE | `keyring` crate is cross-platform |
| UI string updates | LOW | "Keychain" -> "secure storage" |

---

## 9. Window Identification for Per-Window History

### macOS Current Implementation

`bundle_id:shell_pid` as window key. Bundle ID from `NSRunningApplication`. Shell PID from process tree walk.

### Windows Equivalent

**Window identification:** Use `GetForegroundWindow()` -> `GetWindowThreadProcessId()` to get the PID of the foreground process. Use `QueryFullProcessImageNameW` to get the executable path (replaces bundle ID).

**Window key format:** `exe_name:shell_pid` (e.g., `WindowsTerminal.exe:12345`)

**Process tree for shell PID:** Same algorithm as macOS but using Windows Toolhelp32 APIs:
- `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS)` -> iterate -> build parent map
- Find shell child of terminal process (walk through `conhost.exe`, `OpenConsole.exe`)
- Known shells list: same as macOS + add `pwsh.exe`, `powershell.exe`, `cmd.exe`

### Feature Table

| Feature | Complexity | Notes |
|---------|-----------|-------|
| Foreground window PID | LOW | `GetForegroundWindow` + `GetWindowThreadProcessId` |
| Exe name (replaces bundle_id) | LOW | `QueryFullProcessImageNameW` |
| Window key computation | MEDIUM | Port process tree walk to Windows APIs |
| Per-window history/context | NONE | Rust state logic is platform-independent |

---

## Table Stakes vs Differentiators vs Anti-Features

### Table Stakes (Must Have for Windows Launch)

Features Windows users expect. Missing any of these makes the product feel broken or incomplete.

| Feature | Why Expected | Complexity | macOS Code Reuse |
|---------|--------------|------------|------------------|
| Acrylic frosted glass overlay | Windows 11 Fluent Design language; users expect modern material | LOW | None (new, but simple API) |
| Always-on-top floating panel | Core product behavior | LOW | Position logic reusable |
| Ctrl+Shift+K global hotkey (configurable) | Power users expect system-wide shortcuts | LOW | Hotkey registration reusable |
| Terminal CWD detection (PS, CMD, WT) | "Know my context" is core value prop | HIGH | Algorithm reusable, APIs must be ported |
| Auto-paste into active terminal | Core workflow: generate -> paste | MEDIUM | Simpler than macOS (uniform SendInput) |
| Windows Credential Manager for API key | Secure storage expected | NONE | `keyring` crate handles it |
| System tray icon with menu | Background daemon UX convention | NONE | Tauri tray API is cross-platform |
| NSIS installer with WebView2 | Users expect standard Windows installer | LOW | Tauri bundler built-in |
| Per-window command history | Already built for macOS v0.1.1 | MEDIUM | State logic reusable, PID capture ported |
| Settings panel | Users need to configure API key, hotkey | NONE | React UI is cross-platform |
| Safety warnings for destructive commands | Core safety feature | NONE | Regex logic is platform-independent |
| AI streaming with xAI/Grok | Core feature | NONE | Entirely platform-independent |

### Differentiators (Competitive Advantage on Windows)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Zero-setup terminal context (no PowerShell profile edits) | Competitors require `$PROFILE` modifications or shell plugins. CMD+K reads CWD via process inspection. | HIGH | NtQueryInformationProcess approach is unique; most tools rely on OSC 9;9 which requires shell config |
| Works across all terminal types | Not just Windows Terminal -- also PowerShell standalone, CMD, Git Bash (partial), Hyper | MEDIUM | Process tree walk handles diverse terminal architectures |
| Per-terminal-window AI context | Most Windows AI tools are global. CMD+K gives each terminal tab its own conversation thread. | MEDIUM | Port window key computation |
| No admin required | Per-user install, no UAC prompt, no elevated permissions | LOW | NSIS per-user mode |
| Instant overlay (no Electron startup) | Tauri is ~20MB, starts in <1s. Electron alternatives are 200MB+. | NONE | Inherent Tauri advantage |

### Anti-Features (Do NOT Build)

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| WSL CWD detection | WSL2 runs in a VM with separate PID namespace. Cannot read Linux process CWD from Windows side without WSL-specific tooling. Scope explosion. | Detect WSL shell, return "WSL" as context. CWD shows as unknown. User can still type queries. |
| Run as admin by default | Would allow reading elevated process CWD but creates UAC prompts, security concerns, and bad UX | Run as normal user. Gracefully degrade when target is elevated. |
| ConPTY injection for output reading | Injecting into another process's ConPTY stream to read terminal output. Invasive, fragile, antivirus will flag it. | Use UIA for output reading. Accept that some terminals (mintty, GPU) won't have output context. |
| Custom DWM composition for overlay | Custom DirectComposition for pixel-perfect overlay matching. Massive complexity. | Use Tauri's built-in window + `window-vibrancy` Acrylic. Good enough. |
| MSI installer (primary) | Only buildable on Windows. Consumer users expect EXE installers, not MSI. MSI is for enterprise IT. | NSIS as primary. Add MSI as optional for enterprise users later. |
| MSIX containerized package | MSIX restricts filesystem access, may conflict with process inspection features. Modern but too constrained. | NSIS gives full Win32 access needed for process inspection. |
| PowerShell profile auto-modification | Adding OSC 9;9 to user's `$PROFILE` to get CWD. Violates zero-setup constraint. Users hate tools that modify their shell config. | NtQueryInformationProcess for CWD. Zero shell modifications. |

---

## Feature Dependencies

```
Overlay UX (Acrylic + always-on-top + focus management)
    requires -> Window creation with WS_EX_NOACTIVATE or focus-restore pattern
    requires -> Tauri transparent window working on Windows (known v2 issues)
    requires -> window-vibrancy crate Acrylic support

Terminal Context (CWD + shell type + output)
    requires -> NtQueryInformationProcess + ReadProcessMemory FFI module
    requires -> Windows Toolhelp32 process tree walking
    requires -> Windows UIA reader module (parallel to ax_reader.rs)
    requires -> Known terminals list updated for Windows

Auto-Paste
    requires -> Clipboard write (clipboard-win or Win32 API)
    requires -> SendInput for Ctrl+V keystrokes
    requires -> Focus restore to terminal HWND
    depends on -> Terminal Context (need HWND of target terminal)

Window Identification
    requires -> GetForegroundWindow + GetWindowThreadProcessId
    requires -> QueryFullProcessImageNameW (replaces bundle_id)
    requires -> Process tree walk (Windows Toolhelp32)
    enables -> Per-window history
    enables -> Per-window AI context

Installer
    requires -> Tauri NSIS bundle config
    requires -> WebView2 bootstrapper embedding
    independent of all runtime features

Auto-Start
    requires -> tauri-plugin-autostart
    requires -> Settings UI toggle
    independent of other features

Permissions/Onboarding
    requires -> Remove macOS accessibility flow (conditional compile)
    requires -> Simplified Windows onboarding (API key only)
    simplifies -> Overall first-run experience
```

---

## Platform Abstraction Requirements

These macOS-specific modules need Windows implementations behind `#[cfg(target_os)]`:

| Module | macOS Implementation | Windows Implementation Needed |
|--------|---------------------|-------------------------------|
| `terminal/process.rs` | libproc FFI (proc_pidinfo, proc_pidpath, proc_listchildpids) | Toolhelp32 + NtQueryInformationProcess |
| `terminal/ax_reader.rs` | Accessibility API (AXUIElement) | Windows UI Automation (IUIAutomation) |
| `terminal/detect.rs` | NSRunningApplication bundle IDs | QueryFullProcessImageNameW + exe name mapping |
| `commands/paste.rs` | CGEventPost + AppleScript | SendInput + clipboard-win |
| `commands/hotkey.rs` | NSWorkspace frontmostApplication | GetForegroundWindow + GetWindowThreadProcessId |
| `commands/permissions.rs` | AXIsProcessTrusted, AX probe | No-op (Windows needs no special permission) |
| `commands/window.rs` | tauri-nspanel (NSPanel) | Tauri standard window + focus management |

### Cross-Platform Modules (No Changes Needed)

| Module | Why Cross-Platform |
|--------|-------------------|
| `commands/ai.rs` | HTTP requests to xAI API |
| `commands/xai.rs` | Streaming response parsing |
| `commands/safety.rs` | Regex-based destructive command detection |
| `commands/history.rs` | In-memory HashMap operations |
| `commands/keychain.rs` | `keyring` crate abstracts platform |
| `commands/tray.rs` | Tauri tray API is cross-platform |
| `state.rs` | Pure Rust state management |
| `terminal/filter.rs` | Text processing logic |
| `terminal/browser.rs` | Browser detection (needs Windows browser list) |
| All frontend code | React/TypeScript in WebView |

---

## MVP Recommendation for Windows v0.2.1

### Phase 1: Platform Abstraction Foundation
1. Overlay window without NSPanel (Tauri standard window + Acrylic + always-on-top)
2. Foreground window capture (GetForegroundWindow replaces get_frontmost_pid)
3. Focus restore after overlay hide
4. Hotkey with Ctrl+Shift+K default

### Phase 2: Terminal Context
1. Process tree walking via Toolhelp32
2. CWD via NtQueryInformationProcess
3. Shell type detection (add powershell.exe, pwsh.exe, cmd.exe to known shells)
4. Known terminals list for Windows

### Phase 3: Paste and Interaction
1. Clipboard write (replace pbcopy)
2. SendInput Ctrl+V (replace CGEventPost)
3. Focus management for paste-and-return flow

### Phase 4: Terminal Output (Optional for MVP)
1. Windows UIA reader for terminal text
2. Equivalent of ax_reader.rs for Windows Terminal and PowerShell

### Phase 5: Polish and Distribution
1. NSIS installer configuration
2. Auto-start plugin integration
3. Onboarding flow adaptation (remove accessibility, add Windows-specific help)
4. Settings panel string updates

**Defer:** WSL CWD detection, ConPTY output injection, MSI installer, MSIX package, Git Bash output reading.

---

## Sources

- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) -- overlay window config
- [window-vibrancy crate](https://github.com/tauri-apps/window-vibrancy) -- Acrylic/Mica support
- [Mica Material - Microsoft Learn](https://learn.microsoft.com/en-us/windows/apps/design/style/mica) -- Mica vs Acrylic guidance
- [Extended Window Styles - Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles) -- WS_EX_NOACTIVATE
- [GetForegroundWindow - Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow) -- foreground window API
- [NtQueryInformationProcess - Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess) -- process CWD reading
- [Shell Integration in Windows Terminal](https://learn.microsoft.com/en-us/windows/terminal/tutorials/shell-integration) -- OSC 9;9 escape sequences
- [SendInput - windows-docs-rs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/UI/Input/KeyboardAndMouse/fn.SendInput.html) -- keyboard simulation
- [Tauri v2 Windows Installer](https://v2.tauri.app/distribute/windows-installer/) -- NSIS and WiX configuration
- [tauri-plugin-autostart](https://v2.tauri.app/plugin/autostart/) -- cross-platform auto-start
- [keyring crate](https://docs.rs/keyring) -- Windows Credential Manager integration
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/) -- cross-platform tray API
- [SetForegroundWindow - Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) -- focus management
- [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel) -- macOS-only, no Windows equivalent
- [Tracking Active Process in Windows with Rust](https://hellocode.co/blog/post/tracking-active-process-windows-rust/) -- GetForegroundWindow Rust pattern
- [ConPTY Introduction](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/) -- terminal architecture
- [Windows Notification Area - Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/shell/notification-area) -- system tray guidelines
- Codebase analysis: `window.rs`, `paste.rs`, `detect.rs`, `process.rs`, `ax_reader.rs`, `keychain.rs`, `permissions.rs`, `hotkey.rs`

---

*Feature research for: Windows platform support v0.2.1*
*Researched: 2026-03-01*
