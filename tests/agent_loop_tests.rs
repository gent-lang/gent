use gent::config::Config;
use gent::interpreter::types::AgentValue;
use gent::logging::NullLogger;
use gent::runtime::{run_agent_with_tools, ToolRegistry};

// NOTE: These tests require mock tool_calls support which is not yet implemented
// in the Config-based mock system. They are ignored until that support is added.

#[tokio::test]
async fn test_agent_simple_response() {
    let agent = AgentValue::new("Bot", "You are helpful.");
    let config = Config::mock_with_response("Hello there!");
    let registry = ToolRegistry::new();

    let logger = NullLogger;
    let result =
        run_agent_with_tools(&agent, Some("Hi".to_string()), &config, &registry, &logger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello there!");
}

// TODO: Re-enable when Config supports mock_tool_calls
// These tests require MockLLMClient::with_tool_calls which is not supported by Config yet
#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_with_tool_call() {
    // Test disabled - needs Config::mock_with_tool_calls support
    let config = Config::mock();
    let _ = config;
}

#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_max_steps_exceeded() {
    // Test disabled - needs Config::mock_with_tool_calls support
    let config = Config::mock();
    let _ = config;
}

#[tokio::test]
#[ignore = "Requires mock tool_calls support in Config"]
async fn test_agent_unknown_tool() {
    // Test disabled - needs Config::mock_with_tool_calls support
    let config = Config::mock();
    let _ = config;
}
