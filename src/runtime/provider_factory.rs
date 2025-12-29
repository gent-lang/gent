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
            other => Err(GentError::ProviderError {
                message: format!("Unknown provider '{}'. Supported: openai, claude-code", other),
            }),
        }
    }
}
