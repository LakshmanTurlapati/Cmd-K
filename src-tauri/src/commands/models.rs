use serde::{Deserialize, Serialize};
use tauri_plugin_http::reqwest;

use super::providers::Provider;

/// What the frontend receives: id + display label + tier tag for model dropdown.
#[derive(Serialize, Clone)]
pub struct ModelWithMeta {
    pub id: String,
    pub label: String,
    pub tier: String, // "fast", "balanced", "capable", or "" for uncategorized
    /// Price per 1 million input tokens (USD). None for models without known pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_price_per_m: Option<f64>,
    /// Price per 1 million output tokens (USD). None for models without known pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_price_per_m: Option<f64>,
}

/// Hardcoded curated model list per provider with tier tags.
fn curated_models(provider: &Provider) -> Vec<ModelWithMeta> {
    match provider {
        Provider::OpenAI => vec![
            ModelWithMeta { id: "gpt-4o".into(), label: "GPT-4o".into(), tier: "balanced".into(), input_price_per_m: Some(2.50), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gpt-4o-mini".into(), label: "GPT-4o Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.15), output_price_per_m: Some(0.60) },
            ModelWithMeta { id: "gpt-4.1".into(), label: "GPT-4.1".into(), tier: "capable".into(), input_price_per_m: Some(2.00), output_price_per_m: Some(8.00) },
            ModelWithMeta { id: "gpt-4.1-mini".into(), label: "GPT-4.1 Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.40), output_price_per_m: Some(1.60) },
            ModelWithMeta { id: "gpt-4.1-nano".into(), label: "GPT-4.1 Nano".into(), tier: "fast".into(), input_price_per_m: Some(0.10), output_price_per_m: Some(0.40) },
        ],
        Provider::Anthropic => vec![
            ModelWithMeta { id: "claude-sonnet-4-20250514".into(), label: "Claude Sonnet 4".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "claude-haiku-3-5-20241022".into(), label: "Claude 3.5 Haiku".into(), tier: "fast".into(), input_price_per_m: Some(0.80), output_price_per_m: Some(4.00) },
            ModelWithMeta { id: "claude-opus-4-20250514".into(), label: "Claude Opus 4".into(), tier: "capable".into(), input_price_per_m: Some(15.00), output_price_per_m: Some(75.00) },
        ],
        Provider::Gemini => vec![
            ModelWithMeta { id: "gemini-2.0-flash".into(), label: "Gemini 2.0 Flash".into(), tier: "fast".into(), input_price_per_m: Some(0.10), output_price_per_m: Some(0.40) },
            ModelWithMeta { id: "gemini-2.5-pro-preview-06-05".into(), label: "Gemini 2.5 Pro".into(), tier: "capable".into(), input_price_per_m: Some(1.25), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gemini-2.5-flash-preview-05-20".into(), label: "Gemini 2.5 Flash".into(), tier: "balanced".into(), input_price_per_m: Some(0.15), output_price_per_m: Some(3.50) },
        ],
        Provider::XAI => vec![
            ModelWithMeta { id: "grok-3".into(), label: "Grok 3".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "grok-3-mini".into(), label: "Grok 3 Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.30), output_price_per_m: Some(0.50) },
            ModelWithMeta { id: "grok-4".into(), label: "Grok 4".into(), tier: "capable".into(), input_price_per_m: Some(6.00), output_price_per_m: Some(18.00) },
        ],
        Provider::OpenRouter => vec![],
    }
}

/// Validate an API key for a given provider by making a lightweight request.
#[tauri::command]
pub async fn validate_api_key(provider: Provider, api_key: String) -> Result<(), String> {
    let client = reqwest::Client::new();

    match provider {
        Provider::OpenAI => {
            let resp = client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|_| "Network error: Check your internet connection.".to_string())?;

            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                status => Err(format!("API error: {}", status)),
            }
        }
        Provider::Anthropic => {
            let body = serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 1,
                "messages": [{"role": "user", "content": "hi"}]
            })
            .to_string();

            let resp = client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .body(body)
                .send()
                .await
                .map_err(|_| "Network error: Check your internet connection.".to_string())?;

            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                status => Err(format!("API error: {}", status)),
            }
        }
        Provider::Gemini => {
            let resp = client
                .get(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                ))
                .send()
                .await
                .map_err(|_| "Network error: Check your internet connection.".to_string())?;

            match resp.status().as_u16() {
                200 => Ok(()),
                400 | 403 => Err("invalid_key".to_string()),
                status => Err(format!("API error: {}", status)),
            }
        }
        Provider::XAI => {
            let resp = client
                .get("https://api.x.ai/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|_| "Network error: Check your internet connection.".to_string())?;

            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                404 => {
                    // Fallback: validate via chat/completions
                    let body = serde_json::json!({
                        "model": "grok-3",
                        "messages": [{"role": "user", "content": "hi"}],
                        "max_tokens": 1
                    })
                    .to_string();

                    let fallback = client
                        .post("https://api.x.ai/v1/chat/completions")
                        .header("Authorization", format!("Bearer {}", api_key))
                        .header("Content-Type", "application/json")
                        .body(body)
                        .send()
                        .await
                        .map_err(|_| "Network error: Check your internet connection.".to_string())?;

                    match fallback.status().as_u16() {
                        200 => Ok(()),
                        401 => Err("invalid_key".to_string()),
                        status => Err(format!("API error: {}", status)),
                    }
                }
                status => Err(format!("API error: {}", status)),
            }
        }
        Provider::OpenRouter => {
            let resp = client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|_| "Network error: Check your internet connection.".to_string())?;

            match resp.status().as_u16() {
                200 => Ok(()),
                401 => Err("invalid_key".to_string()),
                status => Err(format!("API error: {}", status)),
            }
        }
    }
}

// ---- API response types for model listing ----

#[derive(Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Deserialize)]
struct OpenAIModel {
    id: String,
}

#[derive(Deserialize)]
struct GeminiModelsResponse {
    models: Option<Vec<GeminiModel>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiModel {
    name: String,
    supported_generation_methods: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Deserialize)]
struct OpenRouterModel {
    id: String,
    context_length: Option<u64>,
}

/// Fetch models for a provider: curated models first (with tier tags),
/// then API-fetched models not in the curated list (with tier="").
#[tauri::command]
pub async fn fetch_models(
    provider: Provider,
    api_key: String,
) -> Result<Vec<ModelWithMeta>, String> {
    let curated = curated_models(&provider);
    let curated_ids: Vec<String> = curated.iter().map(|m| m.id.clone()).collect();

    let api_models = fetch_api_models(&provider, &api_key).await.unwrap_or_default();

    // Merge: curated first, then API models not already in curated list
    let mut result = curated;
    for model in api_models {
        if !curated_ids.contains(&model.id) {
            result.push(model);
        }
    }

    Ok(result)
}

/// Fetch models from a provider's API. Returns empty vec on failure (graceful degradation).
async fn fetch_api_models(
    provider: &Provider,
    api_key: &str,
) -> Result<Vec<ModelWithMeta>, String> {
    let client = reqwest::Client::new();

    match provider {
        Provider::OpenAI => {
            let resp = client
                .get("https://api.openai.com/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
            let parsed: OpenAIModelsResponse =
                serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

            Ok(parsed
                .data
                .into_iter()
                .filter(|m| m.id.contains("gpt"))
                .map(|m| ModelWithMeta {
                    label: m.id.clone(),
                    id: m.id,
                    tier: String::new(),
                    input_price_per_m: None,
                    output_price_per_m: None,
                })
                .collect())
        }
        Provider::Anthropic => {
            // No public model list API -- curated only
            Ok(vec![])
        }
        Provider::Gemini => {
            let resp = client
                .get(format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                ))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
            let parsed: GeminiModelsResponse =
                serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

            Ok(parsed
                .models
                .unwrap_or_default()
                .into_iter()
                .filter(|m| {
                    m.supported_generation_methods
                        .as_ref()
                        .is_some_and(|methods| methods.iter().any(|mt| mt == "generateContent"))
                })
                .map(|m| {
                    // Gemini API returns "models/gemini-..." -- strip the prefix
                    let id = m.name.strip_prefix("models/").unwrap_or(&m.name).to_string();
                    ModelWithMeta {
                        label: id.clone(),
                        id,
                        tier: String::new(),
                        input_price_per_m: None,
                        output_price_per_m: None,
                    }
                })
                .collect())
        }
        Provider::XAI => {
            let resp = client
                .get("https://api.x.ai/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            match resp.status().as_u16() {
                200 => {
                    let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
                    let parsed: OpenAIModelsResponse =
                        serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

                    Ok(parsed
                        .data
                        .into_iter()
                        .filter(|m| !m.id.contains("image") && !m.id.contains("video"))
                        .map(|m| ModelWithMeta {
                            label: m.id.clone(),
                            id: m.id,
                            tier: String::new(),
                            input_price_per_m: None,
                            output_price_per_m: None,
                        })
                        .collect())
                }
                404 => {
                    // GET /v1/models not supported -- return hardcoded list
                    Ok(vec![
                        "grok-3",
                        "grok-3-mini",
                        "grok-4",
                        "grok-4-fast",
                        "grok-4-fast-non-reasoning",
                    ]
                    .into_iter()
                    .map(|id| ModelWithMeta {
                        label: id.to_string(),
                        id: id.to_string(),
                        tier: String::new(),
                        input_price_per_m: None,
                        output_price_per_m: None,
                    })
                    .collect())
                }
                _ => Err(format!("API error: {}", resp.status().as_u16())),
            }
        }
        Provider::OpenRouter => {
            let resp = client
                .get("https://openrouter.ai/api/v1/models")
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
            let parsed: OpenRouterModelsResponse =
                serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

            Ok(parsed
                .data
                .into_iter()
                .filter(|m| m.context_length.unwrap_or(0) > 0)
                .map(|m| ModelWithMeta {
                    label: m.id.clone(),
                    id: m.id,
                    tier: String::new(),
                    input_price_per_m: None,
                    output_price_per_m: None,
                })
                .collect())
        }
    }
}
