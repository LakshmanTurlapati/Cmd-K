import * as vscode from 'vscode';

export type AIProviderType = 'openai' | 'anthropic' | 'xai';

export interface ExtensionConfig {
  provider: AIProviderType;
  openai: {
    apiKey: string;
    model: string;
  };
  anthropic: {
    apiKey: string;
    model: string;
  };
  xai: {
    apiKey: string;
    model: string;
  };
  maxHistoryLines: number;
  showPreviewBox: boolean;
}

export class ConfigManager {
  private static readonly CONFIG_SECTION = 'terminalAI';

  /**
   * Gets current extension configuration
   */
  static getConfig(): ExtensionConfig {
    const config = vscode.workspace.getConfiguration(this.CONFIG_SECTION);

    return {
      provider: config.get<AIProviderType>('provider', 'openai'),
      openai: {
        apiKey: config.get<string>('openai.apiKey', ''),
        model: config.get<string>('openai.model', 'gpt-4o'),
      },
      anthropic: {
        apiKey: config.get<string>('anthropic.apiKey', ''),
        model: config.get<string>('anthropic.model', 'claude-sonnet-4-5-20250929'),
      },
      xai: {
        apiKey: config.get<string>('xai.apiKey', ''),
        model: config.get<string>('xai.model', 'grok-beta'),
      },
      maxHistoryLines: config.get<number>('maxHistoryLines', 20),
      showPreviewBox: config.get<boolean>('showPreviewBox', true),
    };
  }

  /**
   * Validates that the selected provider is configured
   */
  static validateConfig(config: ExtensionConfig): { valid: boolean; message?: string } {
    const provider = config.provider;

    switch (provider) {
      case 'openai':
        if (!config.openai.apiKey) {
          return {
            valid: false,
            message: 'OpenAI API key is not configured. Please set it in Settings > Extensions > Terminal AI Command Generator.',
          };
        }
        break;

      case 'anthropic':
        if (!config.anthropic.apiKey) {
          return {
            valid: false,
            message: 'Anthropic API key is not configured. Please set it in Settings > Extensions > Terminal AI Command Generator.',
          };
        }
        break;

      case 'xai':
        if (!config.xai.apiKey) {
          return {
            valid: false,
            message: 'xAI API key is not configured. Please set it in Settings > Extensions > Terminal AI Command Generator.',
          };
        }
        break;
    }

    return { valid: true };
  }

  /**
   * Opens settings UI for configuration
   */
  static async openSettings(): Promise<void> {
    await vscode.commands.executeCommand('workbench.action.openSettings', '@ext:terminalAI');
  }

  /**
   * Gets list of providers that have API keys configured
   */
  static getAvailableProviders(): AIProviderType[] {
    const config = this.getConfig();
    const available: AIProviderType[] = [];

    if (config.openai.apiKey && config.openai.apiKey.trim() !== '') {
      available.push('openai');
    }
    if (config.anthropic.apiKey && config.anthropic.apiKey.trim() !== '') {
      available.push('anthropic');
    }
    if (config.xai.apiKey && config.xai.apiKey.trim() !== '') {
      available.push('xai');
    }

    return available;
  }

  /**
   * Gets available models for a specific provider
   */
  static getModelsForProvider(provider: AIProviderType): string[] {
    switch (provider) {
      case 'openai':
        return ['gpt-4o', 'gpt-4-turbo', 'gpt-4', 'gpt-3.5-turbo'];

      case 'anthropic':
        return [
          'claude-sonnet-4-5-20250929',
          'claude-opus-4-20250514',
          'claude-3-5-sonnet-20241022',
          'claude-3-opus-20240229'
        ];

      case 'xai':
        return [
          'grok-beta',
          'grok-4',
          'grok-4-fast-reasoning',
          'grok-4-fast-non-reasoning',
          'grok-code-fast-1',
          'grok-3',
          'grok-3-mini',
          'grok-2-latest'
        ];

      default:
        return [];
    }
  }

  /**
   * Gets friendly name for provider
   */
  static getProviderDisplayName(provider: AIProviderType): string {
    switch (provider) {
      case 'openai':
        return 'OpenAI';
      case 'anthropic':
        return 'Anthropic';
      case 'xai':
        return 'xAI';
      default:
        return provider;
    }
  }

  /**
   * Gets friendly name for model
   */
  static getModelDisplayName(model: string): string {
    // Map technical model names to friendly names
    const modelNames: Record<string, string> = {
      'gpt-4o': 'GPT-4o',
      'gpt-4-turbo': 'GPT-4 Turbo',
      'gpt-4': 'GPT-4',
      'gpt-3.5-turbo': 'GPT-3.5 Turbo',
      'claude-sonnet-4-5-20250929': 'Claude Sonnet 4.5',
      'claude-opus-4-20250514': 'Claude Opus 4.1',
      'claude-3-5-sonnet-20241022': 'Claude 3.5 Sonnet',
      'claude-3-opus-20240229': 'Claude 3 Opus',
      'grok-beta': 'Grok Beta',
      'grok-4': 'Grok 4',
      'grok-4-fast-reasoning': 'Grok 4 Fast Reasoning',
      'grok-4-fast-non-reasoning': 'Grok 4 Fast',
      'grok-code-fast-1': 'Grok Code Fast',
      'grok-3': 'Grok 3',
      'grok-3-mini': 'Grok 3 Mini',
      'grok-2-latest': 'Grok 2'
    };

    return modelNames[model] || model;
  }
}
