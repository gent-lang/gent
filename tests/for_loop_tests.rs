use gent::parser::parse;
use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::runtime::{llm::MockLLMClient, ToolRegistry};

async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &llm, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

#[test]
fn test_parse_for_loop_array() {
    let source = r#"
        tool test_loop() {
            for item in [1, 2, 3] {
                let x = item
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_for_loop_variable() {
    let source = r#"
        tool test_loop() {
            let items = [1, 2, 3]
            for item in items {
                let x = item
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_for_loop_range() {
    let source = r#"
        tool test_loop() {
            for i in 0..5 {
                let x = i
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_nested_for_loops() {
    let source = r#"
        tool test_loop() {
            for i in [1, 2] {
                for j in [3, 4] {
                    let x = i
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// Evaluation tests

#[tokio::test]
async fn test_eval_for_loop_array() {
    let source = r#"
        tool test_loop() {
            for item in [1, 2, 3] {
                let x = item
            }
        }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_for_loop_range() {
    let source = r#"
        tool test_loop() {
            for i in 0..5 {
                let x = i
            }
        }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_for_loop_string() {
    let source = r#"
        tool test_loop() {
            for char in "hello" {
                let x = char
            }
        }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_for_loop_empty_array() {
    let source = r#"
        tool test_loop() {
            for item in [] {
                let x = item
            }
        }
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}