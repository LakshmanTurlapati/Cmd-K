# Quick Start Guide

Get up and running with Terminal AI Command Generator in 5 minutes.

## Step 1: Install Dependencies

```bash
npm install
```

## Step 2: Get an API Key

Choose one provider and get an API key:

- **OpenAI**: https://platform.openai.com/api-keys
- **Anthropic**: https://console.anthropic.com/
- **xAI**: https://console.x.ai/

## Step 3: Configure the Extension

### Option A: Using VSCode UI

1. Press F5 to launch Extension Development Host
2. In the new window, open Settings (Cmd/Ctrl + ,)
3. Search for "Terminal AI"
4. Enter your API key

### Option B: Edit settings.json Directly

In the Extension Development Host, add to your settings:

```json
{
  "terminalAI.provider": "openai",
  "terminalAI.openai.apiKey": "sk-your-api-key-here",
  "terminalAI.openai.model": "gpt-4o"
}
```

For Anthropic:
```json
{
  "terminalAI.provider": "anthropic",
  "terminalAI.anthropic.apiKey": "sk-ant-your-api-key-here",
  "terminalAI.anthropic.model": "claude-sonnet-4-5-20250929"
}
```

For xAI:
```json
{
  "terminalAI.provider": "xai",
  "terminalAI.xai.apiKey": "xai-your-api-key-here",
  "terminalAI.xai.model": "grok-beta"
}
```

## Step 4: Try It Out

1. Open a terminal (Terminal > New Terminal)
2. Press **Cmd+K** (Mac) or **Ctrl+K** (Windows/Linux)
3. Type: `"list all files modified today"`
4. Press Enter
5. Choose "Review in Terminal"
6. See the generated command!

## Example Prompts to Try

### Git Commands
- "commit all changes with a descriptive message"
- "create a new branch called feature/auth"
- "show the last 5 commits"

### File Operations
- "find all JavaScript files in this directory"
- "create a backup of package.json"
- "count lines of code in all TypeScript files"

### Development
- "install express as a dependency"
- "run the development server"
- "build for production"

### System
- "check disk usage"
- "find what's using port 3000"
- "show running processes"

## Troubleshooting

### "No active terminal" error
Open a terminal first: Terminal > New Terminal

### "API key is not configured"
Make sure you've set the API key in settings for your chosen provider

### Extension not working after F5
1. Check Terminal panel for compilation errors
2. Run `npm run compile`
3. Press Cmd/Ctrl+R to reload Extension Development Host

## Development Workflow

### Making Changes

1. Edit code in `src/`
2. Save files (auto-compilation with watch mode)
3. Reload Extension Development Host (Cmd/Ctrl+R)
4. Test changes

### Building for Production

```bash
npm run package
```

This creates a `.vsix` file you can install or distribute.

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Check [CONTRIBUTING.md](CONTRIBUTING.md) to contribute
- See [CHANGELOG.md](CHANGELOG.md) for version history

## Tips for Best Results

1. **Be specific**: "list all .js files modified in the last 7 days" is better than "list files"
2. **Mention tools**: "use git to show recent commits" helps if you want a specific tool
3. **Include context**: "in the current directory" or "recursively" helps clarify scope
4. **Review first**: Always use Review mode for destructive commands

## Support

Having issues?
- Check the Troubleshooting section in README.md
- Look at existing GitHub issues
- Create a new issue with details

Happy commanding!
