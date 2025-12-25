//! Tests for UserToolWrapper functionality

use gent::interpreter::{Environment, UserToolValue};
use gent::parser::ast::{BinaryOp, Block, BlockStmt, Expression, Param, ReturnStmt, TypeName};
use gent::runtime::tools::{Tool, ToolRegistry, UserToolWrapper};
use gent::Span;
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_user_tool_wrapper_execute() {
    // Create a simple tool: greet(name: String) -> String { return "Hello, " + name }
    let tool_value = UserToolValue {
        name: "greet".to_string(),
        params: vec![Param {
            name: "name".to_string(),
            type_name: TypeName::String,
            span: Span::default(),
        }],
        return_type: Some(TypeName::String),
        body: Block {
            statements: vec![BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Binary(
                    BinaryOp::Add,
                    Box::new(Expression::String("Hello, ".to_string(), Span::default())),
                    Box::new(Expression::Identifier("name".to_string(), Span::default())),
                    Span::default(),
                )),
                span: Span::default(),
            })],
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    // Test tool name
    assert_eq!(wrapper.name(), "greet");

    // Test tool description
    assert!(!wrapper.description().is_empty());

    // Test parameters schema
    let schema = wrapper.parameters_schema();
    assert_eq!(schema["type"], "object");
    assert!(schema["properties"]["name"].is_object());
    assert_eq!(schema["required"][0], "name");

    // Test tool execution
    let args = json!({"name": "World"});
    let result = wrapper.execute(args).await.unwrap();
    assert_eq!(result, "Hello, World");
}

#[tokio::test]
async fn test_user_tool_wrapper_with_number_param() {
    // Create a tool: double(x: Number) -> Number { return x * 2 }
    let tool_value = UserToolValue {
        name: "double".to_string(),
        params: vec![Param {
            name: "x".to_string(),
            type_name: TypeName::Number,
            span: Span::default(),
        }],
        return_type: Some(TypeName::Number),
        body: Block {
            statements: vec![BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Binary(
                    BinaryOp::Mul,
                    Box::new(Expression::Identifier("x".to_string(), Span::default())),
                    Box::new(Expression::Number(2.0, Span::default())),
                    Span::default(),
                )),
                span: Span::default(),
            })],
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    assert_eq!(wrapper.name(), "double");

    // Test execution
    let args = json!({"x": 5});
    let result = wrapper.execute(args).await.unwrap();
    assert_eq!(result, "10");
}

#[tokio::test]
async fn test_user_tool_wrapper_with_multiple_params() {
    // Create a tool: add(a: Number, b: Number) -> Number { return a + b }
    let tool_value = UserToolValue {
        name: "add".to_string(),
        params: vec![
            Param {
                name: "a".to_string(),
                type_name: TypeName::Number,
                span: Span::default(),
            },
            Param {
                name: "b".to_string(),
                type_name: TypeName::Number,
                span: Span::default(),
            },
        ],
        return_type: Some(TypeName::Number),
        body: Block {
            statements: vec![BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Binary(
                    BinaryOp::Add,
                    Box::new(Expression::Identifier("a".to_string(), Span::default())),
                    Box::new(Expression::Identifier("b".to_string(), Span::default())),
                    Span::default(),
                )),
                span: Span::default(),
            })],
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    assert_eq!(wrapper.name(), "add");

    // Test execution
    let args = json!({"a": 3, "b": 7});
    let result = wrapper.execute(args).await.unwrap();
    assert_eq!(result, "10");
}

#[tokio::test]
async fn test_user_tool_wrapper_missing_parameter() {
    // Create a simple tool
    let tool_value = UserToolValue {
        name: "greet".to_string(),
        params: vec![Param {
            name: "name".to_string(),
            type_name: TypeName::String,
            span: Span::default(),
        }],
        return_type: Some(TypeName::String),
        body: Block {
            statements: vec![],
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    // Test execution with missing parameter
    let args = json!({});
    let result = wrapper.execute(args).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name"));
}

#[tokio::test]
async fn test_user_tool_wrapper_in_registry() {
    // Create a tool
    let tool_value = UserToolValue {
        name: "test_tool".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String("success".to_string(), Span::default())),
                span: Span::default(),
            })],
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    // Register the tool
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(wrapper));

    // Get the tool from registry
    let tool = registry.get("test_tool").unwrap();
    assert_eq!(tool.name(), "test_tool");

    // Execute through registry
    let result = tool.execute(json!({})).await.unwrap();
    assert_eq!(result, "success");
}
