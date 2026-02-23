import * as vscode from 'vscode';
import { ConfigManager, AIProviderType } from '../config';
import { ContextBuilder } from '../contextBuilder';
import { TerminalManager } from '../terminalManager';
import { OpenAIProvider } from '../aiProviders/openai';
import { AnthropicProvider } from '../aiProviders/anthropic';
import { XAIProvider } from '../aiProviders/xai';

export class StreamHandler {
  public onChunk: ((chunk: string) => void) | undefined;

  constructor(
    private extensionUri: vscode.Uri,
    private terminalManager: TerminalManager,
    private terminal: vscode.Terminal,
    private userPrompt: string,
    private providerOverride?: AIProviderType,
    private modelOverride?: string
  ) {}

  async generate(): Promise<string> {
    const config = ConfigManager.getConfig();

    // Use override provider if specified, otherwise use config default
    const provider = this.providerOverride || config.provider;

    // Get model - use override if specified, otherwise use config for the provider
    let model: string;
    if (this.modelOverride) {
      model = this.modelOverride;
    } else {
      switch (provider) {
        case 'openai':
          model = config.openai.model;
          break;
        case 'anthropic':
          model = config.anthropic.model;
          break;
        case 'xai':
          model = config.xai.model;
          break;
        default:
          throw new Error(`Unknown provider: ${provider}`);
      }
    }

    // Get API key for the provider
    let apiKey: string;
    switch (provider) {
      case 'openai':
        apiKey = config.openai.apiKey;
        break;
      case 'anthropic':
        apiKey = config.anthropic.apiKey;
        break;
      case 'xai':
        apiKey = config.xai.apiKey;
        break;
      default:
        throw new Error(`Unknown provider: ${provider}`);
    }

    if (!apiKey || apiKey.trim() === '') {
      throw new Error(`API key not configured for ${provider}`);
    }

    // Build context
    const contextBuilder = new ContextBuilder(this.terminalManager);
    const context = contextBuilder.buildContext(this.terminal, this.userPrompt);

    // Create provider and generate with streaming
    switch (provider) {
      case 'openai':
        const openaiProvider = new OpenAIProvider(apiKey, model);
        return await openaiProvider.generateCommandStream(context, (chunk) => {
          if (this.onChunk) this.onChunk(chunk);
        });

      case 'anthropic':
        const anthropicProvider = new AnthropicProvider(apiKey, model);
        return await anthropicProvider.generateCommandStream(context, (chunk) => {
          if (this.onChunk) this.onChunk(chunk);
        });

      case 'xai':
        const xaiProvider = new XAIProvider(apiKey, model);
        return await xaiProvider.generateCommandStream(context, (chunk) => {
          if (this.onChunk) this.onChunk(chunk);
        });

      default:
        throw new Error(`Unknown provider: ${provider}`);
    }
  }
}
