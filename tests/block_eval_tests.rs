//! Tests for block evaluation with scoping

use gent::interpreter::{Environment, Value};
use gent::parser::ast::{Block, BlockStmt, Expression, IfStmt, LetStmt, ReturnStmt};
use gent::runtime::tools::ToolRegistry;
use gent::Span;

// Import the block evaluator
use gent::interpreter::block_eval::evaluate_block;

#[tokio::test]
async fn test_block_empty() {
    let block = Block {
        statements: vec![],
        span: Span::new(0, 0),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[tokio::test]
async fn test_block_return_value() {
    // return 42
    let block = Block {
        statements: vec![BlockStmt::Return(ReturnStmt {
            value: Some(Expression::Number(42.0, Span::new(0, 2))),
            span: Span::new(0, 9),
        })],
        span: Span::new(0, 10),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[tokio::test]
async fn test_block_let_binding() {
    // let x = 10
    // return x
    let block = Block {
        statements: vec![
            BlockStmt::Let(LetStmt {
                name: "x".to_string(),
                value: Expression::Number(10.0, Span::new(0, 2)),
                span: Span::new(0, 10),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Identifier("x".to_string(), Span::new(11, 12))),
                span: Span::new(11, 20),
            }),
        ],
        span: Span::new(0, 21),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(10.0));
}

#[tokio::test]
async fn test_block_if_then() {
    // if true { return 1 }
    let block = Block {
        statements: vec![BlockStmt::If(IfStmt {
            condition: Expression::Boolean(true, Span::new(3, 7)),
            then_block: Block {
                statements: vec![BlockStmt::Return(ReturnStmt {
                    value: Some(Expression::Number(1.0, Span::new(10, 11))),
                    span: Span::new(10, 18),
                })],
                span: Span::new(8, 19),
            },
            else_block: None,
            span: Span::new(0, 20),
        })],
        span: Span::new(0, 20),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[tokio::test]
async fn test_block_if_else() {
    // if false { return 1 } else { return 2 }
    let block = Block {
        statements: vec![BlockStmt::If(IfStmt {
            condition: Expression::Boolean(false, Span::new(3, 8)),
            then_block: Block {
                statements: vec![BlockStmt::Return(ReturnStmt {
                    value: Some(Expression::Number(1.0, Span::new(11, 12))),
                    span: Span::new(11, 19),
                })],
                span: Span::new(9, 20),
            },
            else_block: Some(Block {
                statements: vec![BlockStmt::Return(ReturnStmt {
                    value: Some(Expression::Number(2.0, Span::new(28, 29))),
                    span: Span::new(28, 36),
                })],
                span: Span::new(26, 37),
            }),
            span: Span::new(0, 38),
        })],
        span: Span::new(0, 38),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(2.0));
}

#[tokio::test]
async fn test_block_multiple_lets() {
    // let x = 5
    // let y = x + 3
    // return y
    let block = Block {
        statements: vec![
            BlockStmt::Let(LetStmt {
                name: "x".to_string(),
                value: Expression::Number(5.0, Span::new(0, 1)),
                span: Span::new(0, 9),
            }),
            BlockStmt::Let(LetStmt {
                name: "y".to_string(),
                value: Expression::Binary(
                    gent::parser::ast::BinaryOp::Add,
                    Box::new(Expression::Identifier("x".to_string(), Span::new(10, 11))),
                    Box::new(Expression::Number(3.0, Span::new(14, 15))),
                    Span::new(10, 15),
                ),
                span: Span::new(10, 23),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Identifier("y".to_string(), Span::new(24, 25))),
                span: Span::new(24, 32),
            }),
        ],
        span: Span::new(0, 33),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(8.0));
}

#[tokio::test]
async fn test_block_expr_statement() {
    // let x = 1
    // x + 5  // expression statement (side effect only)
    // return x
    let block = Block {
        statements: vec![
            BlockStmt::Let(LetStmt {
                name: "x".to_string(),
                value: Expression::Number(1.0, Span::new(0, 1)),
                span: Span::new(0, 9),
            }),
            BlockStmt::Expr(Expression::Binary(
                gent::parser::ast::BinaryOp::Add,
                Box::new(Expression::Identifier("x".to_string(), Span::new(10, 11))),
                Box::new(Expression::Number(5.0, Span::new(14, 15))),
                Span::new(10, 15),
            )),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::Identifier("x".to_string(), Span::new(16, 17))),
                span: Span::new(16, 24),
            }),
        ],
        span: Span::new(0, 25),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    // x should still be 1 since expression statement doesn't modify it
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[tokio::test]
async fn test_block_scoping() {
    // Test that blocks create their own scope
    // let x = 5
    // if true {
    //     let x = 10
    //     return x  // Should return 10, not 5
    // }
    let block = Block {
        statements: vec![
            BlockStmt::Let(LetStmt {
                name: "x".to_string(),
                value: Expression::Number(5.0, Span::new(0, 1)),
                span: Span::new(0, 9),
            }),
            BlockStmt::If(IfStmt {
                condition: Expression::Boolean(true, Span::new(13, 17)),
                then_block: Block {
                    statements: vec![
                        BlockStmt::Let(LetStmt {
                            name: "x".to_string(),
                            value: Expression::Number(10.0, Span::new(20, 22)),
                            span: Span::new(20, 30),
                        }),
                        BlockStmt::Return(ReturnStmt {
                            value: Some(Expression::Identifier("x".to_string(), Span::new(31, 32))),
                            span: Span::new(31, 39),
                        }),
                    ],
                    span: Span::new(18, 40),
                },
                else_block: None,
                span: Span::new(10, 41),
            }),
        ],
        span: Span::new(0, 42),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(10.0));
}
