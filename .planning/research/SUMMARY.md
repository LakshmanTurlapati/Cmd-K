# Project Research Summary

**Project:** CMD + K (macOS System-Wide AI Command Generator)
**Domain:** macOS overlay application with terminal integration and AI command generation
**Researched:** 2026-02-21
**Confidence:** HIGH

## Executive Summary

CMD + K is a macOS overlay application that uses AI to generate terminal commands from natural language. Research shows this domain combines two distinct patterns: overlay/launcher UX (like Raycast/Alfred) and AI terminal assistance (like Warp AI/GitHub Copilot CLI). The recommended approach is a Tauri v2 application with React frontend, leveraging NSPanel for proper overlay behavior, Accessibility API for terminal context reading, and xAI Grok for command generation.

The core technical stack is well-established (Tauri v2.10.2, React 18, TypeScript, Vite) with high confidence in implementation patterns. The main architectural challenge is terminal context reading without shell plugins—achievable via macOS Accessibility API combined with process inspection (libproc-rs, active-win-pos-rs). Critical risks center on macOS security permissions, AppleScript command injection, and sandboxing limitations. All risks have documented mitigation strategies.

The MVP should focus on the tightest loop: global hotkey → overlay display → AI command generation with current directory context → clipboard copy for manual pasting. This validates core value without building complex integrations. Auto-paste to terminal and advanced context awareness (git, history) can be deferred to post-MVP phases.

## Key Findings

### Recommended Stack

The standard 2025/2026 stack for Tauri-based macOS overlay apps combines Tauri v2 with React 18 + TypeScript + Vite for the frontend, leveraging Tauri's official plugin ecosystem for system integration. Tauri v2 offers 10x smaller bundle size than Electron (2-2.5MB vs 150MB+), native macOS APIs, and better security model.

**Core technologies:**
- **Tauri v2.10.2**: Desktop framework — current stable, native macOS integration, 2-2.5MB bundle size, Rust security model
- **React 18.3+**: Frontend UI — dominant ecosystem, best TypeScript support, mature Tauri integration
- **TypeScript 5.7+**: Type safety — essential for IPC type safety, compile-time error catching
- **Vite 6.x**: Build tool — official Tauri recommendation, fast HMR, optimized builds
- **xAI SDK 0.1.x**: AI integration — official Protocol Buffer definitions, type-safe, supports grok-4-fast-reasoning
- **cocoa crate**: macOS bindings — REQUIRED for NSWindowLevel control (floating overlay above all apps)
- **osascript crate**: AppleScript execution — terminal context reading (cwd, recent commands)
- **sysinfo crate**: Process detection — detect active terminal (Terminal.app, iTerm2, etc.)

**Frontend stack:**
- **shadcn/ui**: React components — 65K+ stars, native-looking, copy-paste pattern (no npm bloat)
- **Tailwind CSS 4.x**: Styling — standard with shadcn/ui, fast styling, dark mode built-in
- **Zustand 5.x**: State management — minimal (3KB), hooks-based, perfect for overlay state

**Critical implementation notes:**
- NSWindowLevel control requires unsafe Cocoa code (Tauri's setAlwaysOnTop insufficient for fullscreen apps)
- Terminal context reading needs AppleScript automation with Accessibility permissions
- Global hotkey Cmd+K conflicts with system apps (recommend Cmd+Shift+K default with user configuration)

### Expected Features

**Must have (table stakes):**
- **Global hotkey activation** — universal pattern (Cmd+K), users expect instant access from anywhere
- **Keyboard-first navigation** — users invoke overlay to avoid mouse, all interactions keyboard-driven
- **Natural language → command** — core value proposition, users describe intent, AI generates shell command
- **Command preview before execution** — safety requirement, never auto-execute
- **Copy to clipboard** — minimum viable output, universal fallback
- **Basic terminal context: cwd** — dramatically improves command quality for file operations
- **Command explanation** — users need to understand what command does

**Should have (competitive):**
- **Destructive command detection** — flag rm -rf, DROP TABLE, force-push, require confirmation
- **Command modification** — allow user to edit AI-generated command before copying
- **Git context awareness** — commands contextualized to current branch, repo state
- **Multi-model support** — let users choose LLM (GPT-4, Claude, Gemini, Grok)
- **Terminal emulator agnostic** — work with Terminal.app, iTerm2, Warp, Alacritty

**Defer (v2+):**
- **Shell history awareness** — complex, privacy-sensitive, not critical for validation
- **Auto-paste to terminal** — fragile, terminal-specific, clipboard approach works universally
- **Streaming responses** — nice UX improvement, not core to value prop
- **Multi-step workflows** — complex, single commands sufficient for MVP
- **Command templates/snippets** — feature creep, focus on AI generation first

**Anti-features (explicitly avoid):**
- **Auto-execution of commands** — catastrophically dangerous, one bad rm -rf ruins trust forever
- **Shell plugin requirement** — friction in onboarding, breaks in SSH/containers
- **Cloud-synced command history** — privacy nightmare, commands contain sensitive data
- **Built-in terminal emulator** — scope explosion, users already have preferences

### Architecture Approach

The architecture follows Tauri's multi-process model with clear boundaries between Rust backend (system integration) and web frontend (UI/streaming). The Core process handles OS integration via NSPanel, Accessibility API, and AppleScript, while WebView processes render UI using React components. Communication uses Tauri IPC with Commands (request-response) for one-time operations and Events (fire-and-forget) for streaming AI responses.

**Major components:**
1. **Hotkey Manager** (Rust Core) — Register/unregister global shortcuts (Cmd+K), trigger window show
2. **Window Manager** (Rust Core) — Create/show/hide NSPanel overlay, manage window state using tauri-nspanel for proper fullscreen behavior
3. **Terminal Reader** (Rust Core) — Read active terminal context (cwd, selected text) via Accessibility API + process inspection (active-win-pos-rs, libproc-rs)
4. **AI Streaming Client** (Rust Core) — HTTP client for xAI API, handle SSE streaming with reqwest-eventsource
5. **State Manager** (Rust Core) — Manage app state (settings, terminal context, AI session) with Mutex
6. **AppleScript Bridge** (Rust Core) — Execute AppleScript for pasting into terminal (Phase 2+)
7. **Overlay UI** (WebView) — Render transparent overlay window, handle user input with shadcn/ui components
8. **AI Stream Renderer** (WebView) — Receive streaming tokens via Events, render markdown with marked.js

**Key architectural patterns:**
- **Event-driven window management** — global hotkey emits app event, Window Manager listens and shows panel
- **Context caching with fallback** — cache terminal context with 100ms TTL, use fallback strategies when Accessibility API fails
- **Streaming with backpressure** — accumulate tokens in Rust, emit in batches (50ms intervals) to avoid IPC flooding
- **Type-safe IPC with Serde** — define shared types between Rust and TypeScript, prevent runtime errors

### Critical Pitfalls

1. **Sandboxing incompatible with Accessibility API** — DO NOT enable app sandboxing. Apple's sandbox blocks inter-application accessibility features even with user-granted permissions. Must use Developer ID distribution (notarization only), not App Store. Add entitlements file with `com.apple.security.automation.apple-events`. *Phase 1 blocker.*

2. **AppleScript command injection via unsanitized xAI responses** — AI responses may contain backticks, semicolons, shell metacharacters. NEVER directly interpolate into AppleScript. Use proper escaping, whitelist safe characters, reject dangerous ones, or use base64 encoding. One injection vulnerability destroys trust. *Phase 2 critical security issue.*

3. **Transparent window rendering glitches on macOS Sonoma** — Sonoma (14.0+) breaks transparent windows with focus changes (Stage Manager). Tauri's setAlwaysOnTop insufficient. **Solution:** Set activation policy to Accessory (app won't appear in Dock/Cmd+Tab, acceptable for menu bar overlay). *Phase 1 architectural decision.*

4. **Global hotkey Cmd+K conflicts with system apps** — Cmd+K used by Safari (search), VS Code (command palette), Slack (quick switcher). Registration may fail silently. **Solution:** Allow user-configurable hotkey, provide alternative defaults (Cmd+Shift+K, Cmd+Option+K), test all three at startup, detect registration failures and notify user. *Phase 1 UX requirement.*

5. **Accessibility permissions silently fail without prompting** — Accessibility API requires manual System Settings navigation (no automatic prompt like camera/mic). API calls fail silently with generic errors. **Solution:** Use tauri-plugin-macos-permissions to detect and guide, create first-run onboarding wizard, handle permission revocation gracefully (Ventura bug causes spontaneous revocation). *Phase 1 onboarding critical.*

## Implications for Roadmap

Based on research, suggested phase structure follows dependency order and risk mitigation:

### Phase 1: Foundation (Overlay + Permissions)
**Rationale:** Establishes core architecture and validates macOS integration challenges before building AI features. Window management and permissions are prerequisites for all subsequent phases.

**Delivers:**
- Working overlay window with global hotkey (Cmd+Shift+K)
- NSPanel integration for proper fullscreen behavior
- Accessibility permission onboarding flow
- Menu bar presence (no dock icon)
- Basic transparent UI with keyboard navigation

**Addresses features:**
- Global hotkey activation (table stakes)
- Clean, minimal UI (table stakes)
- Menu bar presence (table stakes)
- Accessibility permissions onboarding (macOS specific)

**Avoids pitfalls:**
- Pitfall #3: Transparent window glitches (set ActivationPolicy::Accessory immediately)
- Pitfall #4: Hotkey conflicts (build configurable system from start)
- Pitfall #5: Accessibility permissions (create onboarding wizard)

**Critical decisions made in Phase 1:**
- Sandboxing disabled (Developer ID distribution only)
- Activation policy set to Accessory (no Dock icon)
- Hotkey configuration architecture

**Research flags:** None (well-documented Tauri patterns)

### Phase 2: Terminal Context Reading
**Rationale:** Terminal context is essential for command quality. Separating from AI integration allows focused testing of macOS Accessibility API challenges. This is the highest-risk technical component.

**Delivers:**
- Active terminal detection (Terminal.app, iTerm2, Warp)
- Current working directory via process inspection (libproc-rs)
- Selected text reading via Accessibility API
- Context display in overlay UI
- Fallback strategies when API fails

**Uses stack:**
- active-win-pos-rs (active window detection)
- libproc-rs (process inspection for cwd)
- sysinfo (process detection)
- Custom Accessibility API FFI bindings

**Implements architecture:**
- Terminal Reader component
- Context caching with fallback pattern
- State Manager for terminal context storage

**Avoids pitfalls:**
- Pitfall #5: Accessibility permission handling (already solved in Phase 1)
- Pitfall #10: Terminal detection (use process-based detection, not hardcoded app names)

**Research flags:** MEDIUM
- Needs deep research on iTerm2 vs Warp vs Alacritty AppleScript variations
- Accessibility API FFI implementation requires testing across terminal apps
- Fallback strategy testing for permission denial scenarios

### Phase 3: AI Command Generation
**Rationale:** Core value proposition. Built on foundation (Phase 1) without dependency on terminal context (Phase 2 parallel-trackable). Can develop with mocked context then integrate real context from Phase 2.

**Delivers:**
- xAI Grok API integration
- Natural language → command generation
- Command preview with syntax highlighting
- Command explanation (what does this command do?)
- Error handling (API failures, network issues)
- Copy to clipboard

**Uses stack:**
- xai-sdk (official Protocol Buffer definitions)
- reqwest-eventsource (SSE streaming)
- tokio (async runtime)
- serde/serde_json (JSON serialization)
- marked.js frontend (markdown rendering)

**Implements architecture:**
- AI Streaming Client component
- Event-based streaming (backpressure, batched tokens)
- AI Stream Renderer component

**Addresses features:**
- Natural language → command (table stakes)
- Command preview (table stakes)
- Copy to clipboard (table stakes)
- Command explanation (table stakes)
- Error handling and retry (table stakes)

**Avoids pitfalls:**
- Pitfall #9: Streaming IPC bottleneck (use Tauri events with batched 50ms emissions)

**Research flags:** LOW
- xAI API well-documented
- SSE streaming pattern established
- Standard Tauri IPC patterns

### Phase 4: Safety Layer
**Rationale:** Must be in place before considering auto-paste or public release. Builds trust, prevents catastrophic mistakes. Can't defer—safety is launch-critical.

**Delivers:**
- Destructive command detection (rm -rf, git push --force, DROP TABLE)
- Command approval workflow (explicit confirmation required)
- Command modification before execution
- Warning UI for dangerous commands
- Safe mode detection (dry-run flags when available)

**Addresses features:**
- Destructive command flagging (should have, competitive)
- Command modification (should have)
- Command approval workflow (safety critical)

**Implements architecture:**
- Pattern matching (DCG-style whitelist/blocklist)
- Command sanitization layer (preparation for Phase 5)

**Research flags:** LOW
- DCG safety patterns well-documented
- Pattern matching straightforward

### Phase 5: Terminal Pasting (Post-MVP Optional)
**Rationale:** Clipboard workflow (Phase 1-4) is sufficient for MVP validation. Auto-paste adds convenience but is fragile and terminal-specific. Only build if user feedback demands it.

**Delivers:**
- AppleScript bridge for Terminal.app and iTerm2
- Auto-paste to active terminal
- Window focus restoration
- Graceful degradation for unsupported terminals

**Uses stack:**
- osascript crate
- AppleScript execution

**Implements architecture:**
- AppleScript Bridge component

**Avoids pitfalls:**
- Pitfall #2: AppleScript command injection (CRITICAL—implement sanitization before ANY pasting)
- Pitfall #10: Terminal detection (already solved in Phase 2)

**Research flags:** HIGH
- Needs deep research on terminal-specific AppleScript APIs
- iTerm2, Warp, Alacritty have different automation patterns
- Security testing for injection vectors

### Phase Ordering Rationale

**Why this order:**
1. **Foundation first** (Phase 1) because window management and permissions are prerequisites for everything. Architectural decisions (sandboxing, activation policy) can't be changed later without rewrite.

2. **Terminal Context before AI** (Phase 2 before 3) logically, but can be parallel-tracked. AI can develop with mocked context. Separating validates macOS integration challenges independently.

3. **Safety before Pasting** (Phase 4 before 5) because auto-paste without safety is unacceptable risk. Command injection must be solved before any AppleScript execution.

4. **MVP = Phases 1-4** (not Phase 5). Clipboard workflow validates core value without fragile terminal automation. Phase 5 only if users demand convenience.

**Dependency graph:**
```
Phase 1 (Foundation)
  ├── Phase 2 (Terminal Context) — depends on Accessibility permissions
  ├── Phase 3 (AI Generation) — depends on window + permissions
  └── Phase 4 (Safety) — depends on command preview UI
      └── Phase 5 (Pasting) — depends on safety + context + AppleScript
```

**How this avoids pitfalls:**
- Phase 1 addresses all architectural pitfalls before code written
- Phase 2 isolates highest-risk macOS integration
- Phase 4 blocks security-critical pitfall #2 before enabling pasting
- Phase 5 deferred until validation complete (avoid scope creep)

### Research Flags

**Phases needing deeper research during planning:**
- **Phase 2 (Terminal Context):** MEDIUM confidence
  - Accessibility API FFI implementation needs custom code
  - Terminal-specific detection strategies (iTerm2 vs Warp vs Alacritty)
  - Fallback strategy testing for permission denial
  - Recommend: /gsd:research-phase for Accessibility API integration

- **Phase 5 (Terminal Pasting):** HIGH research needs
  - AppleScript API differences across terminals
  - Security testing for command injection vectors
  - Sanitization strategy validation
  - Recommend: /gsd:research-phase before implementation

**Phases with standard patterns (skip research-phase):**
- **Phase 1 (Foundation):** HIGH confidence — Tauri official docs, NSPanel plugin documented, hotkey patterns established
- **Phase 3 (AI Generation):** HIGH confidence — xAI SDK available, SSE streaming well-documented, Tauri IPC patterns established
- **Phase 4 (Safety):** HIGH confidence — DCG patterns documented, pattern matching straightforward

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core stack (Tauri/React/Vite) current and verified. All crates production-tested. xAI SDK official. |
| Features | MEDIUM-HIGH | Overlay/launcher patterns well-established (Raycast/Alfred). AI command generation rapidly evolving (2026). |
| Architecture | HIGH | Tauri v2 architecture documented. NSPanel integration verified via tauri-nspanel plugin. IPC patterns established. |
| Pitfalls | HIGH | All critical pitfalls verified via official docs, Apple security guidelines, Tauri GitHub issues. Mitigation strategies documented. |

**Overall confidence:** HIGH

### Gaps to Address

**Accessibility API implementation details:**
- Research shows pattern (Shellporter blog) but requires custom FFI
- No off-the-shelf crate for macOS Accessibility text reading
- **Handling:** Phase 2 research-phase deep dive, prototype early, have fallback (process inspection only)

**Terminal app compatibility:**
- AppleScript APIs differ between Terminal.app, iTerm2, Warp, Alacritty
- No comprehensive documentation for all terminals
- **Handling:** Start with Terminal.app + iTerm2 (90%+ market), add others based on user demand, community feedback for unsupported terminals

**xAI Grok API pricing and rate limits:**
- API pricing not fully documented as of Feb 2026
- Rate limits unknown for free tier
- **Handling:** Validate during Phase 3 implementation, have multi-provider architecture ready for fallback

**macOS version-specific issues:**
- Sonoma (14.0+) transparent window glitches — SOLVED (ActivationPolicy::Accessory)
- Ventura (13.0+) spontaneous permission revocation — MITIGATED (detect and re-prompt)
- Future macOS versions may break Accessibility API
- **Handling:** Test on macOS 13+ (Ventura, Sonoma, Sequoia), monitor Tauri GitHub for upstream fixes

## Sources

### Primary (HIGH confidence)

**Tauri Official Documentation:**
- [Tauri v2 Documentation](https://v2.tauri.app/) — architecture, IPC, state management
- [Tauri v2 Release Page](https://v2.tauri.app/release/) — version verification
- [Tauri Plugins](https://v2.tauri.app/plugin/) — global-shortcut, clipboard-manager, shell
- [Tauri Window Customization](https://v2.tauri.app/learn/window-customization/) — NSPanel, transparent windows
- [Tauri macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) — notarization, entitlements
- [Tauri IPC Documentation](https://v2.tauri.app/concept/inter-process-communication/) — Commands, Events
- [Tauri State Management](https://v2.tauri.app/develop/state-management/) — Mutex patterns

**Apple Official Documentation:**
- [Apple Shell Script Security](https://developer.apple.com/library/archive/documentation/OpenSource/Conceptual/ShellScripting/ShellScriptSecurity/ShellScriptSecurity.html) — AppleScript injection
- [Apple Support: Accessibility Permissions](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac) — permission flow

**Verified Crates:**
- [active-win-pos-rs](https://crates.io/crates/active-win-pos-rs) — active window detection
- [libproc-rs](https://github.com/andrewdavidmackenzie/libproc-rs) — process inspection
- [reqwest-eventsource](https://docs.rs/reqwest-eventsource/) — SSE streaming
- [tauri-nspanel](https://github.com/ahkohd/tauri-nspanel) — NSPanel integration
- [xai-sdk](https://github.com/0xC0DE666/xai-sdk) — xAI Grok API

**Verified GitHub Issues (Tauri):**
- [Issue #8255](https://github.com/tauri-apps/tauri/issues/8255) — Transparent window glitch Sonoma
- [Issue #9503](https://github.com/tauri-apps/tauri/issues/9503) — Cannot drag window Overlay titleBarStyle
- [Issue #14102](https://github.com/tauri-apps/tauri/issues/14102) — Focusable: false broken macOS
- [Issue #10025](https://github.com/tauri-apps/tauri/issues/10025) — Global shortcut fires twice macOS
- [Issue #11488](https://github.com/tauri-apps/tauri/issues/11488) — visibleOnAllWorkspaces not staying on top

### Secondary (MEDIUM confidence)

**Community Resources:**
- [Building Shellporter: From Idea to Production](https://www.marcogomiero.com/posts/2026/building-shellporter/) — Accessibility API patterns
- [tauri-plugin-macos-permissions](https://github.com/ayangweb/tauri-plugin-macos-permissions) — permission detection
- [iTerm2 Scripting Documentation](https://iterm2.com/documentation-scripting.html) — terminal automation
- [Streaming at Scale: SSE, WebSockets & Real-Time AI APIs](https://learnwithparam.com/blog/streaming-at-scale-sse-websockets-real-time-ai-apis) — SSE patterns

**Competitive Research:**
- [Raycast](https://www.raycast.com/) — overlay UX patterns
- [Warp AI](https://www.warp.dev/warp-ai) — AI terminal features
- [GitHub Copilot CLI](https://github.com/features/copilot/cli) — command generation patterns
- [Amazon Q CLI](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line.html) — terminal integration

### Tertiary (LOW confidence, needs validation)

**Security Research:**
- [ClickFix macOS Campaign: AppleScript Phishing](https://hunt.io/blog/macos-clickfix-applescript-terminal-phishing) — injection vectors
- [MITRE ATT&CK: AppleScript T1059.002](https://attack.mitre.org/techniques/T1059/002/) — attack patterns
- [DCG: Destructive Command Guard Philosophy](https://reading.torqsoftware.com/notes/software/ai-ml/safety/2026-01-26-dcg-destructive-command-guard-safety-philosophy-design-principles/) — safety patterns

---
*Research completed: 2026-02-21*
*Ready for roadmap: YES*
*MVP scope: Phases 1-4*
*Estimated MVP timeline: 3-4 weeks (single developer)*
