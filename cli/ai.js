const OpenAI = require('openai');
const Anthropic = require('@anthropic-ai/sdk');
const os = require('os');

// System prompt for command generation
const SYSTEM_PROMPT = `You are a terminal command generator. Given a natural language description, output ONLY the exact terminal command(s) needed.

Rules:
- Output ONLY the command, no explanations
- Use the appropriate syntax for the user's OS and shell
- For multiple commands, separate with && or ;
- Never include markdown formatting
- Never include backticks or code blocks
- If the request is unclear, output the most likely command`;

/**
 * Generate a command using OpenAI
 */
async function generateWithOpenAI(prompt, apiKey, model = 'gpt-4o') {
  const client = new OpenAI({ apiKey });

  const response = await client.chat.completions.create({
    model,
    messages: [
      { role: 'system', content: SYSTEM_PROMPT },
      { role: 'user', content: buildUserPrompt(prompt) }
    ],
    temperature: 0.3,
    max_tokens: 500
  });

  return response.choices[0]?.message?.content?.trim() || '';
}

/**
 * Generate a command using Anthropic
 */
async function generateWithAnthropic(prompt, apiKey, model = 'claude-sonnet-4-5-20250929') {
  const client = new Anthropic({ apiKey });

  const response = await client.messages.create({
    model,
    max_tokens: 500,
    system: SYSTEM_PROMPT,
    messages: [
      { role: 'user', content: buildUserPrompt(prompt) }
    ]
  });

  const textBlock = response.content.find(block => block.type === 'text');
  return textBlock?.text?.trim() || '';
}

/**
 * Generate a command using xAI (Grok)
 * Uses OpenAI-compatible API
 */
async function generateWithXAI(prompt, apiKey, model = 'grok-code-fast-1') {
  const client = new OpenAI({
    apiKey,
    baseURL: 'https://api.x.ai/v1'
  });

  const response = await client.chat.completions.create({
    model,
    messages: [
      { role: 'system', content: SYSTEM_PROMPT },
      { role: 'user', content: buildUserPrompt(prompt) }
    ],
    temperature: 0.3,
    max_tokens: 500
  });

  return response.choices[0]?.message?.content?.trim() || '';
}

/**
 * Build user prompt with context
 */
function buildUserPrompt(prompt) {
  const platform = os.platform();
  const shell = process.env.SHELL || 'bash';
  const cwd = process.cwd();

  return `OS: ${platform}
Shell: ${shell}
Current directory: ${cwd}

Request: ${prompt}`;
}

/**
 * Generate a command using the specified provider
 */
async function generate(prompt, config) {
  const { provider, apiKey, model } = config;

  if (!apiKey) {
    throw new Error('API key not configured. Use /settings to add one.');
  }

  switch (provider) {
    case 'openai':
      return generateWithOpenAI(prompt, apiKey, model);
    case 'anthropic':
      return generateWithAnthropic(prompt, apiKey, model);
    case 'xai':
      return generateWithXAI(prompt, apiKey, model);
    default:
      throw new Error(`Unknown provider: ${provider}`);
  }
}

// Available models per provider
const MODELS = {
  openai: ['gpt-4o', 'gpt-4o-mini', 'gpt-4-turbo', 'gpt-4', 'gpt-3.5-turbo'],
  anthropic: [
    'claude-sonnet-4-5-20250929',
    'claude-opus-4-20250514',
    'claude-3-5-sonnet-20241022',
    'claude-3-5-haiku-20241022',
    'claude-3-opus-20240229'
  ],
  xai: [
    'grok-beta',
    'grok-4',
    'grok-4-fast-reasoning',
    'grok-4-fast-non-reasoning',
    'grok-code-fast-1',
    'grok-3',
    'grok-3-mini',
    'grok-2-latest'
  ]
};

module.exports = {
  generate,
  MODELS
};
