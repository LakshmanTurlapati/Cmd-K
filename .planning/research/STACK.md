# Technology Stack

**Project:** CMD + K (macOS System-Wide Overlay with AI Command Generation)
**Researched:** 2026-02-21
**Target Platform:** macOS (v1)
**Architecture:** Tauri v2 (Rust backend + web frontend)

## Executive Summary

Standard 2025/2026 stack for Tauri-based macOS overlay apps combines Tauri v2.10.2 with React 18 + TypeScript + Vite for the frontend, leveraging Tauri's official plugin ecosystem for system integration. Key challenges: macOS window level management requires low-level Cocoa bindings beyond standard Tauri APIs, and terminal context reading needs AppleScript automation with Accessibility permissions.

**Confidence:** HIGH for core stack (Tauri, React, Vite), MEDIUM for overlay window implementation (requires custom Cocoa integration), HIGH for AI integration (xAI SDK available), MEDIUM for terminal automation (AppleScript-based approach proven but requires careful implementation).

---

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| Tauri | v2.10.2 | Desktop app framework | Current stable (Feb 2026). Native macOS integration, 2-2.5MB bundle size, Rust security model. V2 has mature plugin ecosystem. | HIGH |
| Rust | 1.88+ | Backend/system integration | Required by Tauri. MSRV policy ensures 6-month minimum. Strong type safety for system APIs. | HIGH |
| React | 18.3+ | Frontend UI framework | Dominant ecosystem, component-based for overlay UI, best TypeScript support. Mature Tauri integration. | HIGH |
| TypeScript | 5.7+ | Frontend type safety | Essential for IPC type safety, compile-time error catching. Tauri v2 encourages strong typing. | HIGH |
| Vite | 6.x | Frontend build tool | Official Tauri recommendation. Fast HMR, optimized builds, first-class TypeScript support. | HIGH |

**Rationale:**
- Tauri v2 over Electron: 10x smaller bundle (2-2.5MB vs 150MB+), native macOS APIs, better security model
- React over Vue/Svelte: Largest ecosystem (65K+ stars for shadcn/ui alone), most third-party integrations, best TypeScript tooling
- Vite over Webpack: Official Tauri recommendation, faster dev server, better tree-shaking

### Tauri Plugins (Official)

| Plugin | Version | Purpose | When to Use | Confidence |
|--------|---------|---------|-------------|------------|
| tauri-plugin-global-shortcut | 2.x | Global Cmd+K hotkey registration | REQUIRED. System-wide keyboard shortcuts. | HIGH |
| tauri-plugin-clipboard-manager | 2.x | Clipboard read/write for pasting commands | REQUIRED. Paste generated commands to terminal. | HIGH |
| tauri-plugin-shell | 2.x | Execute AppleScript for terminal context | REQUIRED. Run osascript for terminal detection and context. | HIGH |
| tauri-plugin-window-state | 2.x | Persist overlay position between launches | OPTIONAL. Remember last window position. | HIGH |
| tauri-plugin-positioner | 2.x | Position window relative to cursor/screen | OPTIONAL. Center overlay on active display. | MEDIUM |

**Note on Missing Plugin:**
- No official plugin for NSWindowLevel control (floating above all apps)
- Requires custom Cocoa bindings via `cocoa` crate (see macOS System Integration below)

### Frontend Stack

| Library | Version | Purpose | Why | Confidence |
|---------|---------|---------|-----|------------|
| shadcn/ui | Latest | React component library (Radix UI + Tailwind) | 65K+ stars, native-looking components, copy-paste pattern (no npm bloat), Tailwind integration. Perfect for overlay UI. | HIGH |
| Tailwind CSS | 4.x | Utility-first CSS framework | Standard with shadcn/ui. Fast styling, small bundles, dark mode support built-in. | HIGH |
| Zustand | 5.x | Client-side state management | Minimal (3KB), hooks-based, perfect for overlay state. No Redux boilerplate. Tauri-friendly (works across windows). | HIGH |
| react-hot-toast | 2.x | Toast notifications | Lightweight (5KB), overlay-friendly, customizable. For user feedback (errors, success). | MEDIUM |
| Lucide React | Latest | Icon library | 1400+ icons, tree-shakeable, matches shadcn/ui design language. | HIGH |

**Rationale:**
- shadcn/ui over Material-UI: Copy-paste pattern means smaller bundle, full customization, better overlay performance
- Zustand over Redux: 10x smaller API surface, no boilerplate, sufficient for single-window overlay state
- Tailwind over CSS-in-JS: Better build-time optimization, no runtime cost, dark mode utilities

### Rust Crates (Backend)

#### AI Integration

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| xai-sdk | 0.1.x | xAI Grok API client (gRPC) | Official Protocol Buffer definitions, type-safe, supports grok-4-fast-reasoning. | HIGH |
| reqwest | 0.13.x | HTTP client (for OpenAI/Anthropic later) | Industry standard, async/await, TLS support. Use native-tls feature for macOS keychain. | HIGH |
| tokio | 1.47.x LTS | Async runtime | Required by reqwest/xai-sdk. LTS until Sept 2026. Most widely used async runtime. | HIGH |
| serde | 1.0.x | JSON serialization | Universal Rust serialization. Required for Tauri IPC and AI API responses. | HIGH |
| serde_json | 1.0.x | JSON parsing | Handles AI API responses, Tauri command arguments. | HIGH |

**Alternative for xAI:**
- `grok-rust-sdk` (0.x) - Community SDK with more batteries included (tool calling, sessions)
- **Recommendation:** Start with `xai-sdk` (official protobuf), switch to `grok-rust-sdk` if need advanced features

#### macOS System Integration

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| cocoa | 0.26.x | macOS Cocoa API bindings | REQUIRED for NSWindowLevel control (floating overlay above all apps). Low-level but necessary. | MEDIUM |
| osascript | 0.3.x or osakit 0.2.x | AppleScript execution | Terminal context reading (cwd, recent commands). `osascript` is simpler, `osakit` supports JavaScript too. | HIGH |
| sysinfo | 0.33.x | Process detection | Detect active terminal (Terminal.app, iTerm2, etc.). Cross-platform, well-maintained. | HIGH |
| macos-accessibility-client | 0.0.x | Accessibility API wrapper | Check if app has Accessibility permission. Required before context reading. | MEDIUM |

**Why these crates:**
- **cocoa:** Only way to set `NSWindowLevel` to float above fullscreen apps. Tauri's `setAlwaysOnTop` insufficient (GitHub issue #11488).
- **osascript vs osakit:** `osascript` simpler for basic AppleScript, `osakit` if need JavaScript automation later. Both require main thread execution.
- **sysinfo:** Standard for process enumeration. Detect frontmost terminal by process name.
- **macos-accessibility-client:** Lightweight wrapper. Must verify permission before attempting automation (macOS blocks without permission).

**Terminal Context Strategy:**
```rust
// 1. Detect active terminal (sysinfo)
// 2. Check Accessibility permission (macos-accessibility-client)
// 3. Execute AppleScript via osascript to get:
//    - Current working directory (pwd)
//    - Recent commands (history | tail -10)
// 4. Parse output, send to frontend
```

#### Supporting Crates

| Crate | Version | Purpose | Why | Confidence |
|-------|---------|---------|-----|------------|
| anyhow | 1.0.x | Error handling | Ergonomic error propagation in Tauri commands. Converts to string for frontend. | HIGH |
| tracing | 0.1.x | Structured logging | Better than println!, structured context, production-ready. | HIGH |
| tracing-subscriber | 0.3.x | Log formatting | Console output during dev, file output for production. | HIGH |

### Development Tools

| Tool | Version | Purpose | Why | Confidence |
|------|---------|---------|-----|------------|
| pnpm | 9.x | Package manager | Faster than npm/yarn, space-efficient, monorepo-ready if add OpenAI/Anthropic SDKs. | HIGH |
| ESLint | 9.x | JavaScript linter | Standard for TypeScript projects. Tauri templates include config. | HIGH |
| Prettier | 3.x | Code formatter | Consistent formatting, integrates with ESLint. | HIGH |
| rust-analyzer | Latest | Rust LSP | Essential for Rust development. IDE-agnostic. | HIGH |
| cargo-watch | Latest | Auto-rebuild on file change | `cargo watch -x run` for faster iteration. | MEDIUM |

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Desktop Framework | Tauri v2 | Electron | 60x larger bundles (150MB vs 2.5MB), security concerns, slower startup |
| Frontend | React | Svelte | Smaller ecosystem, fewer UI libraries, less TypeScript tooling |
| Frontend | React | Vue | Smaller ecosystem, less shadcn/ui equivalent quality |
| State Management | Zustand | Redux | Massive boilerplate for single-window overlay, overkill |
| State Management | Zustand | Jotai/Recoil | Less mature Tauri integration, smaller community |
| AI HTTP Client | reqwest | hyper | Lower-level, reqwest built on hyper anyway, less ergonomic |
| AppleScript Execution | osascript | osakit | osascript simpler for basic needs, osakit adds JavaScript support (not needed v1) |
| Process Detection | sysinfo | libproc-rs | sysinfo more mature, better API, cross-platform (helpful for Linux later) |
| macOS Bindings | cocoa | objc2 | cocoa higher-level, established patterns, better docs |

---

## Installation

### Prerequisites
```bash
# Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (via Homebrew)
brew install node

# pnpm
npm install -g pnpm

# Tauri CLI
cargo install tauri-cli --version "^2.0.0"
```

### Project Setup
```bash
# Create Tauri app with React + TypeScript + Vite
pnpm create tauri-app --template react-ts

# Navigate to project
cd cmd-k

# Frontend dependencies
pnpm add zustand react-hot-toast
pnpm add -D tailwindcss postcss autoprefixer
pnpm add -D @shadcn/ui lucide-react

# Tauri plugins
cargo add tauri-plugin-global-shortcut
cargo add tauri-plugin-clipboard-manager
cargo add tauri-plugin-shell
cargo add tauri-plugin-window-state
cargo add tauri-plugin-positioner

# Rust backend dependencies
cd src-tauri
cargo add xai-sdk
cargo add reqwest --features native-tls,json
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
cargo add cocoa
cargo add osascript
cargo add sysinfo
cargo add macos-accessibility-client
cargo add anyhow
cargo add tracing
cargo add tracing-subscriber

# Return to root
cd ..
```

### TypeScript Type Safety for IPC

```bash
# Optional: Type-safe IPC layer
pnpm add -D taurpc
cargo add taurpc
```

**TauRPC Benefits:**
- Generate TypeScript types from Rust commands
- Compile-time IPC type safety
- Eliminates runtime type errors

**Alternative:** Manual type definitions with Zod for smaller projects.

---

## Critical Implementation Notes

### 1. Window Level Management (Floating Overlay)

**Problem:** Tauri's `setAlwaysOnTop(true)` does NOT work above fullscreen apps on macOS (Issue #11488).

**Solution:** Use `cocoa` crate to set NSWindowLevel directly.

```rust
use cocoa::appkit::{NSWindow, NSWindowLevel};
use cocoa::base::id;

// In Tauri setup
#[cfg(target_os = "macos")]
fn set_window_level_above_all(window: &tauri::Window) {
    unsafe {
        let ns_window = window.ns_window().unwrap() as id;
        NSWindow::setLevel_(ns_window, NSWindowLevel::NSFloatingWindowLevel + 1);
    }
}
```

**Confidence:** MEDIUM (proven pattern in community, but low-level unsafe code)

### 2. Activation Policy (Hide Dock Icon)

**Problem:** Overlay should not show Dock icon (feels like system UI, not app).

**Solution:** Use `Accessory` activation policy.

```rust
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Tradeoff:** App icon disappears from Dock. Acceptable for overlay-only apps.

### 3. Terminal Context via AppleScript

**Limitations:**
- AppleScript MUST run on main thread (macOS restriction)
- Requires Accessibility permission (user must grant in System Settings)
- Different AppleScript per terminal (Terminal.app vs iTerm2 vs Warp)

**Example for Terminal.app:**
```rust
use std::process::Command;

fn get_terminal_context() -> Result<String, anyhow::Error> {
    let script = r#"
        tell application "Terminal"
            if (count of windows) > 0 then
                do script "pwd; history | tail -5" in front window
            end if
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    Ok(String::from_utf8(output.stdout)?)
}
```

**Confidence:** HIGH (proven pattern, but fragile across terminal apps)

### 4. Accessibility Permission Check

**Before attempting terminal context:**

```rust
use macos_accessibility_client::accessibility;

fn check_accessibility() -> bool {
    accessibility::application_is_trusted_with_prompt()
}
```

**User Experience:**
- First launch: Prompt to enable Accessibility
- App explains why (needed to read terminal context)
- Gracefully degrade if denied (AI generates commands without context)

### 5. Global Hotkey Registration

**Cmd+K is system-reserved on macOS** (Spotlight). Use alternative or override with Accessibility permission.

**Recommendation:** Default to `Cmd+Shift+K`, allow customization.

```rust
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

fn register_hotkey(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut: Shortcut = "CommandOrControl+Shift+K".parse()?;
    app.global_shortcut().register(shortcut)?;
    Ok(())
}
```

### 6. Transparent Window Configuration

**In `tauri.conf.json`:**
```json
{
  "tauri": {
    "windows": [
      {
        "title": "CMD + K",
        "width": 600,
        "height": 400,
        "transparent": true,
        "decorations": false,
        "alwaysOnTop": true,
        "skipTaskbar": true,
        "visible": false,
        "center": true
      }
    ]
  }
}
```

**Note:** `alwaysOnTop` sets basic level. Override with Cocoa for above-fullscreen.

---

## Architecture Patterns for This Stack

### IPC Command Pattern
```rust
// src-tauri/src/commands.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TerminalContext {
    pub cwd: String,
    pub recent_commands: Vec<String>,
}

#[tauri::command]
async fn get_terminal_context() -> Result<TerminalContext, String> {
    // Implementation
    Ok(TerminalContext {
        cwd: "/Users/...".to_string(),
        recent_commands: vec![],
    })
}

#[tauri::command]
async fn generate_command(
    prompt: String,
    context: TerminalContext,
) -> Result<String, String> {
    // Call xAI Grok API
    Ok("ls -la".to_string())
}
```

### Frontend Component Structure
```typescript
// src/components/CommandInput.tsx
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';
import { useStore } from '@/store';

export function CommandInput() {
  const [prompt, setPrompt] = useState('');
  const { context } = useStore();

  const handleGenerate = async () => {
    const command = await invoke<string>('generate_command', {
      prompt,
      context,
    });
    // Paste to clipboard and close overlay
  };

  return (
    // shadcn/ui Input component
  );
}
```

### State Management with Zustand
```typescript
// src/store/index.ts
import { create } from 'zustand';

interface AppState {
  context: TerminalContext | null;
  isLoading: boolean;
  setContext: (context: TerminalContext) => void;
}

export const useStore = create<AppState>((set) => ({
  context: null,
  isLoading: false,
  setContext: (context) => set({ context }),
}));
```

---

## Performance Considerations

| Concern | At Launch | At 1K Users | At 100K Users |
|---------|-----------|-------------|---------------|
| Bundle Size | 2.5MB (Tauri) | N/A (desktop app) | N/A |
| Startup Time | <500ms (Rust + React) | N/A | N/A |
| Hotkey Response | <50ms (global-shortcut) | N/A | N/A |
| AI API Latency | 200-2000ms (Grok) | Same (user-specific) | Same |
| Memory Usage | ~50MB (webview + Rust) | N/A | N/A |

**Desktop App Note:** No scalability concerns like web apps. Each user runs local binary.

**Optimization Priorities:**
1. Fast overlay appearance (<50ms after Cmd+K)
2. Instant UI feedback (loading states while AI processes)
3. Minimize bundle size (lazy-load AI SDKs if add OpenAI/Anthropic)

---

## Security Considerations

### Tauri Security Model
- Allowlist-based IPC (explicitly allow commands)
- Content Security Policy for webview
- No Node.js runtime (safer than Electron)

### AI API Keys
- Store in macOS Keychain (use `keyring` crate)
- Never bundle in app (prompt user on first launch)

### AppleScript Injection Risk
- Sanitize terminal output before sending to AI
- Escape shell metacharacters in generated commands
- Show preview before pasting (user confirms)

---

## Testing Strategy

### Unit Tests
```bash
# Rust backend
cargo test

# Frontend
pnpm test
```

### Integration Tests (Tauri WebDriver)
```bash
cargo add tauri-webdriver-automation --dev
```

**Test scenarios:**
- Hotkey triggers overlay
- Accessibility permission prompt
- Terminal context extraction
- AI command generation
- Clipboard paste

### Manual Testing Checklist
- [ ] Works on macOS 13+ (Ventura, Sonoma, Sequoia)
- [ ] Floats above fullscreen apps
- [ ] Cmd+Shift+K shows overlay
- [ ] ESC hides overlay
- [ ] Terminal.app context extraction
- [ ] iTerm2 context extraction
- [ ] Grok API integration
- [ ] Clipboard paste to terminal

---

## Migration Path to Multi-Provider AI

**V1:** xAI (Grok) only
**V2:** Add OpenAI, Anthropic

**Architecture for V2:**
```rust
// src-tauri/src/ai/mod.rs
pub trait AIProvider {
    async fn generate_command(&self, prompt: &str, context: &TerminalContext) -> Result<String>;
}

pub struct GrokProvider { /* xai-sdk */ }
pub struct OpenAIProvider { /* reqwest */ }
pub struct ClaudeProvider { /* reqwest */ }

impl AIProvider for GrokProvider { /* ... */ }
impl AIProvider for OpenAIProvider { /* ... */ }
impl AIProvider for ClaudeProvider { /* ... */ }
```

**Frontend Selection:**
```typescript
// User selects provider in settings
const provider = useStore(state => state.aiProvider);
// 'grok' | 'openai' | 'claude'
```

**Why this matters for stack choice:**
- `reqwest` chosen for future OpenAI/Anthropic HTTP APIs
- `serde` handles different response schemas
- `anyhow` abstracts provider-specific errors

---

## Known Limitations

### macOS-Specific
1. **Global Hotkey Conflicts:** Cmd+K reserved by Spotlight (recommend Cmd+Shift+K)
2. **Fullscreen Overlay:** Requires unsafe Cocoa code (NSWindowLevel)
3. **Terminal Fragmentation:** Different AppleScript per terminal app
4. **Accessibility Permission:** Friction on first launch (system modal)

### Tauri v2
1. **No Native Window Level API:** Must drop to Cocoa (Issue #4620 open since 2022)
2. **visibleOnAllWorkspaces Broken:** Overlay won't show on all desktops (Issue #11488)

### AI Integration
1. **Latency:** 200ms-2s per generation (network-dependent)
2. **Rate Limits:** xAI free tier limits (need upgrade path)
3. **Offline Mode:** No command generation without internet

---

## Version Lock Recommendations

**Lock these (breaking changes likely):**
- `tauri`: `^2.10.0` (lock minor, allow patch)
- `cocoa`: `=0.26.0` (unsafe API, lock exact)
- `xai-sdk`: `^0.1.0` (early stage, lock minor)

**Allow updates (stable APIs):**
- `react`: `^18.0.0`
- `typescript`: `^5.0.0`
- `serde`: `^1.0.0`
- `tokio`: `^1.47.0`
- `reqwest`: `^0.13.0`

---

## Sources

### Tauri Documentation
- [Tauri v2 Official Documentation](https://v2.tauri.app/)
- [Tauri v2 Release Page](https://v2.tauri.app/release/)
- [Tauri Plugins](https://v2.tauri.app/plugin/)
- [Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/)
- [Tauri GitHub Releases](https://github.com/tauri-apps/tauri/releases)

### Tauri Issues & Discussions
- [Issue #11488: visibleOnAllWorkspaces not staying on top](https://github.com/tauri-apps/tauri/issues/11488)
- [Issue #4620: Feature request for window.set_level API](https://github.com/tauri-apps/tauri/issues/4620)
- [Discussion #9685: Does alwaysOnTop really mean all windows](https://github.com/orgs/tauri-apps/discussions/9685)

### Frontend Framework Research
- [Tauri Frontend Configuration](https://v2.tauri.app/start/frontend/)
- [CrabNebula: Best UI Libraries for Tauri](https://crabnebula.dev/blog/the-best-ui-libraries-for-cross-platform-apps-with-tauri/)
- [Complete Guide to React + TypeScript + Vite 2026](https://medium.com/@robinviktorsson/complete-guide-to-setting-up-react-with-typescript-and-vite-2025-468f6556aaf2)

### Rust Crates
- [xai-sdk on GitHub](https://github.com/0xC0DE666/xai-sdk)
- [grok-rust-sdk on crates.io](https://crates.io/crates/grok-rust-sdk)
- [reqwest Documentation](https://docs.rs/reqwest/)
- [Tokio Official Site](https://tokio.rs/)
- [sysinfo Documentation](https://docs.rs/sysinfo/)
- [cocoa crate - NSWindow](https://docs.rs/cocoa/latest/cocoa/appkit/trait.NSWindow.html)
- [osascript Documentation](https://docs.rs/osascript/)
- [macos-accessibility-client](https://crates.io/crates/macos-accessibility-client)
- [Serde JSON](https://docs.rs/serde_json/)

### UI & State Management
- [shadcn/ui Official](https://ui.shadcn.com/)
- [shadcn UI Complete Guide 2026](https://designrevision.com/blog/shadcn-ui-guide)
- [Zustand GitHub](https://github.com/pmndrs/zustand)
- [Zustand with Tauri State Sync](https://www.gethopp.app/blog/tauri-window-state-sync)
- [react-hot-toast](https://react-hot-toast.com/)

### macOS Development
- [2026 macOS Always-on-Top Landscape](https://www.floatytool.com/posts/macos-always-on-top-landscape/)
- [Floaty Tool - How to Keep Windows on Top](https://www.floatytool.com/posts/how-to-keep-any-window-always-on-top-macos/)

---

## Confidence Assessment

| Area | Confidence | Reasoning |
|------|------------|-----------|
| Core Stack (Tauri/React/Vite) | HIGH | Official recommendations, current versions verified via WebSearch Feb 2026 |
| Tauri Plugins | HIGH | Official plugins, docs current |
| AI Integration (xAI) | HIGH | Official SDK available, community alternatives exist |
| Window Level Control | MEDIUM | Requires unsafe Cocoa code, proven pattern but low-level |
| Terminal Context Reading | MEDIUM | AppleScript approach proven but fragile across terminals |
| Accessibility Permissions | HIGH | Standard macOS pattern, well-documented |
| Frontend UI Stack | HIGH | shadcn/ui + Tailwind standard for Tauri in 2026 |
| State Management (Zustand) | HIGH | Mature library, Tauri-specific examples exist |
| Overall Stack | HIGH | All components current, verified, battle-tested |

**Gaps Identified:**
- Terminal.app vs iTerm2 vs Warp AppleScript differences (needs phase-specific research)
- Grok API rate limits and pricing (needs validation before production)
- macOS 13 vs 14 vs 15 Accessibility API changes (needs testing matrix)

---

## Next Steps for Roadmap Creation

**Phase Structure Recommendations:**

1. **Foundation (Tauri + Overlay):** Basic overlay window, global hotkey, window level control
   - Validates: Cocoa integration, activation policy, transparent window
   - Risk: NSWindowLevel floating might fail in specific macOS configs

2. **Terminal Detection:** Process detection, AppleScript execution, context extraction
   - Validates: Accessibility permissions, AppleScript differences per terminal
   - Risk: Fragile across terminal apps (needs robust fallback)

3. **AI Integration:** xAI SDK, command generation, error handling
   - Validates: API key storage, rate limits, latency UX
   - Risk: API changes, costs at scale

4. **Polish:** Clipboard paste, keyboard shortcuts, settings persistence
   - Validates: End-to-end flow, edge cases
   - Risk: Low (standard Tauri patterns)

**Research Flags:**
- Phase 2 (Terminal Detection): Deep research on iTerm2, Warp, Alacritty AppleScript variations
- Phase 3 (AI): Validate Grok pricing, explore fallback providers if rate limited

**Architecture Ready:** Stack supports clean separation (Rust backend for system, React for UI, Tauri IPC bridge). No major blockers identified.
