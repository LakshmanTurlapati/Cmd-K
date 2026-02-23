import OpenAI from 'openai';
import { BaseAIProvider, CommandGenerationContext } from './base';

export class OpenAIProvider extends BaseAIProvider {
  name = 'OpenAI';
  private client: OpenAI;

  constructor(apiKey: string, model: string) {
    super({ apiKey, model });
    this.validateConfig();
    this.client = new OpenAI({
      apiKey: this.config.apiKey,
    });
  }

  async generateCommand(context: CommandGenerationContext): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const response = await this.client.chat.completions.create({
        model: this.config.model,
        messages: [
          {
            role: 'system',
            content: systemPrompt
          }
        ],
        temperature: 0.1,
        max_tokens: 500,
      });

      const command = response.choices[0]?.message?.content?.trim() || '';
      return this.cleanCommand(command);
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('OpenAI API key is invalid. Please check your settings.');
      }
      throw new Error(`OpenAI Error: ${error.message || 'Unknown error occurred'}`);
    }
  }

  async generateCommandStream(
    context: CommandGenerationContext,
    onChunk: (chunk: string) => void
  ): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const stream = await this.client.chat.completions.create({
        model: this.config.model,
        messages: [
          {
            role: 'system',
            content: systemPrompt
          }
        ],
        temperature: 0.1,
        max_tokens: 500,
        stream: true,
      });

      let fullCommand = '';

      for await (const chunk of stream) {
        const content = chunk.choices[0]?.delta?.content || '';
        if (content) {
          fullCommand += content;
          onChunk(content);
        }
      }

      return this.cleanCommand(fullCommand);
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('OpenAI API key is invalid. Please check your settings.');
      }
      throw new Error(`OpenAI Error: ${error.message || 'Unknown error occurred'}`);
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
