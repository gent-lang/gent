use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

#[tokio::test]
async fn test_evaluate_agent_with_max_steps() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o-mini"
            maxSteps: 5
        }
        let result = Bot.invoke()
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_with_model() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o"
        }
        let result = Bot.invoke()
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_with_tools() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o-mini"
            use web_fetch, read_file
        }
        let result = Bot.invoke()
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_all_fields() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            use web_fetch
            maxSteps: 20
            model: "gpt-4o-mini"
        }
        let result = Bot.invoke()
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;
    assert!(result.is_ok());
}
