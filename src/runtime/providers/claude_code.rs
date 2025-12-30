//! Claude Code CLI client

use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use tokio::process::Command;

use crate::errors::{GentError, GentResult};
use crate::runtime::llm::{LLMClient, LLMResponse, Message, Role, ToolDefinition};

/// Response from Claude CLI
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    result: Option<String>,
    #[serde(default)]
    is_error: bool,
}

/// Claude Code CLI client
pub struct ClaudeCodeClient {
    model: Option<String>,
    #[allow(dead_code)]
    cli_path: String,
    skip_permissions: bool,
}

impl ClaudeCodeClient {
    /// Create a new Claude Code client
    pub fn new() -> GentResult<Self> {
        Ok(Self {
            model: None,
            cli_path: "claude".to_string(),
            skip_permissions: false,
        })
    }

    /// Set the model to use
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// Set whether to skip permission prompts (dangerous!)
    pub fn with_skip_permissions(mut self, skip: bool) -> Self {
        self.skip_permissions = skip;
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

    /// Extract system prompt from messages (first system message)
    fn extract_system_prompt(&self, messages: &[Message]) -> Option<String> {
        messages
            .iter()
            .find(|m| matches!(m.role, Role::System))
            .map(|m| m.content.clone())
    }

    /// Build user prompt from non-system messages
    fn build_user_prompt(&self, messages: &[Message]) -> String {
        let mut parts = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => {
                    // Handled separately via --system-prompt
                }
                Role::User => {
                    parts.push(msg.content.clone());
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
        messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
        model: Option<&str>,
        _json_mode: bool,
    ) -> GentResult<LLMResponse> {
        self.ensure_available().await?;

        // Extract system prompt and user prompt separately
        let system_prompt = self.extract_system_prompt(&messages);
        let user_prompt = self.build_user_prompt(&messages);

        // Build CLI args
        let mut args = vec!["--print", "--output-format", "json"];

        // Add --dangerously-skip-permissions if enabled (allows unattended execution)
        if self.skip_permissions {
            args.push("--dangerously-skip-permissions");
        }

        let model_to_use = model.or(self.model.as_deref());
        let model_string;
        if let Some(m) = model_to_use {
            model_string = m.to_string();
            args.push("--model");
            args.push(&model_string);
        }

        // Add system prompt if present
        let system_prompt_string;
        if let Some(ref sp) = system_prompt {
            system_prompt_string = sp.clone();
            args.push("--system-prompt");
            args.push(&system_prompt_string);
        }

        // Add user prompt as positional argument (must be last)
        // Only add if not empty - Claude CLI requires input when using --print
        if user_prompt.trim().is_empty() {
            return Err(GentError::ProviderError {
                message: "No user prompt provided. Claude Code requires a prompt.".to_string(),
            });
        }
        args.push(&user_prompt);

        // Spawn CLI process (no timeout - let Claude Code run as long as needed)
        let output = Command::new(&self.cli_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
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
