# Anthropic Provider Support

## Summary

Add Claude/Anthropic as an LLM provider alongside OpenAI, with automatic provider detection based on model name.

## Provider Detection

Auto-detect provider from model name prefix:
- `claude*` → Anthropic
- `gpt*`, `o1*`, `o3*` → OpenAI
- Anything else → Error with helpful message

```rust
pub enum Provider {
    OpenAI,
    Anthropic,
}

pub fn detect_provider(model: &str) -> Result<Provider, GentError> {
    if model.starts_with("claude") {
        Ok(Provider::Anthropic)
    } else if model.starts_with("gpt")
           || model.starts_with("o1")
           || model.starts_with("o3") {
        Ok(Provider::OpenAI)
    } else {
        Err(GentError::UnknownProvider { model: model.to_string() })
    }
}
```

Error message: "Unknown model provider for '{model}'. Use claude-* for Anthropic or gpt-*/o1-*/o3-* for OpenAI."

## AnthropicClient

New file `src/runtime/providers/anthropic.rs`:

```rust
pub struct AnthropicClient {
    api_key: String,
    model: String,
    base_url: String,  // https://api.anthropic.com
    client: reqwest::Client,
}
```

Implementation details:
- **System prompt**: Extract `Role::System` messages from array, concatenate into `system` param
- **Message format**: Convert string content to `[{"type": "text", "text": "..."}]`
- **Tools**: Use `input_schema` instead of `parameters`
- **Headers**: `x-api-key` and `anthropic-version: 2023-06-01`

No changes to `LLMClient` trait - different serialization handled internally.

## Per-Agent Client Creation

Each agent creates its own LLM client based on its `model` field:

```rust
pub fn create_llm_client(model: &str, config: &Config) -> Result<Box<dyn LLMClient>, GentError> {
    match detect_provider(model)? {
        Provider::OpenAI => {
            let key = config.require_openai_key()?;
            Ok(Box::new(OpenAIClient::new(key.to_string()).with_model(model)))
        }
        Provider::Anthropic => {
            let key = config.require_anthropic_key()?;
            Ok(Box::new(AnthropicClient::new(key.to_string()).with_model(model)))
        }
    }
}
```

Flow changes:
- `main.rs` passes `Config` instead of pre-built client
- `run_agent_with_tools` creates client based on agent's model
- `--mock` flag bypasses detection, uses `MockLLMClient` for all agents
- Agents without `model` field will error

## Files to Change

**Modify:**
- `src/runtime/providers/mod.rs` - Provider enum, detect_provider(), export AnthropicClient
- `src/errors/mod.rs` - UnknownProvider error variant
- `src/config.rs` - require_anthropic_key() method
- `src/main.rs` - Pass config, handle mock mode
- `src/interpreter/evaluator.rs` - Update signature to take &Config
- `src/runtime/agent.rs` - Create client per-agent

**New:**
- `src/runtime/providers/anthropic.rs` - Full client implementation

**Unchanged:**
- `src/runtime/llm.rs` - Trait stays the same
- Grammar/parser - No syntax changes

## Testing

- Unit test `detect_provider()` with various model strings
- Integration tests with `--mock` flag (existing tests unchanged)
- Manual test with real `ANTHROPIC_API_KEY`
