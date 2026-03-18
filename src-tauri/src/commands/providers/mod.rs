pub mod anthropic;
pub mod gemini;
pub mod openai_compat;

use serde::{Deserialize, Serialize};

/// Supported AI providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Gemini,
    #[serde(rename = "xai")]
    XAI,
    OpenRouter,
    Ollama,
    #[serde(rename = "lmstudio")]
    LMStudio,
}

/// Groups providers by their streaming API format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    OpenAICompat,
    Anthropic,
    Gemini,
}

impl Provider {
    /// Whether this provider runs locally (no API key, configurable base URL).
    pub fn is_local(&self) -> bool {
        matches!(self, Provider::Ollama | Provider::LMStudio)
    }

    /// Whether this provider requires an API key stored in the keychain.
    #[cfg(test)]
    pub fn requires_api_key(&self) -> bool {
        !self.is_local()
    }

    /// Default base URL for local providers. Empty string for cloud providers.
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Provider::Ollama => "http://localhost:11434",
            Provider::LMStudio => "http://localhost:1234",
            _ => "",
        }
    }

    /// Settings store key for the user-configured base URL. Empty for cloud providers.
    pub fn base_url_store_key(&self) -> &'static str {
        match self {
            Provider::Ollama => "ollama_base_url",
            Provider::LMStudio => "lmstudio_base_url",
            _ => "",
        }
    }

    /// Keychain account name for this provider's API key.
    pub fn keychain_account(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai_api_key",
            Provider::Anthropic => "anthropic_api_key",
            Provider::Gemini => "gemini_api_key",
            Provider::XAI => "xai_api_key",
            Provider::OpenRouter => "openrouter_api_key",
            Provider::Ollama | Provider::LMStudio => "",
        }
    }

    /// API endpoint URL for chat completions.
    pub fn api_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1/chat/completions",
            Provider::Anthropic => "https://api.anthropic.com/v1/messages",
            Provider::Gemini => "https://generativelanguage.googleapis.com/v1beta/models/",
            Provider::XAI => "https://api.x.ai/v1/chat/completions",
            Provider::OpenRouter => "https://openrouter.ai/api/v1/chat/completions",
            Provider::Ollama => "http://localhost:11434/v1/chat/completions",
            Provider::LMStudio => "http://localhost:1234/v1/chat/completions",
        }
    }

    /// Default streaming timeout in seconds.
    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            Provider::XAI => 10,
            Provider::Ollama | Provider::LMStudio => 120,
            _ => 30,
        }
    }

    /// Human-readable provider name for UI and error messages.
    pub fn display_name(&self) -> &'static str {
        match self {
            Provider::OpenAI => "OpenAI",
            Provider::Anthropic => "Anthropic",
            Provider::Gemini => "Google Gemini",
            Provider::XAI => "xAI",
            Provider::OpenRouter => "OpenRouter",
            Provider::Ollama => "Ollama",
            Provider::LMStudio => "LM Studio",
        }
    }

    /// URL where the user can manage their API key.
    pub fn console_url(&self) -> &'static str {
        match self {
            Provider::OpenAI => "platform.openai.com",
            Provider::Anthropic => "console.anthropic.com",
            Provider::Gemini => "aistudio.google.com",
            Provider::XAI => "console.x.ai",
            Provider::OpenRouter => "openrouter.ai/keys",
            Provider::Ollama => "ollama.com",
            Provider::LMStudio => "lmstudio.ai",
        }
    }

    /// Which streaming adapter handles this provider.
    pub fn adapter_kind(&self) -> AdapterKind {
        match self {
            Provider::OpenAI | Provider::XAI | Provider::OpenRouter | Provider::Ollama | Provider::LMStudio => {
                AdapterKind::OpenAICompat
            }
            Provider::Anthropic => AdapterKind::Anthropic,
            Provider::Gemini => AdapterKind::Gemini,
        }
    }
}

/// Check HTTP status and return a provider-specific error message.
pub fn handle_http_status(provider: &Provider, status: u16) -> Result<(), String> {
    match status {
        200 => Ok(()),
        401 => Err(format!(
            "{}: Authentication failed. Check your API key at {}.",
            provider.display_name(),
            provider.console_url()
        )),
        429 => Err(format!(
            "{}: Rate limited. Wait a moment and try again.",
            provider.display_name()
        )),
        _ => Err(format!(
            "{}: API error ({}). Try again.",
            provider.display_name(),
            status
        )),
    }
}

/// Normalize user-supplied base URL: ensure http:// prefix, strip trailing slash.
#[cfg(test)]
pub fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    }
}

/// Read the configured base URL for a local provider from settings.json.
/// Falls back to the provider's default base URL if not configured.
pub fn get_provider_base_url(app_handle: &tauri::AppHandle, provider: &Provider) -> String {
    use tauri_plugin_store::StoreExt;
    let key = provider.base_url_store_key();
    if key.is_empty() {
        return provider.default_base_url().to_string();
    }
    app_handle
        .store("settings.json")
        .ok()
        .and_then(|s| s.get(key))
        .and_then(|v| v.as_str().map(String::from))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| provider.default_base_url().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_local() {
        assert!(Provider::Ollama.is_local());
        assert!(Provider::LMStudio.is_local());
        assert!(!Provider::OpenAI.is_local());
        assert!(!Provider::Anthropic.is_local());
        assert!(!Provider::Gemini.is_local());
        assert!(!Provider::XAI.is_local());
        assert!(!Provider::OpenRouter.is_local());
    }

    #[test]
    fn test_requires_api_key() {
        assert!(!Provider::Ollama.requires_api_key());
        assert!(!Provider::LMStudio.requires_api_key());
        assert!(Provider::OpenAI.requires_api_key());
    }

    #[test]
    fn test_default_base_url() {
        assert_eq!(Provider::Ollama.default_base_url(), "http://localhost:11434");
        assert_eq!(Provider::LMStudio.default_base_url(), "http://localhost:1234");
        assert_eq!(Provider::OpenAI.default_base_url(), "");
    }

    #[test]
    fn test_base_url_store_key() {
        assert_eq!(Provider::Ollama.base_url_store_key(), "ollama_base_url");
        assert_eq!(Provider::LMStudio.base_url_store_key(), "lmstudio_base_url");
        assert_eq!(Provider::OpenAI.base_url_store_key(), "");
    }

    #[test]
    fn test_normalize_base_url() {
        assert_eq!(normalize_base_url("localhost:11434"), "http://localhost:11434");
        assert_eq!(normalize_base_url("http://localhost:11434"), "http://localhost:11434");
        assert_eq!(normalize_base_url("http://localhost:11434/"), "http://localhost:11434");
        assert_eq!(normalize_base_url("https://myserver:1234"), "https://myserver:1234");
        assert_eq!(normalize_base_url("  localhost:1234/  "), "http://localhost:1234");
    }

    #[test]
    fn test_display_name() {
        assert_eq!(Provider::Ollama.display_name(), "Ollama");
        assert_eq!(Provider::LMStudio.display_name(), "LM Studio");
    }

    #[test]
    fn test_default_timeout_secs() {
        assert_eq!(Provider::Ollama.default_timeout_secs(), 120);
        assert_eq!(Provider::LMStudio.default_timeout_secs(), 120);
        assert_eq!(Provider::OpenAI.default_timeout_secs(), 30);
    }

    #[test]
    fn test_adapter_kind() {
        assert_eq!(Provider::Ollama.adapter_kind(), AdapterKind::OpenAICompat);
        assert_eq!(Provider::LMStudio.adapter_kind(), AdapterKind::OpenAICompat);
    }

    #[test]
    fn test_serde_roundtrip() {
        let json = serde_json::to_string(&Provider::Ollama).unwrap();
        assert_eq!(json, "\"ollama\"");
        let json = serde_json::to_string(&Provider::LMStudio).unwrap();
        assert_eq!(json, "\"lmstudio\"");
        let parsed: Provider = serde_json::from_str("\"ollama\"").unwrap();
        assert_eq!(parsed, Provider::Ollama);
        let parsed: Provider = serde_json::from_str("\"lmstudio\"").unwrap();
        assert_eq!(parsed, Provider::LMStudio);
    }
}
