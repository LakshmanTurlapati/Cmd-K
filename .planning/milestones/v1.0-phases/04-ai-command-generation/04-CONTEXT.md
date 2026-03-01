# Phase 4: AI Command Generation - Context

**Gathered:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

User types natural language into the overlay and receives an AI-generated response via xAI (Grok), streamed token-by-token. In terminal mode (shell detected), the response is strictly a terminal command. In assistant mode (no shell), the response is a concise answer. Auto-copies to clipboard on completion.

Requirements: AICG-01, AICG-02. (AICG-03 destructive command warnings are Phase 5. TERM-01 terminal pasting is Phase 6.)

</domain>

<decisions>
## Implementation Decisions

### Prompt and context design
- Send ALL available context to xAI but condensed: CWD, shell type, last N lines of terminal output (not full buffer), running process name, app name, console last line (if browser with DevTools)
- Terminal output truncated to last ~20-30 lines (most recent = most relevant)
- System prompt includes macOS and shell type (e.g., zsh) so AI generates platform-appropriate commands
- Two distinct system prompts:
  - **Terminal mode** (shell detected, including editors with integrated terminals): strict command-only output. AI returns executable command(s) and nothing else. Compound commands (pipes, && chains) allowed.
  - **Assistant mode** (no shell -- browser, Finder, WhatsApp, Notion, etc.): concise conversational responses. 2-3 sentences max. Still useful, not a chatbot.
- Mode selection rule: if `appContext.terminal.shell_type` is present, use terminal mode. Otherwise, assistant mode.
- Console last line included in prompt when browser with DevTools is open (helps debug JS errors)
- Running process name included in prompt (e.g., if node server is running, AI can suggest Ctrl+C first)
- Terminal prompt instructs AI to prefer common POSIX tools (grep, find, sed, awk) over modern alternatives (rg, fd, jq) for maximum compatibility
- System prompt template stored in Tauri plugin-store (JSON). Editable by power users manually (no Settings UI for it).
- Fixed low temperature (0.1-0.3) for deterministic, reliable command generation
- Respect user's model selection from Settings (Phase 2)
- Session memory: up to 7 turns within a single overlay session. History cleared when overlay closes.
- Context captured once on overlay open, reused for all turns in the session (no re-detection on follow-ups)
- API calls routed through Rust backend (Tauri IPC), not frontend fetch. API key stays in backend.

### Streaming result display
- Character-by-character streaming as tokens arrive from xAI SSE response
- Monospace font with syntax highlighting (no heavy code block chrome -- clean, not a full code editor widget)
- Block cursor (full character width) blinks at the end of streaming text while response is generating. Disappears when streaming completes.
- Always monospace, even in assistant mode
- Adaptive formatting: code blocks for commands (terminal mode), plain monospace text for conversational answers (assistant mode). Both streamed the same way.

### Copy and keyboard flow
- Auto-copy to clipboard when streaming completes (no visual feedback on auto-copy)
- Click-to-copy on the output area: clicking the output re-copies to clipboard and shows a small "Copied to clipboard" indicator on the bottom-right of the output area (only on click, not on auto-copy)
- Subtle hover effect on the output area to hint clickability
- Input field transforms into output display on submit. Same area, different content. Minimal, tight.
- First Escape: toggles back to input mode with the previous query text restored (for editing/refining)
- Second Escape: closes the overlay
- Follow-up queries replace the previous response (not chat-style scroll). Only latest response visible.
- Input disabled while streaming. User must wait for completion before submitting follow-up.
- No explicit regenerate button. User retypes or refines as a follow-up.
- Badge (zsh/Chrome/etc.) stays visible below the output area
- Overlay grows vertically to fit multi-line output, capped at ~60% screen height (scroll beyond that)
- Auto-copy happens in both terminal and assistant modes
- Placeholder text: "Ask anything..."

### Error and edge cases
- No API key configured: inline error in the output area ("No API key configured. Open Settings to add one.")
- API errors (rate limit, auth failure, server error): shown in the output area with muted/red styling
- Timeout: spinner shown while waiting, hard timeout (~10 seconds) cancels and shows "Request timed out. Try again."
- Empty input submit: brief shake animation on the input field (no-op, no API call)

### Claude's Discretion
- Exact system prompt wording and template structure
- Syntax highlighting implementation (library choice or CSS-based)
- SSE parsing and streaming architecture details
- Exact hover effect styling
- Timeout duration (somewhere around 10s, exact value flexible)
- How session memory messages are structured in the API call

</decisions>

<specifics>
## Specific Ideas

- Input area transforms into output on submit -- same physical space, just different content. Escape toggles between the two states. This makes the overlay feel like a single-purpose tool, not a chat window.
- The two-mode system (terminal vs assistant) means the overlay is useful from ANY app, not just terminals. WhatsApp, Notion, Figma -- users can ask anything.
- Auto-copy with no feedback keeps the flow frictionless. Click-to-copy with feedback is the "manual" path for when clipboard was overwritten.
- Block cursor during streaming gives it a terminal feel.

</specifics>

<deferred>
## Deferred Ideas

- Direct terminal command injection (auto-paste into active terminal) -- Phase 6, already in roadmap
- Phase 6 should auto-inject alongside clipboard copy, not as an alternative to it

</deferred>

---

*Phase: 04-ai-command-generation*
*Context gathered: 2026-02-22*
