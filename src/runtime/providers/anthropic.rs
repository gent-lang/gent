//! Anthropic API client

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::errors::{GentError, GentResult};
use crate::runtime::llm::{LLMClient, LLMResponse, Message, Role, ToolCall, ToolDefinition};

/// Anthropic API client
pub struct AnthropicClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub fn with_base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}
