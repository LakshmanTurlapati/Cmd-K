use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tauri_plugin_http::reqwest;

use crate::state::TokenUsage;
use super::{handle_http_status, Provider};

/// Stream tokens from an OpenAI-compatible API (OpenAI, xAI, OpenRouter).
///
/// All three providers share the same SSE format:
/// - `data: {JSON}` with `choices[0].delta.content`
/// - `data: [DONE]` sentinel to end the stream
pub async fn stream(
    provider: &Provider,
    api_key: &str,
    model: &str,
    messages: Vec<serde_json::Value>,
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<TokenUsage, String> {
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": true,
        "stream_options": { "include_usage": true },
        "temperature": 0.1
    })
    .to_string();

    let client = reqwest::Client::new();
    let mut request = client
        .post(provider.api_url())
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json");

    // OpenRouter requires referrer and title headers
    if *provider == Provider::OpenRouter {
        request = request
            .header("HTTP-Referer", "https://cmdkapp.com")
            .header("X-Title", "CMD+K");
    }

    let response = request
        .body(body)
        .send()
        .await
        .map_err(|e| format!("{}: Network error: {}", provider.display_name(), e))?;

    let status = response.status().as_u16();
    eprintln!("[{}] HTTP status={}", provider.display_name(), status);
    handle_http_status(provider, status)?;

    let mut stream = response.bytes_stream().eventsource();

    let mut token_usage = TokenUsage::default();

    let result = tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    let data = event.data;
                    if data == "[DONE]" {
                        eprintln!("[{}] received [DONE], stream complete", provider.display_name());
                        break;
                    }
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&data) {
                        if let Some(token) = chunk["choices"][0]["delta"]["content"].as_str() {
                            if !token.is_empty() {
                                on_token
                                    .send(token.to_string())
                                    .map_err(|e| format!("{}: Channel error: {}", provider.display_name(), e))?;
                            }
                        }
                        // Extract usage from final chunk (choices is empty, usage object present)
                        if chunk.get("usage").is_some() {
                            token_usage.input_tokens = chunk["usage"]["prompt_tokens"].as_u64();
                            token_usage.output_tokens = chunk["usage"]["completion_tokens"].as_u64();
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("{}: Stream error: {}", provider.display_name(), e));
                }
            }
        }
        Ok::<(), String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(token_usage),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(format!(
            "{}: Request timed out. Try again.",
            provider.display_name()
        )),
    }
}
