# Architecture Patterns

**Domain:** Multi-provider AI, WSL terminal context, and auto-updater integration into existing Tauri v2 app
**Researched:** 2026-03-08
**Confidence:** HIGH (provider abstraction, keychain), MEDIUM (WSL detection, auto-updater config)

---

## Current Architecture Snapshot

```
Frontend (React + Zustand)                  Backend (Rust / Tauri)
-----------------------------               ----------------------------------
src/store/index.ts                          src-tauri/src/
  useOverlayStore (Zustand)                   lib.rs         (plugin init, IPC handler registration)
    - apiKeyStatus, selectedModel             state.rs       (AppState: Mutex-wrapped fields)
    - submitQuery() -> invoke("stream_ai")    commands/
                                                ai.rs        (stream_ai_response: hardcoded xAI endpoint)
src/components/Onboarding/                      xai.rs       (validate_and_fetch_models: xAI-specific)
  StepApiKey.tsx (xAI-only key input)           keychain.rs  (save/get/delete with hardcoded "xai_api_key")
  StepModelSelect.tsx                           terminal.rs  (get_app_context -> terminal::detect_full_with_hwnd)
                                                hotkey.rs    (capture PID/HWND before overlay)
src/components/Settings/                        paste.rs     (AppleScript/Win32 paste)
  AccountTab.tsx (xAI-only key mgmt)            window.rs    (show/hide overlay)
  ModelTab.tsx                                  safety.rs    (destructive command detection)
                                              terminal/
                                                mod.rs       (detect, detect_full, detect_full_with_hwnd)
                                                detect.rs    (macOS bundle ID classification)
                                                detect_windows.rs (Windows exe classification)
                                                process.rs   (process tree walk, CWD via libproc/PEB)
                                                ax_reader.rs (macOS Accessibility API text)
                                                uia_reader.rs (Windows UI Automation text)
                                                filter.rs    (sensitive data redaction)
                                                browser.rs   (DevTools console detection)
```

**Key coupling points for this milestone:**
- `commands/ai.rs` line 8: hardcoded `ACCOUNT: &str = "xai_api_key"`
- `commands/ai.rs` line 259: hardcoded `https://api.x.ai/v1/chat/completions`
- `commands/ai.rs` line 307-308: SSE parsing assumes OpenAI-compatible `choices[0].delta.content`
- `commands/xai.rs`: entire file is xAI-specific (model validation, fallback list)
- `commands/keychain.rs` line 4: hardcoded `ACCOUNT: &str = "xai_api_key"`
- `src/store/index.ts`: `XaiModelWithMeta` type name, no provider concept
- `src/components/Onboarding/StepApiKey.tsx` line 98: "Enter your xAI API key"
- `src/components/Settings/AccountTab.tsx` line 97: "Paste your xAI API key"
- `terminal/detect_windows.rs`: no `wsl.exe` or `wslhost.exe` in known lists
- `terminal/process.rs`: Windows process tree walk does not enter WSL namespace

---

## Feature 1: Multi-Provider AI Support

### Recommended Architecture: Provider Enum with Per-Variant Dispatch

**Why enum dispatch, not `Box<dyn Trait>`:** The provider set is fixed at compile time (5 variants). Async trait methods with `Box<dyn>` require boxing futures and lose the compiler's exhaustiveness checking. Enum dispatch is zero-cost, and the `match` arms catch missing implementations at compile time.

**New module structure:**

```
src-tauri/src/commands/providers/
  mod.rs          -- Provider enum, from_str, ModelInfo struct
  openai.rs       -- OpenAI implementation
  anthropic.rs    -- Anthropic implementation
  google.rs       -- Google Gemini implementation
  xai.rs          -- xAI/Grok (extract from current ai.rs + xai.rs)
  openrouter.rs   -- OpenRouter implementation
```

### Provider Enum Design

```rust
// commands/providers/mod.rs

pub mod openai;
pub mod anthropic;
pub mod google;
pub mod xai;
pub mod openrouter;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    OpenAI,
    Anthropic,
    Google,
    Xai,
    OpenRouter,
}

impl Provider {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "openai" => Ok(Self::OpenAI),
            "anthropic" => Ok(Self::Anthropic),
            "google" => Ok(Self::Google),
            "xai" => Ok(Self::Xai),
            "openrouter" => Ok(Self::OpenRouter),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }

    pub fn keychain_account(&self) -> String {
        match self {
            Self::OpenAI => "openai_api_key",
            Self::Anthropic => "anthropic_api_key",
            Self::Google => "google_api_key",
            Self::Xai => "xai_api_key",       // matches existing keychain entry
            Self::OpenRouter => "openrouter_api_key",
        }.to_string()
    }
}
```

### Provider-Specific Differences (Why Simple URL Swap Fails)

| Aspect | OpenAI / xAI / OpenRouter | Anthropic | Google Gemini |
|--------|---------------------------|-----------|---------------|
| Endpoint | `/v1/chat/completions` | `/v1/messages` | `/v1beta/models/{model}:streamGenerateContent` |
| Auth header | `Authorization: Bearer {key}` | `x-api-key: {key}` (different header name) | `?key={key}` query param |
| System prompt | `{"role":"system","content":"..."}` in messages array | Top-level `"system"` field, NOT in messages | `"systemInstruction"` field |
| SSE token path | `choices[0].delta.content` | `content_block_delta.delta.text` | `candidates[0].content.parts[0].text` |
| SSE done signal | `data: [DONE]` | `event: message_stop` | `finishReason: "STOP"` in candidate |
| Required fields | `model`, `messages`, `stream` | `model`, `messages`, `max_tokens` (required!) | `contents`, `generationConfig` |
| Extra headers | None | `anthropic-version: 2023-06-01` | None |

**Anthropic is the most divergent.** Three unique aspects: different auth header name, system prompt not in messages, required `max_tokens` field, and completely different SSE event structure. This alone justifies the provider abstraction.

### Per-Provider Module Contract

Each provider module exports these functions (not a trait -- just convention):

```rust
// Example: commands/providers/anthropic.rs

/// Endpoint URL for streaming requests
pub fn endpoint() -> &'static str {
    "https://api.anthropic.com/v1/messages"
}

/// Build HTTP headers for this provider
pub fn headers(api_key: &str) -> Vec<(String, String)> {
    vec![
        ("x-api-key".to_string(), api_key.to_string()),
        ("anthropic-version".to_string(), "2023-06-01".to_string()),
        ("content-type".to_string(), "application/json".to_string()),
    ]
}

/// Build the request body. System prompt handled differently per provider.
pub fn build_body(model: &str, system_prompt: &str, messages: &[serde_json::Value]) -> String {
    serde_json::json!({
        "model": model,
        "system": system_prompt,  // Anthropic: top-level field
        "messages": messages,     // NO system message in array
        "max_tokens": 1024,       // Required for Anthropic
        "stream": true
    }).to_string()
}

/// Extract token from an SSE data chunk. Returns None if no token.
pub fn extract_token(data: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(data).ok()?;
    if v["type"] == "content_block_delta" {
        v["delta"]["text"].as_str().map(|s| s.to_string())
    } else {
        None
    }
}

/// Check if this SSE event signals stream completion
pub fn is_done(data: &str) -> bool {
    let v: serde_json::Value = serde_json::from_str(data).ok().unwrap_or_default();
    v["type"] == "message_stop"
}

/// Validate API key and return available models
pub async fn validate_and_fetch_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    // Anthropic: GET /v1/models with x-api-key header
    // ...
}
```

### Refactored stream_ai_response

```rust
// commands/ai.rs -- refactored

#[tauri::command]
pub async fn stream_ai_response(
    provider: String,           // NEW parameter
    query: String,
    model: String,
    context_json: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    let provider_enum = Provider::from_str(&provider)?;

    // 1. Read API key for this provider from keychain
    let account = provider_enum.keychain_account();
    let entry = keyring::Entry::new(SERVICE, &account)
        .map_err(|e| format!("Keyring error: {}", e))?;
    let api_key = entry.get_password()
        .map_err(|_| "No API key configured. Open Settings to add one.".to_string())?;

    // 2. Parse context, build system prompt (unchanged logic)
    let ctx: AppContextView = serde_json::from_str(&context_json).unwrap_or_else(/* ... */);
    let system_prompt = build_system_prompt(&ctx);

    // 3. Build messages array (system message excluded for Anthropic/Google)
    let messages = build_messages_for_provider(&provider_enum, &system_prompt, &history, &query, &ctx);

    // 4. Dispatch to provider-specific request building
    let (endpoint, headers, body) = match provider_enum {
        Provider::OpenAI => (openai::endpoint(), openai::headers(&api_key), openai::build_body(&model, &system_prompt, &messages)),
        Provider::Anthropic => (anthropic::endpoint(), anthropic::headers(&api_key), anthropic::build_body(&model, &system_prompt, &messages)),
        Provider::Google => (google::endpoint(&model), google::headers(&api_key), google::build_body(&system_prompt, &messages)),
        Provider::Xai => (xai::endpoint(), xai::headers(&api_key), xai::build_body(&model, &system_prompt, &messages)),
        Provider::OpenRouter => (openrouter::endpoint(), openrouter::headers(&api_key), openrouter::build_body(&model, &system_prompt, &messages)),
    };

    // 5. Make HTTP request
    let client = reqwest::Client::new();
    let mut req = client.post(endpoint);
    for (name, value) in &headers {
        req = req.header(name, value);
    }
    let response = req.body(body).send().await.map_err(|e| format!("Network error: {}", e))?;

    // 6. Parse SSE stream with provider-specific token extraction
    let mut stream = response.bytes_stream().eventsource();
    let timeout_duration = tokio::time::Duration::from_secs(10);

    tokio::time::timeout(timeout_duration, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    let data = &event.data;
                    // Provider-specific done check
                    let is_done = match provider_enum {
                        Provider::OpenAI | Provider::Xai | Provider::OpenRouter => data == "[DONE]",
                        Provider::Anthropic => anthropic::is_done(data),
                        Provider::Google => google::is_done(data),
                    };
                    if is_done { break; }

                    // Provider-specific token extraction
                    let token = match provider_enum {
                        Provider::OpenAI | Provider::Xai | Provider::OpenRouter => openai::extract_token(data),
                        Provider::Anthropic => anthropic::extract_token(data),
                        Provider::Google => google::extract_token(data),
                    };
                    if let Some(t) = token {
                        if !t.is_empty() {
                            on_token.send(t).map_err(|e| format!("Channel error: {}", e))?;
                        }
                    }
                }
                Err(e) => return Err(format!("Stream error: {}", e)),
            }
        }
        Ok(())
    }).await.map_err(|_| "Request timed out. Try again.".to_string())??;

    Ok(())
}
```

**Note:** OpenAI, xAI, and OpenRouter share the same SSE format (OpenAI-compatible). This means 3 of 5 providers reuse the same extraction logic. Only Anthropic and Google need custom parsers.

### Keychain Changes

```rust
// commands/keychain.rs -- parameterized

const SERVICE: &str = "com.lakshmanturlapati.cmd-k";

#[tauri::command]
pub fn save_api_key(provider: String, key: String) -> Result<(), String> {
    let account = format!("{}_api_key", provider);
    let entry = Entry::new(SERVICE, &account)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry.set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}

#[tauri::command]
pub fn get_api_key(provider: String) -> Result<Option<String>, String> {
    let account = format!("{}_api_key", provider);
    let entry = Entry::new(SERVICE, &account)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key(provider: String) -> Result<(), String> {
    let account = format!("{}_api_key", provider);
    let entry = Entry::new(SERVICE, &account)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry.delete_credential()
        .map_err(|e| format!("Failed to delete: {}", e))
}
```

**Migration safety:** `format!("{}_api_key", "xai")` produces `"xai_api_key"`, identical to the current hardcoded `ACCOUNT` constant. Existing xAI API keys remain accessible with zero migration.

### Frontend Changes for Multi-Provider

**Zustand store additions:**

```typescript
// New state in useOverlayStore:
selectedProvider: string;                   // "openai" | "anthropic" | "google" | "xai" | "openrouter"
setSelectedProvider: (p: string) => void;

// Rename type: XaiModelWithMeta -> ModelInfo
// (same shape: { id: string; label: string }, just generic name)
```

**Onboarding flow change:**

```
Current:  Step 1 (Accessibility) -> Step 2 (API Key) -> Step 3 (Model) -> Step 4 (Done)
New:      Step 1 (Accessibility) -> Step 2 (Provider) -> Step 3 (API Key) -> Step 4 (Model) -> Step 5 (Done)
```

New component: `StepProviderSelect.tsx` -- card/radio selection of 5 providers. Sets `selectedProvider` in store. Persisted to Tauri store plugin.

**Modified components:**
- `StepApiKey.tsx` -- Dynamic label: "Enter your {Provider} API key". Pass `provider` to `validate_and_fetch_models` invoke.
- `AccountTab.tsx` -- Show current provider name, dynamic label, pass `provider` to keychain invokes.
- `submitQuery()` -- Add `provider: state.selectedProvider` to `stream_ai_response` invoke args.
- `validate_and_fetch_models` IPC -- New signature: `invoke("validate_and_fetch_models", { provider, apiKey })`.

**Provider selection persistence:** Use existing `tauri-plugin-store` with keys `"selected_provider"` and `"selected_model"`.

### New IPC Command: validate_and_fetch_models (Provider-Aware)

```rust
// commands/providers/mod.rs (replaces commands/xai.rs)

#[tauri::command]
pub async fn validate_and_fetch_models(
    provider: String,
    api_key: String,
) -> Result<Vec<ModelInfo>, String> {
    let p = Provider::from_str(&provider)?;
    match p {
        Provider::OpenAI => openai::validate_and_fetch_models(&api_key).await,
        Provider::Anthropic => anthropic::validate_and_fetch_models(&api_key).await,
        Provider::Google => google::validate_and_fetch_models(&api_key).await,
        Provider::Xai => xai::validate_and_fetch_models(&api_key).await,
        Provider::OpenRouter => openrouter::validate_and_fetch_models(&api_key).await,
    }
}
```

---

## Feature 2: WSL Terminal Context

### Current Windows Terminal Detection Path

```
Hotkey fires -> capture HWND + PID
  -> detect_app_context_windows(pid)
    -> get_exe_name_for_pid(pid) -> "WindowsTerminal.exe"
    -> is_known_terminal_exe() -> true
    -> process::get_foreground_info(pid)
      -> CreateToolhelp32Snapshot -> walk process tree
      -> find shell child (powershell.exe, cmd.exe, bash.exe)
      -> get_process_cwd(shell_pid) via PEB ReadProcessMemory
    -> detect_full_with_hwnd() -> UIA text reading
```

### WSL Problem

WSL shells run inside the WSL2 VM. When a user has a WSL tab in Windows Terminal:

1. **Process tree:** Windows Terminal -> OpenConsole.exe -> wsl.exe -> (WSL namespace). The bash/zsh process inside WSL is NOT visible in the Win32 process tree. The ConPTY fallback in `process.rs` (line 798-823) finds shells parented by OpenConsole, but `wsl.exe` is not in `KNOWN_SHELL_EXES`.

2. **PEB CWD:** Reading CWD from `wsl.exe`'s PEB returns the Windows-side launcher CWD, not the user's CWD inside WSL.

3. **Shell type:** The process name is `wsl.exe`, not `bash` or `zsh`.

### New Component: `terminal/wsl.rs`

```rust
// terminal/wsl.rs

/// Detect if a process chain involves WSL
pub fn is_wsl_process(exe_name: &str) -> bool {
    matches!(exe_name.to_lowercase().as_str(), "wsl.exe" | "wslhost.exe")
}

/// Get WSL terminal context by shelling into WSL from the Windows side.
/// Each command has an independent timeout to avoid blocking the overlay.
pub fn get_wsl_context() -> Option<WslContext> {
    // 1. Shell type: query default shell inside WSL
    let shell_type = wsl_command(&["-e", "sh", "-c", "basename \"$SHELL\""], 200)?;

    // 2. CWD: get working directory of the most recent user shell
    //    Strategy: find the newest interactive shell PID, read its /proc/{pid}/cwd
    let cwd = wsl_command(
        &["-e", "sh", "-c",
          "readlink /proc/$(ps -o pid= -t $(tty) 2>/dev/null | tail -1 | tr -d ' ')/cwd 2>/dev/null || pwd"],
        300
    );

    Some(WslContext {
        shell_type: Some(shell_type),
        cwd,
        running_process: None, // Could be extended later
    })
}

/// Run a wsl.exe command with a hard timeout.
/// Returns None on timeout or failure.
fn wsl_command(args: &[&str], timeout_ms: u64) -> Option<String> {
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::time::Duration;

    let mut child = Command::new("wsl.exe")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .ok()?;

    let (tx, rx) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let output = child.wait_with_output();
        let _ = tx.send(output);
    });

    match rx.recv_timeout(Duration::from_millis(timeout_ms)) {
        Ok(Ok(output)) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        _ => {
            // Timeout or failure -- don't leak the thread
            drop(handle);
            None
        }
    }
}

pub struct WslContext {
    pub shell_type: Option<String>,
    pub cwd: Option<String>,
    pub running_process: Option<String>,
}
```

### Integration Point: `terminal/mod.rs`

```rust
// In detect_app_context_windows(), modify the process tree walk:

#[cfg(target_os = "windows")]
fn detect_app_context_windows(previous_app_pid: i32, _pre_captured_text: Option<String>) -> Option<AppContext> {
    // ... existing exe detection ...

    // Walk process tree to find shell
    let proc_info = process::get_foreground_info(previous_app_pid);

    // NEW: Check if process tree contains WSL
    let has_wsl = check_for_wsl_in_tree(previous_app_pid);

    let terminal = if has_wsl {
        // WSL session: shell into WSL for real context
        match wsl::get_wsl_context() {
            Some(wsl_ctx) => Some(TerminalContext {
                shell_type: wsl_ctx.shell_type,
                cwd: wsl_ctx.cwd,
                visible_output: None, // UIA fills this later
                running_process: wsl_ctx.running_process,
            }),
            None => {
                // WSL detected but context reading failed
                // Fall back to "bash" default so terminal mode is used
                Some(TerminalContext {
                    shell_type: Some("bash".to_string()),
                    cwd: None,
                    visible_output: None,
                    running_process: None,
                })
            }
        }
    } else if has_shell {
        // ... existing native Windows shell handling ...
    };

    // ... rest unchanged (UIA text reading, app context assembly) ...
}

/// Check if the process tree rooted at pid contains wsl.exe or wslhost.exe
fn check_for_wsl_in_tree(app_pid: i32) -> bool {
    // Walk process tree looking for wsl.exe in the chain
    // Uses existing CreateToolhelp32Snapshot infrastructure
    // Also check ConPTY children (OpenConsole -> wsl.exe pattern)
}
```

### detect_windows.rs Changes

```rust
// Add to KNOWN_TERMINAL_EXES:
"wsl.exe",       // WSL direct launch

// Add to KNOWN_SHELL_EXES:
// wsl.exe is NOT a shell, but it IS a process that hosts shells
// Don't add it here -- instead detect it specially in wsl.rs
```

### System Prompt for WSL

The AI needs to know it should generate Linux commands, not Windows commands, when the user is in WSL.

```rust
// In commands/ai.rs, modify system prompt selection:

// Detect WSL from context: CWD starts with "/" (Linux path) not "C:\" (Windows path)
let is_wsl = ctx.terminal.as_ref()
    .and_then(|t| t.cwd.as_ref())
    .map(|cwd| cwd.starts_with('/'))
    .unwrap_or(false);

let system_prompt = if is_wsl {
    // WSL: Linux commands even though running on Windows
    TERMINAL_SYSTEM_PROMPT_TEMPLATE_WSL.replace("{shell_type}", shell_type)
} else if is_terminal_mode {
    TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type)
} else {
    ASSISTANT_SYSTEM_PROMPT.to_string()
};
```

```rust
#[cfg(target_os = "windows")]
const TERMINAL_SYSTEM_PROMPT_TEMPLATE_WSL: &str =
    "You are a terminal command generator for Linux (WSL on Windows). Given the user's task \
     description and terminal context, output ONLY the exact command(s) to run. No explanations, \
     no markdown, no code fences. Just the raw command(s). If multiple commands are needed, \
     separate them with && or use pipes. Prefer common POSIX tools (grep, find, sed, awk). \
     The user is running WSL (Windows Subsystem for Linux) with {shell_type} shell.";
```

### Supported WSL Hosts

| Host | Detection | CWD/Shell Source | Visible Text |
|------|-----------|-----------------|--------------|
| Windows Terminal | ConPTY tree has wsl.exe child of OpenConsole | `wsl.exe -e` commands | UIA (works) |
| VS Code Remote-WSL | Code.exe with wsl.exe in tree | `wsl.exe -e` commands | UIA (limited) |
| Standalone wsl.exe | Direct wsl.exe as foreground process | `wsl.exe -e` commands | UIA on conhost |
| Cursor | Same as VS Code pattern | `wsl.exe -e` commands | UIA (limited) |

### Visible Output for WSL

UIA text reading from Windows Terminal already captures WSL terminal output correctly -- it reads what is displayed in the terminal regardless of whether the underlying session is native or WSL. No changes needed for visible output capture.

---

## Feature 3: Auto-Updater

### Architecture

Uses `tauri-plugin-updater` (official Tauri v2 plugin). The plugin checks a JSON manifest at a URL, compares versions, downloads the update, and installs it.

### New Files

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/updater.rs` | Rust IPC commands for check/install |
| `src/components/UpdateBanner.tsx` | Non-blocking update notification in overlay |
| `.github/workflows/release.yml` | Modified to generate `latest.json` manifest |

### Cargo.toml Addition

```toml
[dependencies]
tauri-plugin-updater = "2"
```

### tauri.conf.json Addition

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/user/cmd-k/releases/latest/download/latest.json"
      ],
      "pubkey": "<generated-public-key>"
    }
  }
}
```

### Plugin Initialization (lib.rs)

```rust
// Add to builder chain in lib.rs:
.plugin(tauri_plugin_updater::Builder::new().build())
```

This goes alongside the existing plugins (global-shortcut, positioner, store, http).

### Rust IPC Commands

```rust
// commands/updater.rs
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub body: Option<String>,
}

/// Check if an update is available. Returns None if current version is latest.
/// Non-blocking -- called on app launch as fire-and-forget.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater()
        .map_err(|e| format!("Updater init error: {}", e))?;

    match updater.check().await {
        Ok(Some(update)) => Ok(Some(UpdateInfo {
            version: update.version.clone(),
            body: update.body.clone(),
        })),
        Ok(None) => Ok(None),
        Err(e) => {
            eprintln!("[updater] check failed: {}", e);
            Ok(None) // Fail silently -- updates are non-critical
        }
    }
}

/// Download and install the pending update.
/// On macOS: replaces app bundle, prompts relaunch.
/// On Windows: downloads installer, runs on next launch (passive install mode).
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let updater = app.updater()
        .map_err(|e| format!("Updater error: {}", e))?;

    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        update.download_and_install(|_, _| {}, || {}).await
            .map_err(|e| format!("Install failed: {}", e))?;
    }
    Ok(())
}
```

### Frontend Integration

**App startup check (in App.tsx or equivalent setup):**

```typescript
// Fire-and-forget update check on mount
useEffect(() => {
    invoke<UpdateInfo | null>("check_for_update").then(info => {
        if (info) {
            useOverlayStore.getState().setUpdateAvailable(info);
        }
    }).catch(() => {}); // Silent fail
}, []);
```

**Zustand store additions:**

```typescript
updateAvailable: UpdateInfo | null;
setUpdateAvailable: (info: UpdateInfo | null) => void;
isUpdating: boolean;
```

**Update notification options (pick one):**
1. **Tray menu entry:** "Update available: v0.2.7" in the system tray menu. Click triggers install. Least intrusive.
2. **Banner in overlay:** Subtle bar at top of overlay when update is available. "Update available. [Install]"
3. **Settings tab indicator:** Dot badge on settings tab. Update controls in settings.

**Recommendation:** Tray menu entry (option 1) + settings tab indicator (option 3). The overlay should not be cluttered with update UI -- it is a focused command interface.

### CI/CD Changes for Auto-Updater

The updater requires:

1. **Signing keypair generation:** Run `npx tauri signer generate -w ~/.tauri/cmd-k.key` to generate a private key. Store private key as CI secret `TAURI_SIGNING_PRIVATE_KEY`. Public key goes in `tauri.conf.json` `plugins.updater.pubkey`.

2. **Build with signing:** Set `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` env vars during `pnpm tauri build`. This produces `.sig` signature files alongside installers.

3. **Generate latest.json manifest:** Post-build step assembles the manifest with version, URLs, and signatures.

4. **Upload manifest to release:** Add `latest.json` to the GitHub Release artifacts.

**Critical note on macOS DMG:** The current build uses a custom `scripts/build-dmg.sh` that produces a signed+notarized DMG outside of Tauri's build system. The updater plugin expects Tauri's built-in updater format. Two options:

- **Option A:** Switch macOS build to `pnpm tauri build` (produces .app.tar.gz with .sig), add custom signing/notarization as post-build step. The updater downloads the .tar.gz, not the DMG.
- **Option B:** Keep custom DMG for distribution, generate .sig separately for the DMG. This requires verifying that the updater plugin can handle DMG format.

**Recommendation:** Option A. Use `pnpm tauri build` for the updater-compatible format (.app.tar.gz + .sig). Keep the DMG as a separate distribution artifact for manual downloads. The updater uses the .tar.gz; the GitHub Release includes both the DMG (for first-time installs) and the .tar.gz + .sig (for updates).

### Release Workflow Additions

```yaml
# In release.yml, add after build steps:

- name: Generate latest.json manifest
  shell: bash
  run: |
    cat > latest.json << EOF
    {
      "version": "${{ env.VERSION }}",
      "notes": "CMD+K ${{ github.ref_name }}",
      "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
      "platforms": {
        "darwin-universal": {
          "url": "https://github.com/.../releases/download/${{ github.ref_name }}/CMD+K.app.tar.gz",
          "signature": "$(cat CMD+K.app.tar.gz.sig)"
        },
        "windows-x86_64": {
          "url": "https://github.com/.../releases/download/${{ github.ref_name }}/CMD+K-setup.exe",
          "signature": "$(cat CMD+K-setup.exe.sig)"
        }
      }
    }
    EOF

- name: Upload latest.json to release
  # Upload alongside other artifacts
```

---

## Component Boundaries Summary

| Component | Responsibility | Status | Communicates With |
|-----------|---------------|--------|-------------------|
| `commands/providers/mod.rs` | Provider enum, ModelInfo, dispatch | **NEW** | ai.rs, keychain.rs |
| `commands/providers/openai.rs` | OpenAI request/response handling | **NEW** | providers/mod.rs |
| `commands/providers/anthropic.rs` | Anthropic request/response handling | **NEW** | providers/mod.rs |
| `commands/providers/google.rs` | Google Gemini request/response handling | **NEW** | providers/mod.rs |
| `commands/providers/xai.rs` | xAI request/response (from current ai.rs+xai.rs) | **NEW** (extracted) | providers/mod.rs |
| `commands/providers/openrouter.rs` | OpenRouter request/response handling | **NEW** | providers/mod.rs |
| `commands/ai.rs` | Stream orchestration, prompt building | **MODIFIED** | providers/, keychain |
| `commands/xai.rs` | -- | **DELETED** (moved to providers/xai.rs) | -- |
| `commands/keychain.rs` | Per-provider key storage | **MODIFIED** (add `provider` param) | providers/ |
| `commands/updater.rs` | Update check and install IPC | **NEW** | tauri-plugin-updater |
| `terminal/wsl.rs` | WSL detection and context via wsl.exe | **NEW** | terminal/mod.rs |
| `terminal/mod.rs` | Detection orchestration | **MODIFIED** (add WSL branch) | wsl.rs |
| `terminal/detect_windows.rs` | Exe classification | **MODIFIED** (add WSL patterns) | terminal/mod.rs |
| `src/store/index.ts` | Zustand state | **MODIFIED** (add provider, update, rename types) | All frontend |
| `StepProviderSelect.tsx` | Provider picker in onboarding | **NEW** | store |
| `StepApiKey.tsx` | Dynamic provider-aware key input | **MODIFIED** | store, keychain |
| `AccountTab.tsx` | Provider-aware key management | **MODIFIED** | store, keychain |
| `UpdateBanner.tsx` or tray entry | Update notification | **NEW** | updater commands |
| `lib.rs` | Plugin initialization | **MODIFIED** (add updater plugin) | -- |
| `Cargo.toml` | Dependencies | **MODIFIED** (add tauri-plugin-updater) | -- |
| `tauri.conf.json` | Updater config | **MODIFIED** (add plugins.updater) | -- |
| `.github/workflows/release.yml` | CI/CD | **MODIFIED** (signing, latest.json) | -- |

---

## Data Flow: Complete Provider Request Lifecycle

```
User types query -> submitQuery(query) in Zustand store
  |
  v
Read selectedProvider and selectedModel from store
  |
  v
invoke("stream_ai_response", { provider, query, model, contextJson, history, onToken })
  |
  v
[Rust] ai.rs::stream_ai_response
  |-- Provider::from_str(provider) -> Provider::Anthropic (example)
  |-- Read "anthropic_api_key" from keychain
  |-- Build system prompt (WSL-aware if CWD starts with "/")
  |-- anthropic::build_body(model, system_prompt, messages)
  |     -> { model, system: "...", messages: [...], max_tokens: 1024, stream: true }
  |-- anthropic::headers(api_key)
  |     -> [("x-api-key", key), ("anthropic-version", "2023-06-01"), ...]
  |-- POST to "https://api.anthropic.com/v1/messages"
  |-- SSE loop:
  |     for each event:
  |       anthropic::is_done(data)? -> break
  |       anthropic::extract_token(data) -> "content_block_delta".delta.text
  |       on_token.send(token)
  |
  v
[Frontend] onToken callback -> set({ streamingText: fullText })
  |
  v
Stream complete -> update turnHistory, persist to Rust history, check destructive
```

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: URL-Only Provider Switching
**What:** Parameterize just the endpoint URL and keep the same request body / SSE parsing.
**Why bad:** Anthropic uses `x-api-key` header (not `Authorization: Bearer`), requires `max_tokens`, puts system prompt outside messages, and has completely different SSE events (`content_block_delta` not `choices[0].delta`). Google uses query param auth and different response structure. Would break 2 of 5 providers.
**Instead:** Per-provider `build_body()`, `headers()`, `extract_token()`, `is_done()` functions.

### Anti-Pattern 2: WSL Detection via Environment Variables
**What:** Reading `$WSL_DISTRO_NAME` or checking `/proc/version` to detect WSL.
**Why bad:** Those are only available INSIDE the WSL namespace. The Rust code runs on the Windows side and cannot read WSL-internal environment variables.
**Instead:** Detect WSL by examining the Windows process tree for `wsl.exe` / `wslhost.exe`, then shell into WSL via `wsl.exe -e` commands for context.

### Anti-Pattern 3: Blocking Update Check on Launch
**What:** Synchronous network call to check for updates during app startup.
**Why bad:** The overlay must appear instantly on hotkey press. A slow/failed network request would block the main thread and delay the first overlay show.
**Instead:** Fire-and-forget async update check. Store result. Show notification on next overlay open or in tray menu.

### Anti-Pattern 4: Single API Key Slot
**What:** One keychain entry that changes based on the selected provider.
**Why bad:** Users may want multiple providers configured and switch without re-entering keys. A single slot forces key re-entry on every provider switch.
**Instead:** Per-provider keychain entries (`openai_api_key`, `anthropic_api_key`, etc.). All can exist simultaneously.

### Anti-Pattern 5: Abstract Trait Object for Providers
**What:** `Box<dyn AiProvider>` with dynamic dispatch.
**Why bad:** Async methods require boxing futures. Only 5 fixed variants. Enum match gives exhaustiveness checking at compile time. No runtime allocation. Simpler error messages.
**Instead:** `Provider` enum with `match` dispatch.

---

## Build Order (Dependency-Driven)

### Phase 1: Provider Abstraction (Rust-side only)
1. Create `commands/providers/` module with Provider enum
2. Implement xai.rs provider (extract from current ai.rs + xai.rs)
3. Implement openai.rs provider (near-identical to xAI, same SSE format)
4. Implement anthropic.rs provider (most divergent -- different everything)
5. Implement google.rs provider (different auth and response format)
6. Implement openrouter.rs provider (identical to OpenAI format)
7. Refactor `commands/ai.rs` to accept `provider` parameter and delegate
8. Refactor `commands/keychain.rs` to accept `provider` parameter
9. Delete `commands/xai.rs`
10. **Backward compatibility:** Default to "xai" if provider param is missing (transitional)

### Phase 2: Frontend Multi-Provider
1. Add `selectedProvider` to Zustand store, persist to Tauri store
2. Rename `XaiModelWithMeta` -> `ModelInfo` throughout frontend
3. Create `StepProviderSelect.tsx` onboarding step
4. Update `StepApiKey.tsx` for dynamic provider labels
5. Update `AccountTab.tsx` for provider-aware key management
6. Update `submitQuery()` to pass `provider` in invoke calls
7. Update `validate_and_fetch_models` invoke to pass `provider`
8. Remove "xai" default -- require explicit provider selection

### Phase 3: WSL Terminal Context (independent of Phases 1-2)
1. Create `terminal/wsl.rs` with `is_wsl_process()` and `get_wsl_context()`
2. Add WSL detection helper (`check_for_wsl_in_tree`) using existing snapshot infrastructure
3. Integrate WSL branch into `detect_app_context_windows()`
4. Add WSL system prompt template to `commands/ai.rs`
5. Test: Windows Terminal WSL tab, VS Code Remote-WSL, standalone `wsl.exe`, Cursor

### Phase 4: Auto-Updater (independent of Phases 1-3)
1. Generate signing keypair (`npx tauri signer generate`)
2. Add public key to `tauri.conf.json` updater config
3. Add `tauri-plugin-updater` to Cargo.toml and plugin chain in lib.rs
4. Create `commands/updater.rs` with check/install IPC commands
5. Modify CI workflow: add signing env vars, generate `.sig` files, build `latest.json`
6. Add update notification (tray menu entry + settings indicator)
7. Wire async update check into app startup

### Dependency Graph

```
Phase 1 (Provider Rust)
    |
    +---> Phase 2 (Provider Frontend -- depends on Phase 1 IPC)

Phase 3 (WSL -- independent, can run in parallel with 1+2)

Phase 4 (Auto-updater -- independent, can run in parallel with all)
```

**Phases 3 and 4 can be developed in parallel with each other and with Phase 2.** Phase 2 depends on Phase 1 completing first (frontend needs the new Rust IPC signatures).

---

## Sources

- Codebase analysis: `src-tauri/src/commands/ai.rs` -- current xAI-hardcoded streaming (lines 8, 259, 307-308)
- Codebase analysis: `src-tauri/src/commands/keychain.rs` -- current single-account key storage
- Codebase analysis: `src-tauri/src/commands/xai.rs` -- xAI-specific model validation
- Codebase analysis: `src-tauri/src/terminal/mod.rs` -- detection orchestration with WSL gap
- Codebase analysis: `src-tauri/src/terminal/process.rs` -- Windows process tree walk, ConPTY fallback
- Codebase analysis: `src-tauri/src/terminal/detect_windows.rs` -- Windows exe classification (no WSL entries)
- Codebase analysis: `src/store/index.ts` -- Zustand state with xAI-specific types
- Codebase analysis: `src/components/Onboarding/StepApiKey.tsx` -- xAI-hardcoded labels
- Codebase analysis: `.github/workflows/release.yml` -- current CI pipeline for updater integration
- Architecture inference: Provider API differences (OpenAI, Anthropic, Google Gemini SSE formats) -- MEDIUM confidence, verify against current API docs during implementation
- Architecture inference: WSL process tree model (ConPTY -> OpenConsole -> wsl.exe) -- MEDIUM confidence, verify via runtime testing on Windows
- Architecture inference: tauri-plugin-updater v2 setup and latest.json format -- MEDIUM confidence, verify against current plugin documentation during implementation
