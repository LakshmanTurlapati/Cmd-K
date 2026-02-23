use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde::Deserialize;
use tauri_plugin_http::reqwest;

// Keychain constants must match keychain.rs exactly
const SERVICE: &str = "com.lakshmanturlapati.cmd-k";
const ACCOUNT: &str = "xai_api_key";

/// System prompt for terminal mode: strict command-only output.
/// Placeholder {shell_type} is replaced at runtime.
const TERMINAL_SYSTEM_PROMPT_TEMPLATE: &str =
    "You are a terminal command generator for macOS. Given the user's task description and terminal \
     context, output ONLY the exact command(s) to run. No explanations, no markdown, no code fences. \
     Just the raw command(s). If multiple commands are needed, separate them with && or use pipes. \
     Prefer common POSIX tools (grep, find, sed, awk) over modern alternatives (rg, fd, jq). \
     The user is on macOS with {shell_type} shell.";

/// System prompt for assistant mode: concise conversational responses.
const ASSISTANT_SYSTEM_PROMPT: &str =
    "You are a concise assistant accessed via a macOS overlay. Answer in 2-3 sentences maximum. \
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
}

#[derive(Deserialize)]
struct TerminalContextView {
    shell_type: Option<String>,
    cwd: Option<String>,
    visible_output: Option<String>,
    running_process: Option<String>,
}

/// Build the user message string from the app context and raw query.
///
/// Terminal mode: includes App, Shell, CWD, Running process, Terminal output (last 25 lines),
/// Console last line (if browser DevTools open), then the task.
///
/// Assistant mode: includes App name (if available), Console last line (if browser), then the question.
fn build_user_message(query: &str, ctx: &AppContextView) -> String {
    let is_terminal_mode = ctx
        .terminal
        .as_ref()
        .and_then(|t| t.shell_type.as_ref())
        .is_some();

    let mut parts: Vec<String> = Vec::new();

    if is_terminal_mode {
        if let Some(name) = &ctx.app_name {
            parts.push(format!("App: {}", name));
        }
        if let Some(terminal) = &ctx.terminal {
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
        parts.push(query.to_string());
    }

    parts.join("\n")
}

/// Stream AI response tokens to the frontend via a Tauri IPC Channel.
///
/// - Reads the xAI API key from macOS Keychain (never accepted from frontend).
/// - Determines terminal vs assistant mode from context_json.
/// - Builds the system prompt (two modes) and user message with context.
/// - Includes up to 7 turns of session history in the messages array.
/// - POSTs to xAI /v1/chat/completions with stream:true.
/// - Parses SSE chunks via eventsource-stream and forwards each token via on_token.
/// - Hard 10-second timeout wraps the SSE streaming loop.
#[tauri::command]
pub async fn stream_ai_response(
    query: String,
    model: String,
    context_json: String,
    history: Vec<ChatMessage>,
    on_token: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    eprintln!("[ai] stream_ai_response called, model={}", model);

    // 1. Read API key from Keychain
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keyring error: {}", e))?;
    let api_key = entry
        .get_password()
        .map_err(|_| "No API key configured. Open Settings to add one.".to_string())?;

    // 2. Parse the context JSON into a lightweight view struct
    let ctx: AppContextView = serde_json::from_str(&context_json).unwrap_or_else(|e| {
        eprintln!("[ai] Failed to parse context_json: {}", e);
        // Fallback: assistant mode with no context
        AppContextView {
            app_name: None,
            terminal: None,
            console_detected: false,
            console_last_line: None,
        }
    });

    // 3. Determine mode and build system prompt
    let is_terminal_mode = ctx
        .terminal
        .as_ref()
        .and_then(|t| t.shell_type.as_ref())
        .is_some();

    let system_prompt = if is_terminal_mode {
        let shell_type = ctx
            .terminal
            .as_ref()
            .and_then(|t| t.shell_type.as_deref())
            .unwrap_or("zsh");
        TERMINAL_SYSTEM_PROMPT_TEMPLATE.replace("{shell_type}", shell_type)
    } else {
        ASSISTANT_SYSTEM_PROMPT.to_string()
    };

    eprintln!("[ai] mode={}", if is_terminal_mode { "terminal" } else { "assistant" });

    // 4. Build the user message with context
    let user_message = build_user_message(&query, &ctx);

    // 5. Build messages array: system prompt + history (capped at last 7 pairs = 14 msgs) + current user msg
    let mut messages: Vec<serde_json::Value> = Vec::new();

    messages.push(serde_json::json!({
        "role": "system",
        "content": system_prompt
    }));

    // Cap history to last 7 turns (14 messages: 7 user + 7 assistant)
    let history_start = history.len().saturating_sub(14);
    for msg in &history[history_start..] {
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

    // 6. Build request body
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "temperature": 0.1
    })
    .to_string();

    // 7. Make the HTTP request
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    // 8. Check HTTP status before streaming
    let status = response.status().as_u16();
    eprintln!("[ai] HTTP status={}", status);

    match status {
        200 => {
            // Proceed to SSE streaming below
        }
        401 => {
            return Err(
                "Authentication failed. Check your API key in Settings.".to_string(),
            );
        }
        429 => {
            return Err(
                "Rate limit exceeded. Please wait a moment and try again.".to_string(),
            );
        }
        _ => {
            return Err(format!("API error ({}). Try again.", status));
        }
    }

    // 9. Parse SSE stream with 10-second hard timeout
    let mut stream = response.bytes_stream().eventsource();
    let timeout_duration = tokio::time::Duration::from_secs(10);

    let result = tokio::time::timeout(timeout_duration, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    let data = event.data;
                    // Check for [DONE] sentinel BEFORE attempting JSON parse
                    if data == "[DONE]" {
                        eprintln!("[ai] received [DONE], stream complete");
                        break;
                    }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(token) = chunk["choices"][0]["delta"]["content"].as_str() {
                            if !token.is_empty() {
                                on_token
                                    .send(token.to_string())
                                    .map_err(|e| format!("Channel error: {}", e))?;
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Stream error: {}", e));
                }
            }
        }
        Ok::<(), String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Request timed out. Try again.".to_string()),
    }
}
