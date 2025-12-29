# Claude Code Provider Design

**Date:** 2025-12-29
**Status:** Draft
**Goal:** Add Claude Code CLI as an LLM provider in Gent, enabling subscription-based usage (Pro/Max) without API keys.

## Overview

Gent currently only supports OpenAI as an LLM provider. This design adds support for Claude Code CLI as an alternative provider, allowing users to leverage their Claude Pro/Max subscription for running Gent agents.

## Approach

Use the Claude Code CLI (`claude`) as a wrapper rather than integrating the Claude Agent SDK directly. This is simpler to implement in Rust and leverages existing authentication.

**Why CLI over SDK:**
- Users with subscriptions already have Claude Code installed
- No FFI or subprocess IPC complexity with Node.js/Python
- Auth "just works" via existing `claude login` session
- Simpler Rust implementation

## CLI Interface

The `claude` CLI supports non-interactive mode:

```bash
echo "Your prompt" | claude --print --output-format json
```

Returns structured JSON with either text content or tool calls.

## Implementation

### 1. New Provider File

**Location:** `src/runtime/providers/claude_code.rs`

```rust
pub struct ClaudeCodeClient {
    model: Option<String>,
    cli_path: String,
    mock_response: Option<String>,  // For testing
}

impl ClaudeCodeClient {
    pub fn new() -> GentResult<Self> {
        let client = Self {
            model: None,
            cli_path: "claude".to_string(),
            mock_response: None,
        };
        // Validation happens on first use, not construction
        Ok(client)
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    pub fn mock(response: &str) -> Self {
        Self {
            model: None,
            cli_path: "claude".to_string(),
            mock_response: Some(response.to_string()),
        }
    }

    async fn ensure_available(&self) -> GentResult<()> {
        // Check CLI exists
        let version = Command::new(&self.cli_path)
            .arg("--version")
            .output()
            .map_err(|_| GentError::ProviderError {
                message: "Claude Code CLI not found. Install: npm install -g @anthropic-ai/claude-code".into()
            })?;

        if !version.status.success() {
            return Err(GentError::ProviderError {
                message: "Claude Code CLI not working properly".into()
            });
        }

        // Check authentication
        let auth = Command::new(&self.cli_path)
            .args(["auth", "status"])
            .output()?;

        if !auth.status.success() {
            return Err(GentError::ProviderError {
                message: "Not authenticated. Run: claude login".into()
            });
        }

        Ok(())
    }
}
```

### 2. LLMClient Implementation

```rust
#[async_trait]
impl LLMClient for ClaudeCodeClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
        model: Option<&str>,
        json_mode: bool,
    ) -> GentResult<LLMResponse> {
        // Return mock if configured
        if let Some(ref response) = self.mock_response {
            return Ok(LLMResponse::new(response));
        }

        // Ensure CLI is available
        self.ensure_available().await?;

        // Build prompt from messages
        let prompt = self.build_prompt(&messages, &tools, json_mode);

        // Build CLI args
        let mut args = vec!["--print", "--output-format", "json"];

        if let Some(m) = model.or(self.model.as_deref()) {
            args.extend(["--model", m]);
        }

        // Spawn CLI process
        let mut child = Command::new(&self.cli_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| GentError::ProviderError {
                message: format!("Failed to spawn claude: {}", e)
            })?;

        // Write prompt to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await?;
        }

        // Wait for output with timeout
        let output = tokio::time::timeout(
            Duration::from_secs(300),  // 5 minute timeout
            child.wait_with_output()
        ).await
            .map_err(|_| GentError::ProviderError {
                message: "Claude CLI timed out".into()
            })?
            .map_err(|e| GentError::ProviderError {
                message: format!("CLI error: {}", e)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GentError::ProviderError {
                message: format!("Claude CLI failed: {}", stderr)
            });
        }

        // Parse JSON response
        self.parse_response(&output.stdout)
    }
}
```

### 3. Response Parsing

Claude CLI JSON output format:

```json
{
  "content": "Text response here",
  "tool_use": [
    {
      "id": "tool_abc123",
      "name": "web_search",
      "input": {"query": "rust async"}
    }
  ]
}
```

```rust
#[derive(Deserialize)]
struct ClaudeResponse {
    content: Option<String>,
    tool_use: Option<Vec<ClaudeToolUse>>,
}

#[derive(Deserialize)]
struct ClaudeToolUse {
    id: String,
    name: String,
    input: JsonValue,
}

impl ClaudeCodeClient {
    fn parse_response(&self, output: &[u8]) -> GentResult<LLMResponse> {
        let response: ClaudeResponse = serde_json::from_slice(output)
            .map_err(|e| GentError::ProviderError {
                message: format!("Failed to parse CLI response: {}", e)
            })?;

        let tool_calls: Vec<ToolCall> = response.tool_use
            .unwrap_or_default()
            .into_iter()
            .map(|t| ToolCall {
                id: t.id,
                name: t.name,
                arguments: t.input,
            })
            .collect();

        if !tool_calls.is_empty() {
            Ok(LLMResponse::with_tool_calls(tool_calls))
        } else {
            Ok(LLMResponse::new(response.content.unwrap_or_default()))
        }
    }
}
```

### 4. Provider Selection

**Grammar change** (if needed): Add `provider` field to agent definition.

**Interpreter change:** Select client based on provider:

```rust
fn create_llm_client(provider: Option<&str>, model: Option<&str>) -> GentResult<Box<dyn LLMClient>> {
    match provider.unwrap_or("openai") {
        "openai" => {
            let api_key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| GentError::ConfigError {
                    message: "OPENAI_API_KEY not set".into()
                })?;
            let mut client = OpenAIClient::new(api_key);
            if let Some(m) = model {
                client = client.with_model(m);
            }
            Ok(Box::new(client))
        }
        "claude-code" => {
            let mut client = ClaudeCodeClient::new()?;
            if let Some(m) = model {
                client = client.with_model(m);
            }
            Ok(Box::new(client))
        }
        other => Err(GentError::ConfigError {
            message: format!("Unknown provider: {}. Supported: openai, claude-code", other)
        }),
    }
}
```

### 5. Gent Syntax

```gent
agent Researcher {
    provider: "claude-code"
    model: "claude-sonnet-4"  // Optional
    instructions: "You are a research assistant"
    tools: [web_search]
}

agent Coder {
    provider: "openai"  // Default if omitted
    model: "gpt-4o"
    instructions: "You are a coding assistant"
}
```

## Error Handling

| Error | Detection | User Message |
|-------|-----------|--------------|
| CLI not found | `claude --version` fails | "Claude Code CLI not found. Install: `npm install -g @anthropic-ai/claude-code`" |
| Not authenticated | `claude auth status` fails | "Not authenticated. Run: `claude login`" |
| Rate limited | CLI stderr contains rate limit | "Claude subscription rate limit reached. Limits reset every 5 hours." |
| Timeout | No response in 5 minutes | "Claude CLI timed out after 5 minutes" |

## Testing

### Unit Tests
- Mock CLI responses for JSON parsing
- Test error conditions (missing CLI, auth failures)
- Test tool call conversion

### Integration Tests
- `ClaudeCodeClient::mock()` for deterministic responses
- Mirror existing `--mock` flag behavior

### Test File
`tests/claude_code_provider.rs`

## Files to Change

1. **New:** `src/runtime/providers/claude_code.rs` - Provider implementation
2. **Modify:** `src/runtime/providers/mod.rs` - Export new provider
3. **Modify:** `src/lexer/grammar.pest` - Add `provider` field (if not present)
4. **Modify:** `src/parser/ast.rs` - Add provider to AgentDef
5. **Modify:** `src/interpreter/evaluator.rs` - Provider selection logic
6. **New:** `tests/claude_code_provider.rs` - Tests

## Open Questions

1. **Tool definition format:** Need to verify exact CLI flags for passing custom tools
2. **Conversation continuity:** How to pass multi-turn conversations via CLI?
3. **Streaming:** CLI supports streaming, but is it needed for Gent initially?

## Future Enhancements

- Direct Claude Agent SDK integration (TypeScript/Python subprocess)
- Anthropic API provider (for API key users who want Claude without CLI)
- Provider auto-detection based on available credentials
