<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="K-white.png">
  <source media="(prefers-color-scheme: light)" srcset="K.png">
  <img src="K.png" alt="CMD+K Logo" width="200">
</picture>

# CMD+K

*AI-powered terminal commands, one keystroke away.*

Press a hotkey from anywhere on your desktop. Type what you want. Get a working command. That's it.

![macOS](https://img.shields.io/badge/macOS-555555?style=for-the-badge&logo=apple&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-555555?style=for-the-badge&logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-555555?style=for-the-badge&logo=linux&logoColor=white)
[![Website](https://img.shields.io/badge/Website-cmd--k.site-555555?style=for-the-badge&logo=safari&logoColor=white)](https://www.cmd-k.site)

[![Downloads](https://img.shields.io/github/downloads/LakshmanTurlapati/Cmd-K/total?style=for-the-badge&logo=github&logoColor=white&color=555555)](https://github.com/LakshmanTurlapati/Cmd-K/releases)
[![Stars](https://img.shields.io/github/stars/LakshmanTurlapati/Cmd-K?style=for-the-badge&logo=github&logoColor=white&color=555555)](https://github.com/LakshmanTurlapati/Cmd-K/stargazers)
[![Forks](https://img.shields.io/github/forks/LakshmanTurlapati/Cmd-K?style=for-the-badge&logo=github&logoColor=white&color=555555)](https://github.com/LakshmanTurlapati/Cmd-K/network/members)
[![Issues](https://img.shields.io/github/issues/LakshmanTurlapati/Cmd-K?style=for-the-badge&logo=github&logoColor=white&color=555555)](https://github.com/LakshmanTurlapati/Cmd-K/issues)
[![Last Commit](https://img.shields.io/github/last-commit/LakshmanTurlapati/Cmd-K?style=for-the-badge&logo=github&logoColor=white&color=555555)](https://github.com/LakshmanTurlapati/Cmd-K/commits/main)

</div>

---

<div align="center">

[Website](https://www.cmd-k.site) · [Download](#download) · [Features](#features) · [How It Works](#how-it-works) · [Architecture](#architecture) · [Tech Stack](#tech-stack) · [Quick Start](#quick-start) · [Project Structure](#project-structure) · [Contributing](#contributing)

</div>

---

## The Problem

You know *what* you want your terminal to do, but not the exact flags, pipes, or syntax to make it happen. So you context-switch to Stack Overflow, scan three half-relevant answers, copy a command you don't fully trust, and paste it back. Your flow is gone.

Shell plugins need dotfile wiring. Copilot needs a subscription and an IDE. ChatGPT needs a browser tab, a prompt, and manual copy-paste. None of them know what directory you're in or what you just ran.

## The Solution

CMD+K is a **native desktop overlay**. Press a hotkey from any application, type plain English, and get a working terminal command -- streamed in real-time. It reads your terminal context (current directory, recent output, shell type) via platform accessibility APIs and process introspection. No plugins. No rc files. No IDE. No browser tab.

And when the generated command would `rm -rf` your home directory or `DROP` your production database, CMD+K tells you before you run it.

---

## Features

| Feature | Description |
|---|---|
| **System-Wide Overlay** | Floating overlay triggered by a global hotkey from any application. No dock icon, no window clutter -- just a translucent command bar that appears and disappears. |
| **Natural Language Commands** | Describe what you want in plain English. AI models from OpenAI, Anthropic, Google Gemini, xAI, or OpenRouter stream a working terminal command back token-by-token. |
| **Zero-Config Context Detection** | Reads your current working directory, shell type, and recent terminal output via platform accessibility APIs and process introspection. No shell plugins, no rc file edits. |
| **Multi-Terminal Support** | Detects and reads context from Terminal.app, iTerm2, Alacritty, kitty, WezTerm, GNOME Terminal, Konsole, and Windows Terminal. Recognizes shells inside VS Code and Cursor. |
| **Browser DevTools Support** | Detects open DevTools consoles in Chrome, Safari, Firefox, Arc, Edge, and Brave. Switches to a conversational assistant mode for web debugging. |
| **Destructive Command Safety** | 50+ regex patterns flag dangerous commands (rm -rf, DROP TABLE, git push --force, etc.) with a red badge and an AI-generated plain-English explanation of the risk. |
| **Secure API Key Storage** | Your API key lives in your system's secure credential store (macOS Keychain, Windows Credential Manager, Linux libsecret). It never touches a plaintext config file and never leaves the Rust backend. |
| **Smart Onboarding** | A 4-step wizard handles Accessibility permissions, API key validation, and model selection on first launch. No manual setup required. |
| **Configurable Hotkey** | Change the trigger shortcut to any key combination. Preset suggestions and a custom key recorder are built in. |
| **Lightweight Native App** | Built with Tauri 2 and Rust. Runs as a menu bar utility with minimal memory footprint. No Electron. No bundled Chromium. |

---

## How It Works

1. **Press the hotkey** -- Cmd+K on macOS, Ctrl+K on Windows/Linux (default) from any application. The overlay appears centered on your active screen.
2. **Type your intent** -- Describe what you want in natural language. "Find all PDFs modified this week" or "Kill whatever is hogging port 3000."
3. **Context is gathered** -- CMD+K captures the foreground app's PID, resolves the active terminal's working directory, shell type, and recent visible output via platform accessibility APIs.
4. **AI generates the command** -- Your query plus context is sent to your selected AI provider. The response streams token-by-token into the results area with shell syntax highlighting.
5. **Safety check runs** -- Once generation completes, the command is scanned against 50+ destructive patterns. Matches trigger a red badge with an AI-powered risk explanation.
6. **Copy or auto-paste** -- Click the result to copy to clipboard, or let CMD+K paste directly into your terminal.
7. **Dismiss** -- Press Escape. The overlay vanishes. Focus returns to your previous application.

---

## Architecture

```mermaid
graph TB
    subgraph User["User Interaction"]
        HK[Global Hotkey]
        INPUT[Natural Language Input]
        RESULT[Command Output]
    end

    subgraph Tauri["Tauri 2 -- Rust Backend"]
        OVR_WIN[Platform Overlay]
        CTX[Context Detection]
        AI[AI Streaming Engine]
        SAFE[Safety Layer]
        CRED[Credential Storage]
        PASTE[Paste Dispatch]
        TRAY[System Tray]
    end

    subgraph React["React 19 Frontend"]
        STORE[Zustand Store]
        OVR[Overlay UI]
        ONB[Onboarding Wizard]
        SET[Settings Panel]
    end

    subgraph macOS["macOS Platform"]
        NSP[NSPanel + Vibrancy]
        AX[Accessibility API]
        PROC[libproc / Process Table]
        KEYS[macOS Keychain]
        AS[AppleScript Paste]
    end

    subgraph Windows["Windows Platform"]
        W32[Win32 WS_EX_TOOLWINDOW + Acrylic]
        UIA[UI Automation Reader]
        WPROC[Process Tree Walking]
        WCRED[Windows Credential Manager]
        SI[SendInput / arboard Paste]
    end

    subgraph Linux["Linux Platform"]
        X11[X11 Overlay + CSS Glass]
        ATSPI[AT-SPI2 / kitty / WezTerm Reader]
        LPROC[/proc Filesystem]
        LSECRET[libsecret Keyring]
        XDO[xdotool Paste]
    end

    subgraph External["AI Providers"]
        PROVIDERS["OpenAI · Anthropic · Gemini · xAI · OpenRouter"]
    end

    HK --> OVR_WIN
    OVR_WIN --> OVR
    OVR --> INPUT
    INPUT --> STORE
    STORE --> AI
    AI --> PROVIDERS
    AI --> RESULT
    RESULT --> SAFE
    SAFE --> PASTE

    CTX --> AX
    CTX --> UIA
    CTX --> ATSPI
    CTX --> PROC
    CTX --> WPROC
    CTX --> LPROC
    CRED --> KEYS
    CRED --> WCRED
    CRED --> LSECRET
    AI --> CTX
    AI --> CRED
    PASTE --> AS
    PASTE --> SI
    PASTE --> XDO

    TRAY --> SET
    TRAY --> ONB
```

---

## Tech Stack

**Backend (Rust)**

![Rust](https://img.shields.io/badge/Rust-555555?style=flat-square&logo=rust&logoColor=white)
![Tauri 2](https://img.shields.io/badge/Tauri_2-24C8D8?style=flat-square&logo=tauri&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-555555?style=flat-square&logo=rust&logoColor=white)
![Serde](https://img.shields.io/badge/Serde-D4A017?style=flat-square&logo=rust&logoColor=white)

**Frontend**

![React 19](https://img.shields.io/badge/React_19-61DAFB?style=flat-square&logo=react&logoColor=black)
![TypeScript](https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white)
![Tailwind CSS 4](https://img.shields.io/badge/Tailwind_CSS_4-06B6D4?style=flat-square&logo=tailwindcss&logoColor=white)
![Zustand](https://img.shields.io/badge/Zustand-555555?style=flat-square&logo=react&logoColor=white)
![Radix UI](https://img.shields.io/badge/Radix_UI-555555?style=flat-square&logo=radixui&logoColor=white)

**AI**

![OpenAI](https://img.shields.io/badge/OpenAI-555555?style=flat-square&logo=openai&logoColor=white)
![Anthropic](https://img.shields.io/badge/Anthropic-555555?style=flat-square&logo=anthropic&logoColor=white)
![Google Gemini](https://img.shields.io/badge/Google_Gemini-555555?style=flat-square&logo=googlegemini&logoColor=white)
![xAI](https://img.shields.io/badge/xAI_Grok-555555?style=flat-square&logo=x&logoColor=white)
![OpenRouter](https://img.shields.io/badge/OpenRouter-555555?style=flat-square&logo=openrouter&logoColor=white)
![SSE Streaming](https://img.shields.io/badge/SSE_Streaming-FF6600?style=flat-square&logo=lightning&logoColor=white)

---

## Download

Grab the latest release from the [Releases page](https://github.com/LakshmanTurlapati/Cmd-K/releases).

| Platform | Format | Notes |
|---|---|---|
| **macOS** (Universal) | `.dmg` | Signed + notarized. Gatekeeper-ready. |
| **Windows** (x64) | `.exe` (NSIS) | Auto-installs per-user. |
| **Linux** (x86_64) | `.AppImage` | `chmod +x` and run. |
| **Linux** (aarch64) | `.AppImage` | ARM64 build for Raspberry Pi, etc. |

All builds are compiled and signed automatically via GitHub Actions -- nothing touches a local machine. Auto-updates are built in on all platforms. You can audit every step in the [workflow source](.github/workflows/).

---

## Quick Start

### Prerequisites

- macOS 13+ (Ventura or later), Windows 10+, or Linux (Ubuntu 22.04+, X11)
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 18+
- [pnpm](https://pnpm.io/)
- An API key from one of the supported providers: [OpenAI](https://platform.openai.com/api-keys), [Anthropic](https://console.anthropic.com/), [Google Gemini](https://aistudio.google.com/apikey), [xAI](https://console.x.ai/), or [OpenRouter](https://openrouter.ai/keys)

### Install & Run

```bash
# Clone the repository
git clone https://github.com/LakshmanTurlapati/Cmd-K.git
cd Cmd-K

# Install frontend dependencies
pnpm install

# Start in development mode
pnpm tauri dev
```

### First Run

On first launch, the onboarding wizard will walk you through:

1. **Accessibility Permission** -- CMD+K needs this to read terminal context. The wizard links you to System Settings.
2. **API Key** -- Enter your API key. It's validated against the provider's API and stored securely in your system credential store.
3. **Model Selection** -- Choose your AI provider and preferred model.

### Build for Production

```bash
pnpm tauri build
```

The installer will be in `src-tauri/target/release/bundle/` (`.dmg` on macOS, `.exe` on Windows, `.AppImage` on Linux).

---

## Project Structure

```
cmd-k/
├── src/                          # React 19 frontend
│   ├── main.tsx                  # App entrypoint
│   ├── App.tsx                   # Initialization, onboarding gate
│   ├── store/
│   │   └── index.ts              # Zustand store (modes, streaming, settings)
│   ├── components/
│   │   ├── Overlay.tsx           # Main overlay container, dismiss handlers
│   │   ├── CommandInput.tsx      # Auto-growing textarea, /settings trigger
│   │   ├── ResultsArea.tsx       # Streaming display, syntax highlighting, copy
│   │   ├── DestructiveBadge.tsx  # Red warning badge with AI tooltip
│   │   ├── HotkeyConfig.tsx      # Hotkey rebinding dialog (platform-aware)
│   │   ├── HotkeyRecorder.tsx    # Custom key capture
│   │   ├── Onboarding/           # 4-step setup wizard
│   │   └── Settings/             # Tabbed preferences panel
│   ├── hooks/
│   │   ├── useKeyboard.ts        # Escape dismiss, Ctrl+C cancel
│   │   └── useWindowAutoSize.ts  # Dynamic Tauri window sizing
│   ├── utils/
│   │   └── platform.ts           # Platform detection utilities
│   └── lib/
│       └── utils.ts              # Tailwind merge utilities
│
├── src-tauri/                    # Tauri 2 Rust backend
│   ├── Cargo.toml                # Rust deps (cfg-gated per platform)
│   ├── src/
│   │   ├── main.rs               # Binary entrypoint
│   │   ├── lib.rs                # Tauri app init (platform-branched)
│   │   ├── state.rs              # AppState (hotkey, visibility, PID, HWND)
│   │   ├── commands/
│   │   │   ├── ai.rs             # SSE streaming, platform-aware prompts
│   │   │   ├── xai.rs            # API validation, model fetching
│   │   │   ├── safety.rs         # 50+ destructive patterns, AI explanation
│   │   │   ├── terminal.rs       # Context detection IPC bridge
│   │   │   ├── paste.rs          # Platform paste (AppleScript / SendInput / xdotool)
│   │   │   ├── keychain.rs       # Credential storage (Keychain / WCM / libsecret)
│   │   │   ├── hotkey.rs         # Global shortcut + focus capture
│   │   │   ├── window.rs         # Overlay positioning, multi-monitor
│   │   │   ├── tray.rs           # System tray icon and menu
│   │   │   └── permissions.rs    # Accessibility permission check
│   │   └── terminal/
│   │       ├── mod.rs            # Terminal detection orchestrator
│   │       ├── detect.rs         # macOS: Bundle ID, app name cleaning
│   │       ├── detect_windows.rs # Windows: foreground window detection
│   │       ├── detect_linux.rs   # Linux: X11 active window detection
│   │       ├── process.rs        # Process tree walking (libproc / Win32 / /proc)
│   │       ├── ax_reader.rs      # macOS: Accessibility API text extraction
│   │       ├── uia_reader.rs     # Windows: UI Automation text extraction
│   │       ├── linux_reader.rs   # Linux: AT-SPI2 / kitty / WezTerm text reading
│   │       ├── context.rs        # Cross-platform ANSI strip + smart truncation
│   │       ├── browser.rs        # Browser DevTools console detection
│   │       └── filter.rs         # Sensitive data scrubbing
│   └── icons/                    # App icons
│
├── scripts/
│   ├── build-dmg.sh              # macOS: signed + notarized DMG pipeline
│   └── build-windows.sh          # Windows: NSIS installer build
│
├── .github/workflows/
│   └── release.yml               # CI: macOS + Windows + Linux builds → GitHub Release
│
├── showcase/                     # Project website (cmd-k.site)
├── K.png                         # Tray icon
├── LICENSE                       # MIT
└── package.json                  # Frontend deps
```

---

## Supported Terminals

### macOS

| Terminal | Context Detection | Output Reading | Notes |
|---|---|---|---|
| **Terminal.app** | Full (CWD, shell, output) | Accessibility API | Full auto-paste via AppleScript |
| **iTerm2** | Full (CWD, shell, output) | Accessibility API | Full auto-paste via AppleScript |
| **Alacritty** | Partial (CWD, shell) | Not available | GPU-rendered; no AX text exposure |
| **kitty** | Partial (CWD, shell) | Not available | GPU-rendered; no AX text exposure |
| **WezTerm** | Partial (CWD, shell) | Not available | GPU-rendered; no AX text exposure |
| **VS Code / Cursor** | Shell detected inside editor | Via editor AX tree | Integrated terminal recognized as shell |

### Windows

| Terminal | Context Detection | Output Reading | Notes |
|---|---|---|---|
| **Windows Terminal** | Full (CWD, shell, output) | UI Automation | Auto-paste via SendInput |
| **PowerShell** | Full (CWD, shell, output) | UI Automation | Auto-paste via SendInput |
| **Command Prompt** | Full (CWD, shell, output) | UI Automation | Auto-paste via SendInput |
| **VS Code / Cursor** | Shell detected inside editor | Via UIA tree | Integrated terminal recognized as shell |

### Linux

| Terminal | Context Detection | Output Reading | Notes |
|---|---|---|---|
| **GNOME Terminal** | Full (CWD, shell, output) | AT-SPI2 D-Bus | Auto-paste via xdotool (X11) |
| **Tilix** | Full (CWD, shell, output) | AT-SPI2 D-Bus | VTE-based; auto-paste via xdotool |
| **Terminator** | Full (CWD, shell, output) | AT-SPI2 D-Bus | VTE-based; auto-paste via xdotool |
| **Konsole** | Full (CWD, shell, output) | AT-SPI2 D-Bus | Qt-based; auto-paste via xdotool |
| **kitty** | Full (CWD, shell, output) | Remote control API | `kitty @ get-text`; auto-paste via xdotool |
| **WezTerm** | Full (CWD, shell, output) | CLI API | `wezterm cli get-text`; auto-paste via xdotool |
| **Alacritty** | Partial (CWD, shell) | Not available | GPU-rendered; no text API |
| **VS Code / Cursor** | Shell detected inside editor | AT-SPI2 | Integrated terminal recognized as shell |

### Cross-Platform

| Target | Context Detection | Output Reading | Notes |
|---|---|---|---|
| **Browser DevTools** | Console detected | Not applicable | Chrome, Safari, Firefox, Arc, Edge, Brave |

---

## Configuration

Type `/settings` into the overlay to configure these options.

| Setting | Location | Description |
|---|---|---|
| **API Key** | System credential store (macOS Keychain / Windows Credential Manager / Linux libsecret) | API key for your selected provider. Never stored in plaintext. Never sent to the frontend. |
| **Model** | Tauri Store (`settings.json`) | AI provider and model selection. Default provider: xAI. |
| **Hotkey** | Tauri Store (`settings.json`) | Global trigger shortcut. Default: `Cmd+K` (macOS) / `Ctrl+K` (Windows/Linux). Supports any modifier+key combination. |
| **Destructive Detection** | Settings > Preferences | Toggle safety pattern scanning on/off. Default: enabled. |
| **Auto-Paste** | Settings > Preferences | Auto-paste generated commands to active terminal. Default: enabled. |

---

## Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/your-feature`)
3. **Commit** your changes (`git commit -m "Add your feature"`)
4. **Push** to the branch (`git push origin feature/your-feature`)
5. **Open** a Pull Request

**Architecture notes for contributors:** The Rust backend (`src-tauri/src/`) handles all system interactions -- Accessibility API, Keychain, process introspection, AI streaming. The React frontend (`src/`) is purely UI and state. All communication goes through Tauri IPC commands defined in `src-tauri/src/commands/`. If you're adding a new system capability, start with a Rust command and expose it to the frontend via `#[tauri::command]`.

---

## License

[MIT](LICENSE)

---

<div align="center">

Built by [Lakshman Turlapati](https://github.com/LakshmanTurlapati)

If CMD+K saved you from copy-pasting from ChatGPT, consider giving it a star.

</div>
