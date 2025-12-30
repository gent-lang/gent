use gent::config::Config;
use gent::interpreter::AgentValue;
use gent::runtime::{run_agent, run_agent_full, ProviderFactory};

// ============================================
// Basic Execution Tests
// ============================================

#[tokio::test]
async fn test_run_agent_basic() {
    let agent = AgentValue::new("Hello", "You are friendly.");
    let factory = ProviderFactory::mock();

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("friendly"));
}

#[tokio::test]
async fn test_run_agent_with_input() {
    let agent = AgentValue::new("Greeter", "You greet users.");
    let factory = ProviderFactory::mock_with_response("Hello there!");

    let result = run_agent(&agent, Some("Hi!".to_string()), &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello there!");
}

#[tokio::test]
async fn test_run_agent_default_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let factory = ProviderFactory::mock_with_response("Response");

    // When input is None, should use "Hello!"
    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_empty_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let factory = ProviderFactory::mock_with_response("Response");

    let result = run_agent(&agent, Some("".to_string()), &factory).await;
    assert!(result.is_ok());
}

// ============================================
// Agent Prompt Tests
// ============================================

#[tokio::test]
async fn test_run_agent_uses_prompt() {
    let agent = AgentValue::new("Custom", "You are a helpful coding assistant.");
    let factory = ProviderFactory::mock_with_response("I can help with code!");

    let result = run_agent(&agent, Some("Help me code".to_string()), &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "I can help with code!");
}

#[tokio::test]
async fn test_run_agent_long_prompt() {
    let long_prompt = "You are a very detailed assistant. ".repeat(10);
    let agent = AgentValue::new("Verbose", &long_prompt);
    let factory = ProviderFactory::mock_with_response("Got it!");

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_multiline_prompt() {
    let prompt = "You are helpful.\nBe concise.\nStay on topic.";
    let agent = AgentValue::new("Multi", prompt);
    let factory = ProviderFactory::mock_with_response("Understood");

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
}

// ============================================
// Full Response Tests
// ============================================

#[tokio::test]
async fn test_run_agent_full_basic() {
    let agent = AgentValue::new("Test", "Test agent.");
    let factory = ProviderFactory::mock_with_response("Full response");

    let result = run_agent_full(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, Some("Full response".to_string()));
}

#[tokio::test]
async fn test_run_agent_full_with_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let factory = ProviderFactory::mock_with_response("Response to input");

    let result = run_agent_full(&agent, Some("Custom input".to_string()), &factory).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().content,
        Some("Response to input".to_string())
    );
}

// ============================================
// Different Agent Types Tests
// ============================================

// TODO: Re-enable when we can have different mock responses per agent
// This test requires running multiple agents with different mock responses
#[tokio::test]
#[ignore = "Requires different mock responses per agent"]
async fn test_run_multiple_different_agents() {
    let agent1 = AgentValue::new("Agent1", "You are agent 1.");
    let agent2 = AgentValue::new("Agent2", "You are agent 2.");

    let factory1 = ProviderFactory::mock_with_response("Response 1");
    let factory2 = ProviderFactory::mock_with_response("Response 2");

    let r1 = run_agent(&agent1, None, &factory1).await.unwrap();
    let r2 = run_agent(&agent2, None, &factory2).await.unwrap();

    assert_eq!(r1, "Response 1");
    assert_eq!(r2, "Response 2");
}

#[tokio::test]
async fn test_run_same_agent_multiple_times() {
    let agent = AgentValue::new("Repeater", "You repeat things.");
    let factory = ProviderFactory::mock_with_response("Repeated!");

    let r1 = run_agent(&agent, Some("First".to_string()), &factory)
        .await
        .unwrap();
    let r2 = run_agent(&agent, Some("Second".to_string()), &factory)
        .await
        .unwrap();

    assert_eq!(r1, r2); // Mock returns same response
}

// ============================================
// Trait Object Tests
// ============================================

// TODO: Re-enable when Config-based approach supports trait objects
// The new Config-based approach doesn't use LLMClient trait objects directly
#[tokio::test]
async fn test_run_agent_with_factory() {
    let agent = AgentValue::new("Test", "Test.");
    let factory = ProviderFactory::mock_with_response("Factory!");

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Factory!");
}

// ============================================
// Input Variations Tests
// ============================================

#[tokio::test]
async fn test_run_agent_with_special_characters() {
    let agent = AgentValue::new("Test", "Test.");
    let factory = ProviderFactory::mock_with_response("OK");

    let result = run_agent(&agent, Some("Hello! How are you? 你好".to_string()), &factory).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_with_newlines_in_input() {
    let agent = AgentValue::new("Test", "Test.");
    let factory = ProviderFactory::mock_with_response("OK");

    let result = run_agent(&agent, Some("Line 1\nLine 2\nLine 3".to_string()), &factory).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_with_quotes_in_input() {
    let agent = AgentValue::new("Test", "Test.");
    let factory = ProviderFactory::mock_with_response("OK");

    let result = run_agent(&agent, Some("Say \"hello\" to me".to_string()), &factory).await;
    assert!(result.is_ok());
}

// ============================================
// Response Content Tests
// ============================================

#[tokio::test]
async fn test_run_agent_empty_response() {
    let agent = AgentValue::new("Test", "Test.");
    let factory = ProviderFactory::mock_with_response("");

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn test_run_agent_long_response() {
    let agent = AgentValue::new("Test", "Test.");
    let long_response = "This is a long response. ".repeat(100);
    let factory = ProviderFactory::mock_with_response(&long_response);

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), long_response.len());
}

#[tokio::test]
async fn test_run_agent_multiline_response() {
    let agent = AgentValue::new("Test", "Test.");
    let response = "Line 1\nLine 2\nLine 3";
    let factory = ProviderFactory::mock_with_response(response);

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), response);
}

// ============================================
// Hello World Simulation
// ============================================

#[tokio::test]
async fn test_hello_world_agent() {
    // Simulates: agent Hello { prompt: "You are friendly." } run Hello
    let agent = AgentValue::new("Hello", "You are friendly.");
    let factory = ProviderFactory::mock(); // Default: "Hello! I'm a friendly assistant..."

    let result = run_agent(&agent, None, &factory).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.contains("Hello"));
    assert!(response.contains("friendly"));
}
