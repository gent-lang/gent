use gent::interpreter::block_eval::evaluate_block;
use gent::interpreter::{Environment, Value};
use gent::parser::ast::{Block, BlockStmt, Expression, LetStmt, ReturnStmt, TryStmt};
use gent::parser::parse;
use gent::runtime::tools::ToolRegistry;
use gent::Span;

// ============================================
// Evaluation Integration Tests (using evaluate_block)
// ============================================

/// Test: try block succeeds, catch is not executed
#[tokio::test]
async fn test_eval_try_catch_no_error() {
    // try { let x = 42 } catch error { return "caught" }
    // return "success"
    let block = Block {
        statements: vec![
            BlockStmt::Try(TryStmt {
                try_block: Block {
                    statements: vec![BlockStmt::Let(LetStmt {
                        name: "x".to_string(),
                        value: Expression::Number(42.0, Span::new(0, 2)),
                        span: Span::new(0, 10),
                    })],
                    span: Span::new(0, 15),
                },
                error_var: "error".to_string(),
                catch_block: Block {
                    statements: vec![BlockStmt::Return(ReturnStmt {
                        value: Some(Expression::String(
                            vec![gent::parser::ast::StringPart::Literal("caught".to_string())],
                            Span::new(0, 8),
                        )),
                        span: Span::new(0, 15),
                    })],
                    span: Span::new(0, 20),
                },
                span: Span::new(0, 40),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::ast::StringPart::Literal("success".to_string())],
                    Span::new(0, 9),
                )),
                span: Span::new(0, 16),
            }),
        ],
        span: Span::new(0, 60),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("success".to_string()));
}

/// Test: return inside try block exits immediately
#[tokio::test]
async fn test_eval_try_catch_with_return_in_try() {
    // try { return "from try" } catch error { return "caught" }
    // return "after"
    let block = Block {
        statements: vec![
            BlockStmt::Try(TryStmt {
                try_block: Block {
                    statements: vec![BlockStmt::Return(ReturnStmt {
                        value: Some(Expression::String(
                            vec![gent::parser::ast::StringPart::Literal("from try".to_string())],
                            Span::new(0, 10),
                        )),
                        span: Span::new(0, 17),
                    })],
                    span: Span::new(0, 22),
                },
                error_var: "error".to_string(),
                catch_block: Block {
                    statements: vec![BlockStmt::Return(ReturnStmt {
                        value: Some(Expression::String(
                            vec![gent::parser::ast::StringPart::Literal("caught".to_string())],
                            Span::new(0, 8),
                        )),
                        span: Span::new(0, 15),
                    })],
                    span: Span::new(0, 20),
                },
                span: Span::new(0, 50),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::ast::StringPart::Literal("after".to_string())],
                    Span::new(0, 7),
                )),
                span: Span::new(0, 14),
            }),
        ],
        span: Span::new(0, 70),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("from try".to_string()));
}

/// Test: nested try/catch blocks work correctly
#[tokio::test]
async fn test_eval_nested_try_catch() {
    // try { try { let x = 1 } catch innerErr { let y = 2 } } catch outerErr { let z = 3 }
    // return "done"
    let block = Block {
        statements: vec![
            BlockStmt::Try(TryStmt {
                try_block: Block {
                    statements: vec![BlockStmt::Try(TryStmt {
                        try_block: Block {
                            statements: vec![BlockStmt::Let(LetStmt {
                                name: "x".to_string(),
                                value: Expression::Number(1.0, Span::new(0, 1)),
                                span: Span::new(0, 9),
                            })],
                            span: Span::new(0, 14),
                        },
                        error_var: "innerErr".to_string(),
                        catch_block: Block {
                            statements: vec![BlockStmt::Let(LetStmt {
                                name: "y".to_string(),
                                value: Expression::Number(2.0, Span::new(0, 1)),
                                span: Span::new(0, 9),
                            })],
                            span: Span::new(0, 14),
                        },
                        span: Span::new(0, 35),
                    })],
                    span: Span::new(0, 40),
                },
                error_var: "outerErr".to_string(),
                catch_block: Block {
                    statements: vec![BlockStmt::Let(LetStmt {
                        name: "z".to_string(),
                        value: Expression::Number(3.0, Span::new(0, 1)),
                        span: Span::new(0, 9),
                    })],
                    span: Span::new(0, 14),
                },
                span: Span::new(0, 60),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::ast::StringPart::Literal("done".to_string())],
                    Span::new(0, 6),
                )),
                span: Span::new(0, 13),
            }),
        ],
        span: Span::new(0, 80),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("done".to_string()));
}

/// Test: try/catch normal completion flows to next statement
#[tokio::test]
async fn test_eval_try_catch_normal_completion() {
    // try { let x = 1 } catch error { let y = 2 }
    // return "completed"
    let block = Block {
        statements: vec![
            BlockStmt::Try(TryStmt {
                try_block: Block {
                    statements: vec![BlockStmt::Let(LetStmt {
                        name: "x".to_string(),
                        value: Expression::Number(1.0, Span::new(0, 1)),
                        span: Span::new(0, 9),
                    })],
                    span: Span::new(0, 14),
                },
                error_var: "error".to_string(),
                catch_block: Block {
                    statements: vec![BlockStmt::Let(LetStmt {
                        name: "y".to_string(),
                        value: Expression::Number(2.0, Span::new(0, 1)),
                        span: Span::new(0, 9),
                    })],
                    span: Span::new(0, 14),
                },
                span: Span::new(0, 35),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::ast::StringPart::Literal("completed".to_string())],
                    Span::new(0, 11),
                )),
                span: Span::new(0, 18),
            }),
        ],
        span: Span::new(0, 60),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("completed".to_string()));
}

// ============================================
// Parser Tests
// ============================================

#[test]
fn test_parse_try_catch() {
    let source = r#"
        tool test() {
            try {
                let x = 1
            } catch error {
                let msg = error
            }
            return "done"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_function_call() {
    let source = r#"
        tool test() {
            try {
                let result = riskyOperation()
            } catch err {
                let fallback = "error occurred"
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_nested_try_catch() {
    let source = r#"
        tool test() {
            try {
                try {
                    let x = 1
                } catch innerErr {
                    let y = 2
                }
            } catch outerErr {
                let z = 3
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_return() {
    let source = r#"
        tool test() {
            try {
                return "success"
            } catch error {
                return "failure"
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_in_loop() {
    let source = r#"
        tool test() {
            for i in 1..5 {
                try {
                    let x = i
                } catch error {
                    break
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_if() {
    let source = r#"
        tool test() {
            try {
                if true {
                    let x = 1
                }
            } catch error {
                if error == "fatal" {
                    return "abort"
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
