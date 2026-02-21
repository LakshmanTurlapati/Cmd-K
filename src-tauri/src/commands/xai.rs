use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;

#[derive(Serialize, Deserialize, Clone)]
pub struct XaiModel {
    pub id: String,
    pub object: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<XaiModel>,
}

/// What the frontend receives: id + short label for display in the model dropdown.
#[derive(Serialize, Clone)]
pub struct XaiModelWithMeta {
    pub id: String,
    pub label: String,
}

/// Map a raw model id to its display label.
fn model_label(id: &str) -> String {
    if id.contains("grok-code-fast") {
        "Recommended".to_string()
    } else if id.contains("grok-4") {
        "Most capable".to_string()
    } else if id.contains("grok-3-mini") {
        "Fast".to_string()
    } else if id.contains("grok-3") {
        "Balanced".to_string()
    } else {
        String::new()
    }
}

/// Filter the raw model list: drop image/video models and convert to XaiModelWithMeta.
fn to_meta_models(models: Vec<XaiModel>) -> Vec<XaiModelWithMeta> {
    models
        .into_iter()
        .filter(|m| !m.id.contains("image") && !m.id.contains("video"))
        .map(|m| {
            let label = model_label(&m.id);
            XaiModelWithMeta { id: m.id, label }
        })
        .collect()
}

/// Hardcoded fallback model list for when GET /v1/models returns 404.
/// Validated by sending a minimal POST to /v1/chat/completions (max_tokens=1).
async fn fallback_validate_and_models(
    client: &reqwest::Client,
    api_key: &str,
) -> Result<Vec<XaiModelWithMeta>, String> {
    // Validate key with minimal chat completions call.
    // Build JSON body manually since tauri_plugin_http::reqwest re-export
    // does not expose the `json` request builder feature.
    let body = serde_json::json!({
        "model": "grok-3",
        "messages": [{ "role": "user", "content": "hi" }],
        "max_tokens": 1
    })
    .to_string();

    let resp = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    match resp.status().as_u16() {
        200 => {
            // Key is valid; return hardcoded model list.
            let hardcoded = vec![
                "grok-3",
                "grok-3-mini",
                "grok-4",
                "grok-4-fast",
                "grok-code-fast-1",
            ];
            let models = hardcoded
                .into_iter()
                .map(|id| XaiModelWithMeta {
                    label: model_label(id),
                    id: id.to_string(),
                })
                .collect();
            Ok(models)
        }
        401 => Err("invalid_key".to_string()),
        status => Err(format!("API error: {}", status)),
    }
}

#[tauri::command]
pub async fn validate_and_fetch_models(
    api_key: String,
) -> Result<Vec<XaiModelWithMeta>, String> {
    let client = reqwest::Client::new();

    let response = client
        .get("https://api.x.ai/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    match response.status().as_u16() {
        200 => {
            // Parse the JSON body manually using serde_json and bytes.
            let bytes = response
                .bytes()
                .await
                .map_err(|e| format!("Read error: {}", e))?;
            let models_resp: ModelsResponse = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Parse error: {}", e))?;
            Ok(to_meta_models(models_resp.data))
        }
        401 => Err("invalid_key".to_string()),
        404 => {
            // GET /v1/models not supported -- fall back to hardcoded list + completions validation.
            fallback_validate_and_models(&client, &api_key).await
        }
        status => Err(format!("API error: {}", status)),
    }
}
