# Technology Stack: Multi-Provider AI, WSL Context & Auto-Updater (v0.2.6)

**Project:** CMD+K -- Multi-Provider, WSL & Auto-Update
**Researched:** 2026-03-08
**Confidence:** HIGH (overall)

> This document supersedes the v0.2.4 STACK.md. The validated stack (Tauri v2, React 19, TypeScript, Vite, Zustand 5, NSPanel, Win32 APIs, keyring, eventsource-stream, etc.) is not re-researched. Focus is strictly on what the three new features add or change.

---

## Guiding Principle

All five AI providers (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter) use HTTP POST with SSE streaming. The existing `tauri-plugin-http` + `eventsource-stream` + `futures-util` stack handles this already for xAI. The provider abstraction is a Rust trait, NOT a new dependency. WSL context is Win32 API work on the existing `windows-sys` crate. Auto-updater is a single Tauri plugin addition.

**No new HTTP or AI client crates are needed.** The current reqwest (via tauri-plugin-http) + eventsource-stream pattern works for all providers.

---

## Already Cross-Platform (No Changes Needed)

These crates/libraries work across all three features with zero modifications:

| Component | Crate/Library | Notes |
|-----------|---------------|-------|
| HTTP client | tauri-plugin-http (reqwest) | All providers use REST+SSE |
| SSE parsing | eventsource-stream 0.2 | Parses SSE for all OpenAI-compatible providers |
| Stream combinators | futures-util 0.3 | Async stream processing |
| Async runtime | tokio 1 | Used for timeouts, async commands |
| Serialization | serde + serde_json 1 | JSON request/response bodies |
| Secure storage | keyring 3 | Multiple keys (one per provider) |
| Frontend state | Zustand 5 | Provider selection state |
| Persistent config | tauri-plugin-store 2 | Provider + model preferences |
| Process tree walking | windows-sys 0.59 | Already has all needed features for WSL |

**Confidence:** HIGH -- verified by reading current Cargo.toml and crate capabilities.

---

## Feature 1: Multi-Provider AI Support

### API Compatibility Matrix

All five providers follow the OpenAI-compatible chat completions format with minor variations:

| Provider | Endpoint | Auth Header | SSE Format | Key Difference |
|----------|----------|-------------|------------|----------------|
| xAI (Grok) | `https://api.x.ai/v1/chat/completions` | `Bearer {key}` | OpenAI-compatible | Current implementation, no change |
| OpenAI | `https://api.openai.com/v1/chat/completions` | `Bearer {key}` | `data: {"choices":[{"delta":{"content":"..."}}]}` | Identical to xAI SSE format |
| OpenRouter | `https://openrouter.ai/api/v1/chat/completions` | `Bearer {key}` | OpenAI-compatible | Requires `HTTP-Referer` and `X-Title` headers |
| Google Gemini | `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions` | `Bearer {key}` | OpenAI-compatible via their OpenAI-compat endpoint | Use OpenAI-compat endpoint, NOT native Gemini API |
| Anthropic | `https://api.anthropic.com/v1/messages` | `x-api-key: {key}` | Different: `content_block_delta` events with `{"delta":{"text":"..."}}` | Only non-OpenAI-compatible provider |

**Confidence:** HIGH for OpenAI, xAI, OpenRouter (all OpenAI-compatible). MEDIUM for Gemini OpenAI-compat endpoint (Google provides this but it may lag behind native features). HIGH for Anthropic (well-documented, just different format).

### What This Means for Implementation

Four of five providers share identical SSE parsing: `chunk["choices"][0]["delta"]["content"]` -- the exact code in `ai.rs` today. Only Anthropic differs:

- **Anthropic SSE events:** `event: content_block_delta` with `data: {"type":"content_block_delta","delta":{"type":"text_delta","text":"..."}}`
- **Anthropic headers:** `x-api-key` (not `Authorization: Bearer`), `anthropic-version: 2023-06-01`, `content-type: application/json`
- **Anthropic request body:** `{"model":"...","messages":[...],"stream":true,"max_tokens":4096}` -- requires `max_tokens` (others default to model max)

### New Rust Dependencies: NONE

No new crates needed. The provider abstraction is a Rust trait + enum:

```rust
enum Provider { OpenAI, Anthropic, Gemini, Xai, OpenRouter }

trait AiProvider {
    fn endpoint(&self) -> &str;
    fn auth_headers(&self, api_key: &str) -> Vec<(String, String)>;
    fn build_body(&self, model: &str, messages: &[Message], stream: bool) -> serde_json::Value;
    fn parse_sse_token(&self, data: &str) -> Option<String>;
}
```

The existing `reqwest::Client`, `eventsource_stream::Eventsource`, and `futures_util::StreamExt` handle all providers.

### Keychain Storage Changes

Currently: single key at `(SERVICE, "xai_api_key")`.
New: one key per provider, e.g., `(SERVICE, "openai_api_key")`, `(SERVICE, "anthropic_api_key")`, etc.

The `keyring` crate already supports arbitrary account names. No crate changes needed.

### Model Listing Per Provider

Each provider needs a model-fetching strategy:

| Provider | List Models Endpoint | Fallback Strategy |
|----------|---------------------|-------------------|
| xAI | `GET /v1/models` (current `validate_and_fetch_models`) | Hardcoded list + validation (already implemented) |
| OpenAI | `GET /v1/models` | Hardcoded: gpt-4o, gpt-4o-mini, gpt-4.1, gpt-4.1-mini, o4-mini |
| Anthropic | No list endpoint | Hardcoded: claude-sonnet-4-20250514, claude-haiku-4-20250414 |
| Gemini | `GET /v1beta/openai/models` (OpenAI-compat) | Hardcoded: gemini-2.5-flash, gemini-2.5-pro |
| OpenRouter | `GET /api/v1/models` | Hardcoded subset of popular models |

**Confidence:** HIGH for OpenAI and xAI model endpoints. MEDIUM for Gemini OpenAI-compat model listing. HIGH for Anthropic (no list endpoint, hardcode is standard). MEDIUM for OpenRouter model listing (returns hundreds; needs filtering).

### Frontend Changes

| Change | What | Library Impact |
|--------|------|----------------|
| Provider selector in onboarding | New `StepProviderSelect` component | None -- React + Zustand |
| Provider selector in settings | New tab or section in AccountTab | None -- React + Zustand |
| Per-provider model dropdown | `availableModels` becomes provider-dependent | None -- existing pattern |
| Per-provider API key input | One input per provider, stored separately | None -- existing keyring pattern |
| Provider + model in tauri-plugin-store | `{provider: "openai", model: "gpt-4o"}` | None -- existing store pattern |

### Frontend Dependencies: NONE

No new npm packages. The existing `@tauri-apps/api` + `@tauri-apps/plugin-store` + Zustand handle everything.

---

## Feature 2: WSL Terminal Context Detection

### The Problem

When a Windows Terminal tab runs WSL (Ubuntu, Debian, etc.), the process tree from the Windows side looks like:

```
WindowsTerminal.exe
  └── OpenConsole.exe
       └── wsl.exe (or wslhost.exe)
            └── [Linux PID namespace -- invisible to Win32 APIs]
```

The current `get_process_cwd()` reads CWD from the Windows PEB via `NtQueryInformationProcess`. For `wsl.exe`, this returns a Windows path like `C:\Windows\System32` (the wsl.exe binary location), NOT the Linux CWD.

Similarly, `find_shell_pid()` finds `wsl.exe` or `bash.exe` (the WSL init process) but cannot see the actual Linux shell running inside.

### Solution: Cross-Namespace Bridge via `wsl.exe` Commands

The only reliable way to get WSL context from Windows is to invoke WSL commands from the Windows side:

```
wsl.exe -e pwd                     → Linux CWD (e.g., /home/user/project)
wsl.exe -e printenv SHELL          → Linux shell (e.g., /bin/zsh)
wsl.exe -e sh -c "echo $0"        → Current shell name
```

For reading recent terminal output, the existing `uiautomation` crate reads the Windows Terminal UIA tree -- this already works for WSL tabs because the text is rendered by Windows Terminal regardless of whether the shell is native or WSL.

### New Rust Dependencies: NONE

WSL detection uses:
- **`std::process::Command`** to spawn `wsl.exe` subprocesses (stdlib, no crate)
- **`windows-sys`** (already in Cargo.toml) for process tree walking to detect `wsl.exe` in the tree
- **`uiautomation`** (already in Cargo.toml) for reading terminal text via Windows Terminal UIA

### WSL Detection Additions to `windows-sys` Features: NONE

The current `windows-sys` feature set already includes everything needed:
- `Win32_System_Threading` -- process inspection
- `Win32_System_Diagnostics_ToolHelp` -- CreateToolhelp32Snapshot for tree walking
- `Win32_System_Diagnostics_Debug` -- ReadProcessMemory (for PEB CWD, used as fallback)

### WSL-Specific Detection Logic

| Signal | How to Detect | API |
|--------|---------------|-----|
| WSL tab detected | Process tree contains `wsl.exe` or `wslhost.exe` as child of OpenConsole | `CreateToolhelp32Snapshot` (existing) |
| WSL distro name | `wsl.exe -l -q` or check `wsl.exe` command-line args | `std::process::Command` |
| Linux CWD | `wsl.exe -e pwd` | `std::process::Command` |
| Linux shell type | `wsl.exe -e printenv SHELL` or `wsl.exe -e basename "$SHELL"` | `std::process::Command` |
| Terminal visible text | Windows Terminal UIA tree (already works for WSL tabs) | `uiautomation` (existing) |
| VS Code Remote-WSL | VS Code window title contains `[WSL: distro]` | `GetWindowTextW` or UIA (existing) |

### CWD Path Mapping

WSL paths need mapping for the AI prompt context:

| Scenario | WSL CWD | Mapped Windows Path | Strategy |
|----------|---------|---------------------|----------|
| WSL filesystem | `/home/user/project` | `\\wsl$\Ubuntu\home\user\project` | Prefix with `\\wsl$\{distro}\` |
| Windows mount | `/mnt/c/Users/foo` | `C:\Users\foo` | Replace `/mnt/{letter}/` with `{LETTER}:\` |
| No mapping needed | AI just gets the Linux path | N/A | Pass Linux path directly to AI -- it is a Linux terminal |

**Recommendation:** Pass the raw Linux path to the AI. The AI already gets `shell_type: "zsh"` which signals this is a Linux context. No path mapping needed for command generation.

### VS Code Remote-WSL and Cursor

These are special cases where the IDE itself handles the WSL connection:

- **Detection:** Window title contains `[WSL: Ubuntu]` or similar
- **Terminal output:** UIA reads the integrated terminal text (same as native terminals)
- **CWD:** `wsl.exe -e pwd` still works because WSL is running
- **Shell type:** `wsl.exe -e printenv SHELL`

The existing `KNOWN_IDE_EXES` list already includes `Code.exe` and `Cursor.exe`. The WSL detection adds a flag to `ProcessInfo`:

```rust
pub struct ProcessInfo {
    pub cwd: Option<String>,
    pub shell_type: Option<String>,
    pub running_process: Option<String>,
    pub is_wsl: bool,  // NEW: signals Linux context to AI prompt
}
```

**Confidence:** HIGH for `wsl.exe` subprocess approach (well-documented, used by all WSL tooling). MEDIUM for VS Code Remote-WSL detection (window title parsing may vary). HIGH for terminal text reading (Windows Terminal UIA already works for WSL tabs).

---

## Feature 3: Auto-Updater (tauri-plugin-updater)

### New Rust Dependency

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri-plugin-updater | 2 | Check for updates, download, install on restart | Official Tauri v2 plugin, designed for this exact use case |

### New Frontend Dependency

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| @tauri-apps/plugin-updater | ^2 | Frontend JS API for update checking and triggering | Official companion to the Rust plugin |

### How tauri-plugin-updater Works

1. **Check:** App calls `check()` on startup -- hits a JSON endpoint for the latest version
2. **Compare:** Plugin compares current `version` from `tauri.conf.json` against the endpoint response
3. **Download:** If update available, `download_and_install()` downloads the new binary
4. **Install:** On next app restart, the update is applied (NSIS on Windows, DMG/tar.gz on macOS)

### Required Configuration

**Cargo.toml addition:**

```toml
[dependencies]
tauri-plugin-updater = "2"
```

**tauri.conf.json addition:**

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/user/repo/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

**package.json addition:**

```json
{
  "dependencies": {
    "@tauri-apps/plugin-updater": "^2"
  }
}
```

### Signing Requirement

tauri-plugin-updater requires update bundles to be signed. This is separate from macOS code signing or Windows Authenticode:

1. **Generate keypair:** `tauri signer generate -w ~/.tauri/cmd-k.key` (one-time)
2. **Set env vars in CI:** `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
3. **Set pubkey in config:** The `pubkey` field in `tauri.conf.json` plugins.updater

The signing happens automatically during `tauri build` when the env vars are set. The CI pipeline (`release.yml`) needs two new secrets.

### Update Endpoint Format

The `latest.json` file at the endpoint must follow this schema:

```json
{
  "version": "0.2.6",
  "notes": "Release notes here",
  "pub_date": "2026-03-08T00:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/.../CMD+K_0.2.6_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../CMD+K_0.2.6_x64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../CMD+K_0.2.6_x64-setup.nsis.zip"
    }
  }
}
```

GitHub Releases can host this file. The CI pipeline needs to generate and upload `latest.json` alongside the release artifacts.

### Plugin Registration in Rust

```rust
// In main.rs or lib.rs, add to the Tauri builder:
.plugin(tauri_plugin_updater::Builder::new().build())
```

### Frontend Update Flow

```typescript
import { check } from "@tauri-apps/plugin-updater";

// On app startup:
const update = await check();
if (update) {
  // Show update prompt to user
  // User accepts:
  await update.downloadAndInstall();
  // Restart app (or prompt user to restart)
}
```

**Confidence:** HIGH -- tauri-plugin-updater is the official, first-party Tauri v2 plugin for auto-updates. Well-documented with clear API surface. The signing requirement is the main integration complexity.

---

## New Dependencies Summary

### Add to Cargo.toml (cross-platform)

```toml
[dependencies]
tauri-plugin-updater = "2"
```

### Add to package.json

```bash
pnpm add @tauri-apps/plugin-updater
```

### No Other New Dependencies

That is it. One Rust crate, one npm package. Everything else is architecture (trait-based provider abstraction) and logic (WSL detection via subprocess calls to existing APIs).

---

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Multi-provider HTTP | Existing reqwest + eventsource-stream | async-openai crate | async-openai only supports OpenAI-compatible APIs. Anthropic is not OpenAI-compatible. Adding a crate for 4 providers while still needing custom code for the 5th adds complexity without reducing it. |
| Multi-provider HTTP | Existing reqwest + eventsource-stream | genai crate (multi-provider Rust SDK) | Immature, low adoption, adds a large dependency for something we can do in ~200 lines of trait implementation |
| Anthropic API format | Native Messages API with custom SSE parsing | Anthropic OpenAI-compat proxy | Anthropic does not offer an official OpenAI-compat endpoint. Third-party proxies add latency and a point of failure. |
| Gemini API | Google's OpenAI-compat endpoint | Native Gemini API (generateContent) | OpenAI-compat endpoint means we reuse the same SSE parser for 4 of 5 providers. Native Gemini API has a different streaming format (Server-Sent Events with different JSON structure). |
| WSL CWD detection | `wsl.exe -e pwd` subprocess | Parse `/proc/PID/cwd` via `\\wsl$\` UNC path | UNC path approach requires knowing the exact Linux PID and distro name. subprocess is simpler and more reliable. |
| WSL CWD detection | `wsl.exe -e pwd` subprocess | WslApi.dll (`WslLaunch`) | WslApi requires linking against a Windows SDK DLL. `wsl.exe` subprocess is simpler and universally available on WSL-enabled systems. |
| Auto-updater | tauri-plugin-updater | Custom update checker + download | Reinventing verified, signed update mechanisms is error-prone and a security risk. The official plugin handles signature verification, atomic installs, and rollback. |
| Auto-updater | tauri-plugin-updater | Electron-style autoUpdater | Not applicable -- this is a Tauri app |
| Update hosting | GitHub Releases latest.json | Self-hosted update server | GitHub Releases is free, already used for distribution, and has CDN. No reason to self-host. |

---

## Integration Points

### How Provider Abstraction Connects to Existing Code

**Current flow (ai.rs):**
1. Read API key from keyring `(SERVICE, "xai_api_key")`
2. Build messages array
3. POST to `https://api.x.ai/v1/chat/completions`
4. Parse SSE: `chunk["choices"][0]["delta"]["content"]`

**New flow:**
1. Read `provider` from tauri-plugin-store (e.g., `"openai"`)
2. Read API key from keyring `(SERVICE, "{provider}_api_key")`
3. Get provider config: endpoint, headers, body builder, SSE parser
4. Build messages array (same logic)
5. POST to provider-specific endpoint
6. Parse SSE using provider-specific parser

**The `stream_ai_response` command gains a `provider` parameter.** Frontend sends the selected provider alongside model and query.

### How WSL Detection Connects to Existing Code

**Current flow (detect_windows.rs + process.rs):**
1. `get_exe_name(hwnd)` identifies terminal
2. `find_shell_pid()` walks process tree
3. `get_process_cwd()` reads CWD via PEB
4. Returns `ProcessInfo { cwd, shell_type, running_process }`

**New flow for WSL:**
1. `get_exe_name(hwnd)` identifies terminal (same)
2. `find_shell_pid()` walks process tree, finds `wsl.exe` instead of shell
3. **NEW:** If leaf process is `wsl.exe`/`wslhost.exe`, call `wsl.exe -e pwd` for CWD
4. **NEW:** Call `wsl.exe -e printenv SHELL` for shell type
5. Returns `ProcessInfo { cwd: "/home/user/project", shell_type: "zsh", running_process: None, is_wsl: true }`

**The system prompt needs WSL awareness.** When `is_wsl: true`, use the Linux terminal system prompt (commands should be Linux, not Windows).

### How Auto-Updater Connects to Existing Code

**Rust:** Single line in plugin registration chain.
**Frontend:** New `useUpdateCheck` hook called in App.tsx on mount. Shows a small toast/banner when update available.
**CI:** Modified `release.yml` to set `TAURI_SIGNING_PRIVATE_KEY` and generate `latest.json`.

---

## What NOT to Add

| Temptation | Why Not |
|------------|---------|
| `async-openai` crate | Only handles OpenAI-compat. Still need Anthropic custom code. Net complexity increase. |
| `anthropic-sdk` crate | Immature Rust SDK. We need ~30 lines of custom SSE parsing, not a full SDK. |
| `google-genai` crate | Use OpenAI-compat endpoint instead. One less API format to support. |
| Provider-specific Rust crates per provider | Five crates for something achievable with one trait and five implementations of ~20 lines each. |
| `wslapi` crate or WslApi.dll FFI | `wsl.exe` subprocess is simpler and doesn't require SDK linking. |
| Custom update server | GitHub Releases serves the same purpose for free. |
| `reqwest` as direct dependency | Already available via `tauri-plugin-http`'s re-export. Adding it directly risks version conflicts. |
| Changing SSE library | `eventsource-stream` works for all providers. No reason to switch. |

---

## Installation

### Rust

```toml
# Add to [dependencies] in src-tauri/Cargo.toml:
tauri-plugin-updater = "2"
```

### Frontend

```bash
pnpm add @tauri-apps/plugin-updater
```

### CI/CD Secrets (GitHub Actions)

```
TAURI_SIGNING_PRIVATE_KEY     # Generated by `tauri signer generate`
TAURI_SIGNING_PRIVATE_KEY_PASSWORD  # Password for the signing key
```

### One-Time Setup

```bash
# Generate update signing keypair (run once, store securely)
npx tauri signer generate -w ~/.tauri/cmd-k.key
# Output: public key (goes in tauri.conf.json) and private key file
```

---

## Version Pinning Notes

| Dependency | Pin | Reason |
|------------|-----|--------|
| tauri-plugin-updater | `"2"` (semver range) | Matches Tauri v2 plugin ecosystem versioning. Tauri plugins use major version matching. |
| @tauri-apps/plugin-updater | `"^2"` | NPM semver range matching Tauri v2 |

All other existing dependencies remain at their current versions. No updates needed.

**Confidence:** MEDIUM for exact latest patch versions (unable to verify crates.io due to tool restrictions). HIGH for major version compatibility (Tauri v2 plugins use v2 consistently).

---

## Sources

- Current codebase analysis: `ai.rs`, `xai.rs`, `process.rs`, `detect_windows.rs`, `store/index.ts` -- HIGH confidence
- OpenAI Chat Completions API format -- HIGH confidence (training data, well-established API since 2023)
- Anthropic Messages API streaming format -- HIGH confidence (training data, distinct SSE format well-documented)
- Google Gemini OpenAI-compatible endpoint -- MEDIUM confidence (training data; Google launched this in 2024 at `/v1beta/openai/` path)
- OpenRouter API format -- HIGH confidence (explicitly designed as OpenAI-compatible proxy)
- xAI API format -- HIGH confidence (already implemented and working in codebase)
- WSL architecture and `wsl.exe` CLI -- HIGH confidence (training data + Microsoft documentation is well-established)
- tauri-plugin-updater v2 -- MEDIUM confidence (training data; official Tauri plugin, API surface is stable but exact latest patch version unverified)
- Tauri update signing -- MEDIUM confidence (training data; `tauri signer generate` command and env var names are documented)

---
*Stack research for: CMD+K v0.2.6 Multi-Provider AI, WSL Context & Auto-Updater*
*Researched: 2026-03-08*
