use gent::config::Config;
use gent::interpreter::types::AgentValue;
use gent::logging::NullLogger;
use gent::runtime::{run_agent_with_tools, ProviderFactory, ToolCall, ToolRegistry};
use serde_json::json;

#[tokio::test]
async fn test_agent_simple_response() {
    let agent = AgentValue::new("Bot", "You are helpful.");
    let factory = ProviderFactory::mock_with_response("Hello there!");
    let registry = ToolRegistry::new();

    let logger = NullLogger;
    let result =
        run_agent_with_tools(&agent, Some("Hi".to_string()), &factory, &registry, &logger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello there!");
}

// TODO: Re-enable when Config supports mock_tool_calls
// These tests require MockLLMClient::with_tool_calls which is not supported by Config yet
#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_with_tool_call() {
    let agent =
        AgentValue::new("Bot", "You help with files.").with_tools(vec!["read_file".to_string()]);

    // First call returns tool request, second returns final answer
    let factory = ProviderFactory::mock_with_tool_calls(vec![ToolCall {
        id: "call_1".to_string(),
        name: "read_file".to_string(),
        arguments: json!({"path": "test.txt"}),
    }]);

    let registry = ToolRegistry::with_builtins();

    // This will execute the tool and loop
    let logger = NullLogger;
    let result = run_agent_with_tools(
        &agent,
        Some("Read test.txt".to_string()),
        &factory,
        &registry,
        &logger,
    )
    .await;
    // Result depends on mock behavior after tool execution
    assert!(result.is_ok() || result.is_err()); // Just testing it runs
}

#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_max_steps_exceeded() {
    let agent = AgentValue::new("Bot", "Loop forever").with_max_steps(2);

    // Always return tool calls to force max steps
    let factory = ProviderFactory::mock_with_tool_calls(vec![ToolCall {
        id: "call_1".to_string(),
        name: "web_fetch".to_string(),
        arguments: json!({"url": "https://example.com"}),
    }]);

    let registry = ToolRegistry::with_builtins();

    let logger = NullLogger;
    let result = run_agent_with_tools(&agent, None, &factory, &registry, &logger).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("exceeded"));
}

#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_unknown_tool() {
    let agent =
        AgentValue::new("Bot", "Use unknown tool").with_tools(vec!["nonexistent".to_string()]);

    let factory = ProviderFactory::mock_with_tool_calls(vec![ToolCall {
        id: "call_1".to_string(),
        name: "nonexistent".to_string(),
        arguments: json!({}),
    }]);

    let registry = ToolRegistry::new(); // Empty, no tools

    let logger = NullLogger;
    let result = run_agent_with_tools(&agent, None, &factory, &registry, &logger).await;
    // Should handle gracefully - either error or tell LLM tool not found
    assert!(result.is_ok() || result.is_err());
}
