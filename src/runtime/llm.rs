//! LLM client abstraction for GENT

use crate::errors::GentResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Role in a chat conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Role {
    /// System message (sets agent behavior)
    System,
    /// User message (input)
    User,
    /// Assistant message (LLM response)
    Assistant,
    /// Tool result message
    Tool,
}

/// A tool call made by the LLM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Name of the tool to call
    pub name: String,
    /// Arguments to pass to the tool (as JSON)
    pub arguments: JsonValue,
}

/// Result from a tool execution
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    /// ID of the tool call this result corresponds to
    pub call_id: String,
    /// Content returned by the tool
    pub content: String,
    /// Whether this is an error result
    pub is_error: bool,
}

/// Definition of a tool that can be called
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Name of the tool
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON Schema for the tool's parameters
    pub parameters: JsonValue,
}

/// A message in a chat conversation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: Role,
    /// Content of the message
    pub content: String,
    /// Tool call ID (only for Role::Tool messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Tool calls (only for Role::Assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Message {
    /// Create a new message
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            tool_call_id: None,
            tool_calls: None,
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

    /// Create a tool result message
    pub fn tool_result(result: ToolResult) -> Self {
        Self {
            role: Role::Tool,
            content: result.content,
            tool_call_id: Some(result.call_id),
            tool_calls: None,
        }
    }

    /// Create an assistant message with tool calls
    pub fn assistant_with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: Role::Assistant,
            content: String::new(),
            tool_call_id: None,
            tool_calls: Some(tool_calls),
        }
    }
}

/// Response from an LLM
#[derive(Debug, Clone, PartialEq)]
pub struct LLMResponse {
    /// The response content (optional if tool calls are present)
    pub content: Option<String>,
    /// Tool calls requested by the LLM
    pub tool_calls: Vec<ToolCall>,
}

impl LLMResponse {
    /// Create a new LLM response with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            tool_calls: vec![],
        }
    }

    /// Create a response with tool calls
    pub fn with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: None,
            tool_calls,
        }
    }

    /// Create a response with both content and tool calls
    pub fn with_content_and_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            content: Some(content.into()),
            tool_calls,
        }
    }
}

/// Trait for LLM clients
#[async_trait]
pub trait LLMClient: Send + Sync {
    /// Send a chat request to the LLM
    ///
    /// # Arguments
    /// * `messages` - The conversation history
    /// * `tools` - Available tool definitions
    /// * `model` - Optional model override (uses client default if None)
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
        model: Option<&str>,
    ) -> GentResult<LLMResponse>;
}

/// Mock LLM client for testing
#[derive(Debug, Clone)]
pub struct MockLLMClient {
    /// The response to return
    response: String,
    /// Tool calls to return (if any)
    tool_calls: Vec<ToolCall>,
}

impl MockLLMClient {
    /// Create a new mock client with default response
    pub fn new() -> Self {
        Self {
            response: "Hello! I'm a friendly assistant. How can I help you today?".to_string(),
            tool_calls: vec![],
        }
    }

    /// Create a mock client with a custom response
    pub fn with_response(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            tool_calls: vec![],
        }
    }

    /// Create a mock client with tool calls
    pub fn with_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            response: String::new(),
            tool_calls,
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

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn chat(
        &self,
        _messages: Vec<Message>,
        _tools: Vec<ToolDefinition>,
        _model: Option<&str>,
    ) -> GentResult<LLMResponse> {
        if !self.tool_calls.is_empty() {
            Ok(LLMResponse::with_tool_calls(self.tool_calls.clone()))
        } else {
            Ok(LLMResponse::new(&self.response))
        }
    }
}
