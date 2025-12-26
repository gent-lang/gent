use gent::interpreter::{Environment, UserToolValue};
use gent::parser::ast::{Block, BlockStmt, Expression, ForStmt, ReturnStmt, TypeName};
use gent::parser::parse;
use gent::runtime::tools::{Tool, UserToolWrapper};
use gent::Span;
use serde_json::json;
use std::sync::Arc;

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

// ============================================
// Evaluation tests - these actually execute for loops
// ============================================

/// Helper to create a tool that has a for loop and execute it
async fn execute_tool_with_for_loop(
    for_stmt: ForStmt,
    additional_stmts: Vec<BlockStmt>,
) -> Result<String, String> {
    let mut statements = vec![BlockStmt::For(for_stmt)];
    statements.extend(additional_stmts);

    let tool_value = UserToolValue {
        name: "test_loop".to_string(),
        params: vec![],
        return_type: Some(TypeName::String),
        body: Block {
            statements,
            span: Span::default(),
        },
    };

    let env = Arc::new(Environment::new());
    let wrapper = UserToolWrapper::new(tool_value, env);

    wrapper.execute(json!({})).await
}

#[tokio::test]
async fn test_eval_for_loop_array() {
    // for item in [1, 2, 3] { let x = item }
    // return "done"
    let for_stmt = ForStmt {
        variable: "item".to_string(),
        iterable: Expression::Array(
            vec![
                Expression::Number(1.0, Span::default()),
                Expression::Number(2.0, Span::default()),
                Expression::Number(3.0, Span::default()),
            ],
            Span::default(),
        ),
        body: Block {
            statements: vec![BlockStmt::Let(gent::parser::ast::LetStmt {
                name: "x".to_string(),
                value: Expression::Identifier("item".to_string(), Span::default()),
                span: Span::default(),
            })],
            span: Span::default(),
        },
        span: Span::default(),
    };

    let result = execute_tool_with_for_loop(
        for_stmt,
        vec![BlockStmt::Return(ReturnStmt {
            value: Some(Expression::String(
                vec![gent::parser::ast::StringPart::Literal("done".to_string())],
                Span::default(),
            )),
            span: Span::default(),
        })],
    )
    .await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), "done");
}

#[tokio::test]
async fn test_eval_for_loop_range() {
    // for i in 0..3 { let x = i }
    // return "done"
    let for_stmt = ForStmt {
        variable: "i".to_string(),
        iterable: Expression::Range(
            Box::new(Expression::Number(0.0, Span::default())),
            Box::new(Expression::Number(3.0, Span::default())),
            Span::default(),
        ),
        body: Block {
            statements: vec![BlockStmt::Let(gent::parser::ast::LetStmt {
                name: "x".to_string(),
                value: Expression::Identifier("i".to_string(), Span::default()),
                span: Span::default(),
            })],
            span: Span::default(),
        },
        span: Span::default(),
    };

    let result = execute_tool_with_for_loop(
        for_stmt,
        vec![BlockStmt::Return(ReturnStmt {
            value: Some(Expression::String(
                vec![gent::parser::ast::StringPart::Literal("done".to_string())],
                Span::default(),
            )),
            span: Span::default(),
        })],
    )
    .await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), "done");
}

#[tokio::test]
async fn test_eval_for_loop_string() {
    // for char in "abc" { let x = char }
    // return "done"
    let for_stmt = ForStmt {
        variable: "char".to_string(),
        iterable: Expression::String(
            vec![gent::parser::ast::StringPart::Literal("abc".to_string())],
            Span::default(),
        ),
        body: Block {
            statements: vec![BlockStmt::Let(gent::parser::ast::LetStmt {
                name: "x".to_string(),
                value: Expression::Identifier("char".to_string(), Span::default()),
                span: Span::default(),
            })],
            span: Span::default(),
        },
        span: Span::default(),
    };

    let result = execute_tool_with_for_loop(
        for_stmt,
        vec![BlockStmt::Return(ReturnStmt {
            value: Some(Expression::String(
                vec![gent::parser::ast::StringPart::Literal("done".to_string())],
                Span::default(),
            )),
            span: Span::default(),
        })],
    )
    .await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), "done");
}

#[tokio::test]
async fn test_eval_for_loop_empty_array() {
    // for item in [] { let x = item }
    // return "done"
    let for_stmt = ForStmt {
        variable: "item".to_string(),
        iterable: Expression::Array(vec![], Span::default()),
        body: Block {
            statements: vec![BlockStmt::Let(gent::parser::ast::LetStmt {
                name: "x".to_string(),
                value: Expression::Identifier("item".to_string(), Span::default()),
                span: Span::default(),
            })],
            span: Span::default(),
        },
        span: Span::default(),
    };

    let result = execute_tool_with_for_loop(
        for_stmt,
        vec![BlockStmt::Return(ReturnStmt {
            value: Some(Expression::String(
                vec![gent::parser::ast::StringPart::Literal("done".to_string())],
                Span::default(),
            )),
            span: Span::default(),
        })],
    )
    .await;

    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), "done");
}