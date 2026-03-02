# Technology Stack: Windows Support (v0.2.1)

**Project:** CMD+K -- Windows Platform Port
**Researched:** 2026-03-01
**Confidence:** HIGH (overall)

> This document supersedes the v0.1.1 STACK.md. The validated macOS stack (Tauri v2, React 19, TypeScript, Vite, Zustand 5, xAI/Grok, NSPanel, libproc FFI, Accessibility API) is not re-researched. Focus is strictly on what the Windows port adds or changes.

---

## Guiding Principle

The existing Tauri v2 + React 19 + Zustand codebase already compiles for Windows for platform-agnostic layers (HTTP, AI streaming, state, safety checks, UI). This document covers ONLY what must change or be added for Windows-specific functionality. Everything not listed here stays as-is.

---

## Already Cross-Platform (No Changes Needed)

These crates/libraries work on Windows with zero modifications:

| Component | Crate/Library | Notes |
|-----------|---------------|-------|
| Framework | Tauri v2 | Cross-platform by design |
| Frontend | React 19 + Zustand 5 | Runs in WebView2 on Windows |
| Build tooling | Vite | Platform-agnostic bundler |
| AI streaming | tauri-plugin-http + eventsource-stream + futures-util | HTTP is cross-platform |
| Safety checks | regex + once_cell | Pure Rust, no platform deps |
| Serialization | serde + serde_json | Pure Rust |
| Async runtime | tokio | Cross-platform |
| Persistent config | tauri-plugin-store v2 | Cross-platform Tauri plugin |
| State management | AppState (Rust Mutex HashMap) | Pure Rust |
| Global shortcut | tauri-plugin-global-shortcut v2 | Supports Windows, macOS, Linux (verified via official docs) |
| System tray | Tauri tray-icon feature | Supports Windows (requires .ico format icon) |
| Window positioner | tauri-plugin-positioner v2 | Cross-platform |

**Confidence:** HIGH -- verified via official Tauri v2 plugin documentation and crate docs.

---

## Recommended Stack: Windows-Specific Additions

### 1. Overlay Window (Replaces tauri-nspanel + NSVisualEffectView)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri Window API | 2.x (built-in) | `always_on_top`, `skip_taskbar`, `decorations: false`, `transparent` | Replaces NSPanel level/behavior; native Tauri APIs work on Windows |
| window-vibrancy | 0.7.1 (already in Cargo.toml at 0.5 -- update) | `apply_mica()` on Win11, `apply_acrylic()` on Win10 | Already a dependency; has Windows-specific functions alongside macOS |
| windows crate (conditional) | 0.61+ | `WS_EX_NOACTIVATE` via raw HWND if needed for non-activating behavior | Only if Tauri's `always_on_top` proves insufficient |

**Why this stack:** tauri-nspanel is macOS-only (NSPanel is a Cocoa concept). On Windows, the overlay becomes a standard Tauri window configured with `always_on_top: true`, `decorations: false`, `skip_taskbar: true`, and `transparent: true`. The window-vibrancy crate already in use provides `apply_mica()` (Win11) and `apply_acrylic()` (Win10) alongside the macOS vibrancy it already provides.

**Key behavioral difference from NSPanel:** NSPanel's `nonactivating_panel` style lets the panel accept keyboard input without deactivating the underlying app. Windows has no exact equivalent. The overlay will steal focus from the terminal. After paste, `SetForegroundWindow` restores focus to the terminal. If needed, `WS_EX_NOACTIVATE` extended window style can be applied via raw HWND manipulation through the `windows` crate.

**Confidence:** HIGH for vibrancy (verified via window-vibrancy 0.7.1 docs). MEDIUM for non-activating behavior (may need raw Win32 API experimentation).

---

### 2. Foreground Window Detection (Replaces NSWorkspace.frontmostApplication)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| windows crate | 0.61+ | `GetForegroundWindow()` + `GetWindowThreadProcessId()` | Official Microsoft Rust bindings; stable Win32 API |

**Why this stack:** macOS uses `NSWorkspace.frontmostApplication.processIdentifier` via ObjC FFI. Windows equivalent is `GetForegroundWindow()` to get the HWND, then `GetWindowThreadProcessId()` to get the PID. Both are well-documented, stable Win32 APIs with official Rust bindings.

**Additional capability:** The HWND must also be captured (not just PID) because `SetForegroundWindow(hwnd)` is needed later to return focus to the terminal after paste. Store both HWND and PID in AppState.

**Confidence:** HIGH -- GetForegroundWindow is a foundational Win32 API.

---

### 3. Process Tree Walking (Replaces libproc FFI)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| sysinfo | 0.38+ | Process enumeration, parent PID, process name, exe path | Cross-platform, well-maintained (35M+ downloads), replaces libproc for non-CWD operations |
| windows crate | 0.61+ | `NtQueryInformationProcess` + PEB reading for CWD | sysinfo's `cwd()` returns empty on Windows -- must use direct API |

**Critical finding:** `sysinfo::Process::cwd()` is **always empty on Windows**. This is a known, documented limitation of the sysinfo crate on Windows. Getting the CWD of another process on Windows requires:

1. `OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, pid)`
2. `NtQueryInformationProcess(ProcessBasicInformation)` to get PEB address
3. `ReadProcessMemory` to read `RTL_USER_PROCESS_PARAMETERS.CurrentDirectory`

**What sysinfo handles well on Windows:**
- `Process::parent()` -- parent PID (replaces `ps -o ppid=` and `proc_listchildpids`)
- `Process::name()` -- process name (replaces `proc_pidpath`)
- `System::processes()` -- enumerate all processes (replaces `pgrep`)
- `Process::exe()` -- executable path

**Windows process tree patterns (different from macOS):**
- Windows Terminal: `WindowsTerminal.exe` -> `conhost.exe`/`OpenConsole.exe` -> `powershell.exe`/`cmd.exe`/`bash.exe`
- PowerShell standalone: `powershell.exe` or `pwsh.exe` directly
- Git Bash: `mintty.exe` -> `bash.exe`
- CMD: `cmd.exe` directly (or via `conhost.exe`)
- WSL: `wsl.exe` -> Linux processes (not visible to Win32 process APIs)

**Confidence:** HIGH for sysinfo (verified docs). MEDIUM for CWD via NtQueryInformationProcess (well-documented but complex unsafe code requiring PEB traversal and ReadProcessMemory).

---

### 4. Terminal Text Reading (Replaces macOS Accessibility API / AX Reader)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| uiautomation | 0.22+ | Read terminal text via Windows UI Automation tree | Rust wrapper for UIA; mirrors the AX reader pattern |
| windows crate | 0.61+ | `AttachConsole` + `ReadConsoleOutputCharacter` for legacy consoles | Fallback when UIA is unavailable |

**Two-tier approach (mirrors macOS AX + GPU terminal fallback):**

**Tier 1 -- UI Automation (equivalent of AX reader):**
Windows Terminal has UIA providers (ScreenInfoUiaProvider, UiaTextRange) that expose terminal text content to automation clients. The `uiautomation` crate (v0.22+) wraps the COM-based IUIAutomation API:
- `UIAutomation::new()` to initialize
- `TreeWalker` for element tree navigation
- `get_property_value(UIProperty::ValueValue)` to read text
- Works for: Windows Terminal, PowerShell windows, CMD windows

**Tier 2 -- Console Buffer API (fallback for terminals without UIA):**
- `AttachConsole(pid)` to attach to another process's console
- `GetConsoleScreenBufferInfo` to find visible region coordinates
- `ReadConsoleOutputCharacter` to read visible text
- Limitation: a process can only be attached to one console at a time
- Works for: CMD, legacy conhost-based terminals

**Terminal-specific UIA support:**
- Windows Terminal: Full UIA support (confirmed via Microsoft terminal repo PR #1691)
- PowerShell: UIA works when "Use UI Automation" accessibility option enabled
- CMD: UIA works (conhost has UIA providers)
- Git Bash (mintty): May not expose UIA -- Console Buffer API or no-text fallback
- WSL: Text visible via Windows Terminal's UIA tree (WSL runs inside WT)

**Confidence:** MEDIUM -- UIA for Windows Terminal confirmed via Microsoft's terminal repository. Git Bash/mintty UIA support is unverified; may need Console API fallback or graceful degradation (no visible text, CWD-only context).

---

### 5. Auto-Paste to Terminal (Replaces AppleScript + CGEventPost)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| arboard | 3.4+ | Cross-platform clipboard write (replaces `pbcopy`) | Maintained by 1Password; works on both Windows and macOS |
| enigo | 0.6+ | Send Ctrl+V keystroke to paste from clipboard | Cross-platform input simulation; replaces CGEventPost |
| windows crate | 0.61+ | `SetForegroundWindow` to activate terminal before paste | Replaces AppleScript `activate` command |

**Paste sequence on Windows:**
1. Write command to clipboard via `arboard`
2. `SetForegroundWindow(terminal_hwnd)` to activate the terminal
3. Small delay (100-150ms) for focus transfer
4. Send keystrokes via `enigo`:
   - PowerShell/CMD/Windows Terminal: Ctrl+V (paste)
   - Git Bash (mintty): Shift+Insert (mintty's paste shortcut)
5. Optionally restore overlay focus

**Why arboard over clipboard-win:** arboard is cross-platform (replaces both `pbcopy` on macOS and Windows clipboard APIs), reducing platform-specific dependencies. Can eventually replace the current `pbcopy` shell call on macOS.

**Why enigo over raw SendInput:** enigo abstracts Win32 SendInput into a clean cross-platform API. It handles virtual key codes, Unicode input, and modifier keys correctly. Also works on macOS, potentially unifying the paste path in future.

**SetForegroundWindow restriction:** Windows restricts which processes can call `SetForegroundWindow`. Our app qualifies because it has the foreground lock (overlay was focused when the user initiated paste). If restrictions cause issues, `AllowSetForegroundWindow` can be used.

**Confidence:** HIGH for clipboard + Ctrl+V paste. MEDIUM for SetForegroundWindow restrictions (should work given our foreground lock, but needs testing).

---

### 6. API Key Storage (keyring crate -- already cross-platform)

| Technology | Version | Purpose | Change Needed |
|------------|---------|---------|---------------|
| keyring | 3.6.3 | Secure credential storage | Add `windows-native` feature flag |

**Required Cargo.toml change:**

```toml
# Before (macOS only):
keyring = { version = "3", features = ["apple-native"] }

# After (macOS + Windows):
keyring = { version = "3", features = ["apple-native", "windows-native"] }
```

On Windows, keyring uses Windows Credential Manager. Each entry maps to a "generic credential" keyed by service + account -- identical API surface as macOS Keychain.

**No code changes needed.** The `Entry::new(SERVICE, ACCOUNT)`, `set_password()`, `get_password()`, and `delete_credential()` calls in `keychain.rs` are platform-agnostic.

**Confidence:** HIGH -- verified via keyring v3.6.3 docs. Windows is explicitly listed as a supported platform with `windows-native` feature.

---

### 7. App Identity on Windows (Replaces bundle_id)

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| sysinfo | 0.38+ | `Process::exe()` for executable path | Maps PID to exe name for app identity |
| windows crate | 0.61+ | `GetWindowTextW(hwnd)` for window title | Additional identity signal for terminal detection |

**Window key format change:**
- macOS: `"com.apple.Terminal:12345"` (bundle_id:shell_pid)
- Windows: `"WindowsTerminal.exe:12345"` (exe_name:shell_pid)

**Known terminal exe names on Windows:**

| Terminal | Executable | Notes |
|----------|-----------|-------|
| Windows Terminal | `WindowsTerminal.exe` | Most common modern terminal |
| PowerShell 5.x | `powershell.exe` | Legacy, ships with Windows |
| PowerShell 7+ | `pwsh.exe` | Cross-platform PowerShell |
| CMD | `cmd.exe` | Legacy command prompt |
| Git Bash | `mintty.exe` or `git-bash.exe` | Git for Windows package |
| WSL | `wsl.exe` | Shell runs in Linux subsystem |
| Hyper | `Hyper.exe` | Electron-based terminal |
| Alacritty | `alacritty.exe` | GPU-rendered |
| WezTerm | `wezterm-gui.exe` | GPU-rendered |

**Confidence:** HIGH -- standard Win32 APIs, well-documented exe names.

---

### 8. Build and Distribution

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| NSIS installer | Built into Tauri | `.exe` setup installer | Cross-compilable from macOS/Linux; modern UX |
| MSI installer | Built into Tauri (WiX v3) | `.msi` enterprise installer | Windows-only build; some enterprises require MSI |
| OV/EV code signing cert | N/A | Sign binaries to avoid SmartScreen | Required for professional distribution |
| WebView2 | Runtime | Windows WebView engine | Required by Tauri on Windows; bundled or bootstrapped |

**Recommended: NSIS over MSI** because:
- NSIS can be cross-compiled from macOS (MSI requires WiX which is Windows-only)
- NSIS produces a single `-setup.exe` (familiar to Windows users)
- NSIS supports multi-language in a single installer

**tauri.conf.json additions:**
```json
{
  "bundle": {
    "windows": {
      "nsis": {
        "installMode": "currentUser"
      },
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    },
    "icon": ["icons/icon.ico"]
  }
}
```

**System tray icon:** Windows requires `.ico` format. The existing `K.png` needs a `.ico` variant.

**Code signing (tauri.conf.json):**
```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.comodoca.com"
    }
  }
}
```

**Confidence:** HIGH -- official Tauri v2 distribution docs.

---

## New Dependencies Summary

### Add to Cargo.toml (Windows-only, platform-gated)

```toml
[target.'cfg(target_os = "windows")'.dependencies]
# Windows Win32 APIs (foreground window, process info, console buffer, window management)
windows = { version = "0.61", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_System_Threading",
    "Win32_Foundation",
    "Win32_System_Console",
    "Win32_System_Diagnostics_Debug",
    "Wdk_System_Threading"
] }

# UI Automation for reading terminal text (equivalent of macOS AX reader)
uiautomation = "0.22"

# Cross-platform input simulation (replaces CGEventPost for keystrokes)
enigo = { version = "0.6", features = [] }

# Cross-platform clipboard (replaces pbcopy for clipboard write)
arboard = "3.4"

# Process enumeration, parent PID, process name, exe path
sysinfo = "0.38"
```

### Modify Existing Cargo.toml

```toml
# Add Windows keyring backend alongside macOS:
keyring = { version = "3", features = ["apple-native", "windows-native"] }

# Update window-vibrancy for latest Windows support:
window-vibrancy = "0.7"
```

### Platform-Gate macOS-Only Dependencies

These compile only on macOS (already won't compile on Windows, but should be explicitly gated):

```toml
[target.'cfg(target_os = "macos")'.dependencies]
tauri-nspanel = { git = "https://github.com/ahkohd/tauri-nspanel", branch = "v2.1" }
accessibility-sys = "0.2"
core-foundation-sys = "0.8"
```

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Process CWD (Windows) | windows crate (NtQueryInformationProcess + PEB) | sysinfo `cwd()` | sysinfo returns empty CWD on Windows -- known limitation |
| Clipboard | arboard (cross-platform) | clipboard-win (Windows-only) | arboard works on both platforms; reduces total dependency count |
| Input simulation | enigo (cross-platform) | winput, raw SendInput | enigo abstracts platform differences; cleaner API |
| Terminal text reading | uiautomation crate | Raw IUIAutomation COM via windows crate | uiautomation wraps COM complexity into safe Rust API |
| Process enumeration | sysinfo (cross-platform) | tasklist crate, wmic subprocess | sysinfo is the standard Rust ecosystem choice (35M+ downloads) |
| Vibrancy | window-vibrancy 0.7 (existing dep, update) | Raw DWM API calls | Already a dependency; well-maintained by Tauri team |
| Installer | NSIS | MSI | NSIS cross-compiles from macOS; MSI requires Windows build env |
| Overlay non-activating | Tauri always_on_top + SetForegroundWindow restore | WS_EX_NOACTIVATE raw style | Start with simpler approach; only add raw Win32 if needed |

---

## Installation

### Rust Dependencies

```bash
# In src-tauri/ -- the platform-gated deps are handled by Cargo.toml cfg attributes
# No manual cargo add needed; just update Cargo.toml as shown above
```

### System Requirements for Development

```bash
# Windows development requirements:
# - Windows 10 v1809+ or Windows 11 (for Acrylic/Mica)
# - Visual Studio Build Tools 2019+ (C++ workload)
# - WebView2 runtime (ships with Windows 11, downloadable for Win10)
# - Rust toolchain for x86_64-pc-windows-msvc target

# Cross-compilation from macOS (NSIS installer only):
# - Install NSIS: brew install nsis
# - Rust cross-compilation target: rustup target add x86_64-pc-windows-msvc
# - Note: Full cross-compilation requires Windows linker; CI/CD recommended
```

### Icon Preparation

```bash
# Convert existing PNG tray icon to ICO for Windows:
# Use ImageMagick or an online converter
# Required sizes in ICO: 16x16, 32x32, 48x48, 256x256
convert icons/K.png -define icon:auto-resize=256,48,32,16 icons/icon.ico
```

---

## Sources

- [window-vibrancy v0.7.1 (GitHub)](https://github.com/tauri-apps/window-vibrancy) -- HIGH confidence
- [keyring v3.6.3 (docs.rs)](https://docs.rs/keyring) -- HIGH confidence
- [sysinfo v0.38.2 (docs.rs)](https://docs.rs/sysinfo/latest/sysinfo/struct.Process.html) -- HIGH confidence
- [uiautomation v0.22 (GitHub)](https://github.com/leexgone/uiautomation-rs) -- MEDIUM confidence
- [enigo v0.6.1 (docs.rs)](https://docs.rs/enigo/latest/enigo/) -- HIGH confidence
- [arboard v3.4 (GitHub)](https://github.com/1Password/arboard) -- HIGH confidence
- [windows crate (Microsoft docs)](https://microsoft.github.io/windows-docs-rs/) -- HIGH confidence
- [Tauri v2 Window Customization](https://v2.tauri.app/learn/window-customization/) -- HIGH confidence
- [Tauri v2 Windows Installer](https://v2.tauri.app/distribute/windows-installer/) -- HIGH confidence
- [Tauri v2 Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) -- HIGH confidence
- [Tauri v2 Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) -- HIGH confidence
- [Tauri v2 System Tray](https://v2.tauri.app/learn/system-tray/) -- HIGH confidence
- [Windows Terminal UIA Provider (PR #1691)](https://github.com/microsoft/terminal/pull/1691) -- MEDIUM confidence
- [NtQueryInformationProcess (MSDN)](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess) -- HIGH confidence
- [GetForegroundWindow (MSDN)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow) -- HIGH confidence
- [ReadConsoleOutputCharacter (MSDN)](https://learn.microsoft.com/en-us/windows/console/readconsoleoutputcharacter) -- HIGH confidence
- [SetForegroundWindow (MSDN)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow) -- HIGH confidence

---
*Stack research for: CMD+K v0.2.1 Windows platform support*
*Researched: 2026-03-01*
