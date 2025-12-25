use gent::runtime::llm::{LLMClient, Message, ToolDefinition};
use gent::runtime::providers::OpenAIClient;
use serde_json::json;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_openai_client_creation() {
    let client = OpenAIClient::new("test-key".to_string());
    assert!(client.model() == "gpt-4o-mini");
}

#[tokio::test]
async fn test_openai_client_with_model() {
    let client = OpenAIClient::new("test-key".to_string()).with_model("gpt-4o");
    assert!(client.model() == "gpt-4o");
}

#[tokio::test]
async fn test_openai_chat_simple() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello! How can I help?"
                },
                "finish_reason": "stop"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::new("test-key".to_string()).with_base_url(&mock_server.uri());

    let messages = vec![Message::user("Hello")];
    let response = client.chat(messages, vec![], None).await.unwrap();

    assert_eq!(response.content, Some("Hello! How can I help?".to_string()));
    assert!(response.tool_calls.is_empty());
}

#[tokio::test]
async fn test_openai_chat_with_tool_call() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_abc123",
                        "type": "function",
                        "function": {
                            "name": "web_fetch",
                            "arguments": "{\"url\":\"https://example.com\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::new("test-key".to_string()).with_base_url(&mock_server.uri());

    let tools = vec![ToolDefinition {
        name: "web_fetch".to_string(),
        description: "Fetch a URL".to_string(),
        parameters: json!({"type": "object", "properties": {"url": {"type": "string"}}}),
    }];

    let messages = vec![Message::user("Fetch example.com")];
    let response = client.chat(messages, tools, None).await.unwrap();

    assert!(response.content.is_none());
    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].name, "web_fetch");
}

#[tokio::test]
async fn test_openai_api_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error"
            }
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::new("bad-key".to_string()).with_base_url(&mock_server.uri());

    let messages = vec![Message::user("Hello")];
    let result = client.chat(messages, vec![], None).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_openai_chat_with_model_override() {
    let mock_server = MockServer::start().await;

    // Set up mock that expects gpt-4 in the request body
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("\"model\":\"gpt-4\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Response from gpt-4"
                },
                "finish_reason": "stop"
            }]
        })))
        .mount(&mock_server)
        .await;

    // Client defaults to gpt-4o-mini, but we override with gpt-4
    let client = OpenAIClient::new("test-key".to_string()).with_base_url(&mock_server.uri());

    let messages = vec![Message::user("Hello")];
    let response = client.chat(messages, vec![], Some("gpt-4")).await.unwrap();

    assert_eq!(response.content, Some("Response from gpt-4".to_string()));
}

#[tokio::test]
async fn test_openai_chat_uses_client_default_when_no_override() {
    let mock_server = MockServer::start().await;

    // Set up mock that expects gpt-4o-mini (the default) in the request body
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_string_contains("\"model\":\"gpt-4o-mini\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "chatcmpl-123",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Response from default model"
                },
                "finish_reason": "stop"
            }]
        })))
        .mount(&mock_server)
        .await;

    let client = OpenAIClient::new("test-key".to_string()).with_base_url(&mock_server.uri());

    let messages = vec![Message::user("Hello")];
    // Pass None for model - should use client default (gpt-4o-mini)
    let response = client.chat(messages, vec![], None).await.unwrap();

    assert_eq!(
        response.content,
        Some("Response from default model".to_string())
    );
}
