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
}

/// Groups providers by their streaming API format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdapterKind {
    OpenAICompat,
    Anthropic,
    Gemini,
}

impl Provider {
    /// Keychain account name for this provider's API key.
    pub fn keychain_account(&self) -> &'static str {
        match self {
            Provider::OpenAI => "openai_api_key",
            Provider::Anthropic => "anthropic_api_key",
            Provider::Gemini => "gemini_api_key",
            Provider::XAI => "xai_api_key",
            Provider::OpenRouter => "openrouter_api_key",
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
        }
    }

    /// Default streaming timeout in seconds.
    pub fn default_timeout_secs(&self) -> u64 {
        match self {
            Provider::XAI => 10,
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
        }
    }

    /// Which streaming adapter handles this provider.
    pub fn adapter_kind(&self) -> AdapterKind {
        match self {
            Provider::OpenAI | Provider::XAI | Provider::OpenRouter => AdapterKind::OpenAICompat,
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
