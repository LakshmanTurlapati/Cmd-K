use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tauri_plugin_http::reqwest;

use super::{handle_http_status, Provider};

/// Stream tokens from the Anthropic Messages API.
///
/// Critical differences from OpenAI-compatible APIs:
/// - Auth: `x-api-key` header (not `Authorization: Bearer`)
/// - Required: `anthropic-version: 2023-06-01` header
/// - System prompt: top-level `"system"` field (not in messages array)
/// - Messages: only `user` and `assistant` roles
/// - Required: `"max_tokens": 4096`
/// - SSE: named events, filter on `content_block_delta`, extract `delta.text`
/// - Stream ends with `event: message_stop` (not `[DONE]`)
pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<(), String> {
    let provider = Provider::Anthropic;

    let body = serde_json::json!({
        "model": model,
        "system": system_prompt,
        "messages": messages,
        "max_tokens": 4096,
        "stream": true,
        "temperature": 0.1
    })
    .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post(provider.api_url())
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Anthropic: Network error: {}", e))?;

    let status = response.status().as_u16();
    eprintln!("[Anthropic] HTTP status={}", status);
    handle_http_status(&provider, status)?;

    let mut stream = response.bytes_stream().eventsource();

    let result = tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    match event.event.as_str() {
                        "content_block_delta" => {
                            if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&event.data) {
                                if let Some(text) = chunk["delta"]["text"].as_str() {
                                    if !text.is_empty() {
                                        on_token
                                            .send(text.to_string())
                                            .map_err(|e| format!("Anthropic: Channel error: {}", e))?;
                                    }
                                }
                            }
                        }
                        "message_stop" => {
                            eprintln!("[Anthropic] received message_stop, stream complete");
                            break;
                        }
                        // Ignore: ping, message_start, content_block_start, content_block_stop, message_delta
                        _ => {}
                    }
                }
                Err(e) => {
                    return Err(format!("Anthropic: Stream error: {}", e));
                }
            }
        }
        Ok::<(), String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Anthropic: Request timed out. Try again.".to_string()),
    }
}
