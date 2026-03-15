//! Smart terminal context preparation for AI prompts.
//!
//! Replaces the hard-coded 25-line truncation with an intelligent pipeline:
//! 1. Strip ANSI escape sequences and non-printable control characters
//! 2. Compute a token budget from the selected model's context window
//! 3. Segment terminal output by command boundaries (shell prompt patterns)
//! 4. Truncate oldest complete command+output segments to fit within budget
//!
//! This module is purely cross-platform -- no `cfg(target_os)` anywhere.

use once_cell::sync::Lazy;
use regex::Regex;

/// Terminal context budget as a fraction of the model's context window.
const TERMINAL_BUDGET_FRACTION: f64 = 0.12;

/// Default context window (in tokens) for unknown/custom models.
const DEFAULT_CONTEXT_WINDOW: u32 = 128_000;

/// Approximate characters per token for budget estimation.
const CHARS_PER_TOKEN: u32 = 4;

/// Matches all common ANSI escape sequences:
/// 1. CSI sequences: \x1b[ ... (letter) -- covers colors, cursor movement, erase
/// 2. OSC sequences: \x1b] ... (BEL or ST) -- covers title setting, hyperlinks
/// 3. Character set selection: \x1b( or \x1b) followed by charset designator
/// 4. Simple ESC sequences: \x1b followed by single char
/// 5. CSI with ? prefix: \x1b[? ... (letter) -- covers DEC private modes
static ANSI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"\x1b\[\??[0-9;]*[A-Za-z]",           // CSI sequences (including DEC private)
        r"|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)", // OSC sequences (BEL or ST terminated)
        r"|\x1b[()][AB012]",                    // Character set selection
        r"|\x1b[>=NHMDEc7\-~]",                // Simple ESC sequences
    ))
    .unwrap()
});

/// Matches non-printable control characters EXCEPT \n (\x0A) and \t (\x09).
/// Covers \x00-\x08, \x0B-\x0C, \x0E-\x1F, and \x7F (DEL).
static CONTROL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]").unwrap()
});

/// Shell prompt patterns for command boundary detection.
/// Uses multiline mode so `^` matches line starts.
static PROMPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"(?m)^(?:",
        r"\w+@[\w.\-]+:[^\s]*[$#]\s?",        // user@host:/path$ (bash/zsh)
        r"|PS [A-Z]:\\[^>]*>\s?",              // PS C:\path> (PowerShell)
        r"|PS>\s?",                             // PS> (PowerShell minimal)
        r"|[A-Z]:\\[^>]*>\s?",                  // C:\path> (CMD)
        r"|[%$#>] ",                            // Generic prompt chars followed by space
        r"|>>>? ",                              // Python REPL (>>> or >>)
        r"|\w+@[\w.\-]+\s+[^\s]+[>$#]\s?",    // fish: user@host /path>
        r")",
    ))
    .unwrap()
});

/// Known context window sizes by model ID prefix (in tokens).
static CONTEXT_WINDOWS: Lazy<Vec<(&'static str, u32)>> = Lazy::new(|| {
    vec![
        // Anthropic Claude -- 200K
        ("claude-", 200_000),
        // OpenAI GPT-4o family -- 128K
        ("gpt-4o", 128_000),
        ("gpt-4.1", 1_047_576),
        ("gpt-5", 1_047_576),
        // OpenAI o-series -- 200K
        ("o1", 200_000),
        ("o3", 200_000),
        ("o4", 200_000),
        // Gemini -- 1M
        ("gemini-2.5", 1_048_576),
        ("gemini-3", 1_048_576),
        ("gemini-2.0", 1_048_576),
        // xAI Grok -- 131K
        ("grok-", 131_072),
    ]
});

/// Internal representation of a parsed command+output pair.
struct CommandSegment {
    /// The prompt line + command text (may be empty for pre-prompt output).
    prompt_and_command: String,
    /// Output lines following the command.
    output: String,
}

/// Strip ANSI escape sequences and non-printable control characters.
/// Also normalizes `\r\n` to `\n`.
fn strip_ansi_and_control(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let no_ansi = ANSI_RE.replace_all(&normalized, "");
    let no_control = CONTROL_RE.replace_all(&no_ansi, "");
    no_control.into_owned()
}

/// Compute the terminal context budget in characters.
/// Uses 12% of the context window, multiplied by 4 chars/token.
fn compute_budget_chars(context_window: u32) -> usize {
    let cw = if context_window == 0 {
        DEFAULT_CONTEXT_WINDOW
    } else {
        context_window
    };
    let budget_tokens = (cw as f64 * TERMINAL_BUDGET_FRACTION) as usize;
    budget_tokens * CHARS_PER_TOKEN as usize
}

/// Segment terminal text into command+output pairs by detecting shell prompt patterns.
fn segment_commands(text: &str) -> Vec<CommandSegment> {
    let lines: Vec<&str> = text.lines().collect();
    let mut segments: Vec<CommandSegment> = Vec::new();
    let mut current_prompt = String::new();
    let mut current_output = String::new();

    for line in &lines {
        if PROMPT_RE.is_match(line) {
            // Save previous segment if non-empty
            if !current_prompt.is_empty() || !current_output.is_empty() {
                segments.push(CommandSegment {
                    prompt_and_command: current_prompt,
                    output: current_output,
                });
            }
            current_prompt = line.to_string();
            current_output = String::new();
        } else {
            current_output.push_str(line);
            current_output.push('\n');
        }
    }

    // Push last segment
    if !current_prompt.is_empty() || !current_output.is_empty() {
        segments.push(CommandSegment {
            prompt_and_command: current_prompt,
            output: current_output,
        });
    }

    segments
}

/// Truncate text to fit within a character budget, preserving the most recent
/// complete command+output segments and dropping the oldest.
fn smart_truncate(text: &str, budget_chars: usize) -> String {
    // Fast path: fits in budget
    if text.len() <= budget_chars {
        return text.to_string();
    }

    let segments = segment_commands(text);

    // Fallback: no prompt patterns found -- simple tail truncation
    if segments.is_empty()
        || (segments.len() == 1 && segments[0].prompt_and_command.is_empty())
    {
        let start = text.len().saturating_sub(budget_chars);
        // Find next newline to avoid cutting mid-line
        let start = text[start..]
            .find('\n')
            .map(|i| start + i + 1)
            .unwrap_or(start);
        return text[start..].to_string();
    }

    // Build from newest to oldest, dropping oldest when over budget
    let mut selected: Vec<&CommandSegment> = Vec::new();
    let mut total_chars: usize = 0;

    for segment in segments.iter().rev() {
        let seg_chars = segment.prompt_and_command.len() + segment.output.len();
        if total_chars + seg_chars > budget_chars && !selected.is_empty() {
            break; // Drop this and all older segments
        }
        // Always include the most recent segment even if it exceeds budget
        selected.push(segment);
        total_chars += seg_chars;
    }

    selected.reverse();

    // If single segment exceeds budget, tail-truncate its output
    if selected.len() == 1 && total_chars > budget_chars {
        let seg = selected[0];
        let prompt_len = seg.prompt_and_command.len() + 1; // +1 for newline
        let output_budget = budget_chars.saturating_sub(prompt_len);
        let output = &seg.output;
        let start = output.len().saturating_sub(output_budget);
        let start = output[start..]
            .find('\n')
            .map(|i| start + i + 1)
            .unwrap_or(start);
        return format!("{}\n{}", seg.prompt_and_command, &output[start..]);
    }

    // Reassemble selected segments
    selected
        .iter()
        .map(|s| {
            if s.output.is_empty() {
                s.prompt_and_command.clone()
            } else {
                format!("{}\n{}", s.prompt_and_command, s.output.trim_end())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Prepare terminal context for AI consumption.
///
/// Pipeline: normalize CRLF -> strip ANSI + control chars -> compute budget -> smart truncate.
/// Returns empty string for empty input.
///
/// Note: `filter_sensitive()` should be called AFTER this function in the prompt building path.
pub fn prepare_terminal_context(raw_text: &str, context_window: u32) -> String {
    if raw_text.is_empty() {
        return String::new();
    }
    let stripped = strip_ansi_and_control(raw_text);
    if stripped.is_empty() {
        return String::new();
    }
    let budget_chars = compute_budget_chars(context_window);
    smart_truncate(&stripped, budget_chars)
}

/// Look up context window size (in tokens) for a model by its ID.
/// Uses prefix matching against known model families.
/// Returns `DEFAULT_CONTEXT_WINDOW` (128K) for unrecognized models.
pub(crate) fn context_window_for_model(model_id: &str) -> u32 {
    for (prefix, window) in CONTEXT_WINDOWS.iter() {
        if model_id.starts_with(prefix) {
            return *window;
        }
    }
    DEFAULT_CONTEXT_WINDOW
}

#[cfg(test)]
mod tests {
    use super::*;

    // === ANSI stripping tests ===

    #[test]
    fn test_strip_ansi_basic() {
        let input = "\x1b[31mred\x1b[0m";
        assert_eq!(strip_ansi_and_control(input), "red");
    }

    #[test]
    fn test_strip_ansi_osc() {
        let input = "\x1b]0;title\x07text";
        assert_eq!(strip_ansi_and_control(input), "text");
    }

    #[test]
    fn test_strip_control_chars() {
        // \x00, \x0B, \x7F should be removed; \n and \t preserved
        assert_eq!(strip_ansi_and_control("\x00"), "");
        assert_eq!(strip_ansi_and_control("\x0B"), "");
        assert_eq!(strip_ansi_and_control("\x7F"), "");
        assert_eq!(strip_ansi_and_control("\n"), "\n");
        assert_eq!(strip_ansi_and_control("\t"), "\t");
    }

    #[test]
    fn test_strip_combined() {
        let input = "\x1b[32mhello\x1b[0m \x00world\x7F";
        assert_eq!(strip_ansi_and_control(input), "hello world");
    }

    #[test]
    fn test_crlf_normalization() {
        let input = "line1\r\nline2\r\n";
        assert_eq!(strip_ansi_and_control(input), "line1\nline2\n");
    }

    // === Budget calculation tests ===

    #[test]
    fn test_budget_calculation() {
        // 200K context window: 200000 * 0.12 * 4 = 96000
        assert_eq!(compute_budget_chars(200_000), 96_000);
        // 128K default: 128000 * 0.12 * 4 = 61440
        assert_eq!(compute_budget_chars(128_000), 61_440);
    }

    #[test]
    fn test_budget_zero_window() {
        // 0 context window should use DEFAULT_CONTEXT_WINDOW (128K)
        assert_eq!(compute_budget_chars(0), compute_budget_chars(128_000));
    }

    // === Context window lookup tests ===

    #[test]
    fn test_context_window_lookup() {
        assert_eq!(context_window_for_model("claude-sonnet-4-6"), 200_000);
        assert_eq!(context_window_for_model("gpt-4o"), 128_000);
        assert_eq!(context_window_for_model("unknown-model"), 128_000);
    }

    // === Segmentation tests ===

    #[test]
    fn test_segment_single_command() {
        let text = "user@host:~$ ls\nfile1\nfile2";
        let segments = segment_commands(text);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].prompt_and_command, "user@host:~$ ls");
        assert!(segments[0].output.contains("file1"));
        assert!(segments[0].output.contains("file2"));
    }

    #[test]
    fn test_segment_multiple_commands() {
        let text = "user@host:~$ ls\nfile1\nuser@host:~$ pwd\n/home/user";
        let segments = segment_commands(text);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].prompt_and_command, "user@host:~$ ls");
        assert_eq!(segments[1].prompt_and_command, "user@host:~$ pwd");
    }

    #[test]
    fn test_segment_no_prompts() {
        let text = "some random output\nwithout any prompts";
        let segments = segment_commands(text);
        assert_eq!(segments.len(), 1);
        assert!(segments[0].prompt_and_command.is_empty());
    }

    // === Truncation tests ===

    #[test]
    fn test_truncate_fits_budget() {
        let text = "short text";
        assert_eq!(smart_truncate(text, 1000), "short text");
    }

    #[test]
    fn test_truncate_drops_oldest() {
        // 3 segments, budget fits ~2 segments
        let text = "user@host:~$ cmd1\noutput1 padding\nuser@host:~$ cmd2\noutput2 padding\nuser@host:~$ cmd3\noutput3 padding";
        let segments = segment_commands(text);
        assert_eq!(segments.len(), 3);

        // Budget that fits 2 segments but not 3
        let seg2_size = segments[1].prompt_and_command.len() + segments[1].output.len();
        let seg3_size = segments[2].prompt_and_command.len() + segments[2].output.len();
        let budget = seg2_size + seg3_size + 5; // fits 2 but not 3

        let result = smart_truncate(text, budget);
        assert!(!result.contains("cmd1"), "oldest segment should be dropped");
        assert!(result.contains("cmd2"));
        assert!(result.contains("cmd3"));
    }

    #[test]
    fn test_truncate_single_overflow() {
        // One command with a lot of output, budget is small
        let mut text = String::from("user@host:~$ bigcmd\n");
        for i in 0..100 {
            text.push_str(&format!("output line {}\n", i));
        }
        let result = smart_truncate(&text, 200);
        assert!(result.contains("user@host:~$ bigcmd"));
        assert!(result.len() <= 250); // some slack for line boundary
        // Should contain the tail of the output, not the beginning
        assert!(result.contains("output line 99"));
        assert!(!result.contains("output line 0\n"));
    }

    #[test]
    fn test_truncate_no_prompts_fallback() {
        // No prompt patterns -- should do simple tail truncation
        let mut text = String::new();
        for i in 0..100 {
            text.push_str(&format!("line {}\n", i));
        }
        let result = smart_truncate(&text, 100);
        // Should contain tail lines, not head lines
        assert!(result.contains("line 99"));
        assert!(!result.contains("line 0\n"));
    }

    // === Full pipeline tests ===

    #[test]
    fn test_prepare_full_pipeline() {
        let raw = "\x1b[32muser@host:~$\x1b[0m ls\nfile1\nfile2\n\x1b[32muser@host:~$\x1b[0m pwd\n/home/user";
        let result = prepare_terminal_context(raw, 128_000);
        // ANSI codes should be stripped
        assert!(!result.contains("\x1b"));
        // Content should be preserved
        assert!(result.contains("file1"));
        assert!(result.contains("/home/user"));
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(prepare_terminal_context("", 128_000), "");
    }
}
