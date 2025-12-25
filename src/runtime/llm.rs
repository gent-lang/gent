//! LLM client abstraction for GENT

use crate::errors::GentResult;

/// Role in a chat conversation
#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    /// System message (sets agent behavior)
    System,
    /// User message (input)
    User,
    /// Assistant message (LLM response)
    Assistant,
}

/// A message in a chat conversation
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Content of the message
    pub content: String,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }

    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(Role::System, content)
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(Role::User, content)
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(Role::Assistant, content)
    }
}

/// Response from an LLM
#[derive(Debug, Clone, PartialEq)]
pub struct LLMResponse {
    /// The response content
    pub content: String,
}

impl LLMResponse {
    /// Create a new LLM response
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
        }
    }
}

/// Trait for LLM clients
pub trait LLMClient: Send + Sync {
    /// Send a chat request to the LLM
    fn chat(&self, messages: Vec<Message>) -> GentResult<LLMResponse>;
}

/// Mock LLM client for testing
#[derive(Debug, Clone)]
pub struct MockLLMClient {
    /// The response to return
    response: String,
}

impl MockLLMClient {
    /// Create a new mock client with default response
    pub fn new() -> Self {
        Self {
            response: "Hello! I'm a friendly assistant. How can I help you today?".to_string(),
        }
    }

    /// Create a mock client with a custom response
    pub fn with_response(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
        }
    }

    /// Get the configured response
    pub fn response(&self) -> &str {
        &self.response
    }
}

impl Default for MockLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LLMClient for MockLLMClient {
    fn chat(&self, _messages: Vec<Message>) -> GentResult<LLMResponse> {
        Ok(LLMResponse::new(&self.response))
    }
}
