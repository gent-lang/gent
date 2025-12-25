//! OpenAI API client

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::errors::{GentError, GentResult};
use crate::runtime::llm::{LLMClient, LLMResponse, Message, Role, ToolCall, ToolDefinition};

/// OpenAI API client
pub struct OpenAIClient {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gpt-4o-mini".to_string(),
            base_url: "https://api.openai.com".to_string(),
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

    fn to_openai_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages.iter().map(|m| self.to_openai_message(m)).collect()
    }

    fn to_openai_message(&self, message: &Message) -> OpenAIMessage {
        OpenAIMessage {
            role: match message.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::Tool => "tool".to_string(),
            },
            content: if message.content.is_empty() {
                None
            } else {
                Some(message.content.clone())
            },
            tool_call_id: message.tool_call_id.clone(),
            tool_calls: message.tool_calls.as_ref().map(|tcs| {
                tcs.iter().map(|tc| OpenAIToolCall {
                    id: tc.id.clone(),
                    r#type: "function".to_string(),
                    function: OpenAIFunction {
                        name: tc.name.clone(),
                        arguments: tc.arguments.to_string(),
                    },
                }).collect()
            }),
        }
    }

    fn to_openai_tools(&self, tools: &[ToolDefinition]) -> Vec<OpenAITool> {
        tools.iter().map(|t| OpenAITool {
            r#type: "function".to_string(),
            function: OpenAIFunctionDef {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.parameters.clone(),
            },
        }).collect()
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn chat(
        &self,
        messages: Vec<Message>,
        tools: Vec<ToolDefinition>,
    ) -> GentResult<LLMResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);

        let mut body = json!({
            "model": self.model,
            "messages": self.to_openai_messages(&messages),
        });

        if !tools.is_empty() {
            body["tools"] = json!(self.to_openai_tools(&tools));
        }

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| GentError::ApiError {
                message: format!("Request failed: {}", e),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(GentError::ApiError {
                message: format!("API error ({}): {}", status, text),
            });
        }

        let api_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| GentError::ApiError {
                message: format!("Failed to parse response: {}", e),
            })?;

        let choice = api_response.choices.into_iter().next().ok_or_else(|| {
            GentError::ApiError {
                message: "No choices in response".to_string(),
            }
        })?;

        let tool_calls = choice.message.tool_calls
            .unwrap_or_default()
            .into_iter()
            .map(|tc| ToolCall {
                id: tc.id,
                name: tc.function.name,
                arguments: serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(JsonValue::Null),
            })
            .collect();

        Ok(LLMResponse {
            content: choice.message.content,
            tool_calls,
        })
    }
}

// OpenAI API types
#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    r#type: String,
    function: OpenAIFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIFunction {
    name: String,
    arguments: String,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    r#type: String,
    function: OpenAIFunctionDef,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionDef {
    name: String,
    description: String,
    parameters: JsonValue,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}
