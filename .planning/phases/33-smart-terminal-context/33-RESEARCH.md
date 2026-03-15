# Phase 33: Smart Terminal Context - Research

**Researched:** 2026-03-14
**Domain:** Terminal text processing, ANSI stripping, token budgeting, command segmentation
**Confidence:** HIGH

## Summary

Phase 33 replaces the current hard-coded 25-line terminal output truncation in `build_user_message()` (ai.rs line 136-142) with an intelligent context preparation pipeline. The pipeline strips ANSI escape sequences and control characters, segments terminal output by command boundaries using shell prompt patterns, and truncates oldest segments to fit within a token budget derived from the selected model's context window.

This is a purely backend, cross-platform module. The existing `infer_shell_from_text()` function in `terminal/mod.rs` already has prompt pattern matching for bash/zsh/fish/PowerShell/CMD that can be repurposed for command boundary detection. The `ModelWithMeta` struct in `models.rs` needs a `context_window` field added. The new module lives at `src-tauri/src/terminal/context.rs` and exposes a single public function `prepare_terminal_context()` that ai.rs calls.

**Primary recommendation:** Build a pure-function pipeline in `terminal/context.rs` with regex-based ANSI stripping, prompt-pattern command segmentation, and chars/4 token estimation. Integrate by replacing the 25-line truncation in `build_user_message()` with a call to `prepare_terminal_context(raw_text, context_window_tokens)`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Regex-based ANSI stripping (not state-machine parser, not external crate)
- Strip at AI call time, not at capture time -- raw text preserved for other uses
- Strip ALL non-printable control characters (\x00-\x1F except \n and \t, plus \x7F) in addition to ANSI escape sequences
- Processing order: ANSI strip first -> sensitive data filter (filter.rs) second
- Add `context_window` field to existing curated model structs in models.rs
- Default 128K context window for unknown/custom models (OpenRouter user-added)
- Budget of 10-15% covers terminal output ONLY -- independent of conversation history turnLimit
- Token estimation via ~4 chars/token heuristic (no tokenizer dependency)
- Detect command boundaries via shell prompt pattern matching (user@host:path$, PS C:\>, >>>, etc.)
- Build on existing `infer_shell_from_text()` prompt patterns
- Truncation preserves most recent complete command+output segments, drops oldest segments entirely
- Last command's output is never truncated (most relevant for AI)
- Single-command overflow: keep tail of output only
- Fallback if no prompt patterns match: simple tail truncation to fit budget
- New `src-tauri/src/terminal/context.rs` module
- ai.rs calls `context::prepare_terminal_context(raw_text, model_context_window)` -> gets back clean, budgeted text
- Current 25-line hard cap REPLACED ENTIRELY by token budget
- Purely backend -- zero frontend changes needed

### Claude's Discretion
- Exact regex patterns for ANSI escape sequence matching
- Which shell prompt patterns to support beyond existing ones
- Exact percentage within the 10-15% range (e.g., 12% as default)
- Internal data structures for command segments
- Test strategy and edge case handling

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SCTX-01 | ANSI escape sequence stripping from terminal output before sending to AI | Regex patterns for CSI, OSC, ESC sequences and control chars documented below |
| SCTX-02 | Token budget allocation -- terminal context uses ~10-15% of model's context window | context_window field on ModelWithMeta + chars/4 heuristic + 12% default budget |
| SCTX-03 | Command-output pairing -- truncation removes oldest complete command+output segments | Prompt pattern regex for segmentation, CommandSegment struct, oldest-first drop |
| SCTX-04 | Cross-platform module -- smart truncation applies to macOS, Windows, and Linux equally | Pure text processing in context.rs, no platform-specific code needed |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| regex | (already in Cargo.toml) | ANSI pattern matching, prompt detection | Already used by filter.rs, no new dependency |
| once_cell | (already in Cargo.toml) | Lazy-compiled regex patterns | Already used by filter.rs for SENSITIVE_PATTERNS |

### Supporting
No new dependencies required. This phase uses only existing crate dependencies.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Regex ANSI strip | `strip-ansi-escapes` crate | External dep for a simple problem; regex gives full control |
| Regex ANSI strip | VTE state machine parser | Overkill for stripping; we don't need terminal emulation |
| chars/4 heuristic | `tiktoken-rs` crate | Adds 5MB+ dependency for marginal accuracy gain on a budget estimate |

## Architecture Patterns

### Recommended Project Structure
```
src-tauri/src/terminal/
├── mod.rs              # Add: pub mod context;
├── context.rs          # NEW: prepare_terminal_context() pipeline
├── filter.rs           # Existing: filter_sensitive() called AFTER context prep
├── detect.rs           # Existing: unchanged
├── detect_linux.rs     # Existing: unchanged
├── detect_windows.rs   # Existing: unchanged
├── process.rs          # Existing: unchanged
├── ax_reader.rs        # Existing: unchanged
├── browser.rs          # Existing: unchanged
└── uia_reader.rs       # Existing: unchanged
```

### Pattern 1: Pure Processing Pipeline
**What:** A chain of pure functions transforming raw terminal text to AI-ready context
**When to use:** When data flows through sequential transformations with no side effects
**Example:**
```rust
// context.rs - the full pipeline
pub fn prepare_terminal_context(raw_text: &str, context_window: u32) -> String {
    let stripped = strip_ansi_and_control(raw_text);
    let budget_chars = compute_budget_chars(context_window);
    let truncated = smart_truncate(&stripped, budget_chars);
    // NOTE: filter_sensitive() is called AFTER this function returns, in ai.rs
    truncated
}
```

### Pattern 2: Command Segment Struct
**What:** Internal representation for parsed command+output pairs
**When to use:** For the segmentation step between stripping and truncation
**Example:**
```rust
struct CommandSegment {
    /// The prompt line + command text
    prompt_and_command: String,
    /// Output lines following the command (may be empty)
    output: String,
}
```

### Pattern 3: Lazy Static Regex (existing project pattern)
**What:** Compile regex patterns once, reuse across calls via `once_cell::sync::Lazy`
**When to use:** For ANSI strip patterns and prompt detection patterns
**Example:**
```rust
use once_cell::sync::Lazy;
use regex::Regex;

static ANSI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\x1b\[[0-9;]*[A-Za-z]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[()][AB012]|\x1b[=>NH]").unwrap()
});
```

### Anti-Patterns to Avoid
- **Stripping at capture time:** Raw text is needed for other purposes (e.g., WSL detection, CWD inference). Strip only at AI call time.
- **Per-call regex compilation:** Use `Lazy<Regex>` like filter.rs does, not `Regex::new()` inside the function.
- **Tokenizer dependency:** A tokenizer would add significant binary size and complexity for a budget estimate that's deliberately approximate.
- **Modifying TerminalContext struct:** The processing happens in the prompt building path, not in the detection path. Don't change visible_output to store cleaned text.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ANSI escape matching | Character-by-character state machine | Regex covering CSI, OSC, ESC sequences | Regex is simpler, project already uses regex crate, stripping doesn't need full VTE compliance |
| Token counting | BPE tokenizer implementation | chars / 4 heuristic | Sufficient accuracy for a budget that's intentionally a range (10-15%), avoids tokenizer dep |

**Key insight:** This is a budget estimation problem, not a precise token counting problem. The 10-15% range is itself approximate, so chars/4 is adequate.

## Common Pitfalls

### Pitfall 1: Incomplete ANSI Regex
**What goes wrong:** Naive `\x1b\[[0-9;]*m` only catches SGR (color) sequences, missing cursor movement, OSC (title setting), and other escape types
**Why it happens:** Terminal output contains far more than just color codes
**How to avoid:** Cover all major ANSI escape sequence categories (see Code Examples)
**Warning signs:** Output still contains `\x1b` bytes or garbage characters after stripping

### Pitfall 2: Prompt Pattern False Positives
**What goes wrong:** Output text that looks like a prompt (e.g., "user@host:" in a log message) causes incorrect command boundary detection
**Why it happens:** Prompt patterns are heuristic, not deterministic
**How to avoid:** Require prompt patterns to match at line start; prefer patterns that include the prompt character ($, #, >) at the end; accept that edge cases will fall through to tail truncation
**Warning signs:** Command segments that contain partial outputs from a different command

### Pitfall 3: Budget Calculation Off-by-One
**What goes wrong:** Budget calculated from context_window but not accounting for units (tokens vs chars)
**Why it happens:** context_window is in tokens, budget percentage gives tokens, but string operations work in chars
**How to avoid:** Explicitly convert: `budget_chars = context_window * 4 * percentage` (multiply by 4 to convert tokens to chars, then apply percentage). Or equivalently: `budget_chars = (context_window * percentage) * 4`.
**Warning signs:** Terminal context is 4x too small or 4x too large

### Pitfall 4: Empty Segment Handling
**What goes wrong:** Consecutive prompt lines (no output between commands) create empty segments that waste budget
**Why it happens:** Users press Enter on empty prompts, or commands produce no output
**How to avoid:** Collapse consecutive prompt-only lines into the next segment, or skip empty segments during budget accounting

### Pitfall 5: Windows Line Endings
**What goes wrong:** `\r\n` line endings from Windows terminals cause double-spacing or broken line counting
**Why it happens:** UIA reader on Windows returns CRLF, macOS AX reader returns LF
**How to avoid:** Normalize line endings early in the pipeline (`text.replace("\r\n", "\n")`)

## Code Examples

### ANSI Escape Sequence Regex (Comprehensive)
```rust
use once_cell::sync::Lazy;
use regex::Regex;

/// Matches all common ANSI escape sequences:
/// 1. CSI sequences: \x1b[ ... (letter) -- covers colors, cursor movement, erase
/// 2. OSC sequences: \x1b] ... (BEL or ST) -- covers title setting, hyperlinks
/// 3. Character set selection: \x1b( or \x1b) followed by charset designator
/// 4. Simple ESC sequences: \x1b followed by single char (=, >, N, H, etc.)
/// 5. CSI with ? prefix: \x1b[? ... (letter) -- covers DEC private modes
static ANSI_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"\x1b\[\??[0-9;]*[A-Za-z]",           // CSI sequences (including DEC private)
        r"|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)",  // OSC sequences (BEL or ST terminated)
        r"|\x1b[()][AB012]",                     // Character set selection
        r"|\x1b[>=NHMDEc7-~]",                   // Simple ESC sequences
    )).unwrap()
});

/// Matches non-printable control characters EXCEPT \n (\x0A) and \t (\x09).
/// Covers \x00-\x08, \x0B-\x0C, \x0E-\x1F, and \x7F (DEL).
static CONTROL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\x00-\x08\x0B\x0C\x0E-\x1F\x7F]").unwrap()
});

fn strip_ansi_and_control(text: &str) -> String {
    let no_ansi = ANSI_RE.replace_all(text, "");
    let no_control = CONTROL_RE.replace_all(&no_ansi, "");
    no_control.into_owned()
}
```

### Shell Prompt Pattern for Segmentation
```rust
/// Prompt patterns for command boundary detection.
/// Reuses and extends patterns from infer_shell_from_text().
static PROMPT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(concat!(
        r"^(?:",
        r"\w+@[\w.-]+:[^\s]*[$#]\s?",          // user@host:/path$ (bash/zsh)
        r"|PS [A-Z]:\\[^>]*>\s?",                // PS C:\path> (PowerShell)
        r"|PS>\s?",                               // PS> (PowerShell minimal)
        r"|[A-Z]:\\[^>]*>\s?",                    // C:\path> (CMD)
        r"|[%$#>]\s",                             // Generic prompt chars at start
        r"|>>>?\s",                               // Python REPL (>>> or >>)
        r"|\w+@[\w.-]+\s+[^\s]+[>$#]\s?",       // fish: user@host /path>
        r")",
    )).unwrap()
});
```

### Token Budget Calculation
```rust
/// Default budget: 12% of context window.
const TERMINAL_BUDGET_FRACTION: f64 = 0.12;
/// Default context window for unknown models (128K tokens).
const DEFAULT_CONTEXT_WINDOW: u32 = 128_000;
/// Approximate chars per token for budget estimation.
const CHARS_PER_TOKEN: u32 = 4;

fn compute_budget_chars(context_window: u32) -> usize {
    let cw = if context_window == 0 { DEFAULT_CONTEXT_WINDOW } else { context_window };
    let budget_tokens = (cw as f64 * TERMINAL_BUDGET_FRACTION) as usize;
    budget_tokens * CHARS_PER_TOKEN as usize
}
```

### Command Segmentation
```rust
fn segment_commands(text: &str) -> Vec<CommandSegment> {
    let lines: Vec<&str> = text.lines().collect();
    let mut segments: Vec<CommandSegment> = Vec::new();
    let mut current_prompt = String::new();
    let mut current_output = String::new();

    for line in &lines {
        if PROMPT_RE.is_match(line) {
            // Save previous segment if non-empty
            if !current_prompt.is_empty() {
                segments.push(CommandSegment {
                    prompt_and_command: current_prompt,
                    output: current_output,
                });
            }
            current_prompt = line.to_string();
            current_output = String::new();
        } else {
            if current_prompt.is_empty() {
                // Lines before first prompt -- treat as output of implicit segment
                current_output.push_str(line);
                current_output.push('\n');
            } else {
                current_output.push_str(line);
                current_output.push('\n');
            }
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
```

### Smart Truncation
```rust
fn smart_truncate(text: &str, budget_chars: usize) -> String {
    // Fast path: fits in budget
    if text.len() <= budget_chars {
        return text.to_string();
    }

    let segments = segment_commands(text);

    // Fallback: no segments found (no prompt patterns matched)
    if segments.is_empty() || (segments.len() == 1 && segments[0].prompt_and_command.is_empty()) {
        // Simple tail truncation
        let start = text.len().saturating_sub(budget_chars);
        // Find next newline to avoid cutting mid-line
        let start = text[start..].find('\n').map(|i| start + i + 1).unwrap_or(start);
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
        let start = output[start..].find('\n').map(|i| start + i + 1).unwrap_or(start);
        return format!("{}\n{}", seg.prompt_and_command, &output[start..]);
    }

    // Reassemble selected segments
    selected.iter().map(|s| {
        if s.output.is_empty() {
            s.prompt_and_command.clone()
        } else {
            format!("{}\n{}", s.prompt_and_command, s.output.trim_end())
        }
    }).collect::<Vec<_>>().join("\n")
}
```

### Context Window Lookup (models.rs extension)
```rust
/// Look up context window size for a model by ID.
/// Returns DEFAULT_CONTEXT_WINDOW (128K) for unknown models.
pub(crate) fn context_window_for_model(model_id: &str) -> u32 {
    // Known context windows by model family prefix
    // Values in tokens
    static CONTEXT_WINDOWS: Lazy<Vec<(&str, u32)>> = Lazy::new(|| vec![
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
        // Gemini 2.5 -- 1M
        ("gemini-2.5", 1_048_576),
        ("gemini-3", 1_048_576),
        ("gemini-2.0", 1_048_576),
        // xAI Grok -- 131K
        ("grok-", 131_072),
    ]);

    for (prefix, window) in CONTEXT_WINDOWS.iter() {
        if model_id.starts_with(prefix) {
            return *window;
        }
    }
    DEFAULT_CONTEXT_WINDOW // 128K default
}
```

### Integration in ai.rs
```rust
// In build_user_message(), replace lines 135-143:
if let Some(output) = &terminal.visible_output {
    // OLD: let lines: Vec<&str> = output.lines().collect();
    //      let start = lines.len().saturating_sub(25);
    //      ...

    // NEW: Smart context preparation
    use crate::terminal::context;
    let context_window = context::context_window_for_model(&model);
    let prepared = context::prepare_terminal_context(output, context_window);
    // filter_sensitive is called after prepare
    let filtered = crate::terminal::filter::filter_sensitive(&prepared);
    let line_count = filtered.lines().count();
    parts.push(format!(
        "Terminal output ({} lines):\n{}",
        line_count,
        filtered
    ));
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Hard 25-line tail truncation | Token-budget-aware smart truncation | This phase | AI gets meaningful context instead of arbitrary line count |
| Raw ANSI codes in AI prompt | ANSI-stripped clean text | This phase | No wasted tokens on invisible formatting |
| No model awareness for context | Model-specific context window budget | This phase | Right-sized context for each model |

## Open Questions

1. **Exact context window values for newer models**
   - What we know: Major model families have documented context windows
   - What's unclear: OpenRouter custom models have varying context windows
   - Recommendation: Use prefix-based lookup for known models, 128K default for unknown. OpenRouter API returns `context_length` per model which could be cached from `fetch_models()` for higher accuracy (future enhancement).

2. **Whether `model` parameter reaches `build_user_message`**
   - What we know: `stream_ai_response` receives `model: String` and calls `build_user_message` which currently doesn't take model
   - What's unclear: N/A -- just needs signature change
   - Recommendation: Pass `model` to `build_user_message` or compute budget before calling it. The model is available in the same function scope.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[cfg(test)] mod tests` |
| Config file | None (standard cargo test) |
| Quick run command | `cargo test --lib -p cmd-k -- terminal::context` |
| Full suite command | `cargo test --lib -p cmd-k` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCTX-01 | ANSI stripping removes CSI, OSC, control chars | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_strip` | Wave 0 |
| SCTX-02 | Budget calculation: 12% of context window in chars | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_budget` | Wave 0 |
| SCTX-03 | Command segmentation and oldest-first truncation | unit | `cargo test --lib -p cmd-k -- terminal::context::tests::test_segment` | Wave 0 |
| SCTX-04 | Cross-platform: no cfg(target_os) in context.rs | manual-only | Code review: grep for cfg(target_os) in context.rs | N/A |

### Sampling Rate
- **Per task commit:** `cargo test --lib -p cmd-k -- terminal::context`
- **Per wave merge:** `cargo test --lib -p cmd-k`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src-tauri/src/terminal/context.rs` -- new file with inline `#[cfg(test)] mod tests`
- No framework install needed -- Rust test framework is built in

## Sources

### Primary (HIGH confidence)
- Codebase inspection: `commands/ai.rs` lines 135-143 -- current 25-line truncation
- Codebase inspection: `commands/models.rs` -- ModelWithMeta struct, curated model lists
- Codebase inspection: `terminal/mod.rs` -- `infer_shell_from_text()` prompt patterns
- Codebase inspection: `terminal/filter.rs` -- `filter_sensitive()` with Lazy<Regex> pattern

### Secondary (MEDIUM confidence)
- ANSI escape sequence specification: ECMA-48 standard defines CSI, OSC, and other escape categories
- Token estimation: ~4 chars/token is a widely-used heuristic for English text with GPT-class tokenizers

### Tertiary (LOW confidence)
- Exact context window values for newest model releases (GPT-5.x, Gemini 3.x) -- verified against provider documentation where available but may change

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - no new dependencies, all existing crate deps
- Architecture: HIGH - follows established project patterns (Lazy regex, terminal/ module, pure functions)
- Pitfalls: HIGH - derived from codebase analysis of existing code patterns and cross-platform considerations

**Research date:** 2026-03-14
**Valid until:** 2026-04-14 (stable domain, no external dependency changes)
