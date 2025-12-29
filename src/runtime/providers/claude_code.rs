//! Claude Code CLI client

use async_trait::async_trait;

use crate::errors::GentResult;
use crate::runtime::llm::{LLMClient, LLMResponse, Message, ToolDefinition};

/// Claude Code CLI client
pub struct ClaudeCodeClient {
    model: Option<String>,
    #[allow(dead_code)]
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
        _messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
        _model: Option<&str>,
        _json_mode: bool,
    ) -> GentResult<LLMResponse> {
        // TODO: Implement in Task 2
        Ok(LLMResponse::new("Claude Code provider not yet implemented"))
    }
}
