use std::collections::HashMap;
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
pub(crate) fn curated_models(provider: &Provider) -> Vec<ModelWithMeta> {
    match provider {
        Provider::OpenAI => vec![
            // GPT-5.x family
            ModelWithMeta { id: "gpt-5.4".into(), label: "GPT-5.4".into(), tier: "capable".into(), input_price_per_m: Some(2.50), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "gpt-5.4-pro".into(), label: "GPT-5.4 Pro".into(), tier: "capable".into(), input_price_per_m: Some(30.00), output_price_per_m: Some(180.00) },
            ModelWithMeta { id: "gpt-5.2".into(), label: "GPT-5.2".into(), tier: "capable".into(), input_price_per_m: Some(1.75), output_price_per_m: Some(14.00) },
            ModelWithMeta { id: "gpt-5.2-pro".into(), label: "GPT-5.2 Pro".into(), tier: "capable".into(), input_price_per_m: Some(21.00), output_price_per_m: Some(168.00) },
            ModelWithMeta { id: "gpt-5.1".into(), label: "GPT-5.1".into(), tier: "capable".into(), input_price_per_m: Some(1.25), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gpt-5".into(), label: "GPT-5".into(), tier: "capable".into(), input_price_per_m: Some(1.25), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gpt-5-pro".into(), label: "GPT-5 Pro".into(), tier: "capable".into(), input_price_per_m: Some(15.00), output_price_per_m: Some(120.00) },
            ModelWithMeta { id: "gpt-5-mini".into(), label: "GPT-5 Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.25), output_price_per_m: Some(2.00) },
            ModelWithMeta { id: "gpt-5-nano".into(), label: "GPT-5 Nano".into(), tier: "fast".into(), input_price_per_m: Some(0.05), output_price_per_m: Some(0.40) },
            // GPT-4.x family
            ModelWithMeta { id: "gpt-4.1".into(), label: "GPT-4.1".into(), tier: "balanced".into(), input_price_per_m: Some(2.00), output_price_per_m: Some(8.00) },
            ModelWithMeta { id: "gpt-4.1-mini".into(), label: "GPT-4.1 Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.40), output_price_per_m: Some(1.60) },
            ModelWithMeta { id: "gpt-4.1-nano".into(), label: "GPT-4.1 Nano".into(), tier: "fast".into(), input_price_per_m: Some(0.10), output_price_per_m: Some(0.40) },
            ModelWithMeta { id: "gpt-4o".into(), label: "GPT-4o".into(), tier: "balanced".into(), input_price_per_m: Some(2.50), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gpt-4o-mini".into(), label: "GPT-4o Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.15), output_price_per_m: Some(0.60) },
            // o-series reasoning models
            ModelWithMeta { id: "o3".into(), label: "o3".into(), tier: "capable".into(), input_price_per_m: Some(2.00), output_price_per_m: Some(8.00) },
            ModelWithMeta { id: "o3-pro".into(), label: "o3 Pro".into(), tier: "capable".into(), input_price_per_m: Some(20.00), output_price_per_m: Some(80.00) },
            ModelWithMeta { id: "o3-mini".into(), label: "o3 Mini".into(), tier: "fast".into(), input_price_per_m: Some(1.10), output_price_per_m: Some(4.40) },
            ModelWithMeta { id: "o4-mini".into(), label: "o4 Mini".into(), tier: "fast".into(), input_price_per_m: Some(1.10), output_price_per_m: Some(4.40) },
            ModelWithMeta { id: "o1".into(), label: "o1".into(), tier: "capable".into(), input_price_per_m: Some(15.00), output_price_per_m: Some(60.00) },
            ModelWithMeta { id: "o1-mini".into(), label: "o1 Mini".into(), tier: "fast".into(), input_price_per_m: Some(1.10), output_price_per_m: Some(4.40) },
            ModelWithMeta { id: "o1-pro".into(), label: "o1 Pro".into(), tier: "capable".into(), input_price_per_m: Some(150.00), output_price_per_m: Some(600.00) },
        ],
        Provider::Anthropic => vec![
            // Claude 4.6 (latest)
            ModelWithMeta { id: "claude-opus-4-6".into(), label: "Claude Opus 4.6".into(), tier: "capable".into(), input_price_per_m: Some(5.00), output_price_per_m: Some(25.00) },
            ModelWithMeta { id: "claude-sonnet-4-6".into(), label: "Claude Sonnet 4.6".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            // Claude 4.5
            ModelWithMeta { id: "claude-haiku-4-5-20251001".into(), label: "Claude Haiku 4.5".into(), tier: "fast".into(), input_price_per_m: Some(1.00), output_price_per_m: Some(5.00) },
            ModelWithMeta { id: "claude-sonnet-4-5-20250929".into(), label: "Claude Sonnet 4.5".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "claude-opus-4-5-20251101".into(), label: "Claude Opus 4.5".into(), tier: "capable".into(), input_price_per_m: Some(5.00), output_price_per_m: Some(25.00) },
            // Claude 4.1
            ModelWithMeta { id: "claude-opus-4-1-20250805".into(), label: "Claude Opus 4.1".into(), tier: "capable".into(), input_price_per_m: Some(15.00), output_price_per_m: Some(75.00) },
            // Claude 4.0
            ModelWithMeta { id: "claude-sonnet-4-20250514".into(), label: "Claude Sonnet 4".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "claude-opus-4-20250514".into(), label: "Claude Opus 4".into(), tier: "capable".into(), input_price_per_m: Some(15.00), output_price_per_m: Some(75.00) },
            // Claude 3.5
            ModelWithMeta { id: "claude-haiku-3-5-20241022".into(), label: "Claude 3.5 Haiku".into(), tier: "fast".into(), input_price_per_m: Some(0.80), output_price_per_m: Some(4.00) },
        ],
        Provider::Gemini => vec![
            // Gemini 3.x
            ModelWithMeta { id: "gemini-3.1-pro-preview".into(), label: "Gemini 3.1 Pro".into(), tier: "capable".into(), input_price_per_m: Some(2.00), output_price_per_m: Some(12.00) },
            ModelWithMeta { id: "gemini-3.1-flash-lite-preview".into(), label: "Gemini 3.1 Flash Lite".into(), tier: "fast".into(), input_price_per_m: Some(0.25), output_price_per_m: Some(1.50) },
            ModelWithMeta { id: "gemini-3-flash-preview".into(), label: "Gemini 3 Flash".into(), tier: "balanced".into(), input_price_per_m: Some(0.50), output_price_per_m: Some(3.00) },
            // Gemini 2.5
            ModelWithMeta { id: "gemini-2.5-pro".into(), label: "Gemini 2.5 Pro".into(), tier: "capable".into(), input_price_per_m: Some(1.25), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gemini-2.5-flash".into(), label: "Gemini 2.5 Flash".into(), tier: "balanced".into(), input_price_per_m: Some(0.30), output_price_per_m: Some(2.50) },
            ModelWithMeta { id: "gemini-2.5-flash-lite".into(), label: "Gemini 2.5 Flash Lite".into(), tier: "fast".into(), input_price_per_m: Some(0.10), output_price_per_m: Some(0.40) },
            // Gemini 2.0
            ModelWithMeta { id: "gemini-2.0-flash".into(), label: "Gemini 2.0 Flash".into(), tier: "fast".into(), input_price_per_m: Some(0.10), output_price_per_m: Some(0.40) },
            // Legacy model IDs (still valid aliases)
            ModelWithMeta { id: "gemini-2.5-pro-preview-06-05".into(), label: "Gemini 2.5 Pro (preview)".into(), tier: "capable".into(), input_price_per_m: Some(1.25), output_price_per_m: Some(10.00) },
            ModelWithMeta { id: "gemini-2.5-flash-preview-05-20".into(), label: "Gemini 2.5 Flash (preview)".into(), tier: "balanced".into(), input_price_per_m: Some(0.30), output_price_per_m: Some(2.50) },
        ],
        Provider::XAI => vec![
            // Grok 4.x
            ModelWithMeta { id: "grok-4-1-fast-reasoning".into(), label: "Grok 4.1 Fast".into(), tier: "fast".into(), input_price_per_m: Some(0.20), output_price_per_m: Some(0.50) },
            ModelWithMeta { id: "grok-4-1-fast-non-reasoning".into(), label: "Grok 4.1 Fast NR".into(), tier: "fast".into(), input_price_per_m: Some(0.20), output_price_per_m: Some(0.50) },
            ModelWithMeta { id: "grok-4-0709".into(), label: "Grok 4".into(), tier: "capable".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "grok-4-fast-reasoning".into(), label: "Grok 4 Fast".into(), tier: "fast".into(), input_price_per_m: Some(0.20), output_price_per_m: Some(0.50) },
            ModelWithMeta { id: "grok-4-fast-non-reasoning".into(), label: "Grok 4 Fast NR".into(), tier: "fast".into(), input_price_per_m: Some(0.20), output_price_per_m: Some(0.50) },
            ModelWithMeta { id: "grok-code-fast-1".into(), label: "Grok Code".into(), tier: "fast".into(), input_price_per_m: Some(0.20), output_price_per_m: Some(1.50) },
            // Grok 3
            ModelWithMeta { id: "grok-3".into(), label: "Grok 3".into(), tier: "balanced".into(), input_price_per_m: Some(3.00), output_price_per_m: Some(15.00) },
            ModelWithMeta { id: "grok-3-mini".into(), label: "Grok 3 Mini".into(), tier: "fast".into(), input_price_per_m: Some(0.30), output_price_per_m: Some(0.50) },
        ],
        Provider::OpenRouter | Provider::Ollama | Provider::LMStudio => vec![],
    }
}

/// Build a pricing lookup map from all curated models across all providers.
/// Returns model_id -> (input_price_per_m, output_price_per_m) for models
/// where both prices are known.
pub(crate) fn curated_models_pricing() -> HashMap<String, (f64, f64)> {
    let providers = [
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::Gemini,
        Provider::XAI,
    ];
    let mut map = HashMap::new();
    for provider in &providers {
        for model in curated_models(provider) {
            if let (Some(inp), Some(out)) = (model.input_price_per_m, model.output_price_per_m) {
                map.insert(model.id, (inp, out));
            }
        }
    }
    map
}

/// Validate an API key for a given provider by making a lightweight request.
/// For local providers (Ollama, LM Studio), performs a health check instead.
#[tauri::command]
pub async fn validate_api_key(
    app_handle: tauri::AppHandle,
    provider: Provider,
    api_key: String,
) -> Result<(), String> {
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
        Provider::Ollama => {
            let base_url = super::providers::get_provider_base_url(&app_handle, &provider);
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(|e| e.to_string())?;

            // Step 1: Check if server is reachable
            match client.get(&base_url).send().await {
                Ok(r) if r.status().is_success() => {
                    // Step 2: Check if any models are loaded
                    let tags_url = format!("{}/api/tags", base_url.trim_end_matches('/'));
                    match client.get(&tags_url).send().await {
                        Ok(resp) => {
                            let bytes = resp.bytes().await.unwrap_or_default();
                            if let Ok(body) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                                let models = body.get("models").and_then(|m| m.as_array());
                                match models {
                                    Some(arr) if arr.is_empty() => {
                                        Err("No models loaded".to_string())
                                    }
                                    Some(_) => Ok(()),
                                    None => Ok(()), // Unexpected format, but server is running
                                }
                            } else {
                                Ok(()) // Could not parse, but server responded -- treat as OK
                            }
                        }
                        Err(e) => Err(format!("Request failed -- {}", e)),
                    }
                }
                Ok(_) => Err("Server not running".to_string()),
                Err(e) if e.is_connect() => Err("Server not running".to_string()),
                Err(e) if e.is_timeout() => Err("Server not running".to_string()),
                Err(e) => Err(format!("Request failed -- {}", e)),
            }
        }
        Provider::LMStudio => {
            let base_url = super::providers::get_provider_base_url(&app_handle, &provider);
            let health_url = format!("{}/v1/models", base_url.trim_end_matches('/'));
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(|e| e.to_string())?;

            match client.get(&health_url).send().await {
                Ok(r) if r.status().is_success() => {
                    // Check if any models are loaded
                    let bytes = r.bytes().await.unwrap_or_default();
                    if let Ok(body) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                        let data = body.get("data").and_then(|d| d.as_array());
                        match data {
                            Some(arr) if arr.is_empty() => {
                                Err("No models loaded".to_string())
                            }
                            Some(_) => Ok(()),
                            None => Ok(()), // Unexpected format, but server is running
                        }
                    } else {
                        Ok(()) // Could not parse, but server responded -- treat as OK
                    }
                }
                Ok(_) => Err("Server not running".to_string()),
                Err(e) if e.is_connect() => Err("Server not running".to_string()),
                Err(e) if e.is_timeout() => Err("Server not running".to_string()),
                Err(e) => Err(format!("Request failed -- {}", e)),
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
    pricing: Option<OpenRouterPricing>,
}

#[derive(Deserialize)]
struct OpenRouterPricing {
    /// USD per token as string, e.g. "0.000003"
    prompt: Option<String>,
    /// USD per token as string
    completion: Option<String>,
}

// ---- Ollama response types for /api/tags ----

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    details: OllamaModelDetails,
}

#[derive(Deserialize, Default)]
struct OllamaModelDetails {
    parameter_size: Option<String>,
    #[allow(dead_code)]
    quantization_level: Option<String>,
    #[allow(dead_code)]
    family: Option<String>,
}

/// Parse parameter size string like "3B", "7B", "70B", "4.3B", "270M" into
/// a numeric value in billions. Returns None if parsing fails.
fn parse_param_size_billions(s: &str) -> Option<f64> {
    let trimmed = s.trim().to_uppercase();
    if let Some(numeric) = trimmed.strip_suffix('B') {
        numeric.parse::<f64>().ok()
    } else if let Some(numeric) = trimmed.strip_suffix('M') {
        numeric.parse::<f64>().ok().map(|v| v / 1000.0)
    } else {
        None
    }
}

/// Assign tier based on parameter count: <7B=fast, 7-30B=balanced, >30B=capable.
/// Returns empty string when size is unknown (model appears in "All Models" section).
fn tier_from_param_size(param_size: Option<&str>) -> String {
    match param_size.and_then(parse_param_size_billions) {
        Some(b) if b < 7.0 => "fast".into(),
        Some(b) if b <= 30.0 => "balanced".into(),
        Some(_) => "capable".into(),
        None => String::new(),
    }
}

/// Fetch models for a provider: curated models first (with tier tags),
/// then API-fetched models not in the curated list (with tier="").
///
/// For OpenRouter, also caches per-model pricing in AppState.openrouter_pricing
/// so that get_usage_stats can calculate costs for dynamically-fetched models.
#[tauri::command]
pub async fn fetch_models(
    app_handle: tauri::AppHandle,
    provider: Provider,
    api_key: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<ModelWithMeta>, String> {
    let curated = curated_models(&provider);
    let curated_ids: Vec<String> = curated.iter().map(|m| m.id.clone()).collect();

    let api_models = fetch_api_models(&provider, &api_key, &state, &app_handle).await.unwrap_or_default();

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
/// For OpenRouter, also caches pricing data in AppState.
async fn fetch_api_models(
    provider: &Provider,
    api_key: &str,
    state: &tauri::State<'_, crate::state::AppState>,
    app_handle: &tauri::AppHandle,
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
                        "grok-4-1-fast-reasoning",
                        "grok-4-1-fast-non-reasoning",
                        "grok-4-0709",
                        "grok-4-fast-reasoning",
                        "grok-4-fast-non-reasoning",
                        "grok-code-fast-1",
                        "grok-3",
                        "grok-3-mini",
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

            // Build pricing cache from OpenRouter response
            let mut pricing_cache = HashMap::new();

            let models: Vec<ModelWithMeta> = parsed
                .data
                .into_iter()
                .filter(|m| m.context_length.unwrap_or(0) > 0)
                .map(|m| {
                    // Parse pricing strings to f64, convert per-token to per-million-token
                    let (input_price, output_price) = m
                        .pricing
                        .as_ref()
                        .and_then(|p| {
                            let prompt = p.prompt.as_ref()?.parse::<f64>().ok()?;
                            let completion = p.completion.as_ref()?.parse::<f64>().ok()?;
                            let inp_per_m = prompt * 1_000_000.0;
                            let out_per_m = completion * 1_000_000.0;
                            pricing_cache.insert(m.id.clone(), (inp_per_m, out_per_m));
                            Some((Some(inp_per_m), Some(out_per_m)))
                        })
                        .unwrap_or((None, None));

                    ModelWithMeta {
                        label: m.id.clone(),
                        id: m.id,
                        tier: String::new(),
                        input_price_per_m: input_price,
                        output_price_per_m: output_price,
                    }
                })
                .collect();

            // Cache pricing in AppState (replace entirely on success)
            *state.openrouter_pricing.lock().unwrap() = pricing_cache;

            Ok(models)
        }
        Provider::Ollama => {
            let base_url = super::providers::get_provider_base_url(app_handle, &Provider::Ollama);
            let tags_url = format!("{}/api/tags", base_url.trim_end_matches('/'));
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(|e| e.to_string())?;

            let resp = client
                .get(&tags_url)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
            let parsed: OllamaTagsResponse =
                serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

            Ok(parsed
                .models
                .into_iter()
                .map(|m| {
                    let tier = tier_from_param_size(m.details.parameter_size.as_deref());
                    ModelWithMeta {
                        id: m.name.clone(),
                        label: m.name, // Raw name per locked decision
                        tier,
                        input_price_per_m: None,
                        output_price_per_m: None,
                    }
                })
                .collect())
        }
        Provider::LMStudio => {
            let base_url = super::providers::get_provider_base_url(app_handle, &Provider::LMStudio);
            let models_url = format!("{}/v1/models", base_url.trim_end_matches('/'));
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
                .map_err(|e| e.to_string())?;

            let resp = client
                .get(&models_url)
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            let bytes = resp.bytes().await.map_err(|e| format!("Read error: {}", e))?;
            let parsed: OpenAIModelsResponse =
                serde_json::from_slice(&bytes).map_err(|e| format!("Parse error: {}", e))?;

            Ok(parsed
                .data
                .into_iter()
                .map(|m| ModelWithMeta {
                    label: m.id.clone(), // Raw ID per locked decision
                    id: m.id,
                    tier: String::new(), // LM Studio has no parameter_size in OpenAI-compat format
                    input_price_per_m: None,
                    output_price_per_m: None,
                })
                .collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_param_size_billions() {
        assert_eq!(parse_param_size_billions("3B"), Some(3.0));
        assert_eq!(parse_param_size_billions("7B"), Some(7.0));
        assert_eq!(parse_param_size_billions("70B"), Some(70.0));
        assert_eq!(parse_param_size_billions("4.3B"), Some(4.3));
        assert_eq!(parse_param_size_billions("1.5B"), Some(1.5));
        assert_eq!(parse_param_size_billions("270M"), Some(0.27));
        assert_eq!(parse_param_size_billions(""), None);
        assert_eq!(parse_param_size_billions("garbage"), None);
    }

    #[test]
    fn test_tier_from_param_size() {
        assert_eq!(tier_from_param_size(Some("3B")), "fast");
        assert_eq!(tier_from_param_size(Some("1.5B")), "fast");
        assert_eq!(tier_from_param_size(Some("270M")), "fast");
        assert_eq!(tier_from_param_size(Some("7B")), "balanced");
        assert_eq!(tier_from_param_size(Some("13B")), "balanced");
        assert_eq!(tier_from_param_size(Some("30B")), "balanced");
        assert_eq!(tier_from_param_size(Some("70B")), "capable");
        assert_eq!(tier_from_param_size(None), "");
        assert_eq!(tier_from_param_size(Some("")), "");
        assert_eq!(tier_from_param_size(Some("garbage")), "");
    }

    #[test]
    fn test_ollama_tags_response_deserialization() {
        let json = r#"{"models":[{"name":"llama3.2:3b-instruct-q4_K_M","details":{"parameter_size":"3B","quantization_level":"Q4_K_M","family":"llama"}}]}"#;
        let parsed: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.models.len(), 1);
        assert_eq!(parsed.models[0].name, "llama3.2:3b-instruct-q4_K_M");
        assert_eq!(parsed.models[0].details.parameter_size.as_deref(), Some("3B"));
        assert_eq!(parsed.models[0].details.quantization_level.as_deref(), Some("Q4_K_M"));
        assert_eq!(parsed.models[0].details.family.as_deref(), Some("llama"));
    }

    #[test]
    fn test_ollama_tags_response_missing_details() {
        let json = r#"{"models":[{"name":"custom-model"}]}"#;
        let parsed: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.models.len(), 1);
        assert_eq!(parsed.models[0].name, "custom-model");
        assert!(parsed.models[0].details.parameter_size.is_none());
        assert!(parsed.models[0].details.quantization_level.is_none());
    }

    #[test]
    fn test_ollama_tags_response_empty_models() {
        let json = r#"{"models":[]}"#;
        let parsed: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.models.len(), 0);
    }
}
