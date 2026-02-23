# Contributing to Terminal AI Command Generator

Thank you for your interest in contributing to Terminal AI Command Generator! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Node.js (v18 or higher)
- npm or yarn
- VSCode (for development and testing)

### Getting Started

1. Clone the repository:
```bash
git clone https://github.com/yourusername/terminal-ai-cmd.git
cd terminal-ai-cmd
```

2. Install dependencies:
```bash
npm install
```

3. Open in VSCode:
```bash
code .
```

4. Start development:
- Press F5 to open Extension Development Host
- Or run `npm run watch` in terminal for continuous compilation

## Project Structure

```
terminal-ai-cmd/
├── src/
│   ├── extension.ts           # Main extension entry point
│   ├── terminalManager.ts     # Terminal interaction & history
│   ├── contextBuilder.ts      # Context assembly
│   ├── config.ts              # Configuration management
│   ├── aiProviders/
│   │   ├── base.ts           # Base provider interface
│   │   ├── openai.ts         # OpenAI implementation
│   │   ├── anthropic.ts      # Anthropic implementation
│   │   └── xai.ts            # xAI implementation
│   └── ui/
│       └── promptInput.ts    # UI components
├── package.json              # Extension manifest
├── tsconfig.json
└── webpack.config.js
```

## Making Changes

### Code Style

- Use TypeScript strict mode
- Follow existing code formatting (ESLint configuration)
- Add JSDoc comments for public methods
- Use meaningful variable and function names

### Adding a New AI Provider

1. Create a new file in `src/aiProviders/` (e.g., `gemini.ts`)
2. Extend `BaseAIProvider` class:

```typescript
import { BaseAIProvider, CommandGenerationContext } from './base';

export class GeminiProvider extends BaseAIProvider {
  name = 'Gemini';

  async generateCommand(context: CommandGenerationContext): Promise<string> {
    // Implementation
  }
}
```

3. Update `src/config.ts` to add configuration options
4. Update `src/extension.ts` to include the new provider in `createProvider()`
5. Update `package.json` to add settings for the new provider

### Testing Your Changes

1. Press F5 to launch Extension Development Host
2. Open a terminal in the development host
3. Press Cmd/Ctrl+K to test the extension
4. Check the Debug Console for any errors

### Manual Testing Checklist

- [ ] CMD+K opens prompt in terminal
- [ ] Prompt accepts natural language input
- [ ] Command is generated successfully
- [ ] Review mode inserts command correctly
- [ ] Execute mode runs command correctly
- [ ] Error handling works for invalid API keys
- [ ] Settings can be configured properly
- [ ] Works with different shells (bash, zsh, PowerShell)
- [ ] Works on different platforms (macOS, Windows, Linux if possible)

## Submitting Changes

### Pull Request Process

1. Fork the repository
2. Create a feature branch:
```bash
git checkout -b feature/your-feature-name
```

3. Make your changes and commit:
```bash
git add .
git commit -m "Add feature: your feature description"
```

4. Push to your fork:
```bash
git push origin feature/your-feature-name
```

5. Create a Pull Request on GitHub

### Pull Request Guidelines

- Provide a clear description of the changes
- Reference any related issues
- Include screenshots/videos for UI changes
- Update README.md if adding new features
- Update CHANGELOG.md with your changes
- Ensure code passes linting (`npm run lint`)
- Test thoroughly before submitting

### Commit Message Format

Use clear, descriptive commit messages:

```
Add feature: brief description

Longer explanation if needed:
- What was changed
- Why it was changed
- Any breaking changes
```

Examples:
- `Add support for Google Gemini provider`
- `Fix: Handle empty terminal history gracefully`
- `Improve: Better error messages for API failures`
- `Docs: Update README with troubleshooting section`

## Feature Requests

Have an idea for a new feature? Great! Please:

1. Check existing issues first
2. Create a new issue with:
   - Clear description of the feature
   - Use case / why it's needed
   - Proposed implementation (if you have ideas)

## Bug Reports

Found a bug? Please create an issue with:

- VSCode version
- Extension version
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Error messages (if any)
- Screenshots (if applicable)

## Code Review Process

All submissions require review. We will:

- Check code quality and style
- Test functionality
- Review for security issues
- Ensure documentation is updated
- Verify backward compatibility

## Development Tips

### Debugging

1. Use Debug Console in VSCode to see logs
2. Add `console.log()` statements (they appear in Debug Console)
3. Use VSCode debugger breakpoints in your code
4. Check Output panel > Extension Host for errors

### Common Issues

**Extension not loading:**
- Check for compilation errors in the Terminal
- Make sure `npm run compile` succeeds
- Restart Extension Development Host

**Changes not appearing:**
- Reload Extension Development Host (Cmd/Ctrl+R)
- Recompile with `npm run compile`

**Testing API providers:**
- Use test API keys (not production)
- Be mindful of API costs during testing
- Mock API responses for unit tests (future improvement)

## Questions?

If you have questions about contributing:
- Check existing issues and discussions
- Create a new discussion on GitHub
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Thank You!

Your contributions help make this extension better for everyone!
