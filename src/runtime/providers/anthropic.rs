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

    /// Extract system messages and return (system_prompt, non_system_messages)
    fn extract_system<'a>(&self, messages: &'a [Message]) -> (Option<String>, Vec<&'a Message>) {
        let system_parts: Vec<&str> = messages
            .iter()
            .filter(|m| m.role == Role::System)
            .map(|m| m.content.as_str())
            .collect();

        let system = if system_parts.is_empty() {
            None
        } else {
            Some(system_parts.join("\n\n"))
        };

        let non_system: Vec<&Message> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .collect();

        (system, non_system)
    }

    fn to_anthropic_messages(&self, messages: &[&Message]) -> Vec<AnthropicMessage> {
        let mut result = Vec::new();

        for msg in messages {
            match msg.role {
                Role::System => continue, // Already extracted
                Role::User => {
                    result.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Text(msg.content.clone()),
                    });
                }
                Role::Assistant => {
                    if let Some(tool_calls) = &msg.tool_calls {
                        let blocks: Vec<AnthropicContentBlock> = tool_calls
                            .iter()
                            .map(|tc| AnthropicContentBlock::ToolUse {
                                id: tc.id.clone(),
                                name: tc.name.clone(),
                                input: tc.arguments.clone(),
                            })
                            .collect();
                        result.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: AnthropicContent::Blocks(blocks),
                        });
                    } else {
                        result.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: AnthropicContent::Text(msg.content.clone()),
                        });
                    }
                }
                Role::Tool => {
                    result.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: AnthropicContent::Blocks(vec![AnthropicContentBlock::ToolResult {
                            tool_use_id: msg.tool_call_id.clone().unwrap_or_default(),
                            content: msg.content.clone(),
                            is_error: false,
                        }]),
                    });
                }
            }
        }

        result
    }

    fn to_anthropic_tools(&self, tools: &[ToolDefinition]) -> Vec<AnthropicTool> {
        tools
            .iter()
            .map(|t| AnthropicTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.parameters.clone(),
            })
            .collect()
    }
}

// Anthropic API types
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<AnthropicTool>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: AnthropicContent,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum AnthropicContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: JsonValue,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: JsonValue,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseBlock>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum AnthropicResponseBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: JsonValue,
    },
}

#[async_trait]
impl LLMClient for AnthropicClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
        model: Option<&str>,
        _json_mode: bool, // Anthropic doesn't have json_mode, handled via prompting
    ) -> GentResult<LLMResponse> {
        let url = format!("{}/v1/messages", self.base_url);
        let model_to_use = model.unwrap_or(&self.model);

        let (system, non_system_messages) = self.extract_system(&messages);
        let anthropic_messages = self.to_anthropic_messages(&non_system_messages);
        let anthropic_tools = self.to_anthropic_tools(&tools);

        let request = AnthropicRequest {
            model: model_to_use.to_string(),
            max_tokens: 4096,
            system,
            messages: anthropic_messages,
            tools: anthropic_tools,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| GentError::ApiError {
                message: format!("Request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GentError::ApiError {
                message: format!("Anthropic API error ({}): {}", status, text),
            });
        }

        let api_response: AnthropicResponse =
            response.json().await.map_err(|e| GentError::ApiError {
                message: format!("Failed to parse Anthropic response: {}", e),
            })?;

        // Extract text content and tool calls from response
        let mut text_parts = Vec::new();
        let mut tool_calls = Vec::new();

        for block in api_response.content {
            match block {
                AnthropicResponseBlock::Text { text } => {
                    text_parts.push(text);
                }
                AnthropicResponseBlock::ToolUse { id, name, input } => {
                    tool_calls.push(ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        let content = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join(""))
        };

        Ok(LLMResponse { content, tool_calls })
    }
}
