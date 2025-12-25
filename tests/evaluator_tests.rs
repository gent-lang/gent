use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

// ============================================
// Basic Evaluation Tests
// ============================================

#[tokio::test]
async fn test_evaluate_empty_program() {
    let program = parse("").unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_evaluate_agent_declaration() {
    let program = parse(r#"agent Hello { prompt: "You are friendly." model: "gpt-4o-mini" }"#).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty()); // No output from just declaring
}

#[tokio::test]
async fn test_evaluate_run_statement() {
    let source = r#"
        agent Hello { prompt: "You are friendly." model: "gpt-4o-mini" }
        run Hello
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Hello there!");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    let outputs = result.unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], "Hello there!");
}

#[tokio::test]
async fn test_evaluate_run_with_input() {
    let source = r#"
        agent Greeter { prompt: "You greet people." model: "gpt-4o-mini" }
        run Greeter with "Hi!"
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Hello! Nice to meet you!");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0], "Hello! Nice to meet you!");
}

// ============================================
// Hello World Test
// ============================================

#[tokio::test]
async fn test_evaluate_hello_world() {
    let source = r#"agent Hello { prompt: "You are friendly." model: "gpt-4o-mini" }
run Hello"#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
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
    let source = "run NonExistent";
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Undefined"));
}

#[tokio::test]
async fn test_evaluate_missing_prompt() {
    let source = r#"
        agent Empty { }
        run Empty
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("prompt"));
}

// ============================================
// Multiple Agents Tests
// ============================================

#[tokio::test]
async fn test_evaluate_multiple_agents() {
    let source = r#"
        agent First { prompt: "You are first." model: "gpt-4o-mini" }
        agent Second { prompt: "You are second." model: "gpt-4o-mini" }
        run First
        run Second
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Response");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_evaluate_same_agent_twice() {
    let source = r#"
        agent Bot { prompt: "You help." model: "gpt-4o-mini" }
        run Bot
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Help!");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
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
        agent Bot { prompt: "Help." model: "gpt-4o-mini" timeout: 30 }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("OK");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    // Should succeed - extra fields are ignored
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_boolean_field() {
    let source = r#"
        agent Bot { prompt: "Help." model: "gpt-4o-mini" verbose: true }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("OK");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

// ============================================
// Complex Program Tests
// ============================================

#[tokio::test]
async fn test_evaluate_complex_program() {
    let source = r#"
        agent Researcher { prompt: "You research topics." model: "gpt-4o-mini" }
        agent Writer { prompt: "You write content." model: "gpt-4o-mini" }
        run Researcher with "Find info about Rust"
        run Writer with "Write about programming"
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Done!");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_evaluate_with_comments() {
    let source = r#"
        // Define an agent
        agent Helper { prompt: "You help." model: "gpt-4o-mini" }
        // Run the agent
        run Helper
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Helping!");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap()[0], "Helping!");
}

// ============================================
// Edge Cases
// ============================================

#[tokio::test]
async fn test_evaluate_empty_prompt() {
    let source = r#"
        agent Empty { prompt: "" model: "gpt-4o-mini" }
        run Empty
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Response");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_long_prompt() {
    let long_text = "You are helpful. ".repeat(50);
    let source = format!(r#"agent Long {{ prompt: "{}" model: "gpt-4o-mini" }} run Long"#, long_text);
    let program = parse(&source).unwrap();
    let llm = MockLLMClient::with_response("OK");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_evaluate_special_characters_in_prompt() {
    let source = r#"
        agent Special { prompt: "Say \"hello\" and use 'quotes'." model: "gpt-4o-mini" }
        run Special
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("OK");
    let tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &llm, &tools).await;
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
            prompt: "Greet users"
            model: "gpt-4o-mini"
            use greet
        }

        run Greeter with "test"
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &llm, &tools).await;
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
            prompt: "Do math"
            model: "gpt-4o-mini"
            use add
        }

        run Calculator with "2 + 2"
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::with_builtins();

    let result = evaluate_with_output(&program, &llm, &tools).await;
    assert!(result.is_ok());
}
