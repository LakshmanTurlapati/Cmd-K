# Requirements: CMD+K

**Defined:** 2026-03-08
**Core Value:** The overlay must appear on top of the currently active application and feel instant

## v0.2.6 Requirements

Requirements for multi-provider AI, WSL terminal context, and auto-updater.

### Provider Abstraction

- [x] **PROV-01**: User can select their AI provider from OpenAI, Anthropic, Google Gemini, xAI, or OpenRouter
- [x] **PROV-02**: User can store a separate API key per provider in the platform keychain
- [x] **PROV-03**: Existing xAI API key is migrated automatically on upgrade from v0.2.4
- [x] **PROV-04**: User can validate their API key for any provider before saving
- [x] **PROV-05**: User can see available models for their selected provider
- [x] **PROV-06**: AI responses stream in real-time from all 5 providers
- [x] **PROV-07**: Provider-specific error messages show the correct provider name and troubleshooting hints

### Provider Frontend

- [x] **PFUI-01**: User can select a provider during first-run onboarding
- [x] **PFUI-02**: User can switch providers in the settings Account tab
- [x] **PFUI-03**: User can pick a model from a dropdown filtered to their selected provider
- [x] **PFUI-04**: Models are grouped by capability tier (Fast, Balanced, Most Capable) across all providers
- [x] **PFUI-05**: User can switch providers without losing conversation history

### OpenRouter

- [x] **ORTR-01**: User can use a single OpenRouter API key to access models from all supported providers
- [x] **ORTR-02**: OpenRouter model list is filtered to chat-capable models with sensible grouping

### WSL Terminal Context

- [x] **WSLT-01**: CMD+K detects WSL sessions in Windows Terminal
- [x] **WSLT-02**: CMD+K detects WSL sessions in VS Code Remote-WSL terminals
- [x] **WSLT-03**: CMD+K detects WSL sessions in Cursor Remote-WSL terminals
- [x] **WSLT-04**: CMD+K detects standalone wsl.exe console sessions
- [x] **WSLT-05**: CMD+K reads the current working directory from WSL sessions
- [x] **WSLT-06**: CMD+K detects the shell type (bash, zsh, fish) in WSL sessions
- [x] **WSLT-07**: CMD+K reads visible terminal output from WSL sessions
- [x] **WSLT-08**: AI generates Linux commands when user is in a WSL session
- [x] **WSLT-09**: Linux destructive command patterns are applied in WSL sessions
- [x] **WSLT-10**: WSL context badge shows "WSL" label when user is in a WSL session

### Auto-Updater

- [ ] **UPDT-01**: App checks for updates on launch without blocking the UI
- [ ] **UPDT-02**: User sees an "Update Available" indicator in the tray menu when an update exists
- [ ] **UPDT-03**: User can download and install the update with one click from the tray
- [ ] **UPDT-04**: Update is applied on next app launch (no forced restart)
- [ ] **UPDT-05**: Updates are cryptographically signed and verified before installation
- [x] **UPDT-06**: CI/CD pipeline generates signed update artifacts and latest.json manifest
- [ ] **UPDT-07**: Background update checks run silently every 24 hours after launch
- [ ] **UPDT-08**: Dismissing the update notification suppresses it until next app launch

## Future Requirements

### Deferred

- **PROV-F01**: Fallback to secondary provider on API error
- **PROV-F02**: Custom API endpoint URLs for self-hosted models
- **PROV-F03**: Provider-specific prompt customization UI
- **UPDT-F01**: Update channel selector (stable/beta)
- **WSLT-F01**: WSL path translation (/mnt/c/... to C:\... for display)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multi-provider simultaneous queries | Wastes API credits, complicates UX |
| Proxy/relay server for API calls | Adds infrastructure cost, privacy concerns, maintenance burden |
| Forced auto-update | Breaks user trust, enterprise users need version pinning |
| WSL file system browsing | Out of scope for a command overlay |
| Full WSL process tree walking | Slow, unnecessary -- wsl.exe -e commands suffice |
| Auto-select "best" provider | Users have strong preferences -- let them choose explicitly |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROV-01 | Phase 21 | Complete |
| PROV-02 | Phase 21 | Complete |
| PROV-03 | Phase 21 | Complete |
| PROV-04 | Phase 21 | Complete |
| PROV-05 | Phase 21 | Complete |
| PROV-06 | Phase 21 | Complete |
| PROV-07 | Phase 21 | Complete |
| PFUI-01 | Phase 22 | Complete |
| PFUI-02 | Phase 22 | Complete |
| PFUI-03 | Phase 22 | Complete |
| PFUI-04 | Phase 22 | Complete |
| PFUI-05 | Phase 22 | Complete |
| ORTR-01 | Phase 22 | Complete |
| ORTR-02 | Phase 22 | Complete |
| WSLT-01 | Phase 23 | Complete |
| WSLT-02 | Phase 23 | Complete |
| WSLT-03 | Phase 23 | Complete |
| WSLT-04 | Phase 23 | Complete |
| WSLT-05 | Phase 23 | Complete |
| WSLT-06 | Phase 23 | Complete |
| WSLT-07 | Phase 23 | Complete |
| WSLT-08 | Phase 23 | Complete |
| WSLT-09 | Phase 23 | Complete |
| WSLT-10 | Phase 23 | Complete |
| UPDT-01 | Phase 24 | Pending |
| UPDT-02 | Phase 24 | Pending |
| UPDT-03 | Phase 24 | Pending |
| UPDT-04 | Phase 24 | Pending |
| UPDT-05 | Phase 24 | Pending |
| UPDT-06 | Phase 24 | Complete |
| UPDT-07 | Phase 24 | Pending |
| UPDT-08 | Phase 24 | Pending |

**Coverage:**
- v0.2.6 requirements: 32 total
- Mapped to phases: 32
- Unmapped: 0

---
*Requirements defined: 2026-03-08*
*Last updated: 2026-03-09 after roadmap creation*
