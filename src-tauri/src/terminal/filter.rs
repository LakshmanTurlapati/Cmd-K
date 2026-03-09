//! Sensitive data filtering for terminal output.
//!
//! Filters API keys, tokens, passwords, and other secrets from captured terminal
//! text before inclusion in TerminalContext. Prevents accidental exposure of
//! credentials to the AI model.
//!
//! Patterns cover the most common credential formats encountered in terminal sessions:
//! AWS access keys, OpenAI/xAI tokens, GitHub tokens, private keys, and generic
//! key=value export patterns.

use once_cell::sync::Lazy;
use regex::Regex;

/// Compiled regex patterns for sensitive data detection.
///
/// Evaluated once at first use (Lazy) to avoid repeated compilation overhead.
/// Each pattern replaces its match with [REDACTED].
static SENSITIVE_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // AWS Access Key ID (begins with AKIA followed by 16 uppercase chars/digits)
        Regex::new(r"AKIA[0-9A-Z]{16}").unwrap(),
        // Generic API key / token / password / secret assignments (e.g., api_key=abc123...)
        Regex::new(
            r#"(?i)(api[_-]?key|token|password|secret|bearer)\s*[=:]\s*['"]?[a-zA-Z0-9+/]{16,}['"]?"#,
        )
        .unwrap(),
        // xAI-style tokens (xai- prefix followed by 32+ alphanumerics)
        Regex::new(r"xai-[a-zA-Z0-9]{32,}").unwrap(),
        // OpenAI-style secret keys (sk- prefix followed by 32+ alphanumerics)
        Regex::new(r"sk-[a-zA-Z0-9]{32,}").unwrap(),
        // GitHub personal access tokens (ghp_, gho_, ghu_, ghs_, ghr_ prefixes)
        Regex::new(r"gh[pousr]_[A-Za-z0-9_]{36,}").unwrap(),
        // PEM private key headers (BEGIN RSA PRIVATE KEY, BEGIN EC PRIVATE KEY, etc.)
        Regex::new(r"-----BEGIN [A-Z]+ PRIVATE KEY-----").unwrap(),
        // Shell export statements with secret variable names (e.g., export SECRET_KEY=abc123...)
        Regex::new(
            r#"(?i)export\s+\w*(secret|key|token|password)\w*\s*=\s*\S{16,}"#,
        )
        .unwrap(),
        // Linux-specific secrets (WSL terminal context)
        // /etc/shadow password hashes ($1$, $5$, $6$, $y$ prefixes)
        Regex::new(r"\$[156y]\$[^\s:]+\$[a-zA-Z0-9./+]+").unwrap(),
        // .env file database/secret URLs (DATABASE_URL=..., REDIS_URL=..., etc.)
        Regex::new(
            r#"(?i)(DATABASE_URL|REDIS_URL|MONGO_URI)\s*=\s*\S{10,}"#,
        )
        .unwrap(),
        // Anthropic API keys (sk-ant- prefix)
        Regex::new(r"sk-ant-[a-zA-Z0-9_-]{32,}").unwrap(),
        // Google API keys (AIza prefix)
        Regex::new(r"AIza[0-9A-Za-z_-]{35}").unwrap(),
    ]
});

/// Filter sensitive data from terminal output text.
///
/// Applies all `SENSITIVE_PATTERNS` in sequence, replacing each match with
/// `[REDACTED]` to prevent accidental credential exposure to the AI model.
///
/// This function is pure (no side effects) and allocation-minimal (uses
/// `replace_all` which only allocates when a match is found).
pub fn filter_sensitive(text: &str) -> String {
    let mut result = text.to_string();
    for pattern in SENSITIVE_PATTERNS.iter() {
        result = pattern.replace_all(&result, "[REDACTED]").into_owned();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_key_redacted() {
        let input = "export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let output = filter_sensitive(input);
        assert!(!output.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_openai_key_redacted() {
        let input = "OPENAI_KEY=sk-abcdefghijklmnopqrstuvwxyz123456";
        let output = filter_sensitive(input);
        assert!(!output.contains("sk-abcdefghijklmnopqrstuvwxyz123456"));
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_safe_text_unchanged() {
        let input = "ls -la /home/user\ncd Documents";
        let output = filter_sensitive(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_shadow_hash_redacted() {
        let input = "root:$6$rounds=5000$salt$longhashvalue.:18000:0:99999:7:::";
        let output = filter_sensitive(input);
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_anthropic_key_redacted() {
        let input = "ANTHROPIC_API_KEY=sk-ant-api03-abcdefghijklmnopqrstuvwxyz1234567890abcdef";
        let output = filter_sensitive(input);
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_google_api_key_redacted() {
        let input = "GOOGLE_KEY=AIzaSyA1234567890abcdefghijklmnopqrstuv";
        let output = filter_sensitive(input);
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_database_url_redacted() {
        let input = "DATABASE_URL=postgres://user:password@localhost:5432/mydb";
        let output = filter_sensitive(input);
        assert!(output.contains("[REDACTED]"));
    }

    #[test]
    fn test_normal_linux_output_unchanged() {
        let input = "user@host:~/projects$ ls -la\ntotal 42\ndrwxr-xr-x 5 user user 4096 Mar 9 10:00 .";
        let output = filter_sensitive(input);
        assert_eq!(output, input);
    }
}
