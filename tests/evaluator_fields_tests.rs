use gent::config::Config;
use gent::interpreter::evaluate_with_output;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{ProviderFactory, ToolRegistry};

#[tokio::test]
async fn test_evaluate_agent_with_max_steps() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o-mini"
            maxSteps: 5
        }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_with_model() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o"
        }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_with_tools() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            model: "gpt-4o-mini"
            tools: [web_fetch, read_file]
        }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_agent_all_fields() {
    let source = r#"
        agent Bot {
            systemPrompt: "Hello"
            tools: [web_fetch]
            maxSteps: 20
            model: "gpt-4o-mini"
        }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}
