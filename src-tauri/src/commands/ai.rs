use serde::Deserialize;

use super::providers::{self, AdapterKind, Provider};

// Keychain service name -- must match keychain.rs
const SERVICE: &str = "com.lakshmanturlapati.cmd-k";

/// System prompt for terminal mode on macOS: strict command-only output.
/// Placeholder {shell_type} is replaced at runtime.
#[cfg(target_os = "macos")]
const TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for macOS. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run. No explanations, no markdown, no code fences. \
     Just the raw command(s). If multiple commands are needed, separate them with && or use pipes. \
     Prefer common POSIX tools (grep, find, sed, awk) over modern alternatives (rg, fd, jq). \
     The user is on macOS with {shell_type} shell.";

/// System prompt for terminal mode on Windows: strict command-only output.
/// Placeholder {shell_type} is replaced at runtime.
#[cfg(target_os = "windows")]
const TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for Windows. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run. No explanations, no markdown, no code fences. \
     Just the raw command(s). If multiple commands are needed, separate them with && or use pipes. \
     The user is on Windows with {shell_type} shell. Use native Windows commands when appropriate. \
     For PowerShell, prefer cmdlets (Get-ChildItem, Select-String, etc.). \
     For CMD, use standard commands (dir, findstr, etc.). \
     For bash/Git Bash, use POSIX tools.";

/// System prompt for WSL terminal mode: generates Linux commands for WSL sessions.
/// Only compiled on Windows since WSL only exists there.
#[cfg(target_os = "windows")]
const WSL_TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for Linux (WSL on Windows). Given the user's task \
     description and terminal context, output ONLY the exact command(s) to run. No explanations, \
     no markdown, no code fences. Just the raw command(s). If multiple commands are needed, \
     separate them with && or use pipes. Prefer common POSIX tools (grep, find, sed, awk). \
     The user is in a WSL Linux terminal with {shell_type} shell. You may reference WSL-Windows \
     interop features (e.g., `code .` to open VS Code, `explorer.exe .` to open Explorer) when relevant.";

/// Fallback system prompt for other platforms.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run. No explanations, no markdown, no code fences. \
     Just the raw command(s). If multiple commands are needed, separate them with && or use pipes. \
     The user has {shell_type} shell.";

/// System prompt for assistant mode: concise conversational responses.
#[cfg(target_os = "macos")]
const ASSISTANT_SYSTEM_PROMPT: &str =
    "You are a concise assistant accessed via a macOS overlay. Answer in 2-3 sentences maximum. \
     Be direct and helpful. No markdown formatting, no code fences unless the user explicitly asks for code.";

#[cfg(target_os = "windows")]
const ASSISTANT_SYSTEM_PROMPT: &str =
    "You are a concise assistant accessed via a Windows overlay. Answer in 2-3 sentences maximum. \
     Be direct and helpful. No markdown formatting, no code fences unless the user explicitly asks for code.";

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const ASSISTANT_SYSTEM_PROMPT: &str =
    "You are a concise assistant accessed via a desktop overlay. Answer in 2-3 sentences maximum. \
     Be direct and helpful. No markdown formatting, no code fences unless the user explicitly asks for code.";

/// Represents a previous conversation turn passed from the frontend.
#[derive(Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Lightweight view of AppContext deserialized from the JSON string sent by the frontend.
/// Only fields needed for prompt building are declared here.
#[derive(Deserialize)]
struct AppContextView {
    app_name: Option<String>,
    terminal: Option<TerminalContextView>,
    console_detected: bool,
    console_last_line: Option<String>,
    visible_text: Option<String>,
}

#[derive(Deserialize)]
struct TerminalContextView {
    shell_type: Option<String>,
    cwd: Option<String>,
    visible_output: Option<String>,
    running_process: Option<String>,
    #[serde(default)]
    is_wsl: bool,
}

/// Build the user message string from the app context and raw query.
///
/// Terminal mode: includes App, Shell, CWD, Running process, Terminal output (last 25 lines),
/// Console last line (if browser DevTools open), then the task.
///
/// Assistant mode: includes App name (if available), Console last line (if browser), then the question.
fn build_user_message(query: &str, ctx: &AppContextView, is_follow_up: bool) -> String {
    let is_terminal_mode = ctx
        .terminal
        .as_ref()
        .and_then(|t| t.shell_type.as_ref())
        .is_some();

    if is_follow_up {
        // Follow-up: just the query, no terminal context (CTXT-03)
        // System prompt already has shell type from the first message
        if is_terminal_mode {
            return format!("Task: {}", query);
        } else {
            return query.to_string();
        }
    }

    let mut parts: Vec<String> = Vec::new();

    if is_terminal_mode {
        if let Some(name) = &ctx.app_name {
            parts.push(format!("App: {}", name));
        }
        if let Some(terminal) = &ctx.terminal {
            if terminal.is_wsl {
                parts.push("OS: WSL on Windows (Linux terminal)".to_string());
            }
            if let Some(shell) = &terminal.shell_type {
                parts.push(format!("Shell: {}", shell));
            }
            if let Some(cwd) = &terminal.cwd {
                parts.push(format!("CWD: {}", cwd));
            }
            if let Some(proc) = &terminal.running_process {
                parts.push(format!("Running: {}", proc));
            }
            if let Some(output) = &terminal.visible_output {
                let lines: Vec<&str> = output.lines().collect();
                let start = lines.len().saturating_sub(25);
                let slice = &lines[start..];
                parts.push(format!(
                    "Terminal output (last {} lines):\n{}",
                    slice.len(),
                    slice.join("\n")
                ));
            }
        }
        if ctx.console_detected {
            if let Some(line) = &ctx.console_last_line {
                parts.push(format!("Console last line: {}", line));
            }
        }
        parts.push(format!("\nTask: {}", query));
    } else {
        // Assistant mode: minimal context
        if let Some(name) = &ctx.app_name {
            parts.push(format!("App: {}", name));
        }
        if ctx.console_detected {
            if let Some(line) = &ctx.console_last_line {
                parts.push(format!("Console last line: {}", line));
            }
        }
        if let Some(text) = &ctx.visible_text {
            let lines: Vec<&str> = text.lines().collect();
            let start = lines.len().saturating_sub(50);
            let slice = &lines[start..];
            parts.push(format!(
                "Screen content ({} lines):\n{}",
                slice.len(),
                slice.join("\n")
            ));
        }
        parts.push(query.to_string());
    }

    parts.join("\n")
}

/// Stream AI response tokens to the frontend via a Tauri IPC Channel.
///
/// - Accepts a `provider` parameter to dispatch to the correct streaming adapter.
/// - Reads the API key from the OS Keychain using the provider's account name.
/// - Determines terminal vs assistant mode from context_json.
/// - Builds the system prompt (two modes) and user message with context.
/// - Includes session history (pre-capped by frontend via configurable turnLimit) in the messages array.
/// - Dispatches to the appropriate adapter based on provider.adapter_kind().
#[tauri::command]
pub async fn stream_ai_response(
    provider: Provider,
    query: String,
    model: String,
    context_json: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    eprintln!(
        "[ai] stream_ai_response called, provider={}, model={}",
        provider.display_name(),
        model
    );

    // 1. Read API key from Keychain using provider-specific account name
    let entry = keyring::Entry::new(SERVICE, provider.keychain_account())
        .map_err(|e| format!("Keyring error: {}", e))?;
    let api_key = entry.get_password().map_err(|_| {
        format!(
            "No {} API key configured. Open Settings to add one.",
            provider.display_name()
        )
    })?;

    // 2. Parse the context JSON into a lightweight view struct
    let ctx: AppContextView = serde_json::from_str(&context_json).unwrap_or_else(|e| {
        eprintln!("[ai] Failed to parse context_json: {}", e);
        // Fallback: assistant mode with no context
        AppContextView {
            app_name: None,
            terminal: None,
            console_detected: false,
            console_last_line: None,
            visible_text: None,
        }
    });

    // 3. Determine mode and build system prompt
    let is_terminal_mode = ctx
        .terminal
        .as_ref()
        .and_then(|t| t.shell_type.as_ref())
        .is_some();

    let is_wsl = ctx
        .terminal
        .as_ref()
        .map(|t| t.is_wsl)
        .unwrap_or(false);

    let system_prompt = if is_terminal_mode {
        let default_shell = if is_wsl { "bash" } else { "zsh" };
        let shell_type = ctx
            .terminal
            .as_ref()
            .and_then(|t| t.shell_type.as_deref())
            .unwrap_or(default_shell);

        if is_wsl {
            #[cfg(target_os = "windows")]
            { WSL_TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type) }
            #[cfg(not(target_os = "windows"))]
            { TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type) }
        } else {
            TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type)
        }
    } else {
        ASSISTANT_SYSTEM_PROMPT.to_string()
    };

    eprintln!(
        "[ai] mode={} wsl={}",
        if is_terminal_mode {
            "terminal"
        } else {
            "assistant"
        },
        is_wsl
    );

    // 4. Build the user message with context (follow-ups omit terminal context)
    let is_follow_up = !history.is_empty();
    let user_message = build_user_message(&query, &ctx, is_follow_up);

    // 5. Build messages array: system prompt + history (pre-capped by frontend via turnLimit) + current user msg
    let mut messages: Vec<serde_json::Value> = Vec::new();

    messages.push(serde_json::json!({
        "role": "system",
        "content": system_prompt
    }));

    // Frontend sends pre-capped history via turnLimit -- no Rust-side capping needed
    for msg in &history {
        messages.push(serde_json::json!({
            "role": msg.role,
            "content": msg.content
        }));
    }

    messages.push(serde_json::json!({
        "role": "user",
        "content": user_message
    }));

    eprintln!("[ai] messages count={}", messages.len());

    // 6. Dispatch to the correct adapter based on provider
    let timeout = tokio::time::Duration::from_secs(provider.default_timeout_secs());

    match provider.adapter_kind() {
        AdapterKind::OpenAICompat => {
            providers::openai_compat::stream(
                &provider, &api_key, &model, messages, &on_token, timeout,
            )
            .await
        }
        AdapterKind::Anthropic => {
            // Anthropic: system prompt is a top-level field, not in messages array
            let non_system_messages: Vec<_> = messages
                .iter()
                .filter(|m| m["role"].as_str() != Some("system"))
                .cloned()
                .collect();
            providers::anthropic::stream(
                &api_key,
                &model,
                &system_prompt,
                non_system_messages,
                &on_token,
                timeout,
            )
            .await
        }
        AdapterKind::Gemini => {
            // Gemini: system prompt via systemInstruction, not in messages
            let non_system_messages: Vec<_> = messages
                .iter()
                .filter(|m| m["role"].as_str() != Some("system"))
                .cloned()
                .collect();
            providers::gemini::stream(
                &api_key,
                &model,
                &system_prompt,
                non_system_messages,
                &on_token,
                timeout,
            )
            .await
        }
    }
}
