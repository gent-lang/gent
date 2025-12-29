//! LLM provider implementations

mod openai;

pub use openai::OpenAIClient;

use crate::errors::GentError;

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    Anthropic,
}

/// Detect the provider from a model name
pub fn detect_provider(model: &str) -> Result<Provider, GentError> {
    if model.starts_with("claude") {
        Ok(Provider::Anthropic)
    } else if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") {
        Ok(Provider::OpenAI)
    } else {
        Err(GentError::UnknownProvider {
            model: model.to_string(),
        })
    }
}
