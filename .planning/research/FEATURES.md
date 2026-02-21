# Feature Landscape

**Domain:** macOS System-Wide Overlay for AI Terminal Command Generation
**Researched:** 2026-02-21
**Confidence:** MEDIUM-HIGH

## Executive Summary

This research analyzes the feature landscape for macOS overlay/launcher apps and AI terminal command generators, synthesizing patterns from Raycast, Alfred, Warp AI, GitHub Copilot CLI, Amazon Q CLI, Superwhisper, and Fig. The domain splits into two categories: launcher/overlay UX patterns and AI command generation capabilities. Table stakes are surprisingly minimal for MVP, while differentiation comes from terminal context awareness and safety mechanisms.

---

## Table Stakes

Features users expect from any overlay/launcher app or AI command generator. Missing any of these makes the product feel incomplete or broken.

### Core Overlay/Launcher Features

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Global hotkey activation** | Universal pattern (Cmd+K, Option+Space). Users expect instant access from anywhere. | Low | Standard macOS pattern. Accessibility permissions required. Most use Cmd/Option + single key. |
| **Keyboard-first navigation** | Users invoke overlay to avoid mouse. All interactions must be keyboard-driven. | Low | Arrow keys for navigation, Enter to confirm, Esc to dismiss. No mouse required. |
| **Fuzzy search/filtering** | Users expect to type partial matches and see relevant results instantly. | Medium | Real-time filtering as user types. Must handle typos, partial matches, abbreviations. |
| **Clean, minimal UI** | Overlay apps emphasize speed and focus. Heavy UI slows users down. | Low | Centered overlay window, blur/transparency effects, minimal chrome. macOS native styling. |
| **Instant dismiss** | Users expect overlay to vanish instantly (same hotkey, Esc, or click outside). | Low | Hide (don't close) window. Restore previous app focus. Preserve input state for next invocation. |
| **Menu bar presence** | macOS convention for background apps. Users need visual indicator app is running. | Low | Status bar icon with minimal menu (Preferences, Quit). Follows macOS Human Interface Guidelines. |

### AI Command Generation Features

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Natural language to command** | Core value proposition. Users describe intent, AI generates shell command. | High | Requires LLM integration. Must handle ambiguous prompts, multiple interpretations. |
| **Command preview before execution** | Safety requirement. Users must see what will run before it runs. | Low | Display generated command with syntax highlighting. Never auto-execute. |
| **Copy to clipboard** | Minimum viable output. Users need command in clipboard to paste manually. | Low | pbcopy integration on macOS. Universal fallback if paste fails. |
| **Command explanation** | Users need to understand what command does, especially for complex/dangerous commands. | Medium | LLM-generated explanation alongside command. Highlight destructive operations. |
| **Error handling and retry** | LLM calls fail. Network issues happen. Users expect graceful degradation. | Medium | Show error messages. Allow retry. Cache last prompt. Don't lose user input on failure. |
| **Basic terminal context awareness** | At minimum: current working directory. Impacts command relevance significantly. | Medium | Read active terminal's cwd. Required for file operations, relative paths. See Pitfalls section. |

### Accessibility & Permissions (macOS Specific)

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Accessibility permission request** | Required for global hotkeys and overlay rendering. macOS enforces this. | Low | System prompt on first launch. Graceful handling if denied. Link to System Settings. |
| **Screen Recording permission** (conditional) | Required if app needs to detect active terminal window or read screen content. | Low | Only if detecting terminal context without shell plugins. May not be needed for MVP. |
| **Permissions onboarding** | Users unfamiliar with macOS security model need guidance. | Low | First-run wizard explaining why permissions needed. Screenshots showing System Settings path. |

---

## Differentiators

Features that set products apart in this crowded space. Not expected, but highly valued when present.

### Terminal Context Intelligence (HIGH VALUE)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Shell history awareness** | Generate commands based on user's actual command patterns. "Do what I did last time but for different file." | High | Access .bash_history or .zsh_history. Privacy concerns. Alternative: local-only processing. Tools like hishtory provide this. |
| **Active terminal detection** | Know which terminal window is active, get its specific context (cwd, env vars, recent output). | High | No standard API. Requires Accessibility + Screen Recording permissions. AppleScript to iTerm2/Terminal.app. See PITFALLS.md for technical challenges. |
| **Git context awareness** | Commands contextualized to current branch, repo state, uncommitted changes. | Medium | Parse .git directory. Run git status. Suggest branch-aware commands. Warp AI does this well. |
| **Project-type detection** | Recognize Node.js project → suggest npm/npx. Python → suggest pip/venv. Improves command relevance. | Medium | Check for package.json, requirements.txt, Cargo.toml, etc. Inject into LLM context. |
| **Environment variable awareness** | Read PATH, SHELL, other env vars to generate compatible commands. | Medium | Read from active shell process or user's shell config files. Privacy: avoid reading sensitive env vars. |

### Safety & Approval Mechanisms (CRITICAL FOR TRUST)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Destructive command detection** | Flag rm -rf, DROP TABLE, force-push, etc. Require explicit confirmation. | Medium | Pattern matching (DCG-style whitelist/blocklist). AI-based danger classification. Prevents catastrophic mistakes. |
| **Command approval workflow** | Never auto-execute. Show command, require explicit user confirmation. | Low | Preview → Approve → Execute flow. Keyboard shortcut for quick approval. Default is "show, don't run." |
| **Command modification before execution** | Allow user to edit AI-generated command before running. | Low | Editable text field in preview. Copy edited version to clipboard or execute directly. |
| **Execution history and rollback** | Log what commands were run, allow undo for file operations. | High | Track executed commands. For destructive ops, snapshot state before execution. Complex rollback logic. |
| **Safe mode / dry run** | Execute commands with --dry-run or similar flags when available. | Medium | Tool-specific. Not all commands support dry run. Helpful for destructive ops like rsync, docker, kubectl. |

### Multi-Model Support (EMERGING DIFFERENTIATOR)

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Model selection** | Let users choose LLM (GPT-4, Claude, Gemini, Grok). Different models excel at different tasks. | Medium | Warp Agents 3.0, GitHub Copilot CLI both offer this. UI for model picker. API key management per provider. |
| **Automatic model routing** | Smart routing: simple commands → fast/cheap model, complex → powerful model. | High | Heuristic or AI-based classification. Cost optimization + quality. Requires multi-provider integration. |
| **Local model support** | Privacy-conscious users want on-device processing (Llama, Mistral, etc). | High | Integration with Ollama or MLX. Slower, lower quality, but private. Niche but growing demand. |

### Workflow & Productivity Features

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Command templates / snippets** | Save frequently used command patterns. "Deploy to staging" → expands to full command. | Medium | Alfred/Raycast pattern. Snippet storage, variable interpolation. Complements AI generation for known tasks. |
| **Multi-step command generation** | Break complex tasks into sequence of commands. "Set up Python project" → multiple commands. | High | LLM must plan multi-step workflows. Present as numbered list. Execute one-by-one or all at once. |
| **Command chaining suggestions** | After user runs command, suggest logical next steps. "You just git add, want to commit?" | High | Stateful. Requires tracking what user executed. Context-aware suggestions. Warp AI-style workflow guidance. |
| **Interactive command refinement** | Conversational: "That's close but use flags -la instead of -l" → AI adjusts command. | High | Chat-style interface. Maintain conversation context. Iterate on command until user satisfied. |
| **Recent commands recall** | Quick access to recently generated commands. Re-run or modify previous suggestions. | Low | Local storage of last N commands. Searchable history within app. Ctrl+R-style UX. |

### Cross-App Integration

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Paste into active terminal** | Automatically paste generated command into frontmost terminal window. | High | AppleScript for Terminal.app/iTerm2. Requires Accessibility permissions. Brittle (see PITFALLS.md). |
| **Execute in background** | Run command in hidden terminal, show output in overlay. | Medium | Spawn subprocess, capture stdout/stderr, display in app. For read-only commands (git status, ls, etc). |
| **Terminal emulator agnostic** | Work with Terminal.app, iTerm2, Warp, Alacritty, Kitty, etc. | High | Each terminal has different automation APIs. AppleScript, Accessibility API, or universal clipboard approach. |
| **Editor integration** | Send commands to VSCode terminal, Cursor terminal, etc. | Medium | Similar to terminal integration. IDE-specific APIs. Extension-based approach may be easier. |

### AI/LLM Optimization

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Streaming responses** | Show command being generated in real-time, not after full completion. Better perceived performance. | Medium | LLM streaming API support. Update UI progressively. Handle partial/invalid commands during stream. |
| **Caching and prediction** | Cache common prompts locally. Predict user intent, pre-generate commands. | High | Local cache of prompt → command mappings. Predictive pre-fetching based on context. Privacy considerations. |
| **Offline mode** | Basic command generation without network (local model or cached responses). | High | Local LLM or extensive command database. Degraded quality but functional offline. |
| **Custom system prompts** | Power users customize AI behavior. "Always use verbose flags" or "Prefer GNU tools over BSD". | Medium | User-defined prompt templates. Inject into LLM system message. Profile/preset management. |

---

## Anti-Features

Features to explicitly NOT build. Common mistakes in this domain or features that seem good but cause problems.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Auto-execution of commands** | Catastrophically dangerous. LLMs make mistakes. Users paste without reading. One bad rm -rf ruins trust forever. | Always preview. Require explicit approval (click, Enter key, etc). Make safety the default, not opt-in. |
| **Shell plugin requirement** | Warp AI, GitHub Copilot CLI require shell integration. Friction in onboarding. Breaks in SSH, containers, restricted shells. | Use clipboard-based approach. Detect context via macOS APIs (Accessibility, AppleScript). Trade some features for universal compatibility. |
| **Cloud-synced command history** | Privacy nightmare. Commands contain sensitive data (passwords, API keys, file paths, server IPs). | Keep all data local-only. If sync needed, encrypt end-to-end and make opt-in with clear warnings. |
| **Complex multi-pane UI** | Overlay apps succeed via simplicity. Adding file browsers, terminal emulators, etc. bloats UX and scope. | Single-purpose focus: generate command, copy/paste. Don't try to be a full IDE or terminal replacement. |
| **Custom terminal emulator** | Massive scope. Terminal emulators are complex (VT100 emulation, performance, etc). Warp tried this, but niche. | Integrate with existing terminals. Clipboard is the universal interface. |
| **Scraping terminal output** | Fragile. OCR or screen scraping breaks with themes, fonts, terminal updates. Privacy violation. | Ask user to paste output if needed, or use shell integration (but see above why that's an anti-feature for v1). |
| **Mandatory account creation** | Friction. Users want to try immediately. Account walls reduce adoption, especially for dev tools. | Make account optional. Local-only mode for free tier. Accounts only for cloud sync or premium features. |
| **Free tier with usage limits** | Annoying for dev tools. Developers hit limits fast during active work. Leads to tool abandonment. | Generous free tier or paid-only. If limits exist, make them high (100+ commands/day). Daily resets, not monthly. |
| **Built-in terminal emulator** | Scope explosion. Users already have terminal preferences (iTerm2, Warp, etc). Won't switch. | Focus on overlay + AI. Integrate with existing workflows rather than replacing them. |
| **Command execution logging to cloud** | Privacy violation. Commands contain secrets, internal tool names, infrastructure details. | Local-only logging. If analytics needed, anonymize heavily (command patterns, not actual commands). |

---

## Feature Dependencies

Understanding which features build on others, informing phase/sprint structure.

```
Core Foundation (Phase 1):
├─ Global hotkey activation
├─ Minimal overlay UI
├─ Keyboard navigation
└─ macOS permissions handling

Basic AI Generation (Phase 1):
├─ Natural language → command (LLM integration)
├─ Command preview
├─ Copy to clipboard
└─ Basic error handling

Context Awareness (Phase 2):
├─ Requires: Active terminal detection
├─ Enables: Current directory awareness
├─ Enables: Shell history integration
└─ Enables: Git context

Safety Layer (Phase 2):
├─ Requires: Command preview
├─ Enables: Destructive command detection
├─ Enables: Approval workflow
└─ Enables: Command modification

Advanced Features (Phase 3+):
├─ Multi-model support
│   └─ Requires: Model selection UI, API key management
├─ Auto-paste to terminal
│   └─ Requires: Active terminal detection, Accessibility permissions
├─ Streaming responses
│   └─ Requires: LLM streaming API integration
└─ Command chaining
    └─ Requires: Execution tracking, context maintenance
```

**Critical Path:**
1. Overlay + hotkey + basic UI → Can't use app without this
2. LLM integration + command preview → Core value proposition
3. Clipboard copy → Minimum viable output
4. Terminal context (cwd) → Dramatically improves command quality
5. Safety features → Builds trust, prevents disasters

**Can Defer:**
- Multi-model support (single provider for MVP)
- Auto-paste (clipboard is sufficient, auto-paste is fragile)
- Streaming (nice-to-have UX, not critical)
- Advanced context (git, env vars, history - add incrementally)

---

## MVP Recommendation

For MVP (v0.1), prioritize:

### Must Have (Core Value)
1. **Global hotkey overlay** (Cmd+K or Option+Space)
2. **Natural language → command generation** (single AI provider: xAI Grok)
3. **Command preview with syntax highlighting**
4. **Copy to clipboard** (primary output method)
5. **Basic terminal context: current working directory**
6. **Command explanation** (what does this command do?)
7. **Error handling** (API failures, network issues)

### Should Have (Safety & Polish)
8. **Destructive command flagging** (warn on rm, git push --force, etc.)
9. **Command modification** (edit before copying)
10. **Accessibility permissions onboarding**
11. **Menu bar app with Preferences/Quit**

### Defer to Post-MVP
- **Multi-model support**: Single provider (Grok) sufficient for v1. Add GPT/Claude later based on demand.
- **Shell history awareness**: Complex, privacy-sensitive. Not critical for initial validation.
- **Auto-paste to terminal**: Fragile, terminal-specific. Clipboard approach works universally.
- **Streaming responses**: Nice UX improvement, not core to value prop.
- **Git context awareness**: Valuable but can add after validating core concept.
- **Command templates/snippets**: Feature creep. Focus on AI generation first.
- **Multi-step workflows**: Complex. Single commands are sufficient for MVP.
- **Execute in background**: Scope expansion. Clipboard + manual execution is fine.

**MVP Rationale:**
The MVP focuses on the tightest possible loop: invoke overlay → describe command in natural language → AI generates command with current directory context → preview and copy to clipboard → paste into existing terminal. This validates the core hypothesis (is AI command generation valuable?) without building a terminal emulator, shell plugin, or complex integrations.

Current directory context is the only terminal awareness included because it's essential for command quality (file paths, relative operations) and achievable without shell plugins (see ARCHITECTURE.md for approaches).

Safety features (destructive command flagging, preview requirement, modification) are included in MVP because one catastrophic mistake destroys user trust permanently. Better to launch with conservative safety than move fast and break production.

---

## Competitive Positioning

### vs. Raycast / Alfred
**Their strength:** General-purpose launcher, 1500+ extensions, workflow automation, file search, calculator, etc.
**Our differentiation:** Terminal-specific. Deep command generation context. Safety for destructive commands. No feature bloat.
**Why users switch to us:** Raycast/Alfred AI features are generic. We're purpose-built for terminal workflows.

### vs. Warp AI / GitHub Copilot CLI
**Their strength:** Integrated into terminal. Full context (history, output, env). Agent-based workflows.
**Our differentiation:** No shell plugin required. Works with any terminal. Overlay invoked from anywhere, not just inside terminal.
**Why users switch to us:** Copilot CLI requires GitHub subscription. Warp requires using Warp terminal. We're universal, zero lock-in.

### vs. Amazon Q CLI
**Their strength:** AWS-specific. Deep integration with AWS services and CLI.
**Our differentiation:** General-purpose commands, not AWS-only. Simpler onboarding (no AWS account needed).
**Why users choose us:** Not everyone uses AWS. We handle git, npm, docker, filesystem ops, etc. Broader scope.

### vs. Fig (deprecated, now Amazon Q)
**Historical note:** Fig pioneered terminal autocomplete overlays. Acquired by AWS, sunset into Amazon Q.
**Lesson learned:** Terminal autocomplete alone wasn't defensible. Need stronger differentiation (AI generation, not just autocomplete).

### vs. Superwhisper
**Their strength:** Voice-to-text dictation, works anywhere on macOS.
**Our differentiation:** Specialized for terminal commands. AI understands command syntax, flags, patterns. Not just transcription.
**Different use case:** They target writing, notes, messaging. We target developers and terminal workflows.

---

## Feature Complexity Assessment

| Complexity | Features | Estimated Effort |
|------------|----------|------------------|
| **Low** | Global hotkey, clipboard copy, menu bar app, basic UI, permissions onboarding | 1-2 days each |
| **Medium** | Fuzzy search, command explanation, error handling, destructive command detection, syntax highlighting, model selection UI | 3-5 days each |
| **High** | LLM integration, terminal context detection (without shell plugin), auto-paste to terminal, shell history awareness, multi-step workflows, streaming responses | 1-2 weeks each |
| **Very High** | Agentic multi-step execution, local LLM support, terminal output parsing, execution rollback | 2-4 weeks each |

**MVP Complexity Budget:**
- Core overlay + hotkey: ~3 days
- LLM integration (Grok API): ~1 week
- Command preview UI: ~3 days
- Terminal context (cwd detection): ~1 week
- Safety features: ~3 days
- Polish and onboarding: ~3 days

**Total MVP estimate: 3-4 weeks** for single developer, assuming no major technical blockers in terminal context detection.

---

## Sources

### macOS Overlay/Launcher Apps
- [Raycast - Your shortcut to everything](https://www.raycast.com/)
- [Raycast - macOS Changelog](https://www.raycast.com/changelog)
- [Alfred - Productivity App for macOS](https://www.alfredapp.com/)
- [Alfred Powerpack - Take Control of Your Mac and macOS](https://www.alfredapp.com/powerpack/)
- [Superwhisper](https://superwhisper.com/)
- [Best Mac Keyboard Shortcut Apps (2026 Edition)](https://textexpander.com/blog/mac-keyboard-shortcut-app)

### AI Terminal Command Generation
- [Warp: The Agentic Development Environment](https://www.warp.dev/)
- [Warp: AI: Natural-Language Coding Agents](https://www.warp.dev/warp-ai)
- [Warp: All Features](https://www.warp.dev/all-features)
- [GitHub Copilot CLI](https://github.com/features/copilot/cli)
- [Using GitHub Copilot CLI - GitHub Docs](https://docs.github.com/en/copilot/how-tos/use-copilot-agents/use-copilot-cli)
- [Amazon Q Developer for command line](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html)
- [Amazon Q CLI Command Reference](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-reference.html)

### Terminal Autocomplete & Context
- [Fig terminal autocomplete](https://github.com/withfig/autocomplete)
- [hishtory - Your shell history: synced, queryable, and in context](https://terminaltrove.com/hishtory/)
- [Giving coding agents situational awareness](https://dave.engineer/blog/2026/01/agent-situations/)
- [zsh-autosuggestions - Fish-like autosuggestions for zsh](https://github.com/zsh-users/zsh-autosuggestions)

### Safety & Best Practices
- [DCG: Destructive Command Guard — Safety Philosophy and Design Principles](https://reading.torqsoftware.com/notes/software/ai-ml/safety/2026-01-26-dcg-destructive-command-guard-safety-philosophy-design-principles/)
- [AI command approval safety confirmation patterns 2026](https://www.gravitee.io/blog/state-of-ai-agent-security-2026-report-when-adoption-outpaces-control)
- [Cline CLI 2.0 (2026): Complete Guide, How It Works & Best Practices](https://iadirecto.com/en/cline-cli-2-0-2026-complete-guide-how-it-works-best-practices-for-ai-powered-terminal-automation/)

### macOS Accessibility & Permissions
- [Allow accessibility apps to access your Mac - Apple Support](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)
- [How to Keep Any Window Always on Top on macOS (2026 Guide)](https://www.floatytool.com/posts/how-to-keep-any-window-always-on-top-macos/)
- [macOS Accessibility Permission](https://jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html)

### macOS Clipboard & Terminal Integration
- [Copy text into a Terminal window on Mac - Apple Support](https://support.apple.com/guide/terminal/copy-text-into-a-terminal-window-trml1019/mac)
- [Terminal and the Clipboard – Scripting OS X](https://scriptingosx.com/2017/03/terminal-and-the-clipboard/)

---

## Research Confidence

| Area | Confidence | Notes |
|------|------------|-------|
| **Overlay/Launcher UX Patterns** | HIGH | Well-established patterns from Raycast, Alfred, etc. Clear industry standards. |
| **AI Command Generation Features** | MEDIUM-HIGH | Rapidly evolving space (2026). Warp, Copilot CLI set patterns, but field is young. |
| **Terminal Context Detection** | MEDIUM | Multiple approaches exist, but all have tradeoffs. No perfect solution without shell plugins. |
| **Safety Mechanisms** | HIGH | DCG and industry best practices well-documented. Critical patterns established. |
| **macOS Permissions & APIs** | HIGH | Official Apple documentation, established patterns from existing apps. |
| **Competitive Landscape** | MEDIUM | Fast-moving space. New tools emerge frequently. Positions may shift post-research. |

**Gaps to investigate in later phases:**
- Specific AppleScript APIs for iTerm2 vs Terminal.app (architecture phase)
- Performance benchmarks for different LLM providers (implementation phase)
- User preferences for auto-paste vs clipboard (beta testing phase)
- macOS Sequoia permission changes and monthly re-approval requirements (deployment phase)
