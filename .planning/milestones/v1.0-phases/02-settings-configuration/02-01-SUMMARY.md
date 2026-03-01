---
phase: 02-settings-configuration
plan: 01
subsystem: api
tags: [rust, tauri, keychain, keyring, reqwest, xai, accessibility, macos]

# Dependency graph
requires:
  - phase: 01-foundation-overlay
    provides: Tauri v2 + React overlay foundation, lib.rs command structure, tauri-plugin-store

provides:
  - save_api_key / get_api_key / delete_api_key Tauri IPC commands via keyring crate
  - validate_and_fetch_models Tauri IPC command with 404 fallback to completions validation
  - open_accessibility_settings Tauri IPC command
  - check_accessibility_permission Tauri IPC command via AXIsProcessTrusted FFI

affects:
  - 02-02 (settings UI frontend uses these Tauri IPC commands)
  - 02-03 (onboarding wizard uses validate_and_fetch_models and check_accessibility_permission)

# Tech tracking
tech-stack:
  added:
    - keyring v3.6.x (apple-native feature) -- macOS User Keychain secure storage
    - tauri-plugin-http v2.5.x -- reqwest re-export for Rust-side HTTP calls
  patterns:
    - Security boundary: API key never touches JS layer; all Keychain and HTTP in Rust commands
    - 404 fallback: validate_and_fetch_models falls back to POST /v1/chat/completions + hardcoded model list if GET /v1/models returns 404
    - AXIsProcessTrusted FFI: stable public C function from ApplicationServices.framework, no plugin dependency
    - Manual JSON parsing via serde_json::from_slice when tauri_plugin_http::reqwest re-export lacks json() feature method

key-files:
  created:
    - src-tauri/src/commands/keychain.rs
    - src-tauri/src/commands/xai.rs
    - src-tauri/src/commands/permissions.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/capabilities/default.json

key-decisions:
  - "keyring crate used directly (no community plugin wrapper): fewer dependencies, full control, same Keychain result"
  - "AXIsProcessTrusted via extern C block: stable macOS public API, avoids tauri-plugin-macos-permissions dependency for a single boolean"
  - "tauri_plugin_http::reqwest re-export lacks json() feature: use .bytes() + serde_json::from_slice for response parsing and .body(json_string) for requests"
  - "404 fallback built from day one: GET /v1/models not explicitly documented by xAI; fallback validates via POST /v1/chat/completions with max_tokens=1 and returns hardcoded model list"

patterns-established:
  - "Pattern: Rust Tauri command as security boundary -- API key only flows through Rust, never stored in Zustand/JS state"
  - "Pattern: serde_json::to_string + .body() for HTTP POST when reqwest re-export lacks json() builder"

requirements-completed: [SETT-01, SETT-03]

# Metrics
duration: 12min
completed: 2026-02-21
---

# Phase 2 Plan 1: Rust Backend Commands for Keychain, xAI API, and Accessibility Summary

**Three Rust Tauri command modules exposing 6 IPC commands: macOS Keychain CRUD via keyring crate, xAI model validation with 404 fallback, and Accessibility permission check via AXIsProcessTrusted FFI**

## Performance

- **Duration:** 12 min
- **Started:** 2026-02-21T00:18:05Z
- **Completed:** 2026-02-21T00:30:08Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- keychain.rs exposes save/get/delete_api_key commands backed by macOS User Keychain (keyring crate, service: com.lakshmanturlapati.cmd-k)
- xai.rs exposes validate_and_fetch_models with primary GET /v1/models path and 404 fallback to POST /v1/chat/completions validation + hardcoded model list
- permissions.rs exposes open_accessibility_settings (open URL) and check_accessibility_permission (AXIsProcessTrusted FFI, no plugin dependency)
- All 6 commands registered in lib.rs invoke_handler; HTTP plugin initialized; capabilities allow https://api.x.ai/**

## Task Commits

Each task was committed atomically:

1. **Task 1: Add keyring/HTTP deps, keychain.rs, xai.rs, permissions.rs, update mod.rs** - `fc551bf` (feat)
2. **Task 2: Register all commands in lib.rs, configure HTTP capabilities** - `c20b322` (feat)

## Files Created/Modified

- `src-tauri/src/commands/keychain.rs` - save_api_key, get_api_key, delete_api_key via keyring::Entry
- `src-tauri/src/commands/xai.rs` - validate_and_fetch_models with 404 fallback + model label mapping
- `src-tauri/src/commands/permissions.rs` - open_accessibility_settings + check_accessibility_permission (AXIsProcessTrusted)
- `src-tauri/src/commands/mod.rs` - Added keychain, xai, permissions module declarations
- `src-tauri/src/lib.rs` - New imports, tauri_plugin_http::init(), 6 commands in generate_handler!
- `src-tauri/Cargo.toml` - Added keyring v3 (apple-native) and tauri-plugin-http v2
- `src-tauri/capabilities/default.json` - Added http:default, api.x.ai allow-list, store:allow-load, store:allow-delete

## Decisions Made

- Used keyring crate directly instead of tauri-plugin-keychain or tauri-plugin-keyring: fewer crate dependencies, same Keychain result, full control over error handling
- Used AXIsProcessTrusted via `extern "C"` block instead of tauri-plugin-macos-permissions: that plugin adds significant crate overhead for a single boolean; the raw FFI is 3 lines
- tauri_plugin_http::reqwest re-export does not expose the json() request builder or response method (the feature flag is not enabled in the plugin's re-export). Fixed by using .body(serde_json::json!(...).to_string()) for requests and .bytes() + serde_json::from_slice() for responses
- Built the GET /v1/models 404 fallback immediately per plan spec: xAI does not explicitly document this endpoint; fallback uses POST /v1/chat/completions with max_tokens=1 to validate the key and returns the hardcoded 5-model list

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] tauri_plugin_http::reqwest re-export missing json() feature**
- **Found during:** Task 1 (xai.rs compilation)
- **Issue:** Plan spec called for `.json(&body)` on RequestBuilder and `.json().await` on Response, but the tauri-plugin-http reqwest re-export does not expose these methods (the json feature flag is absent)
- **Fix:** Used `.body(serde_json::json!(...).to_string())` + `Content-Type: application/json` header for POST body; used `.bytes().await` + `serde_json::from_slice()` for response parsing
- **Files modified:** src-tauri/src/commands/xai.rs
- **Verification:** cargo check passes with zero errors
- **Committed in:** fc551bf (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - bug in implementation approach)
**Impact on plan:** Fix is transparent to callers -- same behavior, different internal serialization path. No scope creep.

## Issues Encountered

- tauri_plugin_http::reqwest does not re-export with the json feature enabled. Resolved by manual serde_json serialization/deserialization (standard Rust pattern, no library additions required).

## User Setup Required

None - no external service configuration required for this plan. Keychain prompts in development (due to unsigned binary) are expected behavior documented in RESEARCH.md Pitfall 1.

## Next Phase Readiness

- All 6 Tauri IPC commands ready for frontend consumption via `invoke()`
- Rust backend security boundary is established: API key flows only through Rust
- Frontend (02-02, 02-03) can call: invoke("save_api_key", { key }), invoke("get_api_key"), invoke("delete_api_key"), invoke("validate_and_fetch_models", { apiKey }), invoke("open_accessibility_settings"), invoke("check_accessibility_permission")
- Note: macOS Keychain will show an allow-access dialog in dev builds (expected; signed release build shows it only once)

---
*Phase: 02-settings-configuration*
*Completed: 2026-02-21*
