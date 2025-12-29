//! LLM client factory

use crate::config::Config;
use crate::errors::GentResult;
use crate::runtime::llm::LLMClient;
use crate::runtime::providers::{detect_provider, AnthropicClient, OpenAIClient, Provider};

/// Create an LLM client based on model name
pub fn create_llm_client(model: &str, config: &Config) -> GentResult<Box<dyn LLMClient>> {
    match detect_provider(model)? {
        Provider::OpenAI => {
            let key = config.require_openai_key()?;
            Ok(Box::new(
                OpenAIClient::new(key.to_string()).with_model(model),
            ))
        }
        Provider::Anthropic => {
            let key = config.require_anthropic_key()?;
            Ok(Box::new(
                AnthropicClient::new(key.to_string()).with_model(model),
            ))
        }
    }
}
