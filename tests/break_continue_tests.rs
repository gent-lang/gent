use gent::interpreter::block_eval::evaluate_block;
use gent::interpreter::{Environment, Value};
use gent::parser::ast::{BinaryOp, Block, BlockStmt, Expression, ForStmt, IfStmt, ReturnStmt};
use gent::parser::{parse, Statement};
use gent::runtime::tools::ToolRegistry;
use gent::Span;

/// Test: break directly in for loop body should exit the loop
#[tokio::test]
async fn test_eval_break_directly_in_for_loop() {
    // for i in [1, 2, 3] { break }
    // return "done"
    let block = Block {
        statements: vec![
            BlockStmt::For(ForStmt {
                variable: "i".to_string(),
                iterable: Expression::Array(
                    vec![
                        Expression::Number(1.0, Span::new(0, 1)),
                        Expression::Number(2.0, Span::new(0, 1)),
                        Expression::Number(3.0, Span::new(0, 1)),
                    ],
                    Span::new(0, 10),
                ),
                body: Block {
                    statements: vec![BlockStmt::Break(Span::new(0, 5))],
                    span: Span::new(0, 10),
                },
                span: Span::new(0, 20),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::StringPart::Literal("done".to_string())],
                    Span::new(0, 6),
                )),
                span: Span::new(0, 13),
            }),
        ],
        span: Span::new(0, 50),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("done".to_string()));
}

/// Test: continue directly in for loop body should skip to next iteration
#[tokio::test]
async fn test_eval_continue_directly_in_for_loop() {
    // for i in [1, 2, 3] { continue }
    // return "done"
    let block = Block {
        statements: vec![
            BlockStmt::For(ForStmt {
                variable: "i".to_string(),
                iterable: Expression::Array(
                    vec![
                        Expression::Number(1.0, Span::new(0, 1)),
                        Expression::Number(2.0, Span::new(0, 1)),
                        Expression::Number(3.0, Span::new(0, 1)),
                    ],
                    Span::new(0, 10),
                ),
                body: Block {
                    statements: vec![BlockStmt::Continue(Span::new(0, 8))],
                    span: Span::new(0, 10),
                },
                span: Span::new(0, 20),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::StringPart::Literal("done".to_string())],
                    Span::new(0, 6),
                )),
                span: Span::new(0, 13),
            }),
        ],
        span: Span::new(0, 50),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("done".to_string()));
}

/// Test: break inside an if statement within a for loop
#[tokio::test]
async fn test_eval_break_in_if_in_for_loop() {
    // for i in [1, 2, 3, 4, 5] {
    //     if i == 3 { break }
    // }
    // return "done"
    let block = Block {
        statements: vec![
            BlockStmt::For(ForStmt {
                variable: "i".to_string(),
                iterable: Expression::Array(
                    vec![
                        Expression::Number(1.0, Span::new(0, 1)),
                        Expression::Number(2.0, Span::new(0, 1)),
                        Expression::Number(3.0, Span::new(0, 1)),
                        Expression::Number(4.0, Span::new(0, 1)),
                        Expression::Number(5.0, Span::new(0, 1)),
                    ],
                    Span::new(0, 15),
                ),
                body: Block {
                    statements: vec![BlockStmt::If(IfStmt {
                        condition: Expression::Binary(
                            BinaryOp::Eq,
                            Box::new(Expression::Identifier("i".to_string(), Span::new(0, 1))),
                            Box::new(Expression::Number(3.0, Span::new(0, 1))),
                            Span::new(0, 6),
                        ),
                        then_block: Block {
                            statements: vec![BlockStmt::Break(Span::new(0, 5))],
                            span: Span::new(0, 10),
                        },
                        else_block: None,
                        span: Span::new(0, 20),
                    })],
                    span: Span::new(0, 25),
                },
                span: Span::new(0, 30),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::StringPart::Literal("done".to_string())],
                    Span::new(0, 6),
                )),
                span: Span::new(0, 13),
            }),
        ],
        span: Span::new(0, 60),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("done".to_string()));
}

/// Test: continue inside an if statement within a for loop
#[tokio::test]
async fn test_eval_continue_in_if_in_for_loop() {
    // for i in [1, 2, 3] {
    //     if i == 2 { continue }
    // }
    // return "done"
    let block = Block {
        statements: vec![
            BlockStmt::For(ForStmt {
                variable: "i".to_string(),
                iterable: Expression::Array(
                    vec![
                        Expression::Number(1.0, Span::new(0, 1)),
                        Expression::Number(2.0, Span::new(0, 1)),
                        Expression::Number(3.0, Span::new(0, 1)),
                    ],
                    Span::new(0, 10),
                ),
                body: Block {
                    statements: vec![BlockStmt::If(IfStmt {
                        condition: Expression::Binary(
                            BinaryOp::Eq,
                            Box::new(Expression::Identifier("i".to_string(), Span::new(0, 1))),
                            Box::new(Expression::Number(2.0, Span::new(0, 1))),
                            Span::new(0, 6),
                        ),
                        then_block: Block {
                            statements: vec![BlockStmt::Continue(Span::new(0, 8))],
                            span: Span::new(0, 12),
                        },
                        else_block: None,
                        span: Span::new(0, 20),
                    })],
                    span: Span::new(0, 25),
                },
                span: Span::new(0, 30),
            }),
            BlockStmt::Return(ReturnStmt {
                value: Some(Expression::String(
                    vec![gent::parser::StringPart::Literal("done".to_string())],
                    Span::new(0, 6),
                )),
                span: Span::new(0, 13),
            }),
        ],
        span: Span::new(0, 60),
    };

    let mut env = Environment::new();
    let tools = ToolRegistry::new();

    let result = evaluate_block(&block, &mut env, &tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    assert_eq!(result.unwrap(), Value::String("done".to_string()));
}

#[test]
fn test_parse_break() {
    let source = r#"
        tool test() {
            for i in [1, 2, 3] {
                break
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    // Verify break is parsed as BlockStmt::Break, not as an identifier expression
    let program = result.unwrap();
    let tool = match &program.statements[0] {
        Statement::ToolDecl(t) => t,
        _ => panic!("Expected ToolDecl"),
    };

    let for_stmt = match &tool.body.statements[0] {
        BlockStmt::For(f) => f,
        _ => panic!("Expected ForStmt"),
    };

    match &for_stmt.body.statements[0] {
        BlockStmt::Break(_) => {} // Success!
        other => panic!("Expected Break, got {:?}", other),
    }
}

#[test]
fn test_parse_continue() {
    let source = r#"
        tool test() {
            for i in [1, 2, 3] {
                continue
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    // Verify continue is parsed as BlockStmt::Continue, not as an identifier expression
    let program = result.unwrap();
    let tool = match &program.statements[0] {
        Statement::ToolDecl(t) => t,
        _ => panic!("Expected ToolDecl"),
    };

    let for_stmt = match &tool.body.statements[0] {
        BlockStmt::For(f) => f,
        _ => panic!("Expected ForStmt"),
    };

    match &for_stmt.body.statements[0] {
        BlockStmt::Continue(_) => {} // Success!
        other => panic!("Expected Continue, got {:?}", other),
    }
}
