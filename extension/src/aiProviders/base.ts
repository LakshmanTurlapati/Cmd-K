export interface AIProviderConfig {
  apiKey: string;
  model: string;
}

export interface CommandGenerationContext {
  os: string;
  shell: string;
  workingDirectory: string;
  terminalHistory: string[];
  userPrompt: string;
}

export interface AIProvider {
  name: string;
  generateCommand(context: CommandGenerationContext): Promise<string>;
  generateCommandStream(context: CommandGenerationContext, onChunk: (chunk: string) => void): Promise<string>;
}

export abstract class BaseAIProvider implements AIProvider {
  abstract name: string;
  protected config: AIProviderConfig;

  constructor(config: AIProviderConfig) {
    this.config = config;
  }

  abstract generateCommand(context: CommandGenerationContext): Promise<string>;
  abstract generateCommandStream(context: CommandGenerationContext, onChunk: (chunk: string) => void): Promise<string>;

  protected buildSystemPrompt(context: CommandGenerationContext): string {
    const historyText = context.terminalHistory.length > 0
      ? context.terminalHistory.map(cmd => `  $ ${cmd}`).join('\n')
      : '  (no recent commands)';

    return `You are a terminal command generator. Generate ONLY the shell command to execute, with no explanations, markdown formatting, or additional text.

Context:
- OS: ${context.os}
- Shell: ${context.shell}
- Working Directory: ${context.workingDirectory}
- Recent commands:
${historyText}

User request: ${context.userPrompt}

IMPORTANT: Respond with ONLY the shell command. No backticks, no markdown, no explanations. Just the raw command to execute.`;
  }

  protected validateConfig(): void {
    if (!this.config.apiKey || this.config.apiKey.trim() === '') {
      throw new Error(`${this.name} API key is not configured. Please set it in VSCode settings.`);
    }
    if (!this.config.model || this.config.model.trim() === '') {
      throw new Error(`${this.name} model is not configured.`);
    }
  }
}
