//! Claude Code CLI client

use async_trait::async_trait;
use tokio::process::Command;

use crate::errors::{GentError, GentResult};
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

    /// Check that Claude Code CLI is available and authenticated
    pub async fn ensure_available(&self) -> GentResult<()> {
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
}

impl Default for ClaudeCodeClient {
    fn default() -> Self {
        Self::new().expect("Failed to create ClaudeCodeClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::GentError;

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
