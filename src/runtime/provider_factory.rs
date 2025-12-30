//! LLM provider factory

use crate::config::Config;
use crate::errors::{GentError, GentResult};
use crate::runtime::{ClaudeCodeClient, LLMClient, MockLLMClient, OpenAIClient, ToolCall};

/// Factory for creating LLM clients based on provider name
pub struct ProviderFactory {
    config: Config,
    use_mock: bool,
    mock_response: Option<String>,
    mock_tool_calls: Option<Vec<ToolCall>>,
}

impl ProviderFactory {
    /// Create a new provider factory
    pub fn new(config: Config) -> Self {
        Self {
            config,
            use_mock: false,
            mock_response: None,
            mock_tool_calls: None,
        }
    }

    /// Create a factory that returns mock clients
    pub fn mock() -> Self {
        Self {
            config: Config::default(),
            use_mock: true,
            mock_response: None,
            mock_tool_calls: None,
        }
    }

    /// Create a factory that returns mock clients with custom response
    pub fn mock_with_response(response: impl Into<String>) -> Self {
        Self {
            config: Config::default(),
            use_mock: true,
            mock_response: Some(response.into()),
            mock_tool_calls: None,
        }
    }

    /// Create a factory that returns mock clients with tool calls
    pub fn mock_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            config: Config::default(),
            use_mock: true,
            mock_response: None,
            mock_tool_calls: Some(tool_calls),
        }
    }

    /// Create an LLM client for the given provider
    pub fn create(&self, provider: Option<&str>) -> GentResult<Box<dyn LLMClient>> {
        self.create_with_options(provider, false)
    }

    /// Create an LLM client with additional options
    pub fn create_with_options(
        &self,
        provider: Option<&str>,
        dangerously_skip_permissions: bool,
    ) -> GentResult<Box<dyn LLMClient>> {
        if self.use_mock {
            return Ok(Box::new(if let Some(ref tool_calls) = self.mock_tool_calls {
                MockLLMClient::with_tool_calls(tool_calls.clone())
            } else if let Some(ref response) = self.mock_response {
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
            "claude-code" => {
                let mut client = ClaudeCodeClient::new()?;
                if dangerously_skip_permissions {
                    client = client.with_skip_permissions(true);
                }
                Ok(Box::new(client))
            }
            other => Err(GentError::ProviderError {
                message: format!("Unknown provider '{}'. Supported: openai, claude-code", other),
            }),
        }
    }
}
