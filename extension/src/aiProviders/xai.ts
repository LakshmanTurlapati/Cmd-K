import axios from 'axios';
import { BaseAIProvider, CommandGenerationContext } from './base';

export class XAIProvider extends BaseAIProvider {
  name = 'xAI';
  private apiBaseUrl = 'https://api.x.ai/v1';

  constructor(apiKey: string, model: string) {
    super({ apiKey, model });
    this.validateConfig();
  }

  async generateCommand(context: CommandGenerationContext): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const response = await axios.post(
        `${this.apiBaseUrl}/chat/completions`,
        {
          model: this.config.model,
          messages: [
            {
              role: 'system',
              content: systemPrompt
            }
          ],
          temperature: 0.1,
          max_tokens: 500,
        },
        {
          headers: {
            'Authorization': `Bearer ${this.config.apiKey}`,
            'Content-Type': 'application/json',
          },
        }
      );

      const command = response.data.choices[0]?.message?.content?.trim() || '';
      return this.cleanCommand(command);
    } catch (error: any) {
      if (error.response?.status === 401) {
        throw new Error('xAI API key is invalid. Please check your settings.');
      }
      const errorMessage = error.response?.data?.error?.message || error.message || 'Unknown error occurred';
      throw new Error(`xAI Error: ${errorMessage}`);
    }
  }

  async generateCommandStream(
    context: CommandGenerationContext,
    onChunk: (chunk: string) => void
  ): Promise<string> {
    try {
      const systemPrompt = this.buildSystemPrompt(context);

      const response = await axios.post(
        `${this.apiBaseUrl}/chat/completions`,
        {
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
        },
        {
          headers: {
            'Authorization': `Bearer ${this.config.apiKey}`,
            'Content-Type': 'application/json',
          },
          responseType: 'stream',
        }
      );

      let fullCommand = '';

      return new Promise((resolve, reject) => {
        response.data.on('data', (chunk: Buffer) => {
          const lines = chunk.toString().split('\n').filter(line => line.trim() !== '');

          for (const line of lines) {
            if (line.startsWith('data: ')) {
              const data = line.slice(6);
              if (data === '[DONE]') continue;

              try {
                const parsed = JSON.parse(data);
                const content = parsed.choices[0]?.delta?.content || '';
                if (content) {
                  fullCommand += content;
                  onChunk(content);
                }
              } catch (e) {
                // Skip invalid JSON
              }
            }
          }
        });

        response.data.on('end', () => {
          resolve(this.cleanCommand(fullCommand));
        });

        response.data.on('error', (error: any) => {
          reject(new Error(`xAI Stream Error: ${error.message}`));
        });
      });
    } catch (error: any) {
      if (error.response?.status === 401) {
        throw new Error('xAI API key is invalid. Please check your settings.');
      }
      const errorMessage = error.response?.data?.error?.message || error.message || 'Unknown error occurred';
      throw new Error(`xAI Error: ${errorMessage}`);
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
