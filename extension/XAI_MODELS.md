# xAI Grok Models Reference

Complete list of xAI Grok models available for the Terminal AI Command Generator extension (as of 2025).

## Available Models

### Latest Generation (Grok 4)

#### grok-4
- **Description**: The most intelligent model in the world
- **Features**: Native tool use, real-time search integration
- **Use Case**: Complex tasks requiring highest intelligence
- **Context Window**: Large (exact size TBD)

#### grok-4-fast-reasoning
- **Description**: Fast variant with reasoning capabilities
- **Context Window**: 2M tokens
- **Use Case**: Tasks requiring reasoning with speed
- **Performance**: Faster than standard grok-4

#### grok-4-fast-non-reasoning
- **Description**: Fast variant without reasoning overhead
- **Context Window**: 2M tokens
- **Use Case**: Simple tasks requiring quick responses
- **Performance**: Fastest Grok 4 variant

### Specialized Models

#### grok-code-fast-1
- **Description**: Speedy and economical reasoning model for coding
- **Specialty**: Agentic coding tasks
- **Pricing**:
  - $0.20 / 1M input tokens
  - $1.50 / 1M output tokens
  - $0.02 / 1M cached input tokens
- **Use Case**: Terminal command generation, coding assistance

### Previous Generation (Grok 3)

#### grok-3
- **Description**: Full previous generation model
- **Knowledge Cutoff**: November 2024
- **Use Case**: General purpose tasks

#### grok-3-mini
- **Description**: Smaller, budget-friendly version of Grok 3
- **Use Case**: Cost-effective for simple tasks
- **Performance**: Faster but less capable than grok-3

### Legacy Models

#### grok-2-latest
- **Description**: Grok 2 latest version (aliased)
- **Status**: Previous generation
- **Use Case**: Fallback option

#### grok-beta
- **Description**: Beta/testing version
- **Status**: May be aliased to latest stable
- **Use Case**: Access to cutting-edge features

### Image Generation

#### grok-2-image-1212
- **Description**: Text-to-image generation model
- **Specialty**: Image generation
- **Use Case**: Not applicable for terminal commands
- **Note**: Not included in this extension

## Model Naming Convention

xAI uses a helpful aliasing system:

- **`grok-4`** → Latest stable Grok 4 version
- **`grok-4-latest`** → Cutting-edge Grok 4 (may be less stable)
- **`grok-3`** → Latest stable Grok 3 version
- **Dated versions** (like `grok-2-image-1212`) → Specific release versions

## Recommendations for Terminal Commands

### Best Overall Performance
**grok-4** - Most intelligent, best for complex commands

### Best Speed-to-Intelligence Ratio
**grok-4-fast-reasoning** - Great balance for most terminal tasks

### Best for Simple Commands
**grok-4-fast-non-reasoning** - Fastest for straightforward tasks

### Best Cost-Effectiveness
**grok-code-fast-1** - Optimized pricing for coding/terminal tasks
**grok-3-mini** - Budget-friendly option

### Recommended Default
**grok-beta** - Usually points to latest stable, good balance

## Usage in Extension

All models are configured in `package.json` and can be selected via VSCode settings:

```json
{
  "terminalAI.provider": "xai",
  "terminalAI.xai.apiKey": "your-xai-api-key",
  "terminalAI.xai.model": "grok-4-fast-reasoning"
}
```

## Model Selection Guide

Choose based on your needs:

| Use Case | Recommended Model | Reason |
|----------|------------------|--------|
| General terminal commands | `grok-4-fast-reasoning` | Best balance of speed and intelligence |
| Simple file operations | `grok-4-fast-non-reasoning` | Fastest for straightforward tasks |
| Complex git workflows | `grok-4` | Highest intelligence for complex scenarios |
| Budget-conscious | `grok-code-fast-1` or `grok-3-mini` | Most cost-effective |
| Latest features | `grok-beta` | Access to newest capabilities |
| Stable production | `grok-4` or `grok-3` | Proven reliability |

## API Compatibility

All xAI models use OpenAI-compatible API format:
- Chat Completions endpoint
- Standard message format
- Temperature and response controls
- Streaming support (not yet implemented in this extension)

## Features Available

xAI models support:
- ✅ Text generation
- ✅ Code generation
- ✅ Real-time search (Grok 4)
- ✅ Tool use (Grok 4)
- ✅ Large context windows (2M tokens for Grok 4 fast variants)
- ❌ Image generation (separate model, not for terminal commands)

## Pricing Tiers

While exact pricing varies, generally:
- **Grok 4**: Premium pricing, highest capability
- **Grok 4 Fast**: Mid-tier pricing, optimized speed
- **Grok Code Fast 1**: Budget pricing for coding ($0.20/$1.50 per 1M tokens)
- **Grok 3**: Standard pricing
- **Grok 3 Mini**: Economy pricing

## Getting Your API Key

1. Visit https://console.x.ai/
2. Sign up or log in
3. Navigate to API Keys section
4. Generate a new API key
5. Add to VSCode settings: `terminalAI.xai.apiKey`

## API Endpoint

All models use the same base endpoint:
```
https://api.x.ai/v1/chat/completions
```

## Notes

- Models are continuously updated by xAI
- New versions may be released without notice
- Aliases (`grok-4`, `grok-beta`) automatically point to latest versions
- Check xAI console for current pricing and availability
- Some models may have usage limits or require special access

## Official Documentation

For the most current information:
- Official xAI API Docs: https://docs.x.ai/
- xAI Console: https://console.x.ai/
- xAI Website: https://x.ai/

---

Last Updated: January 2025
