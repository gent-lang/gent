use gent::interpreter::evaluate_with_output;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{ProviderFactory, ToolRegistry};

// ============================================
// Basic Evaluation Tests
// ============================================

#[tokio::test]
async fn test_evaluate_empty_program() {
    let program = parse("").unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_evaluate_agent_declaration() {
    let program =
        parse(r#"agent Hello { systemPrompt: "You are friendly." model: "gpt-4o-mini" }"#).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty()); // No output from just declaring
}

#[tokio::test]
async fn test_evaluate_run_statement() {
    let source = r#"
        agent Hello { systemPrompt: "You are friendly." model: "gpt-4o-mini" }
        let result = Hello.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Hello there!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    let outputs = result.unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], "Hello there!");
}

#[tokio::test]
async fn test_evaluate_run_with_input() {
    let source = r#"
        agent Greeter { systemPrompt: "You greet people." model: "gpt-4o-mini" }
        let result = Greeter.userPrompt("Hi!").run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Hello! Nice to meet you!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0], "Hello! Nice to meet you!");
}

// ============================================
// Hello World Test
// ============================================

#[tokio::test]
async fn test_evaluate_hello_world() {
    let source = r#"agent Hello { systemPrompt: "You are friendly." model: "gpt-4o-mini" }
let result = Hello.run()"#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    let outputs = result.unwrap();
    assert_eq!(outputs.len(), 1);
    assert!(outputs[0].contains("friendly"));
}

// ============================================
// Error Cases
// ============================================

#[tokio::test]
async fn test_evaluate_undefined_agent() {
    let source = "let result = NonExistent.run()";
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Undefined"));
}

#[tokio::test]
async fn test_evaluate_missing_model() {
    // Prompt is now optional, but model is still required
    let source = r#"
        agent Empty { }
        let result = Empty.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("model"));
}

// ============================================
// Multiple Agents Tests
// ============================================

#[tokio::test]
async fn test_evaluate_multiple_agents() {
    let source = r#"
        agent First { systemPrompt: "You are first." model: "gpt-4o-mini" }
        agent Second { systemPrompt: "You are second." model: "gpt-4o-mini" }
        let r1 = First.run()
        let r2 = Second.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Response");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_evaluate_same_agent_twice() {
    let source = r#"
        agent Bot { systemPrompt: "You help." model: "gpt-4o-mini" }
        let r1 = Bot.run()
        let r2 = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Help!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    let outputs = result.unwrap();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0], outputs[1]);
}

// ============================================
// Expression Tests
// ============================================

#[tokio::test]
async fn test_evaluate_number_field() {
    let source = r#"
        agent Bot { systemPrompt: "Help." model: "gpt-4o-mini" timeout: 30 }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("OK");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    // Should succeed - extra fields are ignored
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_boolean_field() {
    let source = r#"
        agent Bot { systemPrompt: "Help." model: "gpt-4o-mini" verbose: true }
        let result = Bot.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("OK");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

// ============================================
// Complex Program Tests
// ============================================

#[tokio::test]
async fn test_evaluate_complex_program() {
    let source = r#"
        agent Researcher { systemPrompt: "You research topics." model: "gpt-4o-mini" }
        agent Writer { systemPrompt: "You write content." model: "gpt-4o-mini" }
        let r1 = Researcher.userPrompt("Find info about Rust").run()
        let r2 = Writer.userPrompt("Write about programming").run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Done!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_evaluate_with_comments() {
    let source = r#"
        // Define an agent
        agent Helper { systemPrompt: "You help." model: "gpt-4o-mini" }
        // Run the agent
        let result = Helper.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Helping!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0], "Helping!");
}

// ============================================
// Edge Cases
// ============================================

#[tokio::test]
async fn test_evaluate_empty_prompt() {
    let source = r#"
        agent Empty { systemPrompt: "" model: "gpt-4o-mini" }
        let result = Empty.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Response");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_long_prompt() {
    let long_text = "You are helpful. ".repeat(50);
    let source = format!(
        r#"agent Long {{ systemPrompt: "{}" model: "gpt-4o-mini" }} let result = Long.run()"#,
        long_text
    );
    let program = parse(&source).unwrap();
    let factory = ProviderFactory::mock_with_response("OK");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_special_characters_in_prompt() {
    let source = r#"
        agent Special { systemPrompt: "Say \"hello\" and use 'quotes'." model: "gpt-4o-mini" }
        let result = Special.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("OK");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

// ============================================
// Tool Declaration Tests
// ============================================

#[tokio::test]
async fn test_tool_declaration_registers() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }

        agent Greeter {
            systemPrompt: "Greet users"
            model: "gpt-4o-mini"
            tools: [greet]
        }

        let result = Greeter.userPrompt("test").run()
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_tool_declarations() {
    let source = r#"
        tool add(a: number, b: number) -> number {
            return a + b
        }

        tool greet(name: string) -> string {
            return "Hello, " + name
        }

        agent Calculator {
            systemPrompt: "Do math"
            model: "gpt-4o-mini"
            tools: [add]
        }

        let result = Calculator.userPrompt("2 + 2").run()
    "#;

    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok());
}
