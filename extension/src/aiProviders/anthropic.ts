import Anthropic from '@anthropic-ai/sdk';
import { BaseAIProvider, CommandGenerationContext } from './base';

export class AnthropicProvider extends BaseAIProvider {
  name = 'Anthropic';
  private client: Anthropic;

  constructor(apiKey: string, model: string) {
    super({ apiKey, model });
    this.validateConfig();
    this.client = new Anthropic({
      apiKey: this.config.apiKey,
    });
  }

  async generateCommand(context: CommandGenerationContext): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const response = await this.client.messages.create({
        model: this.config.model,
        max_tokens: 500,
        temperature: 0.1,
        messages: [
          {
            role: 'user',
            content: systemPrompt
          }
        ],
      });

      const content = response.content[0];
      const command = content.type === 'text' ? content.text.trim() : '';
      return this.cleanCommand(command);
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('Anthropic API key is invalid. Please check your settings.');
      }
      throw new Error(`Anthropic Error: ${error.message || 'Unknown error occurred'}`);
    }
  }

  async generateCommandStream(
    context: CommandGenerationContext,
    onChunk: (chunk: string) => void
  ): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const stream = await this.client.messages.create({
        model: this.config.model,
        max_tokens: 500,
        temperature: 0.1,
        messages: [
          {
            role: 'user',
            content: systemPrompt
          }
        ],
        stream: true,
      });

      let fullCommand = '';

      for await (const event of stream) {
        if (event.type === 'content_block_delta' && event.delta.type === 'text_delta') {
          const content = event.delta.text;
          fullCommand += content;
          onChunk(content);
        }
      }

      return this.cleanCommand(fullCommand);
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('Anthropic API key is invalid. Please check your settings.');
      }
      throw new Error(`Anthropic Error: ${error.message || 'Unknown error occurred'}`);
    }
  }

  private cleanCommand(command: string): string {
    // Remove markdown code blocks if present
    command = command.replace(/```[\w]*\n?/g, '').trim();

    // Remove surrounding quotes if present
    if ((command.startsWith('"') && command.endsWith('"')) ||
        (command.startsWith("'") && command.endsWith("'"))) {
      command = command.slice(1, -1);
    }

    return command.trim();
  }
}
