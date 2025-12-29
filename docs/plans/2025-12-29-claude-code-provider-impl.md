# Claude Code Provider Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Claude Code CLI as an LLM provider in Gent, enabling per-agent provider selection and subscription-based usage.

**Architecture:** Create `ClaudeCodeClient` implementing `LLMClient` trait, add `provider` field to `AgentValue`, and refactor runtime to use a provider factory instead of a single LLM client.

**Tech Stack:** Rust, tokio (async), serde_json (parsing CLI output)

---

## Task 1: Create ClaudeCodeClient Skeleton

**Files:**
- Create: `src/runtime/providers/claude_code.rs`
- Modify: `src/runtime/providers/mod.rs`

**Step 1: Create the provider file with struct**

```rust
// src/runtime/providers/claude_code.rs
//! Claude Code CLI client

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::errors::{GentError, GentResult};
use crate::runtime::llm::{LLMClient, LLMResponse, Message, Role, ToolCall, ToolDefinition};

/// Claude Code CLI client
pub struct ClaudeCodeClient {
    model: Option<String>,
    cli_path: String,
}

impl ClaudeCodeClient {
    /// Create a new Claude Code client
    pub fn new() -> GentResult<Self> {
        Ok(Self {
            model: None,
            cli_path: "claude".to_string(),
        })
    }

    /// Set the model to use
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }
}

impl Default for ClaudeCodeClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ClaudeCodeClient")
    }
}

#[async_trait]
impl LLMClient for ClaudeCodeClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
        model: Option<&str>,
        json_mode: bool,
    ) -> GentResult<LLMResponse> {
        // TODO: Implement in Task 2
        Ok(LLMResponse::new("Claude Code provider not yet implemented"))
    }
}
```

**Step 2: Export from providers module**

In `src/runtime/providers/mod.rs`, add:

```rust
mod claude_code;

pub use claude_code::ClaudeCodeClient;
```

**Step 3: Run build to verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/runtime/providers/claude_code.rs src/runtime/providers/mod.rs
git commit -m "feat: add ClaudeCodeClient skeleton"
```

---

## Task 2: Implement CLI Availability Check

**Files:**
- Modify: `src/runtime/providers/claude_code.rs`

**Step 1: Write test for CLI check**

Add to bottom of `claude_code.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_cli_not_found() {
        let client = ClaudeCodeClient {
            model: None,
            cli_path: "nonexistent-claude-binary".to_string(),
        };
        let result = client.ensure_available().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, GentError::ProviderError { .. }));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_check_cli_not_found`
Expected: FAIL with "ensure_available not found"

**Step 3: Implement ensure_available method**

Add to `impl ClaudeCodeClient`:

```rust
    /// Check that Claude Code CLI is available and authenticated
    async fn ensure_available(&self) -> GentResult<()> {
        // Check binary exists
        let version_result = Command::new(&self.cli_path)
            .arg("--version")
            .output()
            .await;

        match version_result {
            Ok(output) if output.status.success() => {}
            Ok(_) => {
                return Err(GentError::ProviderError {
                    message: "Claude Code CLI not working properly".to_string(),
                });
            }
            Err(_) => {
                return Err(GentError::ProviderError {
                    message: "Claude Code CLI not found. Install with: npm install -g @anthropic-ai/claude-code".to_string(),
                });
            }
        }

        Ok(())
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_check_cli_not_found`
Expected: PASS

**Step 5: Commit**

```bash
git add src/runtime/providers/claude_code.rs
git commit -m "feat: add ClaudeCodeClient CLI availability check"
```

---

## Task 3: Implement CLI Invocation

**Files:**
- Modify: `src/runtime/providers/claude_code.rs`

**Step 1: Add response parsing structs**

Add after the imports in `claude_code.rs`:

```rust
/// Response from Claude CLI
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    result: Option<String>,
    #[serde(default)]
    is_error: bool,
}
```

**Step 2: Implement the chat method**

Replace the TODO in the `chat` method:

```rust
#[async_trait]
impl LLMClient for ClaudeCodeClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
        model: Option<&str>,
        _json_mode: bool,
    ) -> GentResult<LLMResponse> {
        self.ensure_available().await?;

        // Build prompt from messages
        let prompt = self.build_prompt(&messages);

        // Build CLI args
        let mut args = vec!["--print", "--output-format", "json"];

        let model_to_use = model.or(self.model.as_deref());
        let model_string;
        if let Some(m) = model_to_use {
            model_string = m.to_string();
            args.push("--model");
            args.push(&model_string);
        }

        // Add prompt as argument
        args.push("--prompt");
        args.push(&prompt);

        // Spawn CLI process
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(300), // 5 minute timeout
            Command::new(&self.cli_path)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output(),
        )
        .await
        .map_err(|_| GentError::ProviderError {
            message: "Claude CLI timed out after 5 minutes".to_string(),
        })?
        .map_err(|e| GentError::ProviderError {
            message: format!("Failed to run Claude CLI: {}", e),
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GentError::ProviderError {
                message: format!("Claude CLI failed: {}", stderr),
            });
        }

        // Parse response
        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_response(&stdout)
    }
}
```

**Step 3: Add helper methods**

Add to `impl ClaudeCodeClient`:

```rust
    /// Build a prompt string from messages
    fn build_prompt(&self, messages: &[Message]) -> String {
        let mut parts = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    parts.push(format!("[System]\n{}", msg.content));
                }
                Role::User => {
                    parts.push(format!("[User]\n{}", msg.content));
                }
                Role::Assistant => {
                    if !msg.content.is_empty() {
                        parts.push(format!("[Assistant]\n{}", msg.content));
                    }
                }
                Role::Tool => {
                    parts.push(format!("[Tool Result]\n{}", msg.content));
                }
            }
        }

        parts.join("\n\n")
    }

    /// Parse Claude CLI response
    fn parse_response(&self, output: &str) -> GentResult<LLMResponse> {
        // Try to parse as JSON first
        if let Ok(response) = serde_json::from_str::<ClaudeResponse>(output) {
            if response.is_error {
                return Err(GentError::ProviderError {
                    message: response.result.unwrap_or_else(|| "Unknown error".to_string()),
                });
            }
            return Ok(LLMResponse::new(response.result.unwrap_or_default()));
        }

        // If not JSON, treat as plain text response
        Ok(LLMResponse::new(output.trim()))
    }
```

**Step 4: Run build to verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/runtime/providers/claude_code.rs
git commit -m "feat: implement ClaudeCodeClient CLI invocation"
```

---

## Task 4: Add Provider Field to AgentValue

**Files:**
- Modify: `src/interpreter/types.rs`

**Step 1: Add provider field to AgentValue struct**

In `src/interpreter/types.rs`, find the `AgentValue` struct (around line 306) and add the field:

```rust
pub struct AgentValue {
    pub name: String,
    pub system_prompt: String,
    pub user_prompt: Option<String>,
    pub tools: Vec<String>,
    pub knowledge_config: Option<KnowledgeConfig>,
    pub max_steps: Option<u32>,
    pub model: Option<String>,
    pub provider: Option<String>,  // NEW: Provider to use (openai, claude-code)
    pub output_schema: Option<OutputSchema>,
    pub output_retries: u32,
    pub output_instructions: Option<String>,
    pub retry_prompt: Option<String>,
}
```

**Step 2: Update the new() constructor**

Add `provider: None,` to the `Self` block in `AgentValue::new()`:

```rust
impl AgentValue {
    pub fn new(name: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            system_prompt: system_prompt.into(),
            user_prompt: None,
            tools: Vec::new(),
            knowledge_config: None,
            max_steps: None,
            model: None,
            provider: None,  // NEW
            output_schema: None,
            output_retries: 1,
            output_instructions: None,
            retry_prompt: None,
        }
    }
```

**Step 3: Add with_provider builder method**

Add after `with_model`:

```rust
    /// Set provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
```

**Step 4: Run tests to verify nothing broke**

Run: `cargo test`
Expected: All tests pass

**Step 5: Commit**

```bash
git add src/interpreter/types.rs
git commit -m "feat: add provider field to AgentValue"
```

---

## Task 5: Parse Provider Field in Evaluator

**Files:**
- Modify: `src/interpreter/evaluator.rs`

**Step 1: Find the agent field parsing section**

Around line 637 in `src/interpreter/evaluator.rs`, find the match statement for agent fields.

**Step 2: Add provider field handling**

Add a new case after the `"model"` case (around line 649):

```rust
            "provider" => {
                let value = evaluate_expr(&field.value, env)?;
                provider = Some(match value {
                    Value::String(s) => {
                        // Validate provider name
                        match s.as_str() {
                            "openai" | "claude-code" => s,
                            _ => {
                                return Err(GentError::ConfigError {
                                    message: format!(
                                        "Unknown provider '{}'. Supported: openai, claude-code",
                                        s
                                    ),
                                })
                            }
                        }
                    }
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "String".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
```

**Step 3: Declare provider variable**

Find where `model` is declared (around line 582) and add:

```rust
    let mut provider: Option<String> = None;
```

**Step 4: Use provider when building AgentValue**

Find where `AgentValue` is constructed and update to include provider. Look for the pattern building the agent around line 780-800 and ensure provider is included.

**Step 5: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/interpreter/evaluator.rs
git commit -m "feat: parse provider field in agent declarations"
```

---

## Task 6: Create LLM Provider Factory

**Files:**
- Create: `src/runtime/provider_factory.rs`
- Modify: `src/runtime/mod.rs`

**Step 1: Create provider factory**

```rust
// src/runtime/provider_factory.rs
//! LLM provider factory

use crate::config::Config;
use crate::errors::{GentError, GentResult};
use crate::runtime::{ClaudeCodeClient, LLMClient, MockLLMClient, OpenAIClient};

/// Factory for creating LLM clients based on provider name
pub struct ProviderFactory {
    config: Config,
    use_mock: bool,
    mock_response: Option<String>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new(config: Config) -> Self {
        Self {
            config,
            use_mock: false,
            mock_response: None,
        }
    }

    /// Create a factory that returns mock clients
    pub fn mock() -> Self {
        Self {
            config: Config::default(),
            use_mock: true,
            mock_response: None,
        }
    }

    /// Create a factory that returns mock clients with custom response
    pub fn mock_with_response(response: impl Into<String>) -> Self {
        Self {
            config: Config::default(),
            use_mock: true,
            mock_response: Some(response.into()),
        }
    }

    /// Create an LLM client for the given provider
    pub fn create(&self, provider: Option<&str>) -> GentResult<Box<dyn LLMClient>> {
        if self.use_mock {
            return Ok(Box::new(if let Some(ref response) = self.mock_response {
                MockLLMClient::with_response(response)
            } else {
                MockLLMClient::new()
            }));
        }

        match provider.unwrap_or("openai") {
            "openai" => {
                let api_key = self.config.require_openai_key()?;
                Ok(Box::new(OpenAIClient::new(api_key.to_string())))
            }
            "claude-code" => Ok(Box::new(ClaudeCodeClient::new()?)),
            other => Err(GentError::ConfigError {
                message: format!("Unknown provider '{}'. Supported: openai, claude-code", other),
            }),
        }
    }
}
```

**Step 2: Export from runtime module**

In `src/runtime/mod.rs`, add:

```rust
mod provider_factory;

pub use provider_factory::ProviderFactory;
```

**Step 3: Run build**

Run: `cargo build`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/runtime/provider_factory.rs src/runtime/mod.rs
git commit -m "feat: add ProviderFactory for creating LLM clients"
```

---

## Task 7: Update Agent Runtime to Use Factory

**Files:**
- Modify: `src/runtime/agent.rs`

**Step 1: Add factory parameter to run_agent_with_tools**

Change the signature:

```rust
pub async fn run_agent_with_tools(
    agent: &AgentValue,
    input: Option<String>,
    provider_factory: &ProviderFactory,
    tools: &ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<String> {
```

**Step 2: Create client from factory inside the function**

At the start of the function body, add:

```rust
    let llm = provider_factory.create(agent.provider.as_deref())?;
    let llm = llm.as_ref();
```

**Step 3: Update run_agent function**

Change signature similarly and update to use factory.

**Step 4: Run build (will fail - that's expected)**

Run: `cargo build`
Expected: Compilation errors in files that call these functions

**Step 5: Commit work in progress**

```bash
git add src/runtime/agent.rs
git commit -m "wip: update agent runtime to use ProviderFactory"
```

---

## Task 8: Update Interpreter to Use Factory

**Files:**
- Modify: `src/interpreter/evaluator.rs`
- Modify: `src/interpreter/block_eval.rs`

**Step 1: Update EvalContext to use ProviderFactory**

Change `llm: &'a dyn LLMClient` to `provider_factory: &'a ProviderFactory` in all relevant structs.

**Step 2: Update evaluate_with_output signature**

```rust
pub async fn evaluate_with_output(
    program: &Program,
    provider_factory: &ProviderFactory,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<Vec<String>> {
```

**Step 3: Update all call sites**

Fix all compilation errors by passing `provider_factory` instead of `llm`.

**Step 4: Run build**

Run: `cargo build`
Expected: Compiles without errors

**Step 5: Commit**

```bash
git add src/interpreter/evaluator.rs src/interpreter/block_eval.rs
git commit -m "feat: update interpreter to use ProviderFactory"
```

---

## Task 9: Update main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Update imports**

Change:
```rust
use gent::runtime::{MockLLMClient, OpenAIClient, ToolRegistry};
```
To:
```rust
use gent::runtime::{ProviderFactory, ToolRegistry};
```

**Step 2: Update run function**

Replace the LLM client creation logic:

```rust
    let mut tools = ToolRegistry::with_builtins();

    let provider_factory = if cli.mock {
        logger.log(LogLevel::Info, "cli", "Using mock LLM");
        if let Some(response) = &cli.mock_response {
            ProviderFactory::mock_with_response(response)
        } else {
            ProviderFactory::mock()
        }
    } else {
        let config = Config::load();
        logger.log(LogLevel::Debug, "cli", "Using configured providers");
        ProviderFactory::new(config)
    };

    let outputs = evaluate_with_output(&program, &provider_factory, &mut tools, logger).await?;
```

**Step 3: Run build and tests**

Run: `cargo build && cargo test`
Expected: All pass

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: update CLI to use ProviderFactory"
```

---

## Task 10: Add Integration Test

**Files:**
- Create: `tests/claude_code_provider.rs`

**Step 1: Create test file**

```rust
//! Tests for Claude Code provider

use gent::interpreter::evaluate;
use gent::parser::parse;
use gent::runtime::{ProviderFactory, ToolRegistry};
use gent::logging::NullLogger;

#[tokio::test]
async fn test_agent_with_provider_field() {
    let source = r#"
        agent TestAgent {
            model: "gpt-4"
            provider: "openai"
            instructions: "You are helpful"
        }
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();
    let logger = NullLogger;

    let result = evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_claude_code_provider() {
    let source = r#"
        agent TestAgent {
            model: "claude-sonnet-4"
            provider: "claude-code"
            instructions: "You are helpful"
        }
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();  // Uses mock even for claude-code
    let mut tools = ToolRegistry::with_builtins();
    let logger = NullLogger;

    let result = evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_provider() {
    let source = r#"
        agent TestAgent {
            model: "gpt-4"
            provider: "invalid-provider"
            instructions: "You are helpful"
        }
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();
    let logger = NullLogger;

    let result = evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err());
}
```

**Step 2: Run tests**

Run: `cargo test claude_code_provider`
Expected: All 3 tests pass

**Step 3: Commit**

```bash
git add tests/claude_code_provider.rs
git commit -m "test: add integration tests for Claude Code provider"
```

---

## Task 11: Update lib.rs Exports

**Files:**
- Modify: `src/lib.rs`

**Step 1: Ensure ProviderFactory is exported**

Verify `src/lib.rs` re-exports `ProviderFactory` from runtime:

```rust
pub use runtime::{ProviderFactory, ...};
```

**Step 2: Run full test suite**

Run: `cargo test`
Expected: All tests pass

**Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "chore: export ProviderFactory from lib"
```

---

## Task 12: Add Example File

**Files:**
- Create: `examples/claude_code_agent.gnt`

**Step 1: Create example**

```gent
// Example using Claude Code provider
// Requires: claude login (Claude Code CLI authenticated)

agent Researcher {
    model: "claude-sonnet-4"
    provider: "claude-code"
    instructions: "You are a research assistant. Help the user find information."
}

let result = Researcher("What is the capital of France?")
print(result)
```

**Step 2: Commit**

```bash
git add examples/claude_code_agent.gnt
git commit -m "docs: add Claude Code provider example"
```

---

## Task 13: Run Full Test Suite and Final Verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: All 937+ tests pass

**Step 2: Run clippy**

Run: `cargo clippy`
Expected: No warnings (or only pre-existing ones)

**Step 3: Test with mock**

Run: `cargo run -- examples/claude_code_agent.gnt --mock`
Expected: Runs successfully with mock response

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete Claude Code provider implementation"
```

---

## Summary

After completing all tasks, you'll have:

1. `ClaudeCodeClient` - A new LLM provider using Claude Code CLI
2. `provider` field in agent declarations
3. `ProviderFactory` - Dynamic provider selection at runtime
4. Per-agent provider selection (`provider: "claude-code"`)
5. Integration tests covering the new functionality
6. Example file demonstrating usage
