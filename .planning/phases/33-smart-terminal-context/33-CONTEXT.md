# Phase 33: Smart Terminal Context - Context

**Gathered:** 2026-03-14
**Status:** Ready for planning

<domain>
## Phase Boundary

AI receives intelligently truncated terminal context that maximizes useful information within a token budget. ANSI escape sequences stripped, command-output segments identified, oldest segments dropped to fit budget. Cross-platform module — same logic applies to macOS, Windows, and Linux terminal text.

</domain>

<decisions>
## Implementation Decisions

### ANSI stripping
- Regex-based stripping (not state-machine parser, not external crate)
- Strip at AI call time, not at capture time — raw text preserved for other uses
- Strip ALL non-printable control characters (\x00-\x1F except \n and \t, plus \x7F) in addition to ANSI escape sequences
- Processing order: ANSI strip first → sensitive data filter (filter.rs) second — ensures regex patterns match cleanly against unescaped text

### Token budget & model awareness
- Add `context_window` field to existing curated model structs in models.rs (e.g., Claude Sonnet 4.6: 200K, GPT-4o: 128K)
- Default 128K context window for unknown/custom models (OpenRouter user-added)
- Budget of 10-15% covers terminal output ONLY — conversation history has its own cap (turnLimit), budgets are independent
- Token estimation via ~4 chars/token heuristic (no tokenizer dependency)

### Command-output segmentation
- Detect command boundaries via shell prompt pattern matching (user@host:path$, PS C:\>, >>>, etc.)
- Build on existing `infer_shell_from_text()` prompt patterns
- Truncation preserves most recent complete command+output segments, drops oldest segments entirely
- Last command's output is never truncated (most relevant for AI)
- Single-command overflow (output exceeds entire budget): keep tail of output only
- Fallback if no prompt patterns match: simple tail truncation to fit budget — degrades gracefully

### Integration architecture
- New `src-tauri/src/terminal/context.rs` module — clean separation from ai.rs
- ai.rs calls `context::prepare_terminal_context(raw_text, model_context_window)` → gets back clean, budgeted text
- Current 25-line hard cap on terminal output REPLACED ENTIRELY by token budget — the whole point of this phase
- Model name already passed to `stream_ai_response` — look up `context_window` from model metadata, no new plumbing
- Purely backend — zero frontend changes needed

### Claude's Discretion
- Exact regex patterns for ANSI escape sequence matching
- Which shell prompt patterns to support beyond existing ones
- Exact percentage within the 10-15% range (e.g., 12% as default)
- Internal data structures for command segments
- Test strategy and edge case handling

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `filter.rs`: `filter_sensitive()` — sensitive data redaction, will run AFTER new ANSI stripping step
- `infer_shell_from_text()` in `terminal/mod.rs`: Already has prompt pattern regexes for bash/zsh/fish/PowerShell — extend for command boundary detection
- `models.rs`: Curated model lists with pricing — extend with `context_window` field
- `build_user_message()` in `ai.rs` (lines 93-176): Where terminal text gets assembled into prompts — integration point for smart context

### Established Patterns
- `#[cfg(target_os)]` at function level for platform-specific code
- `terminal/` module houses all terminal-related processing (detection, reading, filtering)
- `ai.rs` builds prompts and dispatches to provider adapters — stays thin
- Model metadata is static/curated in `models.rs` with per-provider lists

### Integration Points
- `build_user_message()` in ai.rs: Currently truncates visible_output to last 25 lines (line 136-142) — replace with `context::prepare_terminal_context()` call
- `stream_ai_response()` in ai.rs: Already receives model name — pass to context preparation for budget lookup
- `terminal/mod.rs`: Add `pub mod context;` to expose new module
- `filter_sensitive()`: Called after ANSI stripping in the new pipeline

</code_context>

<specifics>
## Specific Ideas

- The processing pipeline should be: raw text → ANSI strip → control char strip → command segmentation → budget truncation → sensitive data filter → AI prompt
- Smart context should feel invisible — users don't need to know about it, AI just gets better context automatically

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 33-smart-terminal-context*
*Context gathered: 2026-03-14*
