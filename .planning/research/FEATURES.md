# Feature Landscape: Multi-Provider AI, WSL Context, Auto-Updater

**Domain:** Multi-provider AI support, WSL terminal context, and auto-updater for existing Tauri v2 overlay app
**Milestone:** v0.2.6
**Researched:** 2026-03-08
**Confidence:** MEDIUM (training data only -- WebSearch/WebFetch unavailable; API formats well-known but exact current versions unverified)

---

## Scope

This research covers features needed for v0.2.6 only: multi-provider AI (OpenAI, Anthropic, Google Gemini, xAI/Grok, OpenRouter), WSL terminal context on Windows, and auto-updater. Existing features (single-provider xAI, native terminal context, destructive command detection, per-window history, etc.) are already built.

---

## Table Stakes

Features users expect when these capabilities are advertised. Missing = product feels incomplete.

### Multi-Provider AI Support

| Feature | Why Expected | Complexity | Dependencies on Existing | Notes |
|---------|--------------|------------|--------------------------|-------|
| Provider selector in settings | Users need to choose their preferred AI provider | Low | Refactor `AccountTab.tsx` and onboarding | Currently hardcoded to xAI with single API key input |
| Per-provider API key storage | Each provider has its own API key | Low | Extend `keychain.rs` account naming | Currently stores as `xai_api_key`. Change to `{provider}_api_key` pattern in keyring::Entry |
| Per-provider model listing | Users expect to see available models for their chosen provider | Med | Refactor `xai.rs` model fetching | OpenAI/xAI: `/v1/models` endpoint. Anthropic: hardcode list (no models endpoint). Gemini: `/v1beta/models`. OpenRouter: `/v1/models` |
| Streaming responses from all providers | Non-streaming would be a regression from current xAI experience | Med | Refactor `ai.rs` SSE parsing loop | OpenAI/xAI/OpenRouter share identical SSE format. Anthropic uses different event types. Gemini uses different structure |
| API key validation per provider | Current app validates by fetching models; must work per-provider | Low | Extend `validate_and_fetch_models` | Same pattern, different URLs and auth headers |
| Provider-aware onboarding | First-run must let user pick provider AND enter key | Low | Add step to `OnboardingWizard.tsx` | Insert provider selection step before `StepApiKey` |
| Provider-aware error messages | "Check your xAI API key" is wrong for OpenAI users | Low | Template error strings with provider name | Currently hardcoded in `ai.rs` match arms (401, 429, etc.) |
| Model-specific display labels | Users need human-readable model names, not raw IDs | Low | Extend `model_label()` per provider | Currently maps grok-3/4 IDs to "Fast", "Balanced", etc. Need similar for GPT-4o, Claude Sonnet, Gemini Pro |

### WSL Terminal Context

| Feature | Why Expected | Complexity | Dependencies on Existing | Notes |
|---------|--------------|------------|--------------------------|-------|
| Detect WSL shell sessions | Users in WSL expect CMD+K to recognize their Linux shell | High | Extend `detect_windows.rs` process tree walker | Win32 process tree sees `wsl.exe` but cannot see Linux processes beneath it |
| Read WSL CWD | Users expect CWD context like native Windows terminals | High | New WSL interop module | Must shell out to `wsl.exe -e pwd` since NtQueryInformationProcess cannot read WSL process memory |
| Read WSL visible terminal output | Users expect output context like native terminals | Low | Existing UIA reader likely works already | UIA reads rendered terminal pane text regardless of whether WSL or native produced it |
| WSL shell type detection | System prompt needs correct shell type (bash vs zsh vs fish) | Med | WSL interop commands | `wsl.exe -e echo $SHELL` or `wsl.exe -e basename $SHELL` |
| WSL-aware system prompts | Generate Linux commands when user is in WSL, not Windows commands | Low | Extend system prompt selection in `ai.rs` | Route to Linux/macOS-style prompt instead of Windows prompt when WSL detected |
| WSL-aware destructive patterns | Flag Linux destructive commands in WSL sessions | Low | Existing SAFE-02 Linux patterns already built | Route WSL sessions through Linux pattern set instead of Windows set |

### Auto-Updater

| Feature | Why Expected | Complexity | Dependencies on Existing | Notes |
|---------|--------------|------------|--------------------------|-------|
| Check for updates on launch | Standard desktop app behavior | Low | Add `tauri-plugin-updater` to Cargo.toml and tauri.conf.json | Tauri v2 has first-party updater plugin |
| Show update notification | Users need to know an update exists | Low | New UI element (tray menu item or small banner) | Subtle notification, not a blocking modal |
| Download and install update | One-click update flow | Med | Plugin handles download + apply. Signing keys needed | macOS: replaces .app bundle. Windows: runs NSIS installer |
| Update endpoint (GitHub Releases) | Plugin needs a URL to check | Med | CI/CD pipeline changes | Generate `latest.json` with platform download URLs. Plugin supports GitHub Releases natively |
| Update signing keypair | Tauri verifies update authenticity | Med | Generate keypair, store pubkey in config, privkey in CI secrets | Separate from macOS code signing or Windows Authenticode |

---

## Differentiators

Features that set CMD+K apart. Not expected by default, but high value.

| Feature | Value Proposition | Complexity | Dependencies | Notes |
|---------|-------------------|------------|--------------|-------|
| OpenRouter as meta-provider | One API key accesses 100+ models from all providers | Low | OpenRouter uses OpenAI-compatible API | Users who do not want to manage multiple keys get access to Claude, GPT, Gemini, Llama, Mistral, etc. through one key. Killer UX simplification |
| Provider switching without losing history | Switch from OpenAI to Anthropic mid-session, conversation history preserved | Low | History is already provider-agnostic (plain user/assistant turns) | Store provider used per history entry for transparency, but do not require provider match to use history |
| Automatic WSL distro detection | Show "Ubuntu (WSL)" vs "Debian (WSL)" in context badge | Low | Registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Lxss` | Meaningful for multi-distro users; nice touch in the shell type badge |
| WSL path translation | Translate `/mnt/c/Users/...` to `C:\Users\...` for display (and vice versa) | Low | String manipulation on WSL CWD output | Helps users understand where they are in both namespaces |
| Silent background update checks | No UI unless update is available; no nagging | Low | Timer-based check (every 24h after launch) | Users hate update nags. Check silently, surface once, let dismiss |
| Semantic model grouping | Group models by capability tier ("Fast", "Balanced", "Most Capable") across providers | Low | Extend `model_label()` with per-provider tier mapping | Users do not know which model is "best" -- tiers help them choose |

---

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Auto-select "best" provider | Users have strong preferences (cost, privacy, speed, ideology). Choosing for them is presumptuous | Let user explicitly pick during onboarding. Default to nothing -- force a choice |
| Proxy/relay server for API calls | Adds infrastructure cost, latency, privacy concerns, single point of failure, and an ongoing maintenance burden | Direct client-to-provider API calls. Keys stored locally in platform keychain only |
| Provider-specific prompt customization UI | Complexity explosion. Users should not manage 5 different system prompts | Single system prompt set that works well across all providers. Tune temperature/params internally per provider |
| Forced auto-update | Breaks user trust. Enterprise users may need version pinning | Always prompt, never force. "Update available" with dismiss option. Never auto-restart without consent |
| Update channel selector (stable/beta/nightly) | Premature for current user base. Adds CI/CD complexity and support burden | Ship one stable channel. Add beta channel only when there is real demand |
| Multi-provider simultaneous queries | "Ask all providers and show best result" sounds cool but wastes API credits, adds latency, and complicates the UX | Single provider per query. User picks one. If unsatisfied, they can switch providers and retry |
| Custom API endpoint URLs | Power user feature that adds UI complexity and support burden. Opens up support requests for self-hosted models | Support only official endpoints plus OpenRouter. Users with custom endpoints can use OpenRouter or LiteLLM externally |
| Fallback provider on error | If primary returns 5xx, auto-try secondary provider. Sounds helpful but confusing (different model, different output style, different billing) | Show clear error with "try again" option. Let user manually switch providers if their primary is down |
| WSL file system browsing | Out of scope for a command overlay. Feature creep | Provide CWD context and let the AI know the file structure via the prompt |
| Full WSL process tree walking | Walking the Linux process tree from Windows requires WSL interop commands and is slow | Detect WSL session, get shell type and CWD via `wsl.exe -e` commands. Do not try to replicate the full macOS/Windows process tree walk inside WSL |

---

## Provider API Surface Analysis

Understanding API differences drives the provider abstraction design.

### OpenAI-Compatible Providers (identical SSE format)

**xAI (Grok):** `https://api.x.ai/v1/chat/completions` -- already implemented
- Auth: `Authorization: Bearer {key}`
- SSE chunk: `choices[0].delta.content`
- Stream end: `[DONE]` sentinel

**OpenAI:** `https://api.openai.com/v1/chat/completions`
- Auth: `Authorization: Bearer {key}`
- SSE chunk: `choices[0].delta.content` (identical to xAI)
- Stream end: `[DONE]` sentinel
- Models endpoint: `GET /v1/models`
- Key format: starts with `sk-`

**OpenRouter:** `https://openrouter.ai/api/v1/chat/completions`
- Auth: `Authorization: Bearer {key}`
- Extra headers: `HTTP-Referer: https://cmd-k.app`, `X-Title: CMD+K`
- SSE chunk: `choices[0].delta.content` (identical to xAI/OpenAI)
- Stream end: `[DONE]` sentinel
- Models endpoint: `GET /api/v1/models` (returns 100+ models)
- Key format: starts with `sk-or-`

These three share the same request body shape, SSE format, and done sentinel. The existing `stream_ai_response` logic in `ai.rs` works with only a URL, auth header, and optional extra headers changed.

**Confidence:** HIGH for OpenAI (well-established, stable API). MEDIUM for OpenRouter (known OpenAI-compatible but exact current behavior unverified).

### Non-OpenAI Providers (different formats)

**Anthropic:** `https://api.anthropic.com/v1/messages`
- Auth: `x-api-key: {key}` (NOT `Authorization: Bearer`)
- Required headers: `anthropic-version: 2023-06-01` (or later)
- Request body differences:
  - `max_tokens` is REQUIRED (OpenAI defaults to model max)
  - `system` is a top-level field, NOT a message with `role: "system"`
  - `messages` array has `role: "user"` / `role: "assistant"` only
- SSE event types: `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, `message_delta`, `message_stop`
- Token in: `content_block_delta` event, `delta.text` field
- No `[DONE]` sentinel -- stream ends with `message_stop` event type
- No models listing endpoint -- hardcode list: `claude-sonnet-4-20250514`, `claude-haiku-3.5-20241022`, etc.

**Confidence:** MEDIUM (well-known API, but exact current model names and version header should be verified)

**Google Gemini:** `https://generativelanguage.googleapis.com/v1beta/models/{model}:streamGenerateContent`
- Auth: API key as query parameter `?key={key}` (NOT header-based)
- Request body differences:
  - `contents` array with `parts` (NOT `messages` with `content`)
  - Each content entry: `{ "role": "user"|"model", "parts": [{ "text": "..." }] }`
  - System instruction: separate `systemInstruction` field with `parts` array
- Streaming: SSE but response is JSON array, tokens in `candidates[0].content.parts[0].text`
- Models endpoint: `GET /v1beta/models?key={key}`
- Model names: `gemini-2.0-flash`, `gemini-2.5-pro`, etc.

**Confidence:** MEDIUM (API structure known but v1beta evolves; exact current model names and streaming format details should be verified)

### Provider Abstraction Design Implication

The natural split is:
1. **OpenAI-compatible adapter** -- handles xAI, OpenAI, OpenRouter (URL + headers differ, everything else identical)
2. **Anthropic adapter** -- custom request building (system field, max_tokens) and SSE parsing (content_block_delta)
3. **Gemini adapter** -- custom request building (contents/parts) and response parsing (candidates array)

Each adapter implements a common trait:
- `fn base_url(&self) -> &str`
- `fn build_headers(&self, api_key: &str) -> Vec<(String, String)>`
- `fn build_request_body(&self, model: &str, system_prompt: &str, messages: Vec<ChatMessage>, temperature: f32) -> serde_json::Value`
- `fn parse_sse_chunk(&self, data: &str) -> Option<String>` (extracts token text from SSE data)
- `fn is_stream_done(&self, data: &str) -> bool`
- `fn keychain_account(&self) -> &str`

The `stream_ai_response` function becomes provider-generic: resolve provider from settings, get adapter, build request, stream SSE, parse chunks via adapter.

---

## WSL Terminal Context Analysis

### How WSL Processes Appear to Windows

When a user opens WSL in Windows Terminal:
1. Windows Terminal spawns `wsl.exe` (or `wsl.exe -d Ubuntu`)
2. `wsl.exe` bridges into the WSL2 lightweight VM
3. Linux processes (bash, zsh) run inside the VM -- invisible to Win32 `CreateToolhelp32Snapshot`

The existing process tree walker in `detect_windows.rs` will see `wsl.exe` as a child of `WindowsTerminal.exe` but cannot see the Linux shell underneath.

### Detection Strategy

1. **Detect WSL session:** Process tree walker encounters `wsl.exe` as the "shell" process instead of `powershell.exe`/`cmd.exe`
   - Add `wsl.exe` to `KNOWN_SHELL_EXES` in `detect_windows.rs`
   - When the found shell is `wsl.exe`, flag the session as WSL
2. **Read WSL shell type:** Spawn `wsl.exe -e echo $SHELL` (returns `/bin/bash` or `/bin/zsh`)
   - Use `std::process::Command` with timeout
   - Parse output to extract shell name (basename)
3. **Read WSL CWD:** Spawn `wsl.exe -e pwd`
   - Returns Linux path (e.g., `/home/user/project`)
   - Optionally translate `/mnt/c/...` to `C:\...` for display
4. **Read WSL visible output:** UIA reader already reads the Windows Terminal pane text
   - This should work for WSL sessions since UIA reads what is rendered, not what process produced it
   - Needs validation on real hardware

### WSL Hosting Environments

| Host | How WSL Appears | Detection Method |
|------|----------------|------------------|
| Windows Terminal | `WindowsTerminal.exe` -> `wsl.exe` | Process tree: `wsl.exe` child of WT |
| PowerShell/CMD launching `wsl` | `conhost.exe` -> `wsl.exe` | Process tree: `wsl.exe` child of conhost |
| VS Code Remote-WSL | `Code.exe` integrated terminal runs inside WSL | Window title contains `[WSL: DistroName]` pattern |
| Cursor Remote-WSL | Same as VS Code | Window title contains `[WSL: DistroName]` pattern |
| Standalone `wsl.exe` | `wsl.exe` as direct process | Process tree: `wsl.exe` with no terminal parent |

### Edge Cases and Risks

- **WSL not installed:** `wsl.exe` may not exist. Commands must handle `Command::new("wsl.exe")` failing gracefully
- **WSL distro not running:** `wsl.exe -e pwd` may take 2-5 seconds to start a stopped distro. Need timeout (500ms or so) to avoid blocking overlay
- **Multiple WSL distros:** Default distro used unless `-d` flag detected in process args. Can query `wsl.exe -l -v` for installed distros
- **WSL1 vs WSL2:** WSL1 has different process visibility (processes may be partially visible to Win32). Detection should work the same way since we are using `wsl.exe -e` for context
- **Performance:** Each `wsl.exe -e` command spawns a new WSL process. Batch the shell type and CWD queries into a single command: `wsl.exe -e sh -c 'echo $SHELL && pwd'`

**Confidence:** MEDIUM (WSL interop well-documented, but exact process tree behavior and edge cases need real-device validation)

---

## Auto-Updater Analysis

### Tauri v2 Updater Plugin (`tauri-plugin-updater`)

Tauri v2 provides `tauri-plugin-updater` as the official update mechanism.

**How it works:**
1. Add plugin to `Cargo.toml`: `tauri-plugin-updater = "2"`
2. Configure in `tauri.conf.json` under `plugins.updater`:
   - `endpoints`: array of URLs to check (supports GitHub Releases)
   - `pubkey`: public key for update signature verification
3. App calls `check()` on launch -- hits endpoint, compares versions
4. If update available, call `download_and_install()` or `download()` + `install()`
5. macOS: downloads `.tar.gz` of `.app` bundle, replaces current, restarts
6. Windows: downloads `.nsis.zip`, extracts NSIS installer, runs silently

**GitHub Releases endpoint format:**
The plugin supports GitHub Releases directly. URL pattern: `https://github.com/{owner}/{repo}/releases/latest/download/latest.json`

The `latest.json` file (generated by CI/CD) has the structure:
```json
{
  "version": "0.2.6",
  "notes": "Multi-provider AI support, WSL context, auto-updater",
  "pub_date": "2026-03-15T00:00:00Z",
  "platforms": {
    "darwin-x86_64": { "url": "https://.../.app.tar.gz", "signature": "..." },
    "darwin-aarch64": { "url": "https://.../.app.tar.gz", "signature": "..." },
    "windows-x86_64": { "url": "https://.../.nsis.zip", "signature": "..." }
  }
}
```

**Required CI/CD changes:**
1. Generate updater signing keypair: `tauri signer generate -w ~/.tauri/myapp.key`
2. Store private key as GitHub Secret (`TAURI_SIGNING_PRIVATE_KEY`)
3. Store key password as GitHub Secret (`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`)
4. Add pubkey to `tauri.conf.json` under `plugins.updater.pubkey`
5. CI/CD builds produce signed update bundles (`.tar.gz` for macOS, `.nsis.zip` for Windows)
6. CI/CD generates `latest.json` and uploads it as a release artifact
7. Tauri v2 `tauri build` with `TAURI_SIGNING_PRIVATE_KEY` env var auto-signs update artifacts

**Confidence:** MEDIUM (plugin exists and is well-documented in Tauri v2, but exact v2 config schema and `latest.json` format should be verified against current docs)

### UX Pattern for Background Utility App

Best practice for update flow in a system tray / hotkey-driven app:
1. **Check on launch** (silent, no UI, no blocking)
2. **If update available:** Add "Update Available (vX.Y.Z)" item to tray menu
3. **User clicks tray item:** Show small confirmation ("Download vX.Y.Z? Release notes: ...")
4. **Download with progress** in tray tooltip or small notification
5. **"Restart to update"** or **"Update on next launch"** -- let user choose timing
6. **Never interrupt** the user's workflow with modal dialogs or forced restarts
7. **Dismiss means dismiss** -- do not re-nag until next app launch

---

## Feature Dependencies

```
Provider Abstraction Layer (trait + adapters)
  |-> Per-provider API key storage (keychain account naming)
  |-> Per-provider model listing (adapter-specific API calls)
  |-> Per-provider streaming SSE parsing (adapter parse_sse_chunk)
  |-> Provider selector in settings UI (list of known providers)
  |-> Provider-aware onboarding (provider choice step + key input)
  |-> Provider-aware error messages (provider name in strings)

OpenAI-compatible adapter
  |-> OpenAI provider (URL swap only)
  |-> OpenRouter provider (URL + extra headers)
  |-> xAI provider (existing, refactored into adapter)

Non-OpenAI adapters
  |-> Anthropic adapter (custom request/response)
  |-> Gemini adapter (custom request/response)

WSL Detection (in detect_windows.rs process tree)
  |-> WSL CWD reading (wsl.exe -e commands)
  |-> WSL shell type detection (wsl.exe -e commands)
  |-> WSL-aware system prompts (route to Linux prompt)
  |-> WSL-aware destructive patterns (use Linux pattern set)
  |-> WSL distro detection (optional, registry query)

Auto-Updater Plugin Setup
  |-> Signing keypair generation
  |-> CI/CD pipeline updates (latest.json generation)
  |-> Update check on launch
  |-> Tray menu update notification
  |-> Download and install flow
```

**Cross-cutting dependency:** Provider abstraction is entirely independent of WSL and auto-updater. All three features can be developed in parallel, but provider support delivers the most user value and should be prioritized.

---

## MVP Recommendation

### Phase 1: Provider Abstraction + OpenAI (highest value, lowest risk)

1. **Provider abstraction trait in Rust** -- refactor `ai.rs` to use a trait-based dispatch. Extract xAI logic into an adapter
2. **OpenAI provider** -- nearly zero additional effort since it shares xAI's SSE format. URL + auth header change only
3. **Provider selector in settings** -- dropdown above API key input in `AccountTab.tsx`. Store selected provider in `tauri-plugin-store`
4. **Per-provider keychain storage** -- change keyring::Entry account from `xai_api_key` to `{provider_id}_api_key`
5. **Provider-aware onboarding** -- add provider selection step to `OnboardingWizard.tsx`

**Rationale:** OpenAI is the most requested provider. The abstraction layer validates the design with minimal risk since OpenAI and xAI share identical API format. Every subsequent provider becomes easier.

### Phase 2: Anthropic + Gemini + OpenRouter

6. **Anthropic adapter** -- different auth header (`x-api-key`), different request body (`system` field, `max_tokens` required), different SSE events (`content_block_delta` instead of `choices[0].delta.content`)
7. **Gemini adapter** -- different auth (query param), different body (`contents`/`parts`), different response structure
8. **OpenRouter adapter** -- OpenAI-compatible format, just needs extra `HTTP-Referer` and `X-Title` headers. Adds 100+ model access
9. **Model display labels per provider** -- extend `model_label()` with tier mappings for each provider

**Rationale:** These require more SSE parsing work. Anthropic is the most different from OpenAI format. OpenRouter is trivial but adds massive model coverage. Gemini has its own quirks (query param auth, contents/parts body).

### Phase 3: WSL Terminal Context

10. **WSL session detection** -- add `wsl.exe` to known shell exes, flag WSL sessions in process tree walker
11. **WSL CWD + shell type** -- `wsl.exe -e sh -c 'echo $SHELL && pwd'` with timeout
12. **WSL-aware system prompts** -- route to Linux prompt template when WSL detected
13. **WSL-aware destructive patterns** -- use Linux pattern set (SAFE-02) for WSL sessions
14. **WSL distro badge** -- show "bash (WSL: Ubuntu)" in context badge

**Rationale:** WSL is Windows-only, affects fewer users than multi-provider, and has the highest implementation uncertainty (needs real WSL device testing). Should come after provider work is stable.

### Phase 4: Auto-Updater

15. **Plugin setup** -- add `tauri-plugin-updater`, generate signing keypair, configure endpoints in `tauri.conf.json`
16. **CI/CD updates** -- generate `latest.json`, sign update artifacts, upload to GitHub Releases
17. **Update check on launch** -- silent background check, no blocking UI
18. **Tray notification** -- add "Update Available" item to tray menu. Small confirmation on click. Download + restart flow

**Rationale:** Auto-updater is infrastructure, not user-facing AI functionality. Requires CI/CD pipeline changes, signing key management, and careful testing of the update-and-restart flow on both platforms. Best done last when other features are stable and the release pipeline is proven.

### Defer to Future

- **Fallback provider on error**: Nice-to-have but confusing UX (different model behavior). Show clear error instead.
- **Update channel selector**: No demand yet. Single stable channel.
- **Custom API endpoints**: Use OpenRouter for custom/self-hosted model access.
- **Provider-specific prompt tuning UI**: Tune internally, do not expose to users.

---

## Sources

- Codebase analysis: `src-tauri/src/commands/ai.rs` (current xAI streaming implementation), `src-tauri/src/commands/xai.rs` (model listing), `src-tauri/src/terminal/detect_windows.rs` (Windows process tree), `src-tauri/Cargo.toml` (dependencies), `src-tauri/tauri.conf.json` (app config), `src/store/index.ts` (frontend state), `src/components/Settings/AccountTab.tsx` (current API key UI), `src/components/Onboarding/` (onboarding wizard)
- Project context: `.planning/PROJECT.md` (milestone goals and constraints)
- OpenAI API format: training data (HIGH confidence -- well-established, stable API since 2023)
- Anthropic Messages API format: training data (MEDIUM confidence -- API structure well-known, exact current model list and version header may have changed)
- Google Gemini API format: training data (MEDIUM confidence -- still in v1beta, may have evolved since training cutoff)
- OpenRouter API compatibility: training data (MEDIUM confidence -- known OpenAI-compatible, exact header requirements should be verified)
- Tauri v2 updater plugin: training data (MEDIUM confidence -- plugin exists in Tauri v2 ecosystem, exact config schema and latest.json format should be verified against current docs)
- WSL process model: training data (MEDIUM confidence -- well-documented by Microsoft, but process tree behavior and `wsl.exe -e` performance need on-device validation)

---

*Feature research for: v0.2.6 Multi-Provider, WSL & Auto-Update*
*Researched: 2026-03-08*
