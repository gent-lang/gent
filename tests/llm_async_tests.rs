use gent::runtime::llm::{LLMResponse, Message, Role, ToolCall, ToolDefinition, ToolResult};
use serde_json::json;

#[test]
fn test_tool_call_creation() {
    let call = ToolCall {
        id: "call_123".to_string(),
        name: "web_fetch".to_string(),
        arguments: json!({"url": "https://example.com"}),
    };
    assert_eq!(call.name, "web_fetch");
}

#[test]
fn test_tool_result_success() {
    let result = ToolResult {
        call_id: "call_123".to_string(),
        content: "Page content here".to_string(),
        is_error: false,
    };
    assert!(!result.is_error);
}

#[test]
fn test_tool_result_error() {
    let result = ToolResult {
        call_id: "call_123".to_string(),
        content: "Connection refused".to_string(),
        is_error: true,
    };
    assert!(result.is_error);
}

#[test]
fn test_tool_definition() {
    let def = ToolDefinition {
        name: "web_fetch".to_string(),
        description: "Fetch a web page".to_string(),
        parameters: json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" }
            },
            "required": ["url"]
        }),
    };
    assert_eq!(def.name, "web_fetch");
}

#[test]
fn test_llm_response_with_content() {
    let resp = LLMResponse {
        content: Some("Hello!".to_string()),
        tool_calls: vec![],
    };
    assert!(resp.content.is_some());
    assert!(resp.tool_calls.is_empty());
}

#[test]
fn test_llm_response_with_tool_calls() {
    let resp = LLMResponse {
        content: None,
        tool_calls: vec![ToolCall {
            id: "call_1".to_string(),
            name: "web_fetch".to_string(),
            arguments: json!({"url": "https://example.com"}),
        }],
    };
    assert!(resp.content.is_none());
    assert_eq!(resp.tool_calls.len(), 1);
}

#[test]
fn test_message_tool_result() {
    let msg = Message::tool_result(ToolResult {
        call_id: "call_123".to_string(),
        content: "Success".to_string(),
        is_error: false,
    });
    assert_eq!(msg.role, Role::Tool);
}
