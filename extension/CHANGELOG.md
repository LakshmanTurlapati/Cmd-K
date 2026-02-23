# Change Log

All notable changes to the "Terminal AI Command Generator" extension will be documented in this file.

## [0.1.0] - 2025-01-17

### Added
- Initial release
- CMD+K command generation in terminal
- Support for OpenAI (GPT-4, GPT-4o, GPT-4-turbo)
- Support for Anthropic (Claude Sonnet 4.5, Claude Opus 4.1)
- Support for xAI (Grok)
- Context-aware generation using terminal history and environment
- Review mode: Command inserted into terminal for review
- Execute mode: Command runs immediately
- Configurable settings for API keys and models
- Terminal history tracking (last 20 commands by default)
- Multi-shell support (bash, zsh, PowerShell, fish, cmd)
- Error handling and user feedback
- Comprehensive README with usage examples

### Known Issues
- VSCode InputBox doesn't directly support Cmd/Ctrl+Enter shortcut, so users press Enter to generate and then choose action
- Terminal history is not persisted across VSCode restarts (by design for privacy)
- Generated commands cannot be edited within the UI (must be edited in terminal after insertion)

## [Unreleased]

### Planned Features
- YOLO mode with command allowlists/denylists
- Command history and favorites
- Multi-step command sequences
- Custom prompt templates
- Command explanation mode
- Integration with .cursorrules files
- Sandboxed execution
- Command cost tracking
- Offline mode with local models
- Streaming responses for better UX
- Cmd/Ctrl+Enter direct execution from input box
