import OpenAI from 'openai';
import Anthropic from '@anthropic-ai/sdk';
import axios from 'axios';

export class APIValidator {
  /**
   * Validates OpenAI API key by making a test request
   */
  static async validateOpenAI(apiKey: string, model: string): Promise<boolean> {
    try {
      const client = new OpenAI({ apiKey });

      await client.chat.completions.create({
        model,
        messages: [{ role: 'user', content: 'test' }],
        max_tokens: 5,
      });

      return true;
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('Invalid API key');
      }
      // Other errors might be rate limits, etc. - still valid key
      return error.status !== 401;
    }
  }

  /**
   * Validates Anthropic API key by making a test request
   */
  static async validateAnthropic(apiKey: string, model: string): Promise<boolean> {
    try {
      const client = new Anthropic({ apiKey });

      await client.messages.create({
        model,
        max_tokens: 5,
        messages: [{ role: 'user', content: 'test' }],
      });

      return true;
    } catch (error: any) {
      if (error.status === 401) {
        throw new Error('Invalid API key');
      }
      // Other errors might be rate limits, etc. - still valid key
      return error.status !== 401;
    }
  }

  /**
   * Validates xAI API key by making a test request
   */
  static async validateXAI(apiKey: string, model: string): Promise<boolean> {
    try {
      await axios.post(
        'https://api.x.ai/v1/chat/completions',
        {
          model,
          messages: [{ role: 'user', content: 'test' }],
          max_tokens: 5,
        },
        {
          headers: {
            'Authorization': `Bearer ${apiKey}`,
            'Content-Type': 'application/json',
          },
        }
      );

      return true;
    } catch (error: any) {
      if (error.response?.status === 401) {
        throw new Error('Invalid API key');
      }
      // Other errors might be rate limits, etc. - still valid key
      return error.response?.status !== 401;
    }
  }
}
