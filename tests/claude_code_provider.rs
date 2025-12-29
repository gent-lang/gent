//! Tests for Claude Code provider

use gent::interpreter::evaluate_with_output;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{ProviderFactory, ToolRegistry};

#[tokio::test]
async fn test_agent_with_provider_field() {
    let source = r#"
        agent TestAgent {
            model: "gpt-4"
            provider: "openai"
            systemPrompt: "You are helpful"
        }
        let result = TestAgent.run()
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_claude_code_provider() {
    let source = r#"
        agent TestAgent {
            model: "claude-sonnet-4"
            provider: "claude-code"
            systemPrompt: "You are helpful"
        }
        let result = TestAgent.run()
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock(); // Uses mock even for claude-code
    let mut tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_invalid_provider() {
    let source = r#"
        agent TestAgent {
            model: "gpt-4"
            provider: "invalid-provider"
            systemPrompt: "You are helpful"
        }
        let result = TestAgent.run()
    "#;

    let program = parse(source).unwrap();
    // Use a real factory (not mock) to test provider validation
    let factory = ProviderFactory::new(gent::config::Config::default());
    let mut tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("invalid-provider"));
}
