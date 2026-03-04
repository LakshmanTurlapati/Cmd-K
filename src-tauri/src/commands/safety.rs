use once_cell::sync::Lazy;
use regex::RegexSet;
use tauri_plugin_http::reqwest;

// Keychain constants must match keychain.rs exactly
const SERVICE: &str = "com.lakshmanturlapati.cmd-k";
const ACCOUNT: &str = "xai_api_key";

/// Regex patterns for destructive command detection.
/// Uses word boundaries (\b) to avoid false positives on substrings.
static DESTRUCTIVE_PATTERNS: Lazy<RegexSet> = Lazy::new(|| {
    RegexSet::new([
        // File destruction
        r"\brm\s+-[^-]*r[^-]*f",   // rm -rf, rm -fr, rm -rdf, etc.
        r"\brm\s+-[^-]*f[^-]*r",   // rm -fr variants
        r"\brm\s+-r\b",            // rm -r (recursive without force)
        r"\bshred\b",
        r"\bunlink\b",
        r"\brmdir\b",
        // Git force operations
        r"\bgit\s+push\s+.*--force\b",
        r"\bgit\s+push\s+.*-f\b",
        r"\bgit\s+reset\s+--hard\b",
        r"\bgit\s+clean\s+.*-f\b",
        r"\bgit\s+branch\s+.*-D\b",
        r"\bgit\s+rebase\s+.*--force\b",
        // Database mutations
        r"(?i)\bDROP\s+TABLE\b",
        r"(?i)\bDROP\s+DATABASE\b",
        r"(?i)\bDROP\s+SCHEMA\b",
        r"(?i)\bDROP\s+INDEX\b",
        r"(?i)\bTRUNCATE\s+TABLE\b",
        // DELETE FROM without WHERE (ends at semicolon or end-of-string)
        r"(?i)\bDELETE\s+FROM\s+\S+\s*;",
        r"(?i)\bDELETE\s+FROM\s+\S+\s*$",
        // System / permission / disk operations
        r"\bsudo\s+rm\b",
        r"\bchmod\s+777\b",
        r"\bmkfs\b",
        r"\bdd\s+if=",
        r"\bshutdown\b",
        r"\breboot\b",
        r"\bpkill\s+-9\b",
        r"\bkillall\b",
        r"\bfdisk\b",
        r"\bdiskutil\s+erase\b",
        r"\bformat\s+[A-Za-z]:",
        r">\s*/dev/sd[a-z]",
        r">\s*/dev/disk[0-9]",
        // Windows-specific destructive commands
        r"(?i)\bdel\s+/s\b",
        r"(?i)\brd\s+/s\b",
        r"(?i)\brmdir\s+/s\b",
        r"(?i)\bformat\s+[A-Za-z]:\b",
        r"(?i)\bRemove-Item\s+.*-Recurse\s+.*-Force\b",
        r"(?i)\bRemove-Item\s+.*-Force\s+.*-Recurse\b",
        r"(?i)\bReg\s+Delete\b",
        r"(?i)\bbcdedit\b",
        r"(?i)\bdiskpart\b",
        r"(?i)\btaskkill\s+/f\b",
        r"(?i)\bStop-Process\s+.*-Force\b",
        // CMD file/system commands
        r"(?i)\berase\s+/[sf]\b",
        r"(?i)\bdel\s+/f\b",
        r"(?i)\bcipher\s+/w\b",
        r"(?i)\bshutdown\s+/[srp]\b",
        // Recovery inhibition (MITRE ATT&CK T1490)
        r"(?i)\bvssadmin\s+.*delete\s+shadows\b",
        r"(?i)\bvssadmin\s+.*resize\s+shadowstorage\b",
        r"(?i)\bwmic\s+shadowcopy\s+delete\b",
        r"(?i)\bwbadmin\s+delete\b",
        // Service manipulation (permanent changes only)
        r"(?i)\bsc\s+delete\b",
        r"(?i)\bsc\s+config\s+.*disabled\b",
        r"(?i)\bSet-Service\s+.*Disabled\b",
        // Network destruction
        r"(?i)\bnetsh\s+advfirewall\s+reset\b",
        r"(?i)\bnetsh\s+advfirewall\s+.*state\s+off\b",
        r"(?i)\bnetsh\s+int\s+ip\s+reset\b",
        r"(?i)\bnetsh\s+winsock\s+reset\b",
        r"(?i)\bDisable-NetAdapter\b",
        // User / permission manipulation
        r"(?i)\bnet\s+user\s+.*\/delete\b",
        r"(?i)\bnet\s+user\s+.*\/active:no\b",
        r"(?i)\bnet\s+localgroup\s+.*\/delete\b",
        r"(?i)\bicacls\s+.*\/(grant|deny|remove)\b",
        r"(?i)\btakeown\s+.*\/f\b",
        r"(?i)\bRemove-LocalUser\b",
        // Disk / partition / volume
        r"(?i)\bFormat-Volume\b",
        r"(?i)\bClear-Disk\b",
        r"(?i)\bRemove-Partition\b",
        r"(?i)\bInitialize-Disk\b",
        r"(?i)\bmanage-bde\s+-(lock|off)\b",
        // Registry manipulation
        r"(?i)\breg\s+import\b",
        r"(?i)\breg\s+restore\b",
        r"(?i)\bregedit\s+.*\/s\b",
        // PowerShell dangerous patterns
        r"(?i)\bInvoke-Expression\b",
        r"(?i)\bIEX\s",
        r"(?i)\bSet-ExecutionPolicy\s+(Bypass|Unrestricted)\b",
        r"(?i)\bClear-Content\b",
        r"(?i)\bClear-EventLog\b",
        r"(?i)\bwevtutil\s+cl\b",
        r"(?i)\bRemove-Computer\b",
        r"(?i)\bRestart-Computer\s+.*-Force\b",
        r"(?i)\bStop-Computer\s+.*-Force\b",
        // WMIC destructive commands
        r"(?i)\bwmic\s+process\s+.*\b(delete|call\s+terminate)\b",
        r"(?i)\bwmic\s+product\s+.*call\s+uninstall\b",
        r"(?i)\bwmic\s+os\s+.*call\s+(shutdown|reboot)\b",
        // WSL pass-through
        r"(?i)\bwsl(\.exe)?\s+.*\brm\s+-[^-]*r",
        r"(?i)\bwsl(\.exe)?\s+.*\b(dd\s+if=|mkfs|shred)\b",
        r"(?i)\bwsl(\.exe)?\s+--unregister\b",
        // Boot / system integrity
        r"(?i)\bbootrec\s+\/(rebuildbcd|fixmbr|fixboot)\b",
    ])
    .expect("DESTRUCTIVE_PATTERNS regex set failed to compile")
});

/// Check whether a command string matches any known destructive patterns.
///
/// Returns `true` if the command is potentially destructive, `false` otherwise.
/// Uses a compiled RegexSet for zero-allocation pattern matching at call time.
#[tauri::command]
pub fn check_destructive(command: String) -> bool {
    DESTRUCTIVE_PATTERNS.is_match(&command)
}

/// Get a plain-English explanation of why a command is destructive via the xAI API.
///
/// - Reads the API key from macOS Keychain (same account as ai.rs).
/// - Makes a non-streaming POST to /v1/chat/completions with temperature 0.0.
/// - Sends the result (or a safe fallback) via the IPC Channel.
#[tauri::command]
pub async fn get_destructive_explanation(
    command: String,
    model: String,
    on_result: tauri::ipc::Channel<String>,
) -> Result<(), String> {
    eprintln!("[safety] get_destructive_explanation called, model={}", model);

    // Read API key from Keychain
    let entry = keyring::Entry::new(SERVICE, ACCOUNT)
        .map_err(|e| format!("Keyring error: {}", e))?;
    let api_key = entry
        .get_password()
        .map_err(|_| "No API key configured. Open Settings to add one.".to_string())?;

    let system_prompt = "You are a safety assistant. In one plain-English sentence (max 20 words), \
        explain what the following terminal command does and why it is destructive. \
        Be specific about what data or state it will permanently change or delete. \
        No markdown, no code fences.";

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system_prompt },
            { "role": "user", "content": command }
        ],
        "stream": false,
        "temperature": 0.0
    })
    .to_string();

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.x.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let status = response.status().as_u16();
    eprintln!("[safety] HTTP status={}", status);

    let explanation = if status == 200 {
        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        let parsed: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or(serde_json::Value::Null);

        parsed["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("This command makes irreversible changes.")
            .to_string()
    } else {
        "This command makes irreversible changes.".to_string()
    };

    on_result
        .send(explanation)
        .map_err(|e| format!("Channel error: {}", e))?;

    Ok(())
}
