use std::collections::HashMap;
use serde::Serialize;

/// Per-model usage statistics with optional cost estimation.
#[derive(Serialize)]
pub struct UsageStatEntry {
    pub provider: String,
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub query_count: u32,
    /// Estimated cost in USD. None when pricing is unavailable.
    pub estimated_cost: Option<f64>,
    /// Whether pricing data was available for cost calculation.
    pub pricing_available: bool,
}

/// Aggregated usage stats response returned to the frontend.
#[derive(Serialize)]
pub struct UsageStatsResponse {
    pub entries: Vec<UsageStatEntry>,
    /// Sum of all entries with known pricing. None if no entries have pricing.
    pub session_total_cost: Option<f64>,
}

/// Return accumulated usage stats with estimated costs per model.
///
/// Pricing lookup order:
/// 1. Curated model pricing (hardcoded for known models across all providers)
/// 2. OpenRouter dynamic pricing (cached from their /api/v1/models response)
/// 3. If neither source has pricing, `pricing_available: false` and `estimated_cost: None`
#[tauri::command]
pub fn get_usage_stats(
    state: tauri::State<'_, crate::state::AppState>,
) -> UsageStatsResponse {
    let usage = state.usage.lock().unwrap();
    let curated_pricing = super::models::curated_models_pricing();
    let or_pricing = state.openrouter_pricing.lock().unwrap();

    let mut entries = Vec::new();
    let mut total_cost: f64 = 0.0;
    let mut any_priced = false;

    for ((provider, model), entry) in usage.entries() {
        // Look up pricing: curated first, then OpenRouter cache
        let pricing = curated_pricing
            .get(model.as_str())
            .copied()
            .or_else(|| or_pricing.get(model.as_str()).copied());

        let (estimated_cost, pricing_available) = match pricing {
            Some((input_price, output_price)) => {
                let cost = (entry.total_input_tokens as f64 * input_price / 1_000_000.0)
                    + (entry.total_output_tokens as f64 * output_price / 1_000_000.0);
                any_priced = true;
                total_cost += cost;
                (Some(cost), true)
            }
            None => (None, false),
        };

        entries.push(UsageStatEntry {
            provider: provider.clone(),
            model: model.clone(),
            input_tokens: entry.total_input_tokens,
            output_tokens: entry.total_output_tokens,
            query_count: entry.query_count,
            estimated_cost,
            pricing_available,
        });
    }

    UsageStatsResponse {
        entries,
        session_total_cost: if any_priced { Some(total_cost) } else { None },
    }
}

/// Clear all session usage stats.
#[tauri::command]
pub fn reset_usage(state: tauri::State<'_, crate::state::AppState>) {
    state.usage.lock().unwrap().reset();
}
