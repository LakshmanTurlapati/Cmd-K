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
        Provider::OpenRouter => vec![],
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
    pricing: Option<OpenRouterPricing>,
}

#[derive(Deserialize)]
struct OpenRouterPricing {
    /// USD per token as string, e.g. "0.000003"
    prompt: Option<String>,
    /// USD per token as string
    completion: Option<String>,
}

/// Fetch models for a provider: curated models first (with tier tags),
/// then API-fetched models not in the curated list (with tier="").
///
/// For OpenRouter, also caches per-model pricing in AppState.openrouter_pricing
/// so that get_usage_stats can calculate costs for dynamically-fetched models.
#[tauri::command]
pub async fn fetch_models(
    provider: Provider,
    api_key: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<Vec<ModelWithMeta>, String> {
    let curated = curated_models(&provider);
    let curated_ids: Vec<String> = curated.iter().map(|m| m.id.clone()).collect();

    let api_models = fetch_api_models(&provider, &api_key, &state).await.unwrap_or_default();

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
    }
}
