use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use tauri_plugin_http::reqwest;

use crate::state::TokenUsage;
use super::{handle_http_status, Provider};

/// Stream tokens from the Google Gemini API.
///
/// Critical differences from OpenAI-compatible APIs:
/// - URL: `{base}/{model}:streamGenerateContent?alt=sse&key={api_key}` (key in URL, not header)
/// - Body: `{ "contents": [...], "systemInstruction": {...}, "generationConfig": {...} }`
/// - Roles: `"assistant"` -> `"model"`, content wrapped in `"parts": [{ "text": ... }]`
/// - System prompt via `"systemInstruction"` field
/// - No `[DONE]` sentinel -- stream ends when connection closes
/// - Extract text from `candidates[0].content.parts[0].text`
pub async fn stream(
    api_key: &str,
    model: &str,
    system_prompt: &str,
    messages: Vec<serde_json::Value>,
    on_token: &tauri::ipc::Channel<String>,
    timeout: tokio::time::Duration,
) -> Result<TokenUsage, String> {
    let provider = Provider::Gemini;

    // Build Gemini URL: {base}{model}:streamGenerateContent?alt=sse&key={api_key}
    let url = format!(
        "{}{}:streamGenerateContent?alt=sse&key={}",
        provider.api_url(),
        model,
        api_key
    );

    // Convert OpenAI-format messages to Gemini format
    let contents: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m["role"].as_str() != Some("system"))
        .map(|m| {
            let role = match m["role"].as_str() {
                Some("assistant") => "model",
                Some(r) => r,
                None => "user",
            };
            let text = m["content"].as_str().unwrap_or("");
            serde_json::json!({
                "role": role,
                "parts": [{ "text": text }]
            })
        })
        .collect();

    let body = serde_json::json!({
        "contents": contents,
        "systemInstruction": {
            "parts": [{ "text": system_prompt }]
        },
        "generationConfig": {
            "temperature": 0.1
        }
    })
    .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Google Gemini: Network error: {}", e))?;

    let status = response.status().as_u16();
    eprintln!("[Google Gemini] HTTP status={}", status);
    handle_http_status(&provider, status)?;

    let mut stream = response.bytes_stream().eventsource();

    let mut token_usage = TokenUsage::default();

    let result = tokio::time::timeout(timeout, async {
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => {
                    if let Ok(chunk) = serde_json::from_str::<serde_json::Value>(&event.data) {
                        if let Some(text) =
                            chunk["candidates"][0]["content"]["parts"][0]["text"].as_str()
                        {
                            if !text.is_empty() {
                                on_token
                                    .send(text.to_string())
                                    .map_err(|e| format!("Google Gemini: Channel error: {}", e))?;
                            }
                        }
                        // Extract usage metadata (last chunk has final counts, always overwrite)
                        if chunk.get("usageMetadata").is_some() {
                            token_usage.input_tokens = chunk["usageMetadata"]["promptTokenCount"].as_u64();
                            token_usage.output_tokens = chunk["usageMetadata"]["candidatesTokenCount"].as_u64();
                        }
                    }
                }
                Err(e) => {
                    return Err(format!("Google Gemini: Stream error: {}", e));
                }
            }
        }
        // Gemini stream ends when connection closes -- no sentinel
        eprintln!("[Google Gemini] stream closed, complete");
        Ok::<(), String>(())
    })
    .await;

    match result {
        Ok(Ok(())) => Ok(token_usage),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Google Gemini: Request timed out. Try again.".to_string()),
    }
}
