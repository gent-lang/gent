use gent::config::Config;
use gent::parser::parse;
use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::runtime::{ProviderFactory, ToolRegistry};

async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &factory, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

#[test]
fn test_parse_system_prompt_field() {
    let source = r#"agent Test { systemPrompt: "Hello" model: "gpt-4o-mini" }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_user_prompt_field() {
    let source = r#"agent Test { userPrompt: "Hello" model: "gpt-4o-mini" }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_max_steps_camel() {
    let source = r#"agent Test { systemPrompt: "Hi" model: "gpt-4o-mini" maxSteps: 5 }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_output_retries_camel() {
    let source = r#"agent Test { systemPrompt: "Hi" model: "gpt-4o-mini" outputRetries: 3 }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_with_user_prompt_only() {
    let source = r#"
        agent Test { userPrompt: "Hello" model: "gpt-4o-mini" }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_with_both_prompts() {
    let source = r#"
        agent Test {
            systemPrompt: "You are helpful"
            userPrompt: "Hello"
            model: "gpt-4o-mini"
        }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_with_no_prompts() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
    "#;
    let result = run_program(source).await;
    // Prompts are now optional - agent with only model should work
    assert!(result.is_ok(), "Agent with no prompts should work: {:?}", result.err());
}
