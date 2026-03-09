# Project Research Summary

**Project:** CMD+K v0.2.6 -- Multi-Provider AI, WSL Context & Auto-Updater
**Domain:** Desktop overlay app (Tauri v2) with AI-powered terminal command generation
**Researched:** 2026-03-08
**Confidence:** MEDIUM-HIGH

## Executive Summary

CMD+K is an existing Tauri v2 overlay app that generates terminal commands via AI. The v0.2.6 milestone adds three independent features: multi-provider AI support (OpenAI, Anthropic, Google Gemini, xAI, OpenRouter), WSL terminal context detection on Windows, and an auto-updater via tauri-plugin-updater. The existing stack (Tauri v2, React 19, Zustand 5, reqwest, eventsource-stream, windows-sys) handles nearly everything -- only one new Rust crate (tauri-plugin-updater) and one new npm package (@tauri-apps/plugin-updater) are needed. The provider abstraction is pure architecture: a Rust enum with per-variant dispatch, not new dependencies.

The recommended approach is to build the provider abstraction layer first because all multi-provider UI and AI work depends on it, and because the xAI-hardcoded references are scattered across 4+ files (ai.rs, xai.rs, keychain.rs, store/index.ts). Getting this wrong leaks API keys to wrong endpoints. WSL context and auto-updater are fully independent of each other and of the provider work, so they can proceed in parallel once provider abstraction lands. Four of five AI providers share the OpenAI-compatible SSE format; only Anthropic diverges significantly (different auth header, system prompt location, SSE event structure). Google Gemini can use its OpenAI-compatible endpoint to avoid a third SSE parser.

The top risks are: (1) missing a hardcoded xAI reference during the provider refactor, causing API key leakage to wrong endpoints; (2) shipping the first updater-enabled release without the Ed25519 signing key, permanently preventing auto-updates for those users; and (3) WSL subprocess calls (wsl.exe -e) adding 200-500ms latency to context detection, requiring timeouts and potential caching. All three are preventable with upfront auditing, key generation before first release, and bounded timeouts.

## Key Findings

### Recommended Stack

The existing stack requires minimal additions. The provider abstraction is architectural, not dependency-driven.

**Core technologies (no changes):**
- **Tauri v2 + tauri-plugin-http (reqwest):** HTTP client for all 5 providers -- all use REST + SSE
- **eventsource-stream 0.2:** SSE parsing works for all providers (different JSON paths, same transport)
- **keyring 3:** Per-provider API key storage using different account names -- already supports arbitrary accounts
- **Zustand 5 + tauri-plugin-store 2:** Provider selection state and persistence
- **windows-sys 0.59:** Process tree walking for WSL detection -- already has all needed features

**New additions (auto-updater only):**
- **tauri-plugin-updater 2** (Rust): Official Tauri v2 update plugin -- check, download, verify, install
- **@tauri-apps/plugin-updater ^2** (npm): Frontend API for update check/trigger

### Expected Features

**Must have (table stakes):**
- Provider selector in settings and onboarding (user picks their provider)
- Per-provider API key storage in platform keychain (keys persist across provider switches)
- Streaming responses from all 5 providers (non-streaming would be a regression)
- Per-provider model listing (hardcoded for Anthropic, API-fetched for others)
- WSL session detection with correct CWD and shell type
- WSL-aware system prompts (Linux commands, not Windows)
- Update check on launch with non-blocking notification
- Update signing and CI/CD pipeline for latest.json manifest

**Should have (differentiators):**
- OpenRouter as meta-provider (one key accesses 100+ models -- killer UX simplification)
- Provider switching without losing conversation history
- Automatic WSL distro detection ("Ubuntu (WSL)" badge)
- Silent background update checks (no nagging)
- Semantic model grouping by capability tier across providers

**Defer (v2+):**
- Fallback provider on error (confusing UX, different model behavior)
- Update channel selector (stable/beta -- no demand yet)
- Custom API endpoint URLs (use OpenRouter instead)
- Multi-provider simultaneous queries (wastes credits, complicates UX)
- Provider-specific prompt customization UI (tune internally)

### Architecture Approach

The architecture uses a Provider enum with per-variant match dispatch (not Box<dyn Trait>) because the provider set is fixed at 5 variants, async trait objects require boxing futures, and enum match gives compile-time exhaustiveness checking. Each provider gets its own module (commands/providers/{openai,anthropic,google,xai,openrouter}.rs) exporting endpoint(), headers(), build_body(), extract_token(), and is_done() functions. The stream_ai_response command gains a provider parameter and dispatches to the correct module. WSL detection lives in a new terminal/wsl.rs module that shells out via wsl.exe -e for context, integrating into the existing detect_app_context_windows() flow. Auto-updater is a plugin registration + two IPC commands (check_for_update, install_update).

**Major components:**
1. **commands/providers/** -- Provider enum, per-provider request/response handling (5 modules)
2. **commands/ai.rs** -- Refactored stream orchestration accepting provider parameter
3. **commands/keychain.rs** -- Parameterized per-provider key storage
4. **terminal/wsl.rs** -- WSL detection via process tree + wsl.exe subprocess context
5. **commands/updater.rs** -- Update check/install IPC commands
6. **StepProviderSelect.tsx** -- New onboarding step for provider selection

### Critical Pitfalls

1. **Scattered xAI hardcoding (CRITICAL)** -- 15+ direct xAI references across Rust and TypeScript. Missing one during refactor silently sends API keys to wrong endpoints. Prevention: grep audit of ALL xAI/grok/x.ai references before writing any provider code.

2. **SSE format divergence (CRITICAL)** -- Anthropic uses content_block_delta events (not choices[0].delta.content), different done signal (message_stop, not [DONE]), and requires system prompt as top-level field (not in messages array). Google Gemini also differs. Prevention: per-provider extract_token() and is_done() functions, not a shared parser.

3. **Updater signing key must exist before first release (CRITICAL)** -- If the first updater-enabled release ships without the Ed25519 key, those users can NEVER auto-update. Prevention: generate keypair, add to CI secrets, and test full update cycle BEFORE shipping.

4. **WSL process tree invisible to Win32 (CRITICAL)** -- CreateToolhelp32Snapshot cannot see Linux processes inside WSL. wsl.exe appears as leaf node. Prevention: detect wsl.exe in tree, then use wsl.exe -e subprocess for CWD/shell type.

5. **Keychain migration for existing users (MODERATE)** -- Upgrading from v0.2.4 must preserve the existing xAI API key under the new naming scheme. Prevention: check for legacy "xai_api_key" entry on first launch, migrate to new format, set default provider to xAI.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Provider Abstraction Layer (Rust Backend)
**Rationale:** All multi-provider work depends on this. The xAI-hardcoded architecture must be replaced before any new providers can be added safely. This is the foundation -- get it wrong and API keys leak.
**Delivers:** Provider enum, per-provider modules for all 5 providers, refactored stream_ai_response with provider parameter, parameterized keychain, keychain migration from v0.2.4, settings schema migration.
**Addresses:** Provider selector storage, per-provider API keys, streaming for all providers, model listing per provider, API key validation per provider, error message templating.
**Avoids:** Pitfall 1 (scattered xAI refs), Pitfall 2 (SSE format divergence), Pitfall 6 (system prompt format), Pitfall 7 (keychain migration), Pitfall 12 (settings schema).

### Phase 2: Multi-Provider Frontend
**Rationale:** Depends on Phase 1 IPC signatures being stable. Frontend changes are isolated to React/Zustand and do not affect backend correctness.
**Delivers:** Provider selector in onboarding and settings, dynamic API key input labels, per-provider model dropdown, provider-aware error messages, model display labels.
**Addresses:** Provider-aware onboarding, provider selector in settings, model-specific display labels, OpenRouter as meta-provider differentiator.
**Avoids:** Pitfall 1 (frontend xAI references in store types and component labels).

### Phase 3: WSL Terminal Context
**Rationale:** Independent of provider work. Windows-only, affects fewer users. Highest implementation uncertainty -- needs real WSL device testing. Can start in parallel with Phase 2 since it touches different code paths (terminal/ vs commands/ and components/).
**Delivers:** WSL session detection, Linux CWD and shell type via wsl.exe subprocess, WSL-aware system prompts (Linux commands), WSL-aware destructive command patterns, distro badge.
**Addresses:** All WSL table stakes features.
**Avoids:** Pitfall 4 (invisible process tree), Pitfall 9 (path translation), Pitfall 13 (multi-distro detection).

### Phase 4: Auto-Updater
**Rationale:** Infrastructure feature, not user-facing AI functionality. Requires CI/CD pipeline changes, signing key management, and end-to-end testing of the update-and-restart flow. Best done last when other features are stable. The signing key MUST be generated before the first release with the updater enabled.
**Delivers:** Update check on launch, tray/settings notification, download and install flow, CI/CD pipeline with latest.json generation and signing.
**Addresses:** All auto-updater table stakes features, silent background checks differentiator.
**Avoids:** Pitfall 3 (signing key), Pitfall 5 (endpoint 404), Pitfall 10 (UI blocking).

### Phase Ordering Rationale

- **Phase 1 before Phase 2:** Frontend provider UI depends on Rust IPC signatures being finalized. Building backend first means frontend work does not require rework.
- **Phase 3 parallel with Phase 2:** WSL detection (terminal/) and provider frontend (components/, store/) touch completely different code. No dependency between them.
- **Phase 4 last:** Auto-updater affects the release pipeline itself. Easier to validate when the feature set is frozen. The signing key requirement means this phase has a hard prerequisite (key generation) that should be done early, but the implementation can be last.
- **All 3 features are independent at the architecture level.** Provider abstraction, WSL detection, and auto-updater do not interact. The only shared concern is that Phase 1 must land first because it changes the IPC contract that Phase 2 consumes.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Provider Implementations):** Anthropic SSE event format and Google Gemini OpenAI-compat endpoint need verification against current API docs. Training data confidence is MEDIUM.
- **Phase 3 (WSL Context):** wsl.exe subprocess timing, multi-distro detection, and VS Code Remote-WSL window title parsing need real-device validation. Cannot be fully designed from docs alone.
- **Phase 4 (Auto-Updater CI/CD):** The latest.json format and tauri-plugin-updater v2 config schema should be verified against current Tauri docs. macOS build flow (custom DMG vs Tauri .app.tar.gz) needs resolution.

Phases with standard patterns (skip research-phase):
- **Phase 2 (Frontend Multi-Provider):** Standard React/Zustand state management. Well-documented patterns, no novel integration.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Minimal additions (1 crate, 1 npm package). Existing stack verified against codebase. |
| Features | MEDIUM-HIGH | Feature set clear. API format details for Anthropic/Gemini from training data, not live docs. |
| Architecture | HIGH | Provider enum pattern well-suited. Code structure mapped against actual codebase files and line numbers. |
| Pitfalls | HIGH | Pitfalls derived from codebase analysis (specific files, line numbers). Recovery costs assessed realistically. |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Anthropic model list:** No models endpoint; hardcoded list will drift. Need a process for updating model names on new Anthropic releases.
- **Google Gemini OpenAI-compat endpoint:** Training data says it exists at `/v1beta/openai/`, but this is a newer feature that may have quirks. Verify during Phase 1 implementation.
- **tauri-plugin-updater v2 exact config schema:** Verify latest.json format and tauri.conf.json plugin config against current Tauri v2 documentation before Phase 4.
- **WSL subprocess latency:** 200-500ms per wsl.exe -e call is an estimate. Batch shell type + CWD into single command and measure actual latency on real hardware.
- **macOS updater build format:** Current build uses custom DMG script. Need to decide whether to produce .app.tar.gz for updater alongside DMG for manual downloads.
- **OpenRouter model filtering:** Returns 200+ models. Need a filtering/grouping strategy (chat-only, by provider, by tier) before implementation.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: ai.rs, xai.rs, keychain.rs, process.rs, detect_windows.rs, store/index.ts, AccountTab.tsx, OnboardingWizard, release.yml -- direct file reading with line numbers
- WSL architecture and wsl.exe CLI -- well-established Microsoft documentation
- xAI API format -- already implemented and working in codebase

### Secondary (MEDIUM confidence)
- OpenAI Chat Completions API format -- stable, well-documented (training data)
- Anthropic Messages API streaming format -- well-known but exact model names may have changed
- tauri-plugin-updater v2 -- official Tauri plugin, API surface stable but exact config unverified
- OpenRouter API compatibility -- known OpenAI-compatible, header requirements from training data

### Tertiary (needs validation)
- Google Gemini OpenAI-compatible endpoint -- launched 2024, may have evolved
- Tauri update signing (tauri signer generate) -- command and env var names from training data
- WSL subprocess timing (200-500ms estimate) -- needs real-device measurement

---
*Research completed: 2026-03-08*
*Ready for roadmap: yes*
