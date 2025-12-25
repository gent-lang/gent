use gent::interpreter::AgentValue;
use gent::runtime::{run_agent, run_agent_full, MockLLMClient};

// ============================================
// Basic Execution Tests
// ============================================

#[tokio::test]
async fn test_run_agent_basic() {
    let agent = AgentValue::new("Hello", "You are friendly.");
    let llm = MockLLMClient::new();

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("friendly"));
}

#[tokio::test]
async fn test_run_agent_with_input() {
    let agent = AgentValue::new("Greeter", "You greet users.");
    let llm = MockLLMClient::with_response("Hello there!");

    let result = run_agent(&agent, Some("Hi!".to_string()), &llm).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello there!");
}

#[tokio::test]
async fn test_run_agent_default_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let llm = MockLLMClient::with_response("Response");

    // When input is None, should use "Hello!"
    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_empty_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let llm = MockLLMClient::with_response("Response");

    let result = run_agent(&agent, Some("".to_string()), &llm).await;
    assert!(result.is_ok());
}

// ============================================
// Agent Prompt Tests
// ============================================

#[tokio::test]
async fn test_run_agent_uses_prompt() {
    let agent = AgentValue::new("Custom", "You are a helpful coding assistant.");
    let llm = MockLLMClient::with_response("I can help with code!");

    let result = run_agent(&agent, Some("Help me code".to_string()), &llm).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "I can help with code!");
}

#[tokio::test]
async fn test_run_agent_long_prompt() {
    let long_prompt = "You are a very detailed assistant. ".repeat(10);
    let agent = AgentValue::new("Verbose", &long_prompt);
    let llm = MockLLMClient::with_response("Got it!");

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_multiline_prompt() {
    let prompt = "You are helpful.\nBe concise.\nStay on topic.";
    let agent = AgentValue::new("Multi", prompt);
    let llm = MockLLMClient::with_response("Understood");

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
}

// ============================================
// Full Response Tests
// ============================================

#[tokio::test]
async fn test_run_agent_full_basic() {
    let agent = AgentValue::new("Test", "Test agent.");
    let llm = MockLLMClient::with_response("Full response");

    let result = run_agent_full(&agent, None, &llm).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().content, Some("Full response".to_string()));
}

#[tokio::test]
async fn test_run_agent_full_with_input() {
    let agent = AgentValue::new("Test", "Test agent.");
    let llm = MockLLMClient::with_response("Response to input");

    let result = run_agent_full(&agent, Some("Custom input".to_string()), &llm).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().content,
        Some("Response to input".to_string())
    );
}

// ============================================
// Different Agent Types Tests
// ============================================

#[tokio::test]
async fn test_run_multiple_different_agents() {
    let agent1 = AgentValue::new("Agent1", "You are agent 1.");
    let agent2 = AgentValue::new("Agent2", "You are agent 2.");

    let llm1 = MockLLMClient::with_response("Response 1");
    let llm2 = MockLLMClient::with_response("Response 2");

    let r1 = run_agent(&agent1, None, &llm1).await.unwrap();
    let r2 = run_agent(&agent2, None, &llm2).await.unwrap();

    assert_eq!(r1, "Response 1");
    assert_eq!(r2, "Response 2");
}

#[tokio::test]
async fn test_run_same_agent_multiple_times() {
    let agent = AgentValue::new("Repeater", "You repeat things.");
    let llm = MockLLMClient::with_response("Repeated!");

    let r1 = run_agent(&agent, Some("First".to_string()), &llm)
        .await
        .unwrap();
    let r2 = run_agent(&agent, Some("Second".to_string()), &llm)
        .await
        .unwrap();

    assert_eq!(r1, r2); // Mock returns same response
}

// ============================================
// Trait Object Tests
// ============================================

#[tokio::test]
async fn test_run_agent_with_boxed_client() {
    let agent = AgentValue::new("Test", "Test.");
    let llm: Box<dyn gent::runtime::LLMClient> = Box::new(MockLLMClient::with_response("Boxed!"));

    let result = run_agent(&agent, None, llm.as_ref()).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Boxed!");
}

// ============================================
// Input Variations Tests
// ============================================

#[tokio::test]
async fn test_run_agent_with_special_characters() {
    let agent = AgentValue::new("Test", "Test.");
    let llm = MockLLMClient::with_response("OK");

    let result = run_agent(&agent, Some("Hello! How are you? 你好".to_string()), &llm).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_with_newlines_in_input() {
    let agent = AgentValue::new("Test", "Test.");
    let llm = MockLLMClient::with_response("OK");

    let result = run_agent(&agent, Some("Line 1\nLine 2\nLine 3".to_string()), &llm).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_agent_with_quotes_in_input() {
    let agent = AgentValue::new("Test", "Test.");
    let llm = MockLLMClient::with_response("OK");

    let result = run_agent(&agent, Some("Say \"hello\" to me".to_string()), &llm).await;
    assert!(result.is_ok());
}

// ============================================
// Response Content Tests
// ============================================

#[tokio::test]
async fn test_run_agent_empty_response() {
    let agent = AgentValue::new("Test", "Test.");
    let llm = MockLLMClient::with_response("");

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
}

#[tokio::test]
async fn test_run_agent_long_response() {
    let agent = AgentValue::new("Test", "Test.");
    let long_response = "This is a long response. ".repeat(100);
    let llm = MockLLMClient::with_response(&long_response);

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), long_response.len());
}

#[tokio::test]
async fn test_run_agent_multiline_response() {
    let agent = AgentValue::new("Test", "Test.");
    let response = "Line 1\nLine 2\nLine 3";
    let llm = MockLLMClient::with_response(response);

    let result = run_agent(&agent, None, &llm).await;
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
    let llm = MockLLMClient::new(); // Default: "Hello! I'm a friendly assistant..."

    let result = run_agent(&agent, None, &llm).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(response.contains("Hello"));
    assert!(response.contains("friendly"));
}
