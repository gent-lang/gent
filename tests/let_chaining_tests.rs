use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::parser::{parse, Statement};
use gent::runtime::{llm::MockLLMClient, ToolRegistry};

// ============================================
// Parsing Tests
// ============================================

#[test]
fn test_parse_top_level_let_with_string() {
    let source = r#"let x = "hello""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Statement::LetStmt(stmt) => {
            assert_eq!(stmt.name, "x");
        }
        _ => panic!("Expected LetStmt"),
    }
}

#[test]
fn test_parse_top_level_let_with_number() {
    let source = "let count = 42";
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_top_level_let_with_agent_call() {
    let source = r#"
        agent Greeter { prompt: "Say hi" model: "gpt-4o-mini" }
        let greeting = Greeter("Hello")
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);
}

#[test]
fn test_parse_chained_agent_calls() {
    let source = r#"
        agent Summarizer { prompt: "Summarize" model: "gpt-4o-mini" }
        agent Translator { prompt: "Translate to French" model: "gpt-4o-mini" }
        let summary = Summarizer("Long text here")
        let french = Translator(summary)
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 4);
}

// ============================================
// Evaluation Tests
// ============================================

async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &llm, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

#[tokio::test]
async fn test_eval_top_level_let_string() {
    let result = run_program(r#"let message = "hello world""#).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_top_level_let_number() {
    let result = run_program("let count = 42").await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_let_with_agent_call() {
    let source = r#"
        agent Echo { prompt: "Echo back the input" model: "gpt-4o-mini" }
        let result = Echo("test input")
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_chained_agents() {
    let source = r#"
        agent First { prompt: "Process step 1" model: "gpt-4o-mini" }
        agent Second { prompt: "Process step 2" model: "gpt-4o-mini" }
        let step1 = First("initial input")
        let step2 = Second(step1)
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_variable_in_expression() {
    let source = r#"
        agent Greeter { prompt: "Greet the user" model: "gpt-4o-mini" }
        let name = "Alice"
        Greeter(name)
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_undefined_variable_error() {
    let source = r#"
        agent Greeter { prompt: "Greet" model: "gpt-4o-mini" }
        Greeter(undefined_var)
    "#;
    let result = run_program(source).await;
    assert!(result.is_err(), "Should fail with undefined variable");
    let err = result.unwrap_err();
    assert!(err.contains("undefined") || err.contains("Undefined"), "Error was: {}", err);
}

// ============================================
// Mixed Statement Tests
// ============================================

#[tokio::test]
async fn test_let_mixed_with_other_statements() {
    let source = r#"
        let prefix = "Hello"
        agent Greeter { prompt: "Be friendly" model: "gpt-4o-mini" }
        let result = Greeter(prefix)
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_multiple_lets_and_agents() {
    let source = r#"
        agent A { prompt: "Agent A" model: "gpt-4o-mini" }
        agent B { prompt: "Agent B" model: "gpt-4o-mini" }
        let x = A("input 1")
        let y = B("input 2")
        let z = A(y)
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
