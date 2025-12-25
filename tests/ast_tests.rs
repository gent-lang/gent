use gent::parser::{AgentDecl, AgentField, Expression, Program, AgentCall, Statement};
use gent::Span;

// ============================================
// Span Integration Tests
// ============================================

#[test]
fn test_span_in_ast_nodes() {
    let span = Span::new(0, 10);
    let expr = Expression::String("hello".to_string(), span.clone());
    match expr {
        Expression::String(_, s) => assert_eq!(s, span),
        _ => panic!("Expected String expression"),
    }
}

// ============================================
// Expression Tests
// ============================================

#[test]
fn test_string_expression() {
    let expr = Expression::String("hello".to_string(), Span::new(0, 7));
    match &expr {
        Expression::String(s, _) => assert_eq!(s, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_number_expression_integer() {
    let expr = Expression::Number(42.0, Span::new(0, 2));
    match &expr {
        Expression::Number(n, _) => assert_eq!(*n, 42.0),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_number_expression_float() {
    let expr = Expression::Number(3.14, Span::new(0, 4));
    match &expr {
        Expression::Number(n, _) => assert!((n - 3.14).abs() < 0.001),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_boolean_expression_true() {
    let expr = Expression::Boolean(true, Span::new(0, 4));
    match &expr {
        Expression::Boolean(b, _) => assert!(*b),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_boolean_expression_false() {
    let expr = Expression::Boolean(false, Span::new(0, 5));
    match &expr {
        Expression::Boolean(b, _) => assert!(!*b),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_identifier_expression() {
    let expr = Expression::Identifier("myVar".to_string(), Span::new(0, 5));
    match &expr {
        Expression::Identifier(name, _) => assert_eq!(name, "myVar"),
        _ => panic!("Expected Identifier"),
    }
}

#[test]
fn test_expression_equality() {
    let e1 = Expression::String("test".to_string(), Span::new(0, 6));
    let e2 = Expression::String("test".to_string(), Span::new(0, 6));
    let e3 = Expression::String("other".to_string(), Span::new(0, 7));
    assert_eq!(e1, e2);
    assert_ne!(e1, e3);
}

#[test]
fn test_expression_clone() {
    let e1 = Expression::Number(42.0, Span::new(0, 2));
    let e2 = e1.clone();
    assert_eq!(e1, e2);
}

#[test]
fn test_expression_debug() {
    let expr = Expression::Boolean(true, Span::new(0, 4));
    let debug = format!("{:?}", expr);
    assert!(debug.contains("Boolean"));
    assert!(debug.contains("true"));
}

#[test]
fn test_expression_span() {
    let span = Span::new(5, 10);
    let expr = Expression::String("x".to_string(), span.clone());
    assert_eq!(expr.span(), &span);
}

// ============================================
// AgentField Tests
// ============================================

#[test]
fn test_agent_field_creation() {
    let field = AgentField {
        name: "prompt".to_string(),
        value: Expression::String("You are helpful.".to_string(), Span::new(8, 26)),
        span: Span::new(0, 26),
    };
    assert_eq!(field.name, "prompt");
}

#[test]
fn test_agent_field_equality() {
    let f1 = AgentField {
        name: "prompt".to_string(),
        value: Expression::String("test".to_string(), Span::new(8, 14)),
        span: Span::new(0, 14),
    };
    let f2 = f1.clone();
    assert_eq!(f1, f2);
}

#[test]
fn test_agent_field_debug() {
    let field = AgentField {
        name: "prompt".to_string(),
        value: Expression::String("test".to_string(), Span::new(0, 4)),
        span: Span::new(0, 10),
    };
    let debug = format!("{:?}", field);
    assert!(debug.contains("prompt"));
}

// ============================================
// AgentDecl Tests
// ============================================

#[test]
fn test_agent_decl_creation() {
    let agent = AgentDecl {
        name: "Hello".to_string(),
        fields: vec![AgentField {
            name: "prompt".to_string(),
            value: Expression::String("You are friendly.".to_string(), Span::new(20, 39)),
            span: Span::new(12, 39),
        }],
        tools: vec![],
        output: None,
        span: Span::new(0, 41),
    };
    assert_eq!(agent.name, "Hello");
    assert_eq!(agent.fields.len(), 1);
}

#[test]
fn test_agent_decl_multiple_fields() {
    let agent = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![
            AgentField {
                name: "prompt".to_string(),
                value: Expression::String("Help users.".to_string(), Span::new(0, 11)),
                span: Span::new(0, 11),
            },
            AgentField {
                name: "model".to_string(),
                value: Expression::String("gpt-4".to_string(), Span::new(0, 7)),
                span: Span::new(0, 7),
            },
        ],
        tools: vec![],
        output: None,
        span: Span::new(0, 50),
    };
    assert_eq!(agent.fields.len(), 2);
}

#[test]
fn test_agent_decl_empty_fields() {
    let agent = AgentDecl {
        name: "Empty".to_string(),
        fields: vec![],
        tools: vec![],
        output: None,
        span: Span::new(0, 10),
    };
    assert!(agent.fields.is_empty());
}

#[test]
fn test_agent_decl_equality() {
    let a1 = AgentDecl {
        name: "Test".to_string(),
        fields: vec![],
        tools: vec![],
        output: None,
        span: Span::new(0, 10),
    };
    let a2 = a1.clone();
    assert_eq!(a1, a2);
}

// ============================================
// AgentCall Tests
// ============================================

#[test]
fn test_run_stmt_simple() {
    let run = AgentCall {
        agent_name: "Hello".to_string(),
        input: None,
        span: Span::new(0, 9),
    };
    assert_eq!(run.agent_name, "Hello");
    assert!(run.input.is_none());
}

#[test]
fn test_run_stmt_with_input() {
    let run = AgentCall {
        agent_name: "Greeter".to_string(),
        input: Some(Expression::String(
            "Hi there!".to_string(),
            Span::new(17, 28),
        )),
        span: Span::new(0, 28),
    };
    assert_eq!(run.agent_name, "Greeter");
    assert!(run.input.is_some());
}

#[test]
fn test_run_stmt_equality() {
    let r1 = AgentCall {
        agent_name: "Test".to_string(),
        input: None,
        span: Span::new(0, 8),
    };
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

// ============================================
// Statement Tests
// ============================================

#[test]
fn test_statement_agent_decl() {
    let stmt = Statement::AgentDecl(AgentDecl {
        name: "Hello".to_string(),
        fields: vec![],
        tools: vec![],
        output: None,
        span: Span::new(0, 10),
    });
    match stmt {
        Statement::AgentDecl(a) => assert_eq!(a.name, "Hello"),
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_statement_run_stmt() {
    let stmt = Statement::AgentCall(AgentCall {
        agent_name: "Hello".to_string(),
        input: None,
        span: Span::new(0, 9),
    });
    match stmt {
        Statement::AgentCall(r) => assert_eq!(r.agent_name, "Hello"),
        _ => panic!("Expected AgentCall"),
    }
}

#[test]
fn test_statement_equality() {
    let s1 = Statement::AgentCall(AgentCall {
        agent_name: "X".to_string(),
        input: None,
        span: Span::new(0, 5),
    });
    let s2 = s1.clone();
    assert_eq!(s1, s2);
}

// ============================================
// Program Tests
// ============================================

#[test]
fn test_program_empty() {
    let program = Program {
        statements: vec![],
        span: Span::new(0, 0),
    };
    assert!(program.statements.is_empty());
}

#[test]
fn test_program_single_statement() {
    let program = Program {
        statements: vec![Statement::AgentCall(AgentCall {
            agent_name: "Hello".to_string(),
            input: None,
            span: Span::new(0, 9),
        })],
        span: Span::new(0, 9),
    };
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn test_program_multiple_statements() {
    let program = Program {
        statements: vec![
            Statement::AgentDecl(AgentDecl {
                name: "Hello".to_string(),
                fields: vec![],
                tools: vec![],
                output: None,
                span: Span::new(0, 10),
            }),
            Statement::AgentCall(AgentCall {
                agent_name: "Hello".to_string(),
                input: None,
                span: Span::new(11, 20),
            }),
        ],
        span: Span::new(0, 20),
    };
    assert_eq!(program.statements.len(), 2);
}

#[test]
fn test_program_equality() {
    let p1 = Program {
        statements: vec![],
        span: Span::new(0, 0),
    };
    let p2 = p1.clone();
    assert_eq!(p1, p2);
}

#[test]
fn test_program_debug() {
    let program = Program {
        statements: vec![],
        span: Span::new(0, 0),
    };
    let debug = format!("{:?}", program);
    assert!(debug.contains("Program"));
}

// ============================================
// Full AST Construction Test
// ============================================

#[test]
fn test_hello_world_ast() {
    // Represents: agent Hello { prompt: "You are friendly." } run Hello
    let program = Program {
        statements: vec![
            Statement::AgentDecl(AgentDecl {
                name: "Hello".to_string(),
                fields: vec![AgentField {
                    name: "prompt".to_string(),
                    value: Expression::String("You are friendly.".to_string(), Span::new(22, 41)),
                    span: Span::new(14, 41),
                }],
                tools: vec![],
                output: None,
                span: Span::new(0, 43),
            }),
            Statement::AgentCall(AgentCall {
                agent_name: "Hello".to_string(),
                input: None,
                span: Span::new(44, 53),
            }),
        ],
        span: Span::new(0, 53),
    };

    assert_eq!(program.statements.len(), 2);

    // Verify first statement is agent decl
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.name, "Hello");
            assert_eq!(agent.fields.len(), 1);
            assert_eq!(agent.fields[0].name, "prompt");
        }
        _ => panic!("Expected AgentDecl"),
    }

    // Verify second statement is run stmt
    match &program.statements[1] {
        Statement::AgentCall(run) => {
            assert_eq!(run.agent_name, "Hello");
            assert!(run.input.is_none());
        }
        _ => panic!("Expected AgentCall"),
    }
}

// ============================================
// BinaryOp and UnaryOp Tests
// ============================================

use gent::parser::{BinaryOp, UnaryOp};

#[test]
fn test_binary_op_variants() {
    let ops = vec![
        BinaryOp::Add,
        BinaryOp::Sub,
        BinaryOp::Mul,
        BinaryOp::Div,
        BinaryOp::Mod,
        BinaryOp::Eq,
        BinaryOp::Ne,
        BinaryOp::Lt,
        BinaryOp::Le,
        BinaryOp::Gt,
        BinaryOp::Ge,
        BinaryOp::And,
        BinaryOp::Or,
    ];
    assert_eq!(ops.len(), 13);
}

#[test]
fn test_unary_op_variants() {
    let ops = vec![UnaryOp::Not, UnaryOp::Neg];
    assert_eq!(ops.len(), 2);
}

// ============================================
// TypeName Enum Tests
// ============================================

use gent::parser::TypeName as AstTypeName;

#[test]
fn test_ast_type_name_variants() {
    let types = vec![
        AstTypeName::String,
        AstTypeName::Number,
        AstTypeName::Boolean,
        AstTypeName::Object,
        AstTypeName::Array,
        AstTypeName::Any,
    ];
    assert_eq!(types.len(), 6);
}

// ============================================
// Block and BlockStmt Tests
// ============================================

use gent::parser::{Block, BlockStmt, IfStmt, LetStmt, ReturnStmt};

#[test]
fn test_block_creation() {
    let block = Block {
        statements: vec![],
        span: Span::new(0, 10),
    };
    assert!(block.statements.is_empty());
}

#[test]
fn test_let_stmt() {
    let stmt = BlockStmt::Let(LetStmt {
        name: "x".to_string(),
        value: Expression::Number(42.0, Span::new(0, 2)),
        span: Span::new(0, 10),
    });
    assert!(matches!(stmt, BlockStmt::Let(_)));
}

#[test]
fn test_return_stmt() {
    let stmt = BlockStmt::Return(ReturnStmt {
        value: Some(Expression::Number(1.0, Span::new(0, 1))),
        span: Span::new(0, 8),
    });
    assert!(matches!(stmt, BlockStmt::Return(_)));
}

#[test]
fn test_if_stmt() {
    let stmt = BlockStmt::If(IfStmt {
        condition: Expression::Boolean(true, Span::new(0, 4)),
        then_block: Block {
            statements: vec![],
            span: Span::new(0, 2),
        },
        else_block: None,
        span: Span::new(0, 20),
    });
    assert!(matches!(stmt, BlockStmt::If(_)));
}

#[test]
fn test_block_stmt_expr_variant() {
    let stmt = BlockStmt::Expr(Expression::Number(42.0, Span::new(0, 2)));
    assert!(matches!(stmt, BlockStmt::Expr(_)));
}

// ============================================
// ToolDecl and Param Tests
// ============================================

use gent::parser::{Param, ToolDecl};

#[test]
fn test_tool_decl_creation() {
    let tool = ToolDecl {
        name: "greet".to_string(),
        params: vec![Param {
            name: "name".to_string(),
            type_name: AstTypeName::String,
            span: Span::new(0, 12),
        }],
        return_type: Some(AstTypeName::String),
        body: Block {
            statements: vec![],
            span: Span::new(0, 2),
        },
        span: Span::new(0, 50),
    };
    assert_eq!(tool.name, "greet");
    assert_eq!(tool.params.len(), 1);
}

#[test]
fn test_statement_tool_decl_variant() {
    let tool = ToolDecl {
        name: "test".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![],
            span: Span::new(0, 2),
        },
        span: Span::new(0, 20),
    };
    let stmt = Statement::ToolDecl(tool);
    assert!(matches!(stmt, Statement::ToolDecl(_)));
}

#[test]
fn test_param_creation() {
    let param = Param {
        name: "x".to_string(),
        type_name: AstTypeName::Number,
        span: Span::new(0, 10),
    };
    assert_eq!(param.name, "x");
    assert_eq!(param.type_name, AstTypeName::Number);
}

// ============================================
// Extended Expression Tests (Task 11)
// ============================================

#[test]
fn test_expression_null() {
    let expr = Expression::Null(Span::new(0, 4));
    assert!(matches!(expr, Expression::Null(_)));
}

#[test]
fn test_expression_array() {
    let expr = Expression::Array(
        vec![Expression::Number(1.0, Span::new(1, 2))],
        Span::new(0, 3),
    );
    assert!(matches!(expr, Expression::Array(_, _)));
}

#[test]
fn test_expression_object() {
    let expr = Expression::Object(
        vec![(
            "key".to_string(),
            Expression::String("value".to_string(), Span::new(0, 5)),
        )],
        Span::new(0, 15),
    );
    assert!(matches!(expr, Expression::Object(_, _)));
}

#[test]
fn test_expression_binary() {
    let expr = Expression::Binary(
        BinaryOp::Add,
        Box::new(Expression::Number(1.0, Span::new(0, 1))),
        Box::new(Expression::Number(2.0, Span::new(4, 5))),
        Span::new(0, 5),
    );
    assert!(matches!(expr, Expression::Binary(_, _, _, _)));
}

#[test]
fn test_expression_unary() {
    let expr = Expression::Unary(
        UnaryOp::Neg,
        Box::new(Expression::Number(5.0, Span::new(1, 2))),
        Span::new(0, 2),
    );
    assert!(matches!(expr, Expression::Unary(_, _, _)));
}

#[test]
fn test_expression_call() {
    let expr = Expression::Call(
        Box::new(Expression::Identifier("foo".to_string(), Span::new(0, 3))),
        vec![Expression::Number(1.0, Span::new(4, 5))],
        Span::new(0, 6),
    );
    assert!(matches!(expr, Expression::Call(_, _, _)));
}

#[test]
fn test_expression_member() {
    let expr = Expression::Member(
        Box::new(Expression::Identifier("obj".to_string(), Span::new(0, 3))),
        "prop".to_string(),
        Span::new(0, 8),
    );
    assert!(matches!(expr, Expression::Member(_, _, _)));
}

#[test]
fn test_expression_index() {
    let expr = Expression::Index(
        Box::new(Expression::Identifier("arr".to_string(), Span::new(0, 3))),
        Box::new(Expression::Number(0.0, Span::new(4, 5))),
        Span::new(0, 6),
    );
    assert!(matches!(expr, Expression::Index(_, _, _)));
}
