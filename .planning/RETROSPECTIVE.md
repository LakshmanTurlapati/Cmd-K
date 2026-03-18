# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.2.4 -- Overlay UX, Safety & CI/CD

**Shipped:** 2026-03-04
**Phases:** 4 | **Plans:** 5

### What Was Built
- Overlay z-order fix (Floating level) and drag-to-reposition with session-scoped memory
- Cross-platform destructive command safety expanded from ~80 to 150 patterns
- Windows Terminal ConPTY shell detection fix
- Automated CI/CD pipeline with GitHub Actions for signed macOS DMG and Windows NSIS builds
- Parameterized build script and secrets setup documentation

### What Worked
- Small focused phases (17-18 overlay UX, 19 safety, 20 CI/CD) kept each phase under 15 minutes of execution
- Phase 17 was a single-line change with high impact -- good scoping
- Build script parameterization (Plan 20-01) before workflow creation (Plan 20-02) was the right sequence
- Yolo mode with plan-check enabled: fast execution with quality gates where they matter

### What Was Inefficient
- Requirements were split across three sub-milestone labels (v0.2.2, v0.2.3, v0.3.0) but shipped as one release (v0.2.4) -- cleaner to define a single milestone scope upfront
- Phases 19-20 were added incrementally after v0.2.2 was already defined -- milestone scope creep required re-labeling at archive time
- No milestone audit performed before archiving -- skipped to move faster

### Patterns Established
- PanelLevel::Floating as the standard overlay level for macOS utility panels
- // === Section === headers for organizing large pattern arrays
- Env var with fallback pattern for CI/local dual-path scripts
- Tag-triggered release pipeline with artifact pass-through architecture

### Key Lessons
1. Define milestone scope upfront rather than accumulating phases post-hoc -- avoids version label confusion at archive time
2. Single-line changes can have outsized impact (Phase 17: one line fixed system dialog visibility)
3. CI/CD pipeline should be an early milestone, not a late addition -- every manual release is friction

### Cost Observations
- Model mix: primarily opus for execution, sonnet for planning
- All 5 plans completed in under 20 minutes total execution time
- Notable: Phase 19 (150 pattern expansion) completed in 3 minutes -- regex patterns are high-leverage

---

## Milestone: v0.2.6 -- Multi-Provider, WSL & Auto-Update

**Shipped:** 2026-03-09
**Phases:** 5 | **Plans:** 10

### What Was Built
- Provider abstraction layer with 3 streaming adapters covering 5 AI providers (OpenAI, Anthropic, Gemini, xAI, OpenRouter)
- Provider-aware onboarding and settings UI with tier-grouped model lists and per-provider model memory
- WSL terminal context detection via process ancestry walking for Windows Terminal, VS Code, Cursor, and standalone wsl.exe
- VS Code WSL detection using multi-signal approach: window title patterns, UIA tree walking, CWD path style, shell priority
- Auto-updater with background checks, tray state machine, install-on-quit, and Ed25519 signing
- CI/CD updater pipeline generating .sig files and latest.json manifest

### What Worked
- Provider abstraction with enum dispatch and 3 adapters for 5 providers was clean and minimal
- Tier-grouped model lists (Fast/Balanced/Most Capable) gave consistent UX across all providers
- Process ancestry walking for WSL detection worked reliably across all 4 host terminal types
- Install-on-quit pattern for auto-updater avoided forced restart UX
- Entire milestone (5 phases, 10 plans) completed in a single day

### What Was Inefficient
- v0.2.6 tag initially pointed to docs-only commit -- code changes were uncommitted when tagged, requiring tag recreation
- Phase 23.1 was inserted urgently for VS Code WSL detection but left a known gap (cmd.exe detection)
- try_focused_subtree approach for VS Code UIA was abandoned mid-phase when xterm.js proved inaccessible without screen reader mode

### Patterns Established
- Enum dispatch for compile-time-known provider routing (no trait objects needed)
- Multi-signal detection pattern: window title + UIA + CWD path + shell priority for IDE terminal identification
- UpdateState as separate Tauri managed state when plugin types don't implement Default
- Heredoc-based JSON assembly in CI for structured artifacts without external tools

### Key Lessons
1. Always commit code changes before tagging a release -- docs commits alone don't include working tree changes
2. Multi-signal detection (title + UIA + path + shell) is more robust than relying on any single heuristic for IDE terminals
3. OpenRouter as meta-provider provides excellent onboarding UX -- one key for all providers
4. Scrollable model lists with max-height prevent layout overflow when provider has many models

### Cost Observations
- Model mix: primarily opus for execution
- All 10 plans completed in under 30 minutes total execution time
- Notable: Phase 23.1 pivot (abandoning UIA focused-subtree) cost minimal time due to rapid signal-based fallback

---

## Milestone: v0.2.7 -- Cost Estimation

**Shipped:** 2026-03-10
**Phases:** 2 | **Plans:** 3

### What Was Built
- Token extraction from all 3 streaming adapters (OpenAI-compat, Anthropic, Gemini) with session-scoped per-model accumulation
- Two-tier pricing: 47 curated model prices hardcoded + OpenRouter dynamic pricing from API
- Live cost display in Settings Model tab with dollar amount, token breakdown, greyscale angular sparkline
- Reset button for session stats, fetch-on-tab-open for live updates

### What Worked
- Clean separation: backend tracks raw tokens, cost calculated at read time using pricing data
- Decoupled UsageAccumulator (String keys, no Provider dependency) kept state.rs clean
- Div-based sparkline was simpler than canvas/SVG and matched the existing design system perfectly
- Entire pipeline is pure cross-platform -- no #[cfg(target_os)] anywhere in token/pricing/display code
- Two phases with clear backend/frontend boundary made for fast execution

### What Was Inefficient
- Initial curated pricing only covered 16 models -- expanded to 47 as a follow-up after execution
- Gemini preview model IDs changed (2.5-flash-preview-05-20 → 2.5-flash) requiring both old and new IDs
- Grok 4 model ID and price were wrong in initial hardcoding ($6/$18 → should have been grok-4-0709 at $3/$15)

### Patterns Established
- Per-query metadata stored at record time, costs calculated at read time (pricing changes apply retroactively)
- Two-tier pricing lookup: curated first, dynamic fallback for OpenRouter
- Fetch-on-mount for tab data instead of push-based updates (simpler, IPC is near-instant)
- Angular div-based sparkline pattern with flex layout and proportional heights

### Key Lessons
1. Hardcoded pricing data should be researched comprehensively upfront, not incrementally -- avoids corrections
2. Keep legacy model ID aliases when APIs transition (Gemini preview → GA) for backward compatibility
3. Read-time cost calculation is preferable to record-time -- decouples pricing updates from data collection

### Cost Observations
- Model mix: opus for execution, sonnet for planning/verification
- All 3 plans completed rapidly -- smallest milestone to date
- Notable: Cross-platform verification agent confirmed zero platform-specific code in the entire pipeline

---

## Milestone: v0.2.8 -- Windows Terminal Detection Fix & Provider Icons

**Shipped:** 2026-03-14
**Phases:** 3 | **Plans:** 6

### What Was Built
- ConPTY-aware shell discovery with ProcessSnapshot struct replacing highest-PID heuristic
- PEB command line analysis filtering batch cmd.exe from interactive sessions
- UIA-guided shell type disambiguation for multi-tab IDE terminals
- Multi-signal WSL text detection with scoring threshold (≥2) eliminating false positives
- Scoped UIA tree walk targeting ControlType::List terminal panels with 3-strategy cascade
- Provider SVG icon branding in onboarding and settings for all 5 providers

### What Worked
- TDD approach in Phase 28 (12 tests written before implementation) caught edge cases early
- UAT-driven gap closure: Phase 27 UAT identified multi-tab disambiguation gap, spawned Plan 27-03
- UIA text read before process tree walk was a key architectural insight -- single read serves 3 purposes
- ProcessSnapshot consolidation eliminated redundant system calls across the detection pipeline
- Provider icons phase was clean UI-only work -- completed in a single plan

### What Was Inefficient
- Phase 27 initially planned for 2 plans but needed a 3rd (27-03) after UAT revealed multi-tab disambiguation gap -- better upfront analysis of multi-tab scenarios would have caught this
- Cross-plan compilation errors in Phase 28 when parallel execution created conflicting function signatures

### Patterns Established
- ProcessSnapshot as shared context passed through detection pipeline (capture once, query many times)
- Scoring-based text classification with weighted signals for ambiguous detection scenarios
- 3-strategy cascade pattern for graceful degradation (preferred → scoped → full fallback)
- UIA ControlType targeting for precise element selection in complex accessibility trees

### Key Lessons
1. UAT as a gap-finding mechanism works well -- invest in thorough UAT criteria upfront to catch gaps before verification
2. PEB command line analysis is a reliable Windows process classification technique -- cheaper than WMI, more informative than just exe name
3. Scoring thresholds for multi-signal detection should be set conservatively (≥2) to eliminate false positives at the cost of occasional false negatives

### Cost Observations
- Model mix: opus for execution, sonnet for planning/verification agents
- 6 plans completed in a single day -- mid-size milestone
- Notable: UAT-driven gap closure (Plan 27-03) added minimal overhead and significantly improved detection quality

---

## Milestone: v0.3.9 -- Linux Support & Smart Terminal Context

**Shipped:** 2026-03-15
**Phases:** 7 | **Plans:** 10

### What Was Built
- Linux /proc process detection — CWD, shell type, process tree walking with 18 terminal emulators and 9 IDEs classified
- X11 overlay and hotkey — x11rb EWMH PID capture, CSS backdrop-blur-xl frosted glass, always-on-top floating window
- Linux paste via xdotool on X11, Wayland clipboard fallback with inline amber hint
- Smart terminal context pipeline — ANSI stripping, model-aware 12% token-budget truncation, command-output segmentation (cross-platform)
- Linux terminal text reading — AT-SPI2 D-Bus for VTE, kitty remote control, WezTerm CLI
- AppImage distribution with dual-arch CI (x86_64 + aarch64), auto-updater manifest, write-permission guard
- Showcase website update with OS-detected downloads and privacy policy

### What Worked
- /proc filesystem provided a dramatically simpler process detection implementation than macOS (libproc FFI) or Windows (CreateToolhelp32Snapshot) — zero external dependencies
- Three-way cfg gate pattern (target_os = "macos"/"windows"/"linux") kept platform boundaries crystal clear
- Strategy dispatch by exe_name for terminal text reading was clean and extensible
- Smart context pipeline was fully cross-platform (no cfg gates) — validated architecture choice
- Entire milestone (7 phases, 10 plans) completed in 2 days despite being the largest feature addition since Windows support
- Phase 36 (showcase website) was a good scope addition — ensures user-facing artifacts stay in sync with code

### What Was Inefficient
- zbus API documentation didn't match actual zbus 5 API — cost extra debugging time in Phase 34 (Connection::builder doesn't exist, blocking feature name wrong)
- Phase plan checkboxes in ROADMAP.md inconsistent (some marked [x], others [ ]) for completed plans — cosmetic but confusing
- Milestone version label (v0.3.9) was set during roadmap creation but the jump from v0.2.8 seems large — version numbering could be more systematic

### Patterns Established
- /proc reads return None/empty on error, never panic — safe default for race conditions (process exits between reads)
- Return-value hint communication (Result<String, String>) for platform-specific paste fallback signaling
- Model-aware token budgeting as replacement for hard-coded line limits
- AT-SPI2 tree walk pattern: registry → app by PID → recursive child walk for role=Terminal → GetText
- Native ARM runners for aarch64 builds instead of cross-compilation

### Key Lessons
1. /proc is the easiest platform to implement process detection for — consider Linux-first for future process-related features
2. zbus documentation should be verified against actual crate version (v5 changed significantly from v4)
3. Token-budget approach for context is strictly superior to line-count limits — adapts to model size automatically
4. Showcase website should be updated as part of every user-facing milestone, not as an afterthought

### Cost Observations
- Model mix: primarily opus for execution
- All 10 plans completed in rapid succession — many completed in 3-5 minutes
- Notable: Linux phases were faster than equivalent Windows phases due to /proc simplicity vs Win32 API complexity

---

## Milestone: v0.3.11 -- Local LLM Providers

**Shipped:** 2026-03-18
**Phases:** 4 | **Plans:** 5

### What Was Built
- Ollama and LM Studio added as local LLM providers with keyless auth and dynamic base URL configuration
- Auto-discovered models from Ollama /api/tags and LM Studio /v1/models with parameter-size-based tier grouping
- Local provider streaming via existing OpenAI-compat adapter with 120s cold-start timeout
- Onboarding wizard auto-skips API key step for local providers
- "Free (local)" usage label replaces "$0.00" for local provider queries
- Connection health indicators and provider SVG icons

### What Worked
- Heavy reuse of existing infrastructure: OpenAI-compat adapter, provider enum pattern, PROVIDERS array, tier grouping
- Phase 37 frontend plan (37-02) front-loaded most UI work, making Phase 40 very small (2 tasks + checkpoint)
- Research correctly identified the StepModelSelect model-fetch gap before planning — prevented a runtime bug
- Integration checker caught the URL-change model refresh edge case that manual testing would likely miss

### What Was Inefficient
- Phase 40 was scoped as a full frontend phase but 3/4 requirements were already done in Phase 37 — could have been a single task in Phase 37
- LFUI-01/02/04 should have been in Phase 37's requirements from the start (they were implemented there)
- Nyquist VALIDATION.md was never created for Phase 40 (research had no Validation Architecture section)

### Patterns Established
- Local providers pattern: `is_local()` check → skip keychain → dynamic URL from settings store → health check via validate_api_key
- Step-skip pattern in onboarding: conditional `nextStep` increment (same as Windows/Accessibility)
- "Already done" requirements: legitimate for verification phases to mark pre-done requirements

### Key Lessons
- When a backend phase includes "minimum frontend wiring," the scope often covers most of the frontend phase — plan accordingly
- The OpenAI-compat API is a powerful unifier — both Ollama and LM Studio needed zero custom streaming code

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Phases | Plans | Key Change |
|-----------|--------|-------|------------|
| v0.1.0 | 8 | 21 | Initial MVP, established GSD workflow |
| v0.1.1 | 3 | 6 | Fast follow-up, focused scope |
| v0.2.1 | 7 | 11 | Cross-platform port, parallel branch work |
| v0.2.4 | 4 | 5 | UX polish + infra, yolo mode with plan-check |
| v0.2.6 | 5 | 10 | Multi-provider + WSL + auto-update, single-day milestone |
| v0.2.7 | 2 | 3 | Cost estimation, smallest milestone, fully cross-platform |
| v0.2.8 | 3 | 6 | Terminal detection fix + icons, UAT-driven gap closure, TDD |
| v0.3.9 | 7 | 10 | Full Linux support + smart context, largest since Windows port, 2 days |
| v0.3.11 | 4 | 5 | Local LLM providers, heavy infra reuse, most UI pre-done in backend phase |

### Top Lessons (Verified Across Milestones)

1. Small focused phases execute faster and produce cleaner commits than large multi-concern phases
2. Capture-before-show pattern applies everywhere: get context before changing state
3. In-memory state with no disk persistence is almost always the right v1 choice for transient preferences
4. Always commit code changes before tagging -- verified by v0.2.6 tag recreation incident
5. Multi-signal detection is more robust than single-heuristic approaches for complex environments (IDE terminals, WSL)
6. UAT-driven gap closure catches real-world issues that unit tests miss -- worth the verification overhead
7. Scoring-based classification with weighted signals handles ambiguous detection better than binary heuristics
8. /proc is dramatically simpler than macOS/Windows process APIs — consider Linux-first for future process features
9. Token-budget approach for context is strictly superior to line-count limits — adapts to model size automatically
