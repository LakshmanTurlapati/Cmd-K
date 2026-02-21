# Architecture Patterns

**Domain:** Tauri-based macOS overlay application
**Researched:** 2026-02-21
**Confidence:** HIGH

## Executive Summary

A Tauri v2 macOS overlay app with global hotkey, terminal context reading, and AI streaming requires a multi-process architecture with clear boundaries between the Rust backend (system integration) and web frontend (UI/streaming). The architecture follows Tauri's process model with a Core process handling OS integration and WebView processes for UI rendering, using NSPanel for overlay behavior and IPC for communication.

## Recommended Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     macOS System Layer                       │
│  (Accessibility API, NSPanel, Global Hotkey, AppleScript)   │
└─────────────────────────────────────────────────────────────┘
                            ▲
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                  Tauri Core Process (Rust)                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Hotkey     │  │   Terminal   │  │   AI Streaming   │  │
│  │   Manager    │  │   Reader     │  │   Client         │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Window     │  │   State      │  │   AppleScript    │  │
│  │   Manager    │  │   Manager    │  │   Bridge         │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
└───────────────────────────┬─────────────────────────────────┘
                            │
                    Tauri IPC (Commands/Events)
                            │
                            ▼
┌───────────────────────────┴─────────────────────────────────┐
│              WebView Process (Web Frontend)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │   Overlay    │  │   AI Stream  │  │   Command        │  │
│  │   UI         │  │   Renderer   │  │   Palette        │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────────────────────────────────────────────┐  │
│  │            React/Vue/Svelte Components               │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Component Boundaries

| Component | Layer | Responsibility | Communicates With |
|-----------|-------|---------------|-------------------|
| **Hotkey Manager** | Rust Core | Register/unregister global shortcuts (Cmd+K), trigger window show | Window Manager |
| **Window Manager** | Rust Core | Create/show/hide NSPanel overlay, manage window state | Hotkey Manager, WebView |
| **Terminal Reader** | Rust Core | Read active terminal context (cwd, selected text) via Accessibility API + process inspection | State Manager, WebView |
| **AI Streaming Client** | Rust Core | HTTP client for xAI API, handle SSE streaming | WebView (via Events) |
| **State Manager** | Rust Core | Manage app state (settings, terminal context, AI session) with Mutex | All Rust components |
| **AppleScript Bridge** | Rust Core | Execute AppleScript for pasting into terminal | WebView (triggered via Command) |
| **Overlay UI** | WebView | Render transparent overlay window, handle user input | All Rust components |
| **AI Stream Renderer** | WebView | Receive streaming tokens via Events, render markdown | AI Streaming Client |
| **Command Palette** | WebView | Search/filter UI, command suggestions | State Manager |

## Data Flow

### Primary Flow: Hotkey → Overlay → AI → Terminal

```
1. User presses Cmd+K anywhere on macOS
   ↓
2. Hotkey Manager (Rust) receives global shortcut event
   ↓
3. Window Manager (Rust) triggers NSPanel show + focus
   ↓
4. Terminal Reader (Rust) reads active app context:
   - Get active window process ID (active-win-pos-rs)
   - Read terminal text via Accessibility API
   - Inspect process for cwd (libproc-rs)
   ↓
5. State Manager (Rust) stores context in shared state (Mutex)
   ↓
6. WebView receives "show" event with context payload
   ↓
7. Overlay UI (WebView) displays with context pre-filled
   ↓
8. User types query, presses Enter
   ↓
9. WebView invokes "generate_command" Command (Rust)
   ↓
10. AI Streaming Client (Rust) sends request to xAI Grok
    ↓
11. For each SSE token received:
    - AI Client emits "ai_token" Event
    - WebView AI Stream Renderer appends to UI
    ↓
12. When complete, user presses Cmd+Enter to accept
    ↓
13. WebView invokes "paste_to_terminal" Command (Rust)
    ↓
14. AppleScript Bridge (Rust) executes script to:
    - Focus original terminal window
    - Paste generated command
    ↓
15. Window Manager (Rust) hides overlay NSPanel
```

### Secondary Flow: System Tray Menu

```
1. App runs as menu bar daemon (no dock icon)
   ↓
2. User clicks menu bar icon
   ↓
3. System Tray shows menu (Settings, Quit)
   ↓
4. "Settings" → Window Manager creates settings NSPanel
```

## macOS-Specific Architecture

### NSPanel Integration (tauri-nspanel)

Use `tauri-nspanel` plugin for proper overlay behavior:

```rust
use tauri_nspanel::{ManagerExt, PanelBuilder};

tauri::Builder::default()
  .setup(|app| {
    let panel = PanelBuilder::new("main")
      .is_floating_panel(true)  // Float above other windows
      .can_become_key_window(true)  // Accept keyboard input
      .build(app)?;
    Ok(())
  })
```

**Why NSPanel over standard window:**
- Proper fullscreen app overlay behavior (works over Terminal fullscreen)
- System-appropriate panel styling
- Automatic focus management
- Integration with macOS window levels

**Confidence:** HIGH (verified via [tauri-nspanel docs](https://github.com/ahkohd/tauri-nspanel))

### Accessibility API Integration

Terminal context reading requires three techniques:

**1. Active Window Detection (active-win-pos-rs)**
```rust
use active_win_pos_rs::get_active_window;

let active_window = get_active_window()?;
// Returns: process_id, window_title, position
```

**2. Accessibility API for Text (via Rust bindings)**
```rust
// Use macOS Accessibility API through FFI or wrapper crate
// Read AXValue, AXSelectedText, AXTextArea from active app
// REQUIRES: Accessibility permissions granted by user
```

**Cache Strategy:** Accessibility API can return nil during transitions. Cache last known values and combine with process inspection for resilience.

**Confidence:** MEDIUM (pattern documented in [Shellporter 2026 blog](https://www.marcogomiero.com/posts/2026/building-shellporter/), but requires custom implementation)

**3. Process Inspection for CWD (libproc-rs)**
```rust
use libproc::libproc::proc_pid::pidinfo;

// Get cwd of terminal process
let cwd = get_process_cwd(active_window.process_id)?;
```

**Combined Pattern:**
```rust
#[derive(Default)]
struct TerminalContext {
    process_id: Option<u32>,
    window_title: String,
    selected_text: Option<String>,  // via Accessibility API
    cwd: Option<PathBuf>,           // via process inspection
}

// Cache with 100ms TTL to handle API transitions
struct CachedContext {
    context: TerminalContext,
    last_updated: Instant,
}
```

**Confidence:** HIGH (libraries verified: [active-win-pos-rs](https://crates.io/crates/active-win-pos-rs), [libproc-rs](https://github.com/andrewdavidmackenzie/libproc-rs))

### AppleScript Bridge for Pasting

Terminal pasting requires AppleScript execution from Rust:

```rust
use std::process::Command;

fn paste_to_terminal(text: &str, terminal_app: &str) -> Result<()> {
    let script = format!(r#"
        tell application "{}"
            activate
            tell application "System Events"
                keystroke "{}"
            end tell
        end tell
    "#, terminal_app, text.replace("\"", "\\\""));

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;
    Ok(())
}
```

**Compatibility:**
- Terminal.app: Standard AppleScript
- iTerm2: Enhanced scripting API ([iTerm2 scripting docs](https://iterm2.com/documentation-scripting.html))
- Other terminals: System Events fallback

**Security:** Requires Accessibility permissions (same as context reading)

**Confidence:** HIGH (AppleScript is stable macOS API, pattern widely used)

### Global Hotkey Registration

Use `tauri-plugin-global-shortcut` in Rust setup:

```rust
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, Code, Modifiers};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::init())
        .setup(|app| {
            let shortcut = Shortcut::new(Some(Modifiers::META), Code::KeyK);

            app.global_shortcut().on_shortcut(shortcut, |app, _event| {
                // Show overlay window
                let window = app.get_webview_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running app");
}
```

**Why Rust setup (not frontend):**
- Works even when no window is visible
- Guaranteed to be registered at app start
- Faster response (no IPC overhead)

**Confidence:** HIGH (verified via [Tauri global-shortcut plugin docs](https://v2.tauri.app/plugin/global-shortcut/))

## Tauri IPC Patterns

### Commands (Frontend → Backend, Request-Response)

Use for operations that return a result:

```rust
// Rust backend
#[tauri::command]
async fn generate_command(
    query: String,
    context: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<String, String> {
    let ai_client = state.lock().unwrap().ai_client.clone();
    // Return final result (not streaming)
    ai_client.generate(query, context).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn paste_to_terminal(text: String) -> Result<(), String> {
    paste_via_applescript(&text)
        .map_err(|e| e.to_string())
}
```

```typescript
// Frontend
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('generate_command', {
  query: userInput,
  context: terminalContext,
});
```

**When to use Commands:**
- One-time operations (paste, save settings)
- Operations needing return values
- Error handling with Result types

### Events (Bidirectional, Fire-and-Forget)

Use for streaming and notifications:

```rust
// Rust backend emits events
use tauri::Emitter;

async fn stream_ai_response(app: AppHandle, query: String) {
    let mut stream = ai_client.stream(query).await;

    while let Some(token) = stream.next().await {
        app.emit("ai_token", token).unwrap();
    }

    app.emit("ai_complete", ()).unwrap();
}
```

```typescript
// Frontend listens to events
import { listen } from '@tauri-apps/api/event';

await listen<string>('ai_token', (event) => {
  appendToResponse(event.payload);
});

await listen('ai_complete', () => {
  enableSubmitButton();
});
```

**When to use Events:**
- Streaming data (AI tokens)
- State change notifications
- Progress updates
- Doesn't need return value

**Confidence:** HIGH (verified via [Tauri IPC docs](https://v2.tauri.app/concept/inter-process-communication/))

### State Management Pattern

Use `Mutex` for shared state between commands:

```rust
use std::sync::Mutex;
use tauri::{Builder, Manager, State};

#[derive(Default)]
struct AppState {
    terminal_context: TerminalContext,
    ai_session: Option<String>,
    settings: AppSettings,
}

fn main() {
    Builder::default()
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            generate_command,
            paste_to_terminal,
            get_terminal_context,
        ])
        .run(tauri::generate_context!())
        .unwrap();
}

#[tauri::command]
fn get_terminal_context(
    state: State<'_, Mutex<AppState>>,
) -> TerminalContext {
    let state = state.lock().unwrap();
    state.terminal_context.clone()
}
```

**Pattern for async operations:**
- Use `tokio::sync::Mutex` if holding lock across `.await`
- Use `std::sync::Mutex` for synchronous access
- Keep locks short-lived to avoid blocking

**Confidence:** HIGH (verified via [Tauri state management docs](https://v2.tauri.app/develop/state-management/))

## AI Streaming Architecture

### SSE Client in Rust

Use `reqwest-eventsource` for streaming responses:

```rust
use reqwest_eventsource::{Event, EventSource};
use futures::StreamExt;

async fn stream_grok_response(
    query: String,
    app: AppHandle,
) -> Result<()> {
    let mut es = EventSource::new(
        reqwest::Client::new()
            .post("https://api.x.ai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": "grok-2",
                "messages": [{"role": "user", "content": query}],
                "stream": true,
            }))
    )?;

    while let Some(event) = es.next().await {
        match event {
            Ok(Event::Message(msg)) => {
                let token = parse_sse_data(&msg.data)?;
                app.emit("ai_token", token)?;
            }
            Ok(Event::Open) => {},
            Err(e) => {
                app.emit("ai_error", e.to_string())?;
                break;
            }
        }
    }

    app.emit("ai_complete", ())?;
    Ok(())
}
```

**Why SSE over WebSocket:**
- xAI/Grok API uses SSE (Server-Sent Events)
- Simpler unidirectional streaming
- Automatic reconnection with `reqwest-eventsource`
- Better proxy compatibility

**Confidence:** HIGH (libraries verified: [reqwest-eventsource](https://docs.rs/reqwest-eventsource/), SSE pattern documented in [2026 AI API design](https://learnwithparam.com/blog/streaming-at-scale-sse-websockets-real-time-ai-apis))

### Frontend Streaming Renderer

```typescript
import { listen } from '@tauri-apps/api/event';
import { marked } from 'marked';

let responseBuffer = '';

await listen<string>('ai_token', (event) => {
  responseBuffer += event.payload;
  // Render incrementally with markdown
  responseElement.innerHTML = marked.parse(responseBuffer);
  // Auto-scroll
  responseElement.scrollTop = responseElement.scrollHeight;
});

await listen('ai_complete', () => {
  // Enable actions (copy, paste to terminal)
  enableActions();
});
```

**Pattern:** Accumulate tokens in buffer, re-render markdown on each token (fast enough for real-time feel)

## Background Daemon Architecture

### System Tray with No Dock Icon

```rust
use tauri::{
    Builder, CustomMenuItem, SystemTray, SystemTrayMenu,
    SystemTrayMenuItem, SystemTrayEvent,
};

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let settings = CustomMenuItem::new("settings".to_string(), "Settings");
    let tray_menu = SystemTrayMenu::new()
        .add_item(settings)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    Builder::default()
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| {
            match event {
                SystemTrayEvent::MenuItemClick { id, .. } => {
                    match id.as_str() {
                        "quit" => std::process::exit(0),
                        "settings" => {
                            // Show settings window
                            let window = app.get_window("settings").unwrap();
                            window.show().unwrap();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .unwrap();
}
```

**tauri.conf.json:**
```json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.13",
      "frameworks": [],
      "exceptionDomain": "",
      "signingIdentity": null,
      "entitlements": null,
      "providerShortName": null,
      "dockIcon": false  // Hide from dock
    }
  }
}
```

**Confidence:** HIGH (pattern documented in [Tauri system tray docs](https://v2.tauri.app/learn/system-tray/), [menubar app examples](https://github.com/ahkohd/tauri-macos-menubar-app-example))

## Build Order & Dependencies

### Phase Structure Recommendation

**Phase 1: Basic Overlay (No Dependencies)**
- Tauri setup with NSPanel
- Global hotkey registration
- Show/hide window on Cmd+K
- Basic transparent UI

**Phase 2: Terminal Context (Depends on Phase 1)**
- Implement active window detection
- Add Accessibility API integration
- Process inspection for cwd
- Display context in UI

**Phase 3: AI Integration (Depends on Phase 1)**
- HTTP client setup
- SSE streaming with reqwest-eventsource
- Event-based token streaming to UI
- Markdown rendering

**Phase 4: Terminal Pasting (Depends on Phase 2 + 3)**
- AppleScript bridge
- Paste command integration
- Window focus restoration

**Phase 5: System Tray & Settings (Depends on all)**
- Menu bar integration
- Settings UI
- Persistent configuration

### Component Implementation Order

```
1. Window Management (Rust)
   ├── NSPanel setup with tauri-nspanel
   ├── Global hotkey with tauri-plugin-global-shortcut
   └── Show/hide commands

2. State Management (Rust)
   ├── Define AppState struct
   ├── Mutex wrapper
   └── Access from commands

3. Terminal Reader (Rust)
   ├── Active window detection (active-win-pos-rs)
   ├── Process inspection (libproc-rs)
   └── Accessibility API bindings (custom FFI)

4. UI Components (Frontend)
   ├── Overlay layout (transparent, centered)
   ├── Command palette input
   └── Context display

5. AI Streaming (Rust)
   ├── HTTP client setup (reqwest)
   ├── SSE stream handling (reqwest-eventsource)
   └── Event emission to frontend

6. Stream Renderer (Frontend)
   ├── Event listener setup
   ├── Markdown rendering (marked.js)
   └── Auto-scroll

7. AppleScript Bridge (Rust)
   ├── Script execution
   ├── Terminal app detection
   └── Paste command

8. System Tray (Rust)
   ├── Tray icon + menu
   ├── Settings window
   └── Menu handlers
```

### Critical Path

```
NSPanel + Hotkey → State Management → Terminal Reader → AI Streaming → Pasting
(Week 1)           (Week 1)           (Week 2)          (Week 2-3)      (Week 3)
```

Parallel tracks:
- UI development can proceed alongside Rust backend
- System tray is independent, can be last

## Patterns to Follow

### Pattern 1: Event-Driven Window Management
**What:** Use global hotkey to emit app event, Window Manager listens and shows panel
**When:** Any global shortcut interaction
**Why:** Decouples input handling from window logic
**Example:**
```rust
// Hotkey handler
app.global_shortcut().on_shortcut(shortcut, |app, _| {
    app.emit("show_overlay", ()).unwrap();
});

// Window manager listens
app.listen("show_overlay", |_| {
    let window = app.get_window("main").unwrap();
    window.show().unwrap();
});
```

### Pattern 2: Context Caching with Fallback
**What:** Cache terminal context, use fallback strategies when API fails
**When:** Reading active window/terminal state
**Why:** Accessibility API unreliable during transitions
**Example:**
```rust
struct TerminalReader {
    cache: Arc<Mutex<Option<TerminalContext>>>,
    cache_ttl: Duration,
}

impl TerminalReader {
    fn read_context(&self) -> TerminalContext {
        // Try Accessibility API
        if let Ok(ctx) = self.read_via_accessibility() {
            self.update_cache(ctx.clone());
            return ctx;
        }

        // Fallback to cache
        if let Some(cached) = self.get_cached() {
            return cached;
        }

        // Fallback to process inspection only
        self.read_via_process_inspection()
    }
}
```

### Pattern 3: Streaming with Backpressure
**What:** Accumulate tokens in Rust, emit in batches to avoid event flooding
**When:** Streaming AI responses
**Why:** Too many events can overwhelm IPC
**Example:**
```rust
let mut buffer = String::new();
let mut last_emit = Instant::now();

while let Some(token) = stream.next().await {
    buffer.push_str(&token);

    // Emit every 50ms or every 10 tokens
    if last_emit.elapsed() > Duration::from_millis(50) || buffer.len() > 10 {
        app.emit("ai_token", &buffer)?;
        buffer.clear();
        last_emit = Instant::now();
    }
}

// Emit remaining
if !buffer.is_empty() {
    app.emit("ai_token", &buffer)?;
}
```

### Pattern 4: Type-Safe IPC with Serde
**What:** Define shared types between Rust and TypeScript
**When:** All IPC communication
**Why:** Prevents runtime type errors
**Example:**
```rust
// Shared types
#[derive(Serialize, Deserialize, Clone)]
struct TerminalContext {
    cwd: String,
    selected_text: Option<String>,
    window_title: String,
}

// Command with typed params and return
#[tauri::command]
fn get_context() -> TerminalContext {
    // ...
}
```

```typescript
// Frontend mirrors Rust types
interface TerminalContext {
  cwd: string;
  selected_text?: string;
  window_title: string;
}

const context = await invoke<TerminalContext>('get_context');
```

**Enhancement:** Use `tauri-specta` for automatic TypeScript type generation from Rust types.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Frontend Business Logic
**What:** Implementing terminal reading or AI streaming in JavaScript
**Why bad:**
- No access to native macOS APIs from WebView
- Performance penalty for heavy operations
- Security issues (API keys in frontend)
**Instead:** Keep all system integration in Rust backend

### Anti-Pattern 2: Synchronous IPC Blocking
**What:** Calling slow Rust commands without async/await
**Why bad:** Freezes UI during API calls or file operations
**Instead:**
```rust
// BAD: Synchronous command blocks UI
#[tauri::command]
fn slow_operation() -> String {
    std::thread::sleep(Duration::from_secs(5));
    "done".to_string()
}

// GOOD: Async command doesn't block
#[tauri::command]
async fn slow_operation() -> String {
    tokio::time::sleep(Duration::from_secs(5)).await;
    "done".to_string()
}
```

### Anti-Pattern 3: Window Management in Frontend
**What:** Showing/hiding overlay using JavaScript API
**Why bad:** Race conditions with global hotkey, unreliable focus management
**Instead:** Handle all window lifecycle in Rust, emit events to notify frontend

### Anti-Pattern 4: Polling for Terminal Context
**What:** Frontend repeatedly calling `get_context` command
**Why bad:** Wastes CPU, floods IPC channel
**Instead:**
```rust
// GOOD: Read context once when window shows
app.listen("show_overlay", |app| {
    let context = read_terminal_context();
    app.emit("context_ready", context).unwrap();
});
```

### Anti-Pattern 5: Storing Sensitive Data in Frontend
**What:** Keeping API keys or tokens in localStorage
**Why bad:** Accessible via devtools, persists in clear text
**Instead:** Store in Rust state, use OS keychain for persistence

## Scalability Considerations

| Concern | At MVP | At 100 Active Users | At Production |
|---------|--------|---------------------|---------------|
| **State Size** | Single global Mutex (< 1MB) | Same, state is per-process | Consider splitting state by domain |
| **AI Streaming** | Single concurrent stream | Same, one stream per user session | Add request queuing |
| **Event Flooding** | Emit every token | Batch tokens (50ms intervals) | Implement backpressure |
| **Context Caching** | 100ms TTL, no persistence | Same | Add disk cache for offline mode |
| **Window Management** | Single overlay window | Same | Support multiple workspace contexts |
| **AppleScript Execution** | Synchronous, blocks command | Make async with tokio::task::spawn_blocking | Add timeout + fallback |
| **Memory Management** | No cleanup | Clear AI session after 5 min idle | Implement proper session lifecycle |

## Security Considerations

### Permission Requirements

**macOS Permissions Required:**
1. **Accessibility** - For reading terminal text via Accessibility API
2. **Screen Recording** - For window title reading on macOS (via active-win-pos-rs)
3. **Input Monitoring** - May be required for global hotkey (automatic with tauri-plugin-global-shortcut)

**Request Flow:**
```rust
// Check permissions on startup
fn check_permissions() -> Result<()> {
    if !has_accessibility_permission() {
        show_permission_dialog("Accessibility");
    }
    if !has_screen_recording_permission() {
        show_permission_dialog("Screen Recording");
    }
    Ok(())
}
```

### API Key Management

**DO NOT:**
- Store in frontend localStorage
- Commit to version control
- Embed in binary

**DO:**
- Use macOS Keychain via `keyring-rs` crate
- Prompt user to enter in settings
- Validate on backend before use

```rust
use keyring::Entry;

fn save_api_key(key: &str) -> Result<()> {
    let entry = Entry::new("cmd-k-app", "xai_api_key")?;
    entry.set_password(key)?;
    Ok(())
}

fn load_api_key() -> Result<String> {
    let entry = Entry::new("cmd-k-app", "xai_api_key")?;
    entry.get_password()
}
```

### Sandboxing

**Tauri v2 Security Model:**
- WebView processes sandboxed by default
- Rust Core has full OS access
- Use `allowlist` in tauri.conf.json to restrict IPC

**Recommended allowlist:**
```json
{
  "security": {
    "csp": "default-src 'self'; connect-src https://api.x.ai",
    "dangerousDisableAssetCspModification": false
  }
}
```

## Testing Strategy

### Unit Tests (Rust)
- Terminal context parsing
- AppleScript generation
- State management mutations

### Integration Tests (Rust)
- IPC command handlers (with mocked state)
- Event emission/listening
- Window lifecycle

### E2E Tests (Frontend + Backend)
- Global hotkey → window show
- Context reading → UI display
- AI streaming → markdown render
- Paste command execution

### Manual Testing Requirements
- Test on multiple terminals (Terminal.app, iTerm2, Warp, Alacritty)
- Test in fullscreen mode
- Test with multiple displays
- Test permission denial flows

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Hotkey → Overlay visible | < 100ms | Time from keypress to first paint |
| Context read latency | < 50ms | Time to read active window + cwd |
| AI first token | < 500ms | Time from Enter to first token |
| Token rendering | 60 fps | Maintain smooth scroll during streaming |
| Memory footprint | < 100MB | Rust process + WebView combined |
| Binary size | < 10MB | Tauri app bundle (macOS) |

## Technology Choices Summary

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| **Framework** | Tauri | 2.x | Small binaries, native performance, security |
| **Frontend** | React/Vue/Svelte | Latest | Developer preference, all work well |
| **Overlay Windows** | tauri-nspanel | Latest | Native NSPanel for proper macOS behavior |
| **Global Hotkeys** | tauri-plugin-global-shortcut | Latest | Official Tauri plugin, reliable |
| **Active Window** | active-win-pos-rs | Latest | Cross-platform, returns process ID |
| **Process Info** | libproc-rs | Latest | macOS/Linux process inspection |
| **AI Streaming** | reqwest-eventsource | Latest | SSE support with auto-reconnect |
| **HTTP Client** | reqwest | Latest | Industry standard, async-ready |
| **Markdown** | marked.js | Latest | Fast, widely used, extensible |
| **State** | std::sync::Mutex | stdlib | Sufficient for single-threaded access |
| **Async Runtime** | Tokio | Latest | Required by Tauri, battle-tested |

## Open Questions & Research Flags

### For Later Phases

1. **Multi-terminal support:** Should app detect which terminal is active and adjust pasting strategy?
   - iTerm2 has different AppleScript API than Terminal.app
   - Some terminals (Alacritty) may need different approach
   - **Recommendation:** Start with Terminal.app + iTerm2, add others based on usage

2. **Context reading reliability:** What's the fallback if Accessibility API denies permission?
   - Can still do process inspection (cwd)
   - Can't read selected text
   - **Recommendation:** Graceful degradation, show warning in UI

3. **AI streaming error handling:** How to handle network interruptions mid-stream?
   - reqwest-eventsource has auto-reconnect
   - Need to track partial response state
   - **Recommendation:** Buffer tokens, resume on reconnect

4. **Window positioning:** Should overlay appear at cursor or center screen?
   - Raycast uses cursor position
   - Spotlight uses center screen
   - **Recommendation:** User preference, default to center

5. **Multiple workspaces:** Should app track context per macOS workspace?
   - Would require workspace change detection
   - Separate state per workspace
   - **Recommendation:** Phase 2+ feature, single global context for MVP

## Sources & Confidence Assessment

| Topic | Confidence | Primary Sources |
|-------|------------|-----------------|
| Tauri v2 Architecture | HIGH | [Official docs](https://v2.tauri.app/concept/architecture/), [Process model](https://v2.tauri.app/concept/process-model/) |
| Tauri IPC Patterns | HIGH | [IPC docs](https://v2.tauri.app/concept/inter-process-communication/), [State management](https://v2.tauri.app/develop/state-management/) |
| NSPanel Integration | HIGH | [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel), community examples |
| Global Hotkeys | HIGH | [tauri-plugin-global-shortcut](https://v2.tauri.app/plugin/global-shortcut/) |
| Terminal Context Reading | MEDIUM | [active-win-pos-rs](https://crates.io/crates/active-win-pos-rs), [libproc-rs](https://github.com/andrewdavidmackenzie/libproc-rs), Accessibility API requires custom FFI |
| AppleScript Integration | HIGH | [iTerm2 scripting](https://iterm2.com/documentation-scripting.html), macOS AppleScript stable API |
| AI Streaming (SSE) | HIGH | [reqwest-eventsource](https://docs.rs/reqwest-eventsource/), [SSE patterns](https://learnwithparam.com/blog/streaming-at-scale-sse-websockets-real-time-ai-apis) |
| System Tray | HIGH | [Tauri system tray](https://v2.tauri.app/learn/system-tray/), [menubar examples](https://github.com/ahkohd/tauri-macos-menubar-app-example) |

**Overall Confidence: HIGH**

All major architectural components verified via official documentation or production-tested libraries. Medium confidence on Accessibility API integration due to need for custom FFI, but pattern documented in [Shellporter 2026 blog](https://www.marcogomiero.com/posts/2026/building-shellporter/).

## Sources

- [Tauri v2 Architecture](https://v2.tauri.app/concept/architecture/)
- [Tauri Process Model](https://v2.tauri.app/concept/process-model/)
- [Tauri Inter-Process Communication](https://v2.tauri.app/concept/inter-process-communication/)
- [Tauri State Management](https://v2.tauri.app/develop/state-management/)
- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/)
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/)
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- [tauri-nspanel GitHub](https://github.com/ahkohd/tauri-nspanel)
- [active-win-pos-rs crate](https://crates.io/crates/active-win-pos-rs)
- [libproc-rs GitHub](https://github.com/andrewdavidmackenzie/libproc-rs)
- [reqwest-eventsource docs](https://docs.rs/reqwest-eventsource/)
- [iTerm2 Scripting Documentation](https://iterm2.com/documentation-scripting.html)
- [Streaming at Scale: SSE, WebSockets & Real-Time AI APIs](https://learnwithparam.com/blog/streaming-at-scale-sse-websockets-real-time-ai-apis)
- [Building Shellporter: From Idea to Production in a Week](https://www.marcogomiero.com/posts/2026/building-shellporter/)
- [Tauri macOS Menubar App Example](https://github.com/ahkohd/tauri-macos-menubar-app-example)
- [Raycast Manual - Hotkey](https://manual.raycast.com/hotkey)
