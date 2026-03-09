# Domain Pitfalls: Multi-Provider AI, WSL Terminal Context, and Auto-Updater

**Domain:** Adding multi-provider AI support, WSL terminal context, and auto-updater to existing Tauri v2 app (CMD+K)
**Researched:** 2026-03-08 (v0.2.6 milestone)
**Confidence:** HIGH (verified against codebase architecture, API documentation patterns, and prior Windows pitfalls research)

> This document supersedes the v0.2.1 PITFALLS.md. Prior pitfalls for the Windows port (UIPI, SmartScreen, ConPTY process tree, etc.) remain valid. This document covers ONLY pitfalls specific to the v0.2.6 features: multi-provider AI, WSL terminal context, and tauri-plugin-updater.

---

## Critical Pitfalls

Mistakes that cause rewrites, data loss, or broken update paths.

### Pitfall 1: Hardcoded xAI Constants Scattered Across Multiple Files

**What goes wrong:**
The current codebase has xAI-specific constants, types, and logic hardcoded in at least 4 locations:
- `ai.rs`: Hardcoded `https://api.x.ai/v1/chat/completions` endpoint (line 266), `SERVICE`/`ACCOUNT` keychain constants pointing to `xai_api_key` (lines 7-8)
- `xai.rs`: Hardcoded `https://api.x.ai/v1/models` endpoint (line 105), xAI-specific model filtering (`grok-*` names, lines 23-35), xAI-specific hardcoded fallback models (lines 77-83)
- `keychain.rs`: Single `ACCOUNT = "xai_api_key"` constant -- stores exactly one API key
- `store/index.ts`: Single `apiKeyStatus`, single `selectedModel`, single `availableModels` array -- all assume one provider

Developers attempt to add a provider abstraction layer but miss one of these hardcoded references. The app compiles, appears to work with the default provider, but silently uses the wrong API key or endpoint when the user switches providers. The most insidious case: `ai.rs` reads the API key from keychain using the hardcoded `xai_api_key` account name regardless of which provider is selected, so switching to OpenAI still sends the xAI key to OpenAI's endpoint.

**Why it happens:**
The xAI-only architecture was a deliberate scope decision for v1 (PROJECT.md: "xAI only for v1, provider architecture should allow easy addition later"). But the "should allow easy addition" never materialized as an actual abstraction layer. The xAI references are deeply embedded, not isolated behind an interface.

**Consequences:**
- User's xAI API key sent to OpenAI/Anthropic endpoint (security leak)
- Wrong model names sent to wrong provider (API errors)
- User switches provider but onboarding flow still validates against xAI
- Keychain stores only one key, overwriting the previous provider's key when user configures a new one

**Prevention:**
1. Audit ALL xAI references before writing any provider code. Search for: `x.ai`, `xai`, `grok`, `xai_api_key`, `api.x.ai`. The current count is 15+ direct references across Rust and TypeScript
2. Create a `Provider` trait/enum in Rust FIRST, before touching any existing code:
   ```rust
   enum Provider { OpenAI, Anthropic, Google, XAI, OpenRouter }
   ```
3. Change keychain account to include provider: `"openai_api_key"`, `"anthropic_api_key"`, etc. -- or use a single `"api_keys"` JSON blob
4. The frontend store must change from `apiKeyStatus: string` to `apiKeyStatuses: Record<Provider, Status>` and from `selectedModel: string` to `selectedProvider: Provider` + `selectedModel: string`
5. The `stream_ai_response` command must accept a `provider` parameter, not just `model` -- the model name alone is ambiguous across providers (e.g., "gpt-4o" vs "claude-3.5-sonnet" but what about custom model names on OpenRouter?)

**Detection:**
- Test switching providers in settings and immediately making a query
- Log the actual HTTP request URL and Authorization header in debug mode
- Verify different API keys are stored for different providers

**Phase to address:**
Phase 1 (Provider Abstraction Layer) -- must be the FIRST phase. All other multi-provider work depends on this being correct.

---

### Pitfall 2: SSE Streaming Format Differs Between Providers

**What goes wrong:**
The current `ai.rs` parses SSE (Server-Sent Events) with a specific JSON structure: `chunk["choices"][0]["delta"]["content"]` (line 308). This is the OpenAI-compatible format used by xAI. Developers assume all providers use the same SSE format and reuse the existing parser. But:

- **OpenAI**: `choices[0].delta.content` -- matches current code
- **Anthropic**: Uses a completely different streaming format with `content_block_delta` events and `delta.text` field, NOT `choices[0].delta.content`. Anthropic also sends `message_start`, `content_block_start`, `content_block_delta`, `content_block_stop`, and `message_stop` event types
- **Google Gemini**: Uses `candidates[0].content.parts[0].text` in streaming responses, NOT `choices[0].delta.content`
- **OpenRouter**: Proxies to upstream providers and normalizes to OpenAI format, BUT error responses use OpenRouter's own format, and rate limiting headers are different
- **xAI**: OpenAI-compatible format (current code works)

The app streams partial tokens correctly for xAI and OpenAI but silently produces empty responses for Anthropic and Google because the JSON path `choices[0].delta.content` does not exist in their SSE payloads.

**Why it happens:**
The OpenAI chat completions API has become a de facto standard, and many developers assume all providers follow it. Anthropic and Google explicitly chose different formats. Even providers claiming "OpenAI compatibility" (like OpenRouter) have edge cases in error handling, rate limiting, and streaming termination signals.

**Consequences:**
- Anthropic queries return empty responses (no tokens extracted)
- Google Gemini queries return empty responses
- OpenRouter works for GPT models but errors are misreported
- Users think the provider is broken, not the parsing

**Prevention:**
1. Build a `StreamParser` trait with provider-specific implementations:
   ```rust
   trait StreamParser {
       fn parse_token(&self, event_data: &str) -> Option<String>;
       fn is_done(&self, event_data: &str) -> bool;
   }
   ```
2. Anthropic requires parsing multiple event types (`content_block_delta` for tokens, `message_stop` for completion), not just looking for `[DONE]`
3. Google Gemini uses a different SSE sentinel and response structure
4. Test each provider's streaming with a real API key before considering it "done"
5. The existing 10-second hard timeout (line 295 in `ai.rs`) may be too short for slower providers -- Anthropic Claude and Google Gemini can take 15-30 seconds for complex prompts. Make timeout provider-configurable or increase to 30 seconds

**Detection:**
- Test each provider with a simple prompt and verify tokens appear
- Log raw SSE event data before parsing to diagnose empty responses
- Verify `[DONE]` sentinel handling per provider (Anthropic uses `message_stop` event type, not `[DONE]`)

**Phase to address:**
Phase 2 (Provider Implementations) -- immediately after the abstraction layer. Each provider must be tested individually.

---

### Pitfall 3: Auto-Updater Signing Key Is Different From Code Signing Certificate

**What goes wrong:**
Tauri's auto-updater (`tauri-plugin-updater`) requires a separate Ed25519 signing key pair for update verification. This is NOT the same as the Apple Developer ID certificate (used for macOS code signing) or the Windows code signing certificate (not yet purchased). Developers configure the Apple certificate in CI, assume auto-update signing is handled, and ship a release. The first update check succeeds (downloads the new version), but signature verification fails because the update binary was never signed with the Tauri updater key. The app shows "Update available" but installation fails silently or with a cryptic error.

Worse: if the first release ships WITHOUT the updater key configured, there is NO way to push an auto-update to those users. They must manually download the new version because the old version has no updater key to verify against.

**Why it happens:**
Tauri v2 has THREE separate signing concerns:
1. **macOS code signing**: Apple Developer ID certificate (already configured in CI via `APPLE_CERTIFICATE_BASE64`)
2. **Windows code signing**: Authenticode certificate (not yet purchased, conditional in CI)
3. **Tauri updater signing**: Ed25519 key pair (`TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) -- this is what `tauri-plugin-updater` verifies

The naming is confusing. "Signing" in the Tauri docs refers to updater signing, not code signing. The updater public key goes in `tauri.conf.json` under `plugins.updater.pubkey`, and the private key is used during `tauri build` via environment variables.

**Consequences:**
- First release with updater has no signing key: all users on that version can NEVER auto-update
- Key generated but not stored securely: key loss means future updates break for ALL existing users
- Key generated but not added to CI: local builds work, CI builds produce unsigned updates
- Public key in `tauri.conf.json` but private key not in GitHub Secrets: builds succeed but updates fail signature check

**Prevention:**
1. Generate the Ed25519 key pair BEFORE building the first updater-enabled release: `tauri signer generate -w ~/.tauri/cmd-k.key`
2. Store the private key in a SECURE location (password manager, not git). Loss of this key is catastrophic -- you cannot issue updates to existing users
3. Add `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to GitHub Secrets BEFORE the first CI build with the updater
4. Add the PUBLIC key to `tauri.conf.json` under `plugins.updater.pubkey`
5. Test the full update cycle locally before shipping: build v0.2.5 with updater, build v0.2.6, verify v0.2.5 can detect, download, verify, and install v0.2.6
6. The CI workflow (`release.yml`) must be updated to pass these environment variables to the `tauri build` step

**Detection:**
- Check `tauri.conf.json` for `plugins.updater.pubkey` field
- Check CI workflow for `TAURI_SIGNING_PRIVATE_KEY` environment variable
- Run `tauri build` locally and check if `.sig` files are generated alongside the installer

**Phase to address:**
Phase 1 of auto-updater work -- key generation and CI configuration must happen before the first updater-enabled release ships.

---

### Pitfall 4: WSL Process Tree Is Invisible to Win32 APIs

**What goes wrong:**
The current Windows terminal context detection walks the Win32 process tree via `CreateToolhelp32Snapshot` to find shell PIDs descended from the terminal app (see `find_shell_by_ancestry` in `process.rs`). This works for native Windows shells (PowerShell, CMD, Git Bash). For WSL, the process tree crosses a namespace boundary: `wsl.exe` is a Win32 process, but the actual shell (`bash`, `zsh`) runs inside the WSL Linux namespace. Win32 process APIs cannot see WSL-internal processes. `CreateToolhelp32Snapshot` returns `wsl.exe` as the leaf process -- it cannot enumerate `bash` or `zsh` running inside the Linux namespace.

Current code handles this partially: `detect_windows.rs` lists `bash.exe` in `KNOWN_SHELL_EXES` (line 45), but this is Git Bash's `bash.exe`, not WSL's bash. WSL does not spawn a `bash.exe` visible to Win32. The process tree walk finds `wsl.exe` and stops.

**Why it happens:**
WSL runs a real Linux kernel in a lightweight VM (WSL2) or a translation layer (WSL1). Linux processes inside WSL have their own PID namespace. Win32 APIs like `CreateToolhelp32Snapshot`, `OpenProcess`, `ReadProcessMemory`, and `NtQueryInformationProcess` do not cross the WSL boundary. The only Win32 process visible is `wsl.exe` (or `wslhost.exe` for WSL2).

**Consequences:**
- Shell type detection fails: app cannot determine if user is running bash, zsh, or fish inside WSL
- CWD detection via PEB reading fails: `wsl.exe`'s CWD is its Windows launch directory, not the user's current directory inside the WSL filesystem
- Running process detection fails: cannot see if user is running `vim`, `node`, etc. inside WSL
- Window key computation uses `wsl.exe` PID, which is shared across all WSL sessions -- history/context bleeds between WSL terminals

**Prevention:**
1. Detect WSL sessions by checking if the shell process is `wsl.exe` or `wslhost.exe` in the process tree
2. For CWD: execute `wsl.exe -e pwd` (or `wsl.exe --exec pwd`) to get the current working directory inside WSL. This spawns a quick subprocess that runs inside the WSL namespace and returns the Linux path
3. For shell type: execute `wsl.exe -e basename "$SHELL"` or `wsl.exe -e ps -o comm= -p $$` to get the running shell name
4. For path translation: use `wsl.exe -e wslpath -w "$(pwd)"` to convert Linux paths to Windows paths (e.g., `/home/user/project` -> `\\wsl$\Ubuntu\home\user\project`), or use `wslpath -u` for the reverse
5. Handle the latency cost: `wsl.exe -e` subprocess calls take 200-500ms (WSL must translate the call into the Linux namespace). This is slower than the native Win32 process inspection. Consider caching results with a 2-second TTL
6. For VS Code Remote-WSL and Cursor with WSL: the IDE process tree will show `Code.exe` -> `wslhost.exe` -> (invisible WSL processes). Detect Remote-WSL by checking VS Code's window title (contains "[WSL: distroname]") or by checking for `wslhost.exe` in the process tree

**Detection:**
- Open Windows Terminal with a WSL tab, trigger CMD+K, check if shell_type is detected
- Check CWD -- if it shows `C:\Windows\System32` or the Windows home directory instead of the WSL directory, PEB reading is being used instead of WSL interop
- Verify different WSL distributions (Ubuntu, Debian) produce correct context

**Phase to address:**
Phase 2 (WSL Terminal Context) -- core WSL support. Must handle the namespace boundary explicitly, not attempt to reuse Win32 process tree walking.

---

### Pitfall 5: Auto-Updater Update Endpoint URL Becomes a Single Point of Failure

**What goes wrong:**
Tauri's auto-updater checks a URL endpoint for update metadata (a JSON file containing version, download URL, signature, and release notes). Most Tauri apps use GitHub Releases as the update endpoint. The updater is configured with an `endpoints` URL in `tauri.conf.json` that points to a JSON file served from GitHub (either a static `latest.json` in the release assets, or a dynamic endpoint). If the endpoint URL is wrong, returns 404, or GitHub is down, the app silently fails to check for updates. Users are stuck on old versions with no feedback.

More critically: the endpoint URL format differs between Tauri v1 and v2. Tauri v2 uses a different JSON schema for the update manifest. If you follow a Tauri v1 tutorial (which dominate search results), the update check will fail silently.

**Why it happens:**
The update manifest must be generated and uploaded to a specific URL pattern. GitHub Releases do not automatically create a Tauri-compatible update manifest. You need either:
- A GitHub Action that generates `latest.json` and uploads it to the release
- A static server/CDN that serves the manifest
- The `tauri-action` GitHub Action which handles this automatically

Developers configure the updater plugin, point it at their GitHub releases URL, and assume the manifest exists. It does not -- GitHub Releases serves HTML or the raw binary, not the JSON manifest the updater expects.

**Consequences:**
- App silently never finds updates (endpoint returns HTML or 404)
- Users manually check for updates, see "You're up to date" because the check failed (not because there's no update)
- If using a custom server, server downtime means no updates for anyone

**Prevention:**
1. Use the `tauri-plugin-updater` with GitHub Releases: configure the endpoint as `https://github.com/user/repo/releases/latest/download/latest.json`
2. The CI workflow must generate and upload `latest.json` alongside the installer artifacts. This file must contain: `version`, `notes`, `pub_date`, and per-platform entries with `url` and `signature`
3. Generate platform-specific JSON: the manifest needs `platforms` keys like `darwin-aarch64`, `darwin-x86_64`, `windows-x86_64` mapping to the correct download URLs and `.sig` signature files
4. The `.sig` files are generated automatically by `tauri build` when `TAURI_SIGNING_PRIVATE_KEY` is set -- ensure they are uploaded to the release
5. Test the update flow end-to-end: build v0.2.5, publish to GitHub, build v0.2.6, publish to GitHub, run v0.2.5 and verify it detects and installs v0.2.6
6. Add error logging for update check failures -- the default behavior is silent failure

**Detection:**
- After publishing a release, manually `curl` the endpoint URL and verify it returns valid JSON
- Check that `.sig` files exist alongside the installer in GitHub Releases
- Check app logs for update check errors (network errors, parse errors, signature verification errors)

**Phase to address:**
Phase 3 (Auto-Updater Implementation) -- CI workflow changes and endpoint configuration are part of the updater setup.

---

### Pitfall 6: System Prompt Becomes Provider-Specific Due to Model Capabilities

**What goes wrong:**
The current system prompts in `ai.rs` are tailored for xAI/Grok models (lines 13-54). They use a specific instruction style that works well with Grok. When switching providers, the same system prompt produces different quality results:

- **Anthropic Claude**: Tends to follow "output ONLY the exact command" instructions more literally but may refuse to generate destructive commands even when asked. Claude also has a `system` parameter separate from the messages array (not a `"role": "system"` message)
- **OpenAI GPT**: Follows system prompts well but may add markdown formatting despite being told not to. Models like GPT-4o may add explanatory text
- **Google Gemini**: The `system_instruction` is a separate field in the API, not part of the messages array. Gemini may also wrap responses in markdown code fences despite instructions
- **OpenRouter**: Passes system prompts through to the underlying model, but different models behind OpenRouter behave differently

The most dangerous case: Anthropic's API structure requires the system prompt as a top-level `system` parameter, NOT as a message with `"role": "system"`. If you send `{"role": "system", "content": "..."}` in the messages array to the Anthropic API, it returns an error. The current code builds messages with `"role": "system"` (line 230-232), which works for OpenAI-compatible APIs but breaks for Anthropic.

**Why it happens:**
Each LLM provider has different API conventions, different model behaviors, and different prompt engineering best practices. The "one prompt fits all" approach silently degrades quality rather than failing loudly.

**Consequences:**
- Anthropic API calls fail with 400 error because system prompt is in the wrong location
- Google Gemini returns markdown-wrapped commands that break when pasted into terminal
- Different models produce inconsistent output quality with the same prompt
- Temperature setting (0.1) may be too low for some models, causing repetitive outputs

**Prevention:**
1. Build the system prompt as part of the provider abstraction, not as a shared constant. Each provider implementation constructs its own request body with the system prompt in the correct location
2. For Anthropic: move system prompt to the top-level `system` field in the request body
3. For Google Gemini: use `system_instruction` field, not messages
4. Consider provider-specific prompt tweaks (e.g., adding "Do not use markdown code fences" for models that tend to wrap output)
5. The `temperature` parameter should be provider-configurable -- some models work better at 0.0, others at 0.2
6. Test the ACTUAL output of each provider with 10 common terminal tasks and verify commands are clean, unformatted, and pasteable

**Detection:**
- Anthropic queries fail with HTTP 400
- Google Gemini responses wrapped in triple backticks
- Commands pasted into terminal include markdown formatting (backticks, language tags)

**Phase to address:**
Phase 2 (Provider Implementations) -- system prompt handling is part of each provider's request builder.

---

## Moderate Pitfalls

### Pitfall 7: Keychain Migration -- Existing Users Lose Their API Key

**What goes wrong:**
The current keychain stores a single API key under `SERVICE = "com.lakshmanturlapati.cmd-k"` and `ACCOUNT = "xai_api_key"`. When refactoring to multi-provider, the account naming scheme changes (e.g., `"provider_openai_api_key"` or a JSON blob). If the new code does not check for the legacy `"xai_api_key"` account, existing users who upgrade from v0.2.4 to v0.2.6 will find their API key gone. They must re-enter it, and worse, the old key remains orphaned in the keychain/credential manager.

**Prevention:**
1. On first launch of v0.2.6, check for the legacy `"xai_api_key"` entry in the keychain
2. If found, migrate it to the new provider-specific entry (e.g., `"xai_api_key"` -> new xAI provider storage)
3. Set the user's default provider to xAI if a legacy key is found (preserving their existing workflow)
4. Delete the legacy entry after successful migration to avoid confusion
5. Run migration logic BEFORE showing onboarding -- the user should never see "no API key" if they had one before

**Detection:**
- Upgrade a test build from v0.2.4 to v0.2.6 and verify the API key persists
- Check that the old keychain entry is cleaned up after migration

**Phase to address:**
Phase 1 (Provider Abstraction Layer) -- migration must be in place before the first release that changes keychain structure.

---

### Pitfall 8: Model Listing Endpoints Differ Across Providers

**What goes wrong:**
The current `xai.rs` fetches models from `GET /v1/models` and falls back to a hardcoded list. Each provider has a different model listing approach:

- **OpenAI**: `GET https://api.openai.com/v1/models` -- works, returns all models including fine-tunes and embeddings that must be filtered
- **Anthropic**: NO model listing endpoint. You must hardcode the available models. The Anthropic API does not have a `/v1/models` equivalent
- **Google Gemini**: `GET https://generativelanguage.googleapis.com/v1beta/models?key=API_KEY` -- API key goes in URL parameter, not Authorization header
- **xAI**: `GET https://api.x.ai/v1/models` -- current approach works
- **OpenRouter**: `GET https://openrouter.ai/api/v1/models` -- returns 200+ models from all providers, needs heavy filtering

The developer builds a generic "fetch models" function and expects all providers to support it. Anthropic fails because there is no endpoint. Google fails because the auth mechanism is different (key in URL, not Bearer token). OpenRouter returns hundreds of irrelevant models.

**Prevention:**
1. Make model listing a per-provider method, not a generic API call. Some providers return lists, others require hardcoded lists
2. Anthropic: hardcode `claude-sonnet-4-20250514`, `claude-3.5-haiku-20241022`, etc. Update on new releases. This is what the Anthropic docs recommend
3. Google Gemini: handle the different auth pattern (API key in query parameter) in the provider implementation
4. OpenRouter: filter to only show chat-capable models, group by provider, respect model pricing information
5. Cache model lists to avoid repeated API calls (models change rarely)

**Phase to address:**
Phase 2 (Provider Implementations) -- each provider's model listing is unique.

---

### Pitfall 9: WSL Path Translation Breaks Terminal Context Display

**What goes wrong:**
WSL uses Linux-style paths (`/home/user/project`) while Windows uses Windows-style paths (`C:\Users\user\project`). The WSL filesystem is mounted at `\\wsl$\DistroName\` (or `\\wsl.localhost\DistroName\` on newer Windows). When the overlay displays CWD from a WSL terminal, it must decide: show the Linux path or the Windows path?

If it shows the Linux path (`/home/user/project`), the AI receives a Linux path and generates Linux commands -- correct. But the user may be confused because they see a path that does not exist in Windows Explorer.

If it shows the Windows path (`\\wsl$\Ubuntu\home\user\project`), the AI receives a Windows-style path and may generate Windows commands even though the user is in a WSL bash shell -- incorrect.

Additionally, the system prompt currently says "You are a terminal command generator for Windows" when running on Windows (line 23-30 of `ai.rs`). But WSL sessions should use the macOS/Linux system prompt because the user is running Linux commands.

**Prevention:**
1. When WSL is detected, use the Linux path as CWD in the AI context -- the user wants Linux commands
2. Switch the system prompt to the Linux variant for WSL sessions, not the Windows variant
3. Set shell_type to the WSL shell (`bash`, `zsh`) not `wsl` -- the user's actual shell matters for command generation
4. Include "WSL" context in the system prompt so the AI knows it can reference Windows files via `/mnt/c/` paths
5. Display the Linux path in the overlay badge but consider showing `WSL: /path` for clarity

**Phase to address:**
Phase 2 (WSL Terminal Context) -- path handling is integral to WSL context detection.

---

### Pitfall 10: Auto-Updater Blocks UI on Download or Triggers Unwanted Restarts

**What goes wrong:**
The updater downloads the update binary (10-30MB for a DMG/NSIS installer) and installs it. If the download runs on the main thread or blocks the UI, the overlay becomes unresponsive during the download. If the updater automatically restarts the app after installation, the user loses their current workflow without warning.

Additionally, on macOS, replacing the running app binary requires the app to quit. If the user is in the middle of a terminal task and the updater restarts the app, they lose their overlay session, turn history, and in-progress context.

**Prevention:**
1. Download updates in the background on a separate async task. Show a non-blocking notification in the system tray or overlay: "Update available -- will install on next launch"
2. Do NOT auto-restart. Let the user choose when to restart. The update installs on the next app launch
3. Use the `on_before_restart` hook to save any transient state (though current architecture is session-scoped, so this is less critical)
4. Show download progress if the user opens settings -- do not hide the download entirely
5. Handle download interruption gracefully: if the user loses internet mid-download, do not leave a corrupted partial download. Clean up temp files
6. Milestone spec already says "check on launch, prompt user, install on next launch" -- follow this pattern, do not deviate to auto-install

**Phase to address:**
Phase 3 (Auto-Updater Implementation) -- UX for update flow.

---

### Pitfall 11: OpenRouter Requires Additional Headers and Has Provider-Specific Quirks

**What goes wrong:**
OpenRouter is a meta-provider that routes requests to upstream providers (OpenAI, Anthropic, Google, etc.). It mostly follows the OpenAI API format, but has specific requirements:
- Requires `HTTP-Referer` header (or `X-Title` header) for leaderboard ranking. Without it, requests may be deprioritized
- Rate limiting is per-user-key, not per-model. OpenRouter may return 429 even when the upstream provider has capacity
- Error responses include `provider_name` field indicating which upstream provider failed
- Some models on OpenRouter have different context windows or capabilities than their direct API equivalents
- OpenRouter returns `model` in the response indicating which actual model served the request (it may route to a different model than requested if the requested model is overloaded)

Developers treat OpenRouter as "just another OpenAI-compatible endpoint" and miss these quirks.

**Prevention:**
1. Set `HTTP-Referer` header to the app's homepage URL and `X-Title` to "CMD+K" in OpenRouter requests
2. Parse the `model` field in the response to log which model actually served the request (useful for debugging)
3. Handle OpenRouter-specific error codes (e.g., 402 = insufficient credits, different from standard 401/429)
4. Consider that OpenRouter keys have credit balances -- validate by checking credits, not just model listing
5. Document to users that OpenRouter is a proxy and may have higher latency than direct API access

**Phase to address:**
Phase 2 (Provider Implementations) -- OpenRouter-specific adapter.

---

## Minor Pitfalls

### Pitfall 12: Tauri Plugin Store Settings Schema Changes Break Existing User Preferences

**What goes wrong:**
The current `tauri-plugin-store` stores settings like `selectedModel`, `turnLimit`, `autoPasteEnabled`, etc. Adding multi-provider support requires new fields: `selectedProvider`, per-provider `selectedModel`, etc. If the schema changes without migration, existing users' store files contain the old schema. The app reads `selectedModel` and gets `"grok-3"` but the new code expects `selectedProvider` to be set alongside it. The provider defaults to the first option (possibly OpenAI), and the user's `"grok-3"` model name is sent to OpenAI, causing an error.

**Prevention:**
1. Add a `schemaVersion` field to the store
2. On launch, check schema version and migrate old settings to new format
3. Map legacy `selectedModel` (xAI model names) to `selectedProvider: "xai"` + `selectedModel: "grok-3"`
4. Default new fields gracefully: if `selectedProvider` is missing, infer from the existing model name

**Phase to address:**
Phase 1 (Provider Abstraction Layer) -- settings migration runs alongside keychain migration.

---

### Pitfall 13: WSL Distribution Detection -- Multiple Distros and Default Distro

**What goes wrong:**
Users can have multiple WSL distributions installed (Ubuntu, Debian, Arch, etc.). `wsl.exe` launches the default distribution unless `wsl -d DistroName` is used. When running `wsl.exe -e pwd` to get CWD, it runs in the DEFAULT distribution, which may not be the one the user has open in their terminal. If the user has Ubuntu and Debian installed, with Debian as default, but is working in an Ubuntu terminal, the CWD query goes to Debian and returns the wrong path.

**Prevention:**
1. When detecting WSL, try to identify the specific distribution from the Windows Terminal tab title or `wsl.exe` command-line arguments
2. The `wsl.exe` process for a specific tab has command-line arguments like `wsl.exe -d Ubuntu` -- read these via `QueryFullProcessImageNameW` + command-line extraction to determine which distro
3. Use `wsl.exe -d <distro> -e pwd` instead of just `wsl.exe -e pwd`
4. Fall back to the default distribution if the specific distro cannot be determined

**Phase to address:**
Phase 2 (WSL Terminal Context) -- distribution detection is part of WSL context accuracy.

---

### Pitfall 14: Rate Limiting Behavior Varies Drastically Between Providers

**What goes wrong:**
The current code handles rate limiting with a simple message: "Rate limit exceeded. Please wait a moment and try again." (line 283 in `ai.rs`). Different providers have different rate limiting behaviors:
- **OpenAI**: Returns HTTP 429 with `Retry-After` header
- **Anthropic**: Returns HTTP 429 with `retry-after` header (lowercase) and includes rate limit details in response body
- **Google Gemini**: Returns HTTP 429 but uses a different quota system based on RPM (requests per minute) and TPM (tokens per minute)
- **xAI**: Returns HTTP 429 (current handling)
- **OpenRouter**: Returns HTTP 429 but rate limits are per-key and per-model, with credits-based throttling

**Prevention:**
1. Parse the `Retry-After` header (case-insensitive) and show the user how long to wait
2. For Anthropic, parse the rate limit response body to show specific limits
3. Consider implementing automatic retry with exponential backoff for transient rate limits
4. Display different error messages based on the rate limit type (RPM vs TPM vs credits)

**Phase to address:**
Phase 2 (Provider Implementations) -- error handling is part of each provider's implementation.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Provider Abstraction Layer | Pitfall 1: missed xAI references cause key leaks | Grep audit of ALL xAI references before writing abstraction code |
| Provider Abstraction Layer | Pitfall 7: existing users lose API key on upgrade | Keychain migration logic runs before onboarding |
| Provider Abstraction Layer | Pitfall 12: settings schema breaks | Schema version + migration in store |
| Provider Implementations | Pitfall 2: SSE parsing differs per provider | Per-provider stream parser, not shared parser |
| Provider Implementations | Pitfall 6: system prompt format differs | System prompt construction inside provider, not shared constant |
| Provider Implementations | Pitfall 8: model listing differs | Per-provider model listing, hardcoded for Anthropic |
| WSL Terminal Context | Pitfall 4: process tree invisible across namespace | Use `wsl.exe -e` subprocess for WSL context, not Win32 APIs |
| WSL Terminal Context | Pitfall 9: path translation confuses AI | Use Linux paths for CWD, Linux system prompt for WSL sessions |
| WSL Terminal Context | Pitfall 13: wrong WSL distribution queried | Detect distro from process arguments before querying |
| Auto-Updater Setup | Pitfall 3: updater signing key not configured | Generate Ed25519 key, add to CI, BEFORE first updater release |
| Auto-Updater Setup | Pitfall 5: update endpoint 404 | CI generates and uploads `latest.json` to GitHub Release |
| Auto-Updater UX | Pitfall 10: UI blocked or unwanted restart | Background download, user-initiated install on next launch |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces for v0.2.6.

- [ ] **Provider switching:** Switching provider in settings works -- but verify the API key for the NEW provider is read from keychain, not the old provider's key
- [ ] **Anthropic streaming:** Tokens stream in overlay -- but verify the system prompt is in the `system` parameter, not in the messages array
- [ ] **Google Gemini auth:** API key is validated -- but verify it is sent as a URL parameter, not a Bearer token (Google supports both, but URL param is the basic auth method)
- [ ] **OpenRouter models:** Model list loads -- but verify it is filtered to chat models only, not showing embedding/image models
- [ ] **WSL CWD:** CWD displays in overlay -- but verify it is the Linux path from inside WSL, not `wsl.exe`'s Windows CWD
- [ ] **WSL system prompt:** AI generates commands -- but verify they are Linux commands, not Windows commands (the system prompt must switch to Linux mode for WSL)
- [ ] **Auto-updater signing:** Build succeeds -- but verify `.sig` files are generated alongside installers in CI output
- [ ] **Update manifest:** `latest.json` is uploaded to GitHub Release -- but verify it contains both `darwin-aarch64` and `windows-x86_64` platform entries with correct download URLs
- [ ] **Keychain migration:** New v0.2.6 settings work -- but verify upgrading from v0.2.4 preserves the existing xAI API key
- [ ] **Onboarding flow:** New provider selection step works -- but verify existing users who upgrade skip provider selection and go straight to the overlay (their settings are migrated)
- [ ] **Streaming timeout:** All providers stream responses -- but verify 10-second timeout is increased for slower providers (Anthropic Claude can take 15+ seconds for complex prompts)
- [ ] **Error messages:** Provider errors show in overlay -- but verify error messages reference the correct provider name, not hardcoded "xAI" or "Grok"

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| xAI key sent to wrong provider (Pitfall 1) | LOW | Fix keychain lookup to use provider-scoped account names. No data loss, key still in keychain under old name |
| Empty responses from Anthropic/Gemini (Pitfall 2) | LOW | Add provider-specific SSE parser. Existing xAI/OpenAI path unaffected |
| First release ships without updater signing key (Pitfall 3) | CRITICAL | Cannot push auto-updates to those users. Must ask them to manually download. Consider shipping a "bridge" release that adds the updater before adding multi-provider |
| WSL CWD returns Windows path (Pitfall 4) | MEDIUM | Switch to `wsl.exe -e pwd` approach. Requires adding subprocess call and WSL detection logic |
| Update endpoint returns 404 (Pitfall 5) | LOW | Fix CI to upload `latest.json`. Existing users will get the update on next check after fix |
| Anthropic API rejects system prompt in messages (Pitfall 6) | LOW | Move system prompt to top-level `system` parameter. Isolated change in Anthropic provider |
| Existing users lose API key on upgrade (Pitfall 7) | MEDIUM | Ship a hotfix that checks for legacy keychain entry and migrates. Users must wait for hotfix (manual download since they just broke auto-update) |
| Settings schema breaks (Pitfall 12) | MEDIUM | Ship a patch that handles both old and new schema. May require clearing settings for affected users |

## Sources

### Codebase Analysis
- `src-tauri/src/commands/ai.rs` -- Current streaming implementation with hardcoded xAI endpoints and SSE parsing
- `src-tauri/src/commands/xai.rs` -- xAI-specific model listing with hardcoded fallbacks
- `src-tauri/src/commands/keychain.rs` -- Single-provider keychain storage with hardcoded `xai_api_key` account
- `src-tauri/src/terminal/process.rs` -- Process tree walking with Win32 API (CreateToolhelp32Snapshot) and macOS libproc
- `src-tauri/src/terminal/detect_windows.rs` -- Windows terminal detection with known exe lists
- `src-tauri/src/terminal/mod.rs` -- Full context detection orchestration including UIA text reading
- `src/store/index.ts` -- Frontend state with single-provider assumptions throughout
- `.github/workflows/release.yml` -- Current CI pipeline without updater signing

### Provider API Documentation (from training data -- MEDIUM confidence)
- OpenAI Chat Completions API: `choices[0].delta.content` streaming format, `Authorization: Bearer` header
- Anthropic Messages API: `content_block_delta` event type, `system` top-level parameter (not in messages), `x-api-key` header (not Bearer token)
- Google Gemini API: `candidates[0].content.parts[0].text` response format, API key in URL parameter, `system_instruction` field
- OpenRouter API: OpenAI-compatible with `HTTP-Referer` requirement, 402 credits error code
- xAI API: OpenAI-compatible format (current code works)

### Tauri Plugin Documentation (from training data -- MEDIUM confidence)
- `tauri-plugin-updater`: Ed25519 signing separate from code signing, `plugins.updater.pubkey` in `tauri.conf.json`, `TAURI_SIGNING_PRIVATE_KEY` env var
- Update manifest format: per-platform entries with `url` and `signature` fields

### WSL Architecture (from training data -- HIGH confidence)
- WSL process isolation: Linux processes invisible to Win32 APIs
- `wsl.exe -e` for cross-namespace command execution
- `wslpath` for path translation between Linux and Windows
- `\\wsl$\DistroName\` mount point for WSL filesystem access from Windows

---
*Pitfalls research for: CMD+K v0.2.6 Multi-Provider AI, WSL Terminal Context, and Auto-Updater*
*Researched: 2026-03-08*
