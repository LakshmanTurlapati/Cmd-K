# Requirements: CMD+K

**Defined:** 2026-02-21
**Core Value:** The overlay must appear on top of the active application and feel instant

## v1 Requirements

### Overlay

- [x] **OVRL-01**: System-wide Cmd+K hotkey triggers the overlay from any application
- [x] **OVRL-02**: Overlay appears as a floating panel on top of the currently active window
- [x] **OVRL-03**: User can dismiss overlay with Escape key without executing
- [x] **OVRL-04**: User can configure the trigger hotkey to avoid conflicts
- [x] **OVRL-05**: App runs as background daemon with menu bar icon

### AI Command Generation

- [ ] **AICG-01**: User can type natural language and receive a terminal command via xAI (Grok)
- [ ] **AICG-02**: Command generation streams in real-time as the response is generated
- [ ] **AICG-03**: Destructive commands (rm -rf, format, etc.) are flagged with a warning before paste

### Terminal Integration

- [ ] **TERM-01**: Generated command is pasted into the active terminal (Terminal.app, iTerm2)
- [ ] **TERM-02**: App detects the current working directory of the active terminal without shell plugins
- [ ] **TERM-03**: App reads recent terminal output for context without shell plugins
- [ ] **TERM-04**: Works with Terminal.app, iTerm2, Alacritty, kitty, WezTerm

### Settings & Onboarding

- [x] **SETT-01**: User can store and validate their xAI API key
- [x] **SETT-02**: User can select which Grok model to use
- [x] **SETT-03**: API keys stored securely in macOS Keychain
- [ ] **SETT-04**: First-run onboarding guides user through Accessibility permissions and API key setup

## v2 Requirements

### Additional AI Providers

- **PROV-01**: OpenAI (GPT-4o) provider support
- **PROV-02**: Anthropic (Claude) provider support
- **PROV-03**: User can switch between providers in settings

### Command History

- **HIST-01**: Generated commands saved to searchable history
- **HIST-02**: User can favorite frequently used commands
- **HIST-03**: User can re-run commands from history

### Advanced Features

- **ADVN-01**: Multi-step command workflow generation
- **ADVN-02**: Command explanation mode (explain what a command does)
- **ADVN-03**: Cross-platform support (Windows, Linux)

## Out of Scope

| Feature | Reason |
|---------|--------|
| VS Code extension | Dropped -- standalone overlay is the product |
| Shell plugins (.zshrc) | Zero setup constraint -- use Accessibility API and process inspection instead |
| App Store distribution | Incompatible with Accessibility API requirement |
| Offline mode | Requires internet for AI generation |
| Auto-execution without review | Safety risk -- always paste, never execute directly |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| OVRL-01 | Phase 1 | Complete (01-01) |
| OVRL-02 | Phase 1 | Complete (01-01) |
| OVRL-03 | Phase 1 | Complete |
| OVRL-04 | Phase 1 | Complete (01-03) |
| OVRL-05 | Phase 1 | Complete (01-01) |
| SETT-01 | Phase 2 | Complete |
| SETT-02 | Phase 2 | Complete (02-02) |
| SETT-03 | Phase 2 | Complete |
| SETT-04 | Phase 2 | Pending |
| TERM-02 | Phase 3 | Pending |
| TERM-03 | Phase 3 | Pending |
| TERM-04 | Phase 3 | Pending |
| AICG-01 | Phase 4 | Pending |
| AICG-02 | Phase 4 | Pending |
| AICG-03 | Phase 5 | Pending |
| TERM-01 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 16 total
- Mapped to phases: 16
- Unmapped: 0

---
*Requirements defined: 2026-02-21*
*Last updated: 2026-02-21 after 02-02 completion (SETT-01, SETT-02 complete via 02-01/02-02)*
