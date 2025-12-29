use gent::config::Config;
use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let config = Config::mock();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &config, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

// ============================================
// Method Chain Syntax Parsing Tests
// ============================================

#[test]
fn test_parse_invoke_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_user_prompt_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.userPrompt("Hello").run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_chained_methods() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.systemPrompt("Be helpful").userPrompt("Hi").run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_system_prompt_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.systemPrompt("You are a helpful assistant").run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_full_method_chain() {
    let source = r#"
        agent Translator { model: "gpt-4o-mini" }
        let result = Translator.systemPrompt("You are a translator").userPrompt("Translate to French: Hello").run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_method_chain_with_variable_arg() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let message = "Hello world"
        let result = Test.userPrompt(message).run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_invoke_without_prompt() {
    let source = r#"
        agent Test {
            model: "gpt-4o-mini"
            prompt: "Default prompt"
        }
        let result = Test.run()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================
// Method Chain Evaluation Tests
// ============================================

#[tokio::test]
async fn test_eval_simple_invoke() {
    let source = r#"
        agent Test { systemPrompt: "Say hi" model: "gpt-4o-mini" }
        let result = Test.run()
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_user_prompt_invoke() {
    let source = r#"
        agent Test { systemPrompt: "Be helpful" model: "gpt-4o-mini" }
        let result = Test.userPrompt("Hello").run()
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_chained_invoke() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.systemPrompt("Be helpful").userPrompt("Hi").run()
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
