# Phase 2: Settings & Configuration - Research

**Researched:** 2026-02-21
**Domain:** macOS Keychain secure storage, xAI API validation, Tauri v2 HTTP plugin, macOS Accessibility permissions, settings UI patterns
**Confidence:** MEDIUM-HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Onboarding flow:**
- Step-by-step wizard, not single page
- Step order: Accessibility permission -> API key entry -> Model selection -> Done
- Onboarding appears inside the overlay itself (no separate window)
- If user closes mid-setup, resume where they left off next time (track progress)

**Settings UI access:**
- Two entry points: menu bar tray icon click AND typing /settings in the overlay
- Both routes open the overlay in "settings mode" (reuses the overlay UI)
- Settings organized with tabbed sections (e.g., Account, Model, Preferences)
- Changes auto-save immediately -- no save button needed

**API key experience:**
- Masked input field by default (dots/asterisks) with eye icon toggle to reveal
- Validate immediately on paste/entry (no separate validate button)
- Inline status indicator: green checkmark for valid, red X for invalid
- No helper text or "Get API key" links -- keep it clean
- Single API key only (no multi-key support in v1)
- When stored in Keychain, show last 4 characters by default with reveal toggle for full key
- If key becomes invalid later, show error inline in the overlay with link to settings (don't auto-open settings)

**Model selection:**
- Dropdown list for model selection
- Fetch models dynamically from xAI API (requires valid key first)
- Smart default: pre-select the best general-purpose Grok model
- Dropdown shows model name + short tag (e.g., "Fast", "Most capable")
- Model dropdown disabled/greyed out until API key is validated
- User's model choice persists across app restarts
- Mini usage dashboard in settings panel showing estimated cost

### Claude's Discretion
- Exact onboarding step animations/transitions
- Tab naming and icon choices for settings sections
- Loading states during API key validation
- How to calculate/display estimated cost in usage dashboard
- Error state styling and copy

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| SETT-01 | User can store and validate their xAI API key | xAI API validation via GET /v1/models or chat completions; keyring crate for Keychain storage |
| SETT-02 | User can select which Grok model to use | xAI GET /v1/models endpoint (OpenAI-compatible); dynamic fetch after key validation |
| SETT-03 | API keys stored securely in macOS Keychain (not plaintext config file) | keyring crate v3.6.x stores in macOS User Keychain natively; two plugin options available |
| SETT-04 | First-run onboarding guides user through Accessibility permissions and API key setup | tauri-plugin-macos-permissions provides checkAccessibilityPermission() boolean; onboarding state in tauri-plugin-store |
</phase_requirements>

---

## Summary

Phase 2 builds settings and onboarding on top of the existing Tauri v2 + React 19 + Zustand + tauri-plugin-store foundation from Phase 1. The core technical challenge is threefold: (1) securely storing the xAI API key in macOS Keychain rather than the plaintext settings.json used for the hotkey config, (2) validating the API key against xAI's live API and then fetching available models dynamically, and (3) orchestrating a step-by-step onboarding wizard that tracks progress across app restarts inside the existing overlay UI.

The Keychain storage decision eliminates the most complex option (rolling custom macOS Keychain FFI) because the `keyring` Rust crate (v3.6.x) provides a battle-tested, cross-platform wrapper that targets macOS User Keychain by default. Two community Tauri plugins wrap this crate (`tauri-plugin-keyring` and `tauri-plugin-keychain`), but the simplest and most maintainable approach is to add `keyring` directly to Cargo.toml and expose it through custom Tauri `#[tauri::command]` functions -- avoiding a community plugin dependency for a thin wrapper. The xAI API uses GET /v1/models (OpenAI-compatible endpoint) to both validate the key (a 401 response means invalid) and fetch the model list in one call. HTTP calls must route through a custom Rust Tauri command using reqwest (already available via tauri-plugin-http) because the xAI bearer token must not be visible in the frontend JS bundle.

The settings UI reuses the existing overlay panel component and Zustand store pattern. The tabbed layout can be built with shadcn/ui Tabs (backed by Radix UI, already in the project as `radix-ui` dependency). Onboarding step progress is persisted in `settings.json` via tauri-plugin-store (already integrated). The `tauri-plugin-macos-permissions` plugin provides a clean TypeScript `checkAccessibilityPermission()` -> boolean API for the first onboarding step.

**Primary recommendation:** Use the `keyring` crate directly in Rust (no plugin wrapper), expose `save_api_key`, `get_api_key`, `delete_api_key` as Tauri commands, validate the key via a `validate_api_key` Tauri command that calls GET /v1/models, and gate the model dropdown on validation state in Zustand. Add tauri-plugin-macos-permissions for Accessibility check. Persist onboarding step progress and selected model in settings.json.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| keyring (Rust) | 3.6.x | macOS Keychain secure storage for API key | Widely used, cross-platform, macOS User Keychain by default; no plugin overhead |
| tauri-plugin-macos-permissions | 2.1.x | Check and request Accessibility permission | Only Tauri v2 plugin with checkAccessibilityPermission() boolean API; official-adjacent |
| tauri-plugin-http (reqwest) | 2.x | HTTP calls to xAI API from Rust commands | Already declared in lib.rs; keeps bearer token off frontend |
| tauri-plugin-store | 2.4.2 | Persist onboarding step, model selection | Already installed and working from Phase 1 |
| Zustand | 5.0.11 | Settings/onboarding UI state machine | Already the state management layer; extend existing store |
| shadcn Tabs (Radix UI) | Latest | Tabbed settings panel | `radix-ui` already in package.json; shadcn CLI adds pre-styled component |
| lucide-react | 0.575.0 | Eye icon for reveal toggle, validation indicators | Already installed |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tauri-plugin-keyring | 0.1.x | Alternative: Tauri plugin wrapping keyring crate | Only if direct Rust command approach proves too complex |
| tauri-plugin-keychain | 2.0.2 | Alternative: simpler key/password API | Only if keyring crate direct approach has issues |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| keyring crate direct | tauri-plugin-keychain or tauri-plugin-keyring | Plugins add an extra crate dependency for a thin wrapper; direct approach gives full control and avoids unmaintained community crates |
| tauri-plugin-macos-permissions | AXIsProcessTrustedWithOptions via accessibility-sys | accessibility-sys is low-level FFI; plugin provides safe TypeScript binding with boolean result |
| shadcn Tabs | Custom tab component | Custom tabs require accessibility (keyboard nav, ARIA) handling; shadcn/Radix provides this for free |
| Rust Tauri command for HTTP | @tauri-apps/plugin-http from JS | JS-side fetch would expose API key in renderer memory; Rust command keeps key server-side |

**Installation:**

```bash
# Rust side - add to Cargo.toml
cargo add keyring
cargo add tauri-plugin-macos-permissions

# JS side
pnpm add tauri-plugin-macos-permissions-api
pnpm dlx shadcn@latest add tabs
```

---

## Architecture Patterns

### Recommended Project Structure

```
src/
├── components/
│   ├── Settings/
│   │   ├── SettingsPanel.tsx       # Root tabbed settings component
│   │   ├── AccountTab.tsx          # API key entry + validation status
│   │   ├── ModelTab.tsx            # Model dropdown + usage dashboard
│   │   └── PreferencesTab.tsx      # (Hotkey moved here from HotkeyConfig)
│   ├── Onboarding/
│   │   ├── OnboardingWizard.tsx    # Step orchestrator, reads step from store
│   │   ├── StepAccessibility.tsx   # Step 1: Accessibility permission
│   │   ├── StepApiKey.tsx          # Step 2: API key entry + validation
│   │   ├── StepModelSelect.tsx     # Step 3: Model selection
│   │   └── StepDone.tsx            # Step 4: Done / launch
│   └── Overlay.tsx                 # Existing -- updated to render Settings or Onboarding
├── store/
│   └── index.ts                    # Extended with settings + onboarding state
src-tauri/src/
├── commands/
│   ├── keychain.rs                 # save_api_key, get_api_key, delete_api_key
│   ├── xai.rs                      # validate_api_key, fetch_models
│   ├── hotkey.rs                   # Existing
│   ├── tray.rs                     # Existing (add Settings entry)
│   └── window.rs                   # Existing
```

### Pattern 1: Settings Mode in Overlay (Mode Switching)

**What:** The overlay component switches between three modes: `command` (normal), `onboarding` (step-by-step wizard), `settings` (tabbed panel). The mode is stored in Zustand and determines what the `<Overlay>` renders inside its existing container.

**When to use:** User types `/settings`, clicks menu bar tray, or first run triggers onboarding.

**Example:**
```typescript
// src/store/index.ts - extend existing store
type OverlayMode = "command" | "onboarding" | "settings";

interface OverlayState {
  // ... existing fields ...
  mode: OverlayMode;
  onboardingStep: number;       // 0=accessibility, 1=apikey, 2=model, 3=done
  onboardingComplete: boolean;
  apiKeyStatus: "unknown" | "validating" | "valid" | "invalid";
  selectedModel: string | null;
  availableModels: XaiModel[];

  openSettings: () => void;     // sets mode="settings", shows overlay
  openOnboarding: () => void;   // sets mode="onboarding"
  setOnboardingStep: (step: number) => void;
  setApiKeyStatus: (status: ApiKeyStatus) => void;
  setModels: (models: XaiModel[]) => void;
  setSelectedModel: (model: string) => void;
}
```

### Pattern 2: Rust Tauri Command for Keychain (Security Boundary)

**What:** API key never touches the React/JS layer as plaintext. Frontend invokes Rust commands to read/write/validate. The Rust commands use the `keyring` crate to store and retrieve from macOS Keychain.

**When to use:** Any time the API key needs to be read, written, or used for API calls.

**Example:**
```rust
// src-tauri/src/commands/keychain.rs
use keyring::Entry;

const SERVICE: &str = "com.lakshmanturlapati.cmd-k";
const KEY_ACCOUNT: &str = "xai_api_key";

#[tauri::command]
pub fn save_api_key(key: String) -> Result<(), String> {
    let entry = Entry::new(SERVICE, KEY_ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry.set_password(&key)
        .map_err(|e| format!("Failed to save to Keychain: {}", e))
}

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = Entry::new(SERVICE, KEY_ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    match entry.get_password() {
        Ok(pwd) => Ok(Some(pwd)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("Failed to read from Keychain: {}", e)),
    }
}

#[tauri::command]
pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE, KEY_ACCOUNT)
        .map_err(|e| format!("Keychain entry error: {}", e))?;
    entry.delete_credential()
        .map_err(|e| format!("Failed to delete from Keychain: {}", e))
}
```

### Pattern 3: API Key Validation via xAI GET /v1/models

**What:** Validate the API key AND fetch available models in a single HTTP call. A 200 response means the key is valid and returns the model list. A 401 means invalid key.

**When to use:** Immediately after user pastes or enters an API key (debounced ~800ms).

**Example:**
```rust
// src-tauri/src/commands/xai.rs
use tauri_plugin_http::reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct XaiModel {
    pub id: String,
    pub object: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<XaiModel>,
}

#[tauri::command]
pub async fn validate_and_fetch_models(api_key: String) -> Result<Vec<XaiModel>, String> {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.x.ai/v1/models")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    match response.status().as_u16() {
        200 => {
            let models: ModelsResponse = response.json().await
                .map_err(|e| format!("Parse error: {}", e))?;
            // Filter to text-generation models only (exclude image/video)
            let text_models: Vec<XaiModel> = models.data.into_iter()
                .filter(|m| !m.id.contains("image") && !m.id.contains("video"))
                .collect();
            Ok(text_models)
        },
        401 => Err("invalid_key".to_string()),
        status => Err(format!("API error: {}", status)),
    }
}
```

### Pattern 4: Onboarding Progress Persistence

**What:** Track which onboarding step was last completed. Persisted to settings.json via tauri-plugin-store. On app start, check if onboarding is complete; if not, show onboarding at the persisted step.

**When to use:** Every app launch before the overlay activates.

**Example:**
```typescript
// src/App.tsx - startup check (extend existing loadPersistedHotkey pattern)
useEffect(() => {
  const checkOnboarding = async () => {
    const store = await Store.load("settings.json");
    const onboardingComplete = await store.get<boolean>("onboardingComplete");
    const onboardingStep = await store.get<number>("onboardingStep") ?? 0;
    if (!onboardingComplete) {
      overlayStore.openOnboarding(onboardingStep);
    }
  };
  checkOnboarding();
}, []);
```

### Pattern 5: Deferred Validation with Debounce

**What:** Trigger API validation ~800ms after the user stops typing in the API key field. Show a spinner during validation; show green check or red X on completion.

**When to use:** API key input field onChange handler.

**Example:**
```typescript
// src/components/Onboarding/StepApiKey.tsx
const [inputValue, setInputValue] = useState("");
const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

const handleChange = (val: string) => {
  setInputValue(val);
  if (debounceRef.current) clearTimeout(debounceRef.current);
  if (val.length > 10) {
    debounceRef.current = setTimeout(() => {
      validateKey(val);
    }, 800);
  }
};

const validateKey = async (key: string) => {
  setApiKeyStatus("validating");
  try {
    const models = await invoke<XaiModel[]>("validate_and_fetch_models", { apiKey: key });
    await invoke("save_api_key", { key });
    setModels(models);
    setApiKeyStatus("valid");
  } catch (err) {
    if (err === "invalid_key") setApiKeyStatus("invalid");
    else setApiKeyStatus("error");
  }
};
```

### Pattern 6: Masked Input with Reveal Toggle

**What:** API key input uses `type="password"` by default. An eye icon button toggles `type="text"`. When displaying a stored key (from Keychain), show only last 4 chars with asterisks.

**When to use:** AccountTab in settings, StepApiKey in onboarding.

**Example:**
```typescript
const [revealed, setRevealed] = useState(false);
const maskedKey = storedKey
  ? `${"*".repeat(storedKey.length - 4)}${storedKey.slice(-4)}`
  : "";
// Input uses type={revealed ? "text" : "password"}
// EyeIcon / EyeOffIcon from lucide-react toggles revealed state
```

### Anti-Patterns to Avoid

- **Storing API key in settings.json or Zustand state as plaintext:** settings.json is unencrypted JSON on disk. The API key must only ever pass through Rust Tauri commands and live in macOS Keychain. Never put the full API key in frontend state.
- **Calling xAI API from frontend JavaScript:** The bearer token would be visible in the renderer process memory and network logs. All xAI API calls must go through Rust Tauri commands.
- **Polling for Accessibility permission status:** Check once at onboarding step render; if not granted, show a button to open System Settings, then re-check when the user returns (use app focus event or a manual "Check Again" button).
- **Auto-opening settings on API key error:** The user decision locks this: show inline error in the overlay with a link. Never auto-navigate.
- **Using a save button for settings:** Auto-save on change is the locked decision. Use debounced save or save immediately after each valid change.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| macOS Keychain access | Custom Security.framework FFI via objc crate | keyring crate v3.6.x | keyring handles macOS Keychain API, error types, and fallback keychains; FFI is 200+ lines of unsafe Rust |
| Accessibility permission check | AXIsProcessTrustedWithOptions via raw FFI | tauri-plugin-macos-permissions checkAccessibilityPermission() | Plugin provides safe TypeScript boolean result and handles the system dialog prompt |
| Tabbed UI component | Custom tab state + ARIA roles | shadcn Tabs (Radix UI) | Radix handles keyboard navigation (arrow keys), ARIA roles, focus management; building compliant tabs from scratch is 100+ lines |
| HTTP client in Rust | Hyper or raw TCP | reqwest via tauri-plugin-http | reqwest is already available as a re-export from tauri-plugin-http; adding hyper separately is redundant |
| API key masking display | Custom masking logic | `"*".repeat(n) + key.slice(-4)` pattern | Simple one-liner; no library needed, but pattern must be consistent |
| Debounced validation trigger | Custom timer logic | useRef + setTimeout pattern (see Pattern 5) | Avoids an extra library (lodash debounce); simple enough to hand-roll correctly |

**Key insight:** The security boundary is the critical decision. Every shortcut that puts the API key on the JS side (localStorage, Zustand state, JS fetch) creates a security hole. The keyring + Rust command pattern enforces the boundary at the framework level.

---

## Common Pitfalls

### Pitfall 1: macOS Keychain Prompt on Every Launch

**What goes wrong:** On macOS, accessing the Keychain for the first time from a new app binary (e.g., after a Tauri dev rebuild) triggers an OS-level "allow access" dialog. In development this happens constantly because the binary hash changes on each build.

**Why it happens:** macOS ties Keychain access permissions to the specific binary signature. An unsigned development build gets a different identity each build.

**How to avoid:** In development, expect and dismiss this prompt. For production, the app must be code-signed with a stable Developer ID. The entitlements.plist already has `com.apple.security.automation.apple-events` -- no additional keychain entitlement is needed for the User keychain (only for the app-specific keychain). The prompt will appear once per new binary in dev; in a signed release build it appears only on first use.

**Warning signs:** If the Keychain prompt appears every single time the Tauri command runs (not just on first launch), the binary hash is changing (i.e., unsigned development builds).

### Pitfall 2: xAI /v1/models Not Listed in Core API Reference

**What goes wrong:** The xAI REST API reference page (docs.x.ai/docs/api-reference) does not explicitly document a GET /v1/models endpoint. A developer might conclude the endpoint doesn't exist.

**Why it happens:** xAI positions itself as OpenAI-compatible but their documentation focuses on the chat/completions and responses endpoints. The models endpoint is inherited from OpenAI compatibility.

**How to avoid:** The endpoint `GET https://api.x.ai/v1/models` with `Authorization: Bearer <key>` is confirmed via xAI's OpenAI compatibility claims and community usage. If it doesn't exist or returns 404, fall back to validating with a minimal chat/completions call (low token count, max_tokens=1) and use the hardcoded model list from docs.x.ai/developers/models. Keep the hardcoded list as fallback.

**Warning signs:** GET /v1/models returning 404 means the endpoint is not supported; switch to the completions validation approach.

### Pitfall 3: tauri-plugin-http Requires Capabilities Configuration

**What goes wrong:** Adding the HTTP plugin to lib.rs and calling it from Rust commands works in dev mode but all fetch calls to api.x.ai are blocked in production because the capabilities file doesn't list the allowed URL.

**Why it happens:** Tauri v2 uses a capability-based permission system. The default.json capabilities file must explicitly allow HTTP fetch operations and list permitted domains.

**How to avoid:** Add to `src-tauri/capabilities/default.json`:
```json
{
  "permissions": [
    "http:default",
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://api.x.ai/**" }]
    }
  ]
}
```
Also add `tauri-plugin-http = "2"` to Cargo.toml and `.plugin(tauri_plugin_http::init())` to `lib.rs`.

**Warning signs:** HTTP calls fail with permission errors in production but work in dev. Or calls work from the Rust layer but fail from JS fetch.

### Pitfall 4: Onboarding State Not Reset on Edge Cases

**What goes wrong:** User partially completes onboarding (e.g., grants Accessibility but closes before entering API key). On re-launch, `onboardingComplete: false` correctly resumes onboarding, but the API key status in Zustand is `"unknown"` even though a key might have been saved to Keychain from a previous run.

**Why it happens:** Zustand state is in-memory only; tauri-plugin-store has the `onboardingStep` but not the validation state. On startup, the app must check Keychain for an existing key and validate it before showing the onboarding step.

**How to avoid:** On startup, always call `get_api_key` first. If a key exists, call `validate_and_fetch_models` with it. Set `apiKeyStatus` based on the result before showing onboarding. If valid, skip to the next step after API key.

**Warning signs:** User sees "Enter API key" step even though they already saved one; or key status spinner never resolves.

### Pitfall 5: Settings Tab Not Accessible via Both Entry Points

**What goes wrong:** `/settings` command from the overlay opens the settings tab but the tray icon click does something different (or vice versa). Or the Zustand `openSettings` action is wired to one entry point but not the other.

**Why it happens:** The locked decision requires two entry points (tray icon + `/settings` command). In Phase 1, `openSettings` was wired only to the `open-hotkey-config` Tauri event. Phase 2 must add a separate `open-settings` event from the tray and handle the `/settings` command in `handleSubmit` in App.tsx (already partially stubbed with `openSettings()` call).

**How to avoid:** Audit `src/App.tsx` `handleSubmit` for the existing `/settings` stub -- it already calls `openSettings()`. Wire a new Tauri event `open-settings` from the tray icon "Settings..." menu item to the same `openSettings()` action.

**Warning signs:** Tray icon Settings click doesn't show the settings panel, or `/settings` text command doesn't work.

---

## Code Examples

Verified patterns from official sources and Phase 1 codebase:

### Keychain Read with "Key Not Found" Handling
```rust
// Source: docs.rs/keyring + Phase 2 research
use keyring::{Entry, Error as KeyringError};

#[tauri::command]
pub fn get_api_key() -> Result<Option<String>, String> {
    let entry = Entry::new("com.lakshmanturlapati.cmd-k", "xai_api_key")
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(KeyringError::NoEntry) => Ok(None),  // Key was never saved
        Err(e) => Err(e.to_string()),
    }
}
```

### Accessibility Permission Check (TypeScript)
```typescript
// Source: github.com/ayangweb/tauri-plugin-macos-permissions
import { checkAccessibilityPermission } from "tauri-plugin-macos-permissions-api";

const hasAccess = await checkAccessibilityPermission(); // returns boolean
if (!hasAccess) {
  // Show UI guiding user to System Settings > Privacy & Security > Accessibility
  // Open System Settings URL programmatically:
  // await invoke("open_accessibility_settings");
}
```

### Open Accessibility Settings from Rust
```rust
// Source: jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html
#[tauri::command]
pub fn open_accessibility_settings() {
    let url = "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
    std::process::Command::new("open")
        .arg(url)
        .spawn()
        .ok();
}
```

### tauri-plugin-store for Onboarding Progress
```typescript
// Source: v2.tauri.app/plugin/store/ + Phase 1 pattern (settings.json already used)
import { Store } from "@tauri-apps/plugin-store";

const store = await Store.load("settings.json");
await store.set("onboardingStep", 2);
await store.set("onboardingComplete", false);
await store.save();

// On startup:
const step = await store.get<number>("onboardingStep") ?? 0;
const complete = await store.get<boolean>("onboardingComplete") ?? false;
```

### shadcn Tabs for Settings Panel
```typescript
// Source: ui.shadcn.com/docs/components/radix/tabs
// Install: pnpm dlx shadcn@latest add tabs
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

<Tabs defaultValue="account">
  <TabsList>
    <TabsTrigger value="account">Account</TabsTrigger>
    <TabsTrigger value="model">Model</TabsTrigger>
    <TabsTrigger value="preferences">Preferences</TabsTrigger>
  </TabsList>
  <TabsContent value="account"><AccountTab /></TabsContent>
  <TabsContent value="model"><ModelTab /></TabsContent>
  <TabsContent value="preferences"><PreferencesTab /></TabsContent>
</Tabs>
```

### Cargo.toml Changes for Phase 2
```toml
[dependencies]
# ... existing dependencies ...
keyring = { version = "3", features = ["apple-native"] }
tauri-plugin-http = "2"
tauri-plugin-macos-permissions = "2"
```

### lib.rs Plugin Registration Updates
```rust
// Add to tauri::Builder::default() in lib.rs
.plugin(tauri_plugin_http::init())
.plugin(tauri_plugin_macos_permissions::init())
// Register new commands in invoke_handler:
// save_api_key, get_api_key, delete_api_key,
// validate_and_fetch_models, open_accessibility_settings
```

### Capabilities Update
```json
// src-tauri/capabilities/default.json additions
{
  "permissions": [
    // ... existing permissions ...
    "http:default",
    "macos-permissions:default",
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://api.x.ai/**" }]
    }
  ]
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| electron-store plaintext JSON for API keys | macOS Keychain via keyring crate | Project pivot to Tauri (2026-02-21) | Security: keys encrypted by macOS, not readable from JSON files |
| Plugin wrapper for keyring | Direct keyring crate in Rust commands | Research finding | Simpler: no community plugin dependency; same result via custom Tauri commands |
| Separate settings window | Overlay "settings mode" (reuses existing panel) | User decision (CONTEXT.md) | No window management complexity; same NSPanel handles all modes |
| Static model list hardcoded | Dynamic fetch via GET /v1/models | User decision (CONTEXT.md) | Always shows current models; requires valid key first |

**Deprecated/outdated:**
- electron-store: Was used in the old CLI architecture documented in INTEGRATIONS.md; current architecture is Tauri v2 with tauri-plugin-store + Keychain.
- OpenAI/Anthropic providers: v1 is xAI-only; the old multi-provider architecture in ARCHITECTURE.md is irrelevant to this phase.

---

## Open Questions

1. **Does GET /v1/models exist on api.x.ai?**
   - What we know: xAI claims OpenAI API compatibility; OpenAI has GET /v1/models; xAI is confirmed to have OpenAI-compatible endpoints
   - What's unclear: xAI's API reference page does not explicitly list GET /v1/models; docs focus on chat/completions
   - Recommendation: Attempt GET /v1/models in the implementation. If 404, fall back to validating with a minimal chat/completions call (e.g., max_tokens=1) and use the hardcoded model list from xAI docs. Build the fallback from day one.
   - Hardcoded fallback list: `grok-3`, `grok-3-mini`, `grok-4`, `grok-4-fast`, `grok-code-fast-1`

2. **Which Grok model is the "best general-purpose" default?**
   - What we know: `grok-3` is the full general-purpose model; `grok-3-mini` is lightweight; `grok-4` variants are newest
   - What's unclear: Pricing tier differences and whether grok-4 is gated to paid tiers
   - Recommendation: Default to `grok-3` as the safe general-purpose choice; label it "Balanced" or "Recommended". Allow grok-3-mini as "Fast" and grok-4 as "Most capable".

3. **macOS Keychain prompt behavior in signed vs. unsigned builds**
   - What we know: Unsigned dev builds trigger a Keychain access dialog on first run; signed builds tie to Developer ID
   - What's unclear: Whether the Keychain prompt appears every dev build cycle or just once per binary hash
   - Recommendation: Document in the plan that Keychain prompts are expected in dev. For the human verification step, the reviewer should dismiss any Keychain dialogs as expected behavior.

4. **Usage cost calculation for the mini dashboard**
   - What we know: xAI charges per-token; pricing listed at docs.x.ai/developers/models
   - What's unclear: This is tagged as Claude's Discretion. Simplest approach: show estimated cost as "(estimated)" based on model pricing constants; no request tracking in v1 since the AI call layer isn't built yet
   - Recommendation: In Phase 2, show a placeholder usage dashboard ("No usage recorded yet") with the architecture wired for Phase 4 to populate it. Don't block settings on this.

---

## Sources

### Primary (HIGH confidence)
- `v2.tauri.app/plugin/store/` - tauri-plugin-store API, auto-save behavior, Store.load pattern
- `docs.rs/keyring/latest/aarch64-apple-darwin/keyring/macos/` - keyring crate macOS backend, Entry API, NoEntry error variant
- `github.com/ayangweb/tauri-plugin-macos-permissions` - Plugin API, checkAccessibilityPermission() boolean return, installation, Tauri v2 compatibility confirmed
- `v2.tauri.app/plugin/http-client/` - HTTP plugin capabilities configuration, domain allow-listing in default.json
- Phase 1 codebase (`src/store/index.ts`, `src/components/HotkeyConfig.tsx`, `src-tauri/src/lib.rs`) - established patterns for Zustand, invoke, Store.load, Rust command structure

### Secondary (MEDIUM confidence)
- `docs.rs/crate/tauri-plugin-keychain/latest` - tauri-plugin-keychain API (getItem, saveItem, removeItem); Tauri 2.0.6+ requirement verified
- `github.com/HuakunShen/tauri-plugin-keyring` - tauri-plugin-keyring API (getPassword, setPassword, deletePassword); Tauri v2 compatibility not explicitly stated in docs
- `docs.x.ai/key-information/using-management-api` - xAI Management API for key validation (separate from inference API; requires separate management key -- not relevant for user API key validation)
- `ui.shadcn.com/docs/components/radix/tabs` - shadcn Tabs component install and usage
- `jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html` - NSWorkspace URL for opening Accessibility pane in System Settings

### Tertiary (LOW confidence - needs validation)
- xAI GET /v1/models endpoint existence: Confirmed via OpenAI compatibility claim and community usage patterns, but not explicitly in xAI REST API reference. Must validate empirically during implementation.
- keyring crate Keychain prompt behavior in unsigned Tauri dev builds: inferred from macOS code signing behavior; needs dev-time validation.

---

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM-HIGH -- keyring and tauri-plugin-macos-permissions verified via official docs; xAI /v1/models endpoint is MEDIUM (OpenAI-compatible claim + community evidence, not in xAI official ref)
- Architecture: HIGH -- patterns follow Phase 1 conventions exactly; store/invoke/event patterns verified in codebase
- Pitfalls: MEDIUM-HIGH -- Keychain prompt and capabilities pitfalls confirmed; xAI endpoint uncertainty flagged as open question

**Research date:** 2026-02-21
**Valid until:** 2026-03-21 (30 days; keyring and Tauri plugin APIs are stable; xAI model list changes more frequently)
