//! Tests for built-in functions (print, println)

use gent::interpreter::builtins::{call_builtin, is_builtin};
use gent::interpreter::Value;
use gent::Span;

#[test]
fn test_is_builtin() {
    assert!(is_builtin("print"));
    assert!(is_builtin("println"));
    assert!(!is_builtin("not_a_builtin"));
}

#[test]
fn test_print_single_arg() {
    let args = vec![Value::String("hello".to_string())];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_println_single_arg() {
    let args = vec![Value::String("hello".to_string())];
    let result = call_builtin("println", &args, &Span::default());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_print_multiple_args() {
    let args = vec![
        Value::String("hello".to_string()),
        Value::String("world".to_string()),
    ];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_ok());
}

#[test]
fn test_println_no_args() {
    let args = vec![];
    let result = call_builtin("println", &args, &Span::default());
    assert!(result.is_ok());
}

#[test]
fn test_print_type_error() {
    let args = vec![Value::Number(42.0)];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_err());
}

#[test]
fn test_print_mixed_type_error() {
    let args = vec![
        Value::String("hello".to_string()),
        Value::Number(42.0),
    ];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_err());
}

// Integration tests - test builtins work within tool evaluation

#[tokio::test]
async fn test_println_in_tool() {
    let source = r#"
        tool test() {
            println("Hello, World!")
            return "done"
        }
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed to evaluate: {:?}", result.err());

    // Execute the tool to verify println works
    let tool = tools.get("test").expect("Tool 'test' should be registered");
    let exec_result = tool.execute(serde_json::json!({})).await;
    assert!(exec_result.is_ok(), "Failed to execute tool: {:?}", exec_result.err());
    assert_eq!(exec_result.unwrap(), "done");
}

#[tokio::test]
async fn test_print_multiple_args_in_tool() {
    let source = r#"
        tool test() {
            let name = "World"
            println("Hello", "{name}!")
            return "done"
        }
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed to evaluate: {:?}", result.err());

    // Execute the tool to verify print with multiple args works
    let tool = tools.get("test").expect("Tool 'test' should be registered");
    let exec_result = tool.execute(serde_json::json!({})).await;
    assert!(exec_result.is_ok(), "Failed to execute tool: {:?}", exec_result.err());
    assert_eq!(exec_result.unwrap(), "done");
}

#[tokio::test]
async fn test_print_type_error_in_tool() {
    let source = r#"
        tool test() {
            println(42)
            return "done"
        }
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed to evaluate: {:?}", result.err());

    // Execute the tool - this should fail with type error
    let tool = tools.get("test").expect("Tool 'test' should be registered");
    let exec_result = tool.execute(serde_json::json!({})).await;
    assert!(exec_result.is_err(), "Should have failed with type error");
}
