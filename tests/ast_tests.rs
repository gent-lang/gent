use gent::parser::{AgentDecl, AgentField, Expression, Program, RunStmt, Statement};
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
        span: Span::new(0, 50),
    };
    assert_eq!(agent.fields.len(), 2);
}

#[test]
fn test_agent_decl_empty_fields() {
    let agent = AgentDecl {
        name: "Empty".to_string(),
        fields: vec![],
        span: Span::new(0, 10),
    };
    assert!(agent.fields.is_empty());
}

#[test]
fn test_agent_decl_equality() {
    let a1 = AgentDecl {
        name: "Test".to_string(),
        fields: vec![],
        span: Span::new(0, 10),
    };
    let a2 = a1.clone();
    assert_eq!(a1, a2);
}

// ============================================
// RunStmt Tests
// ============================================

#[test]
fn test_run_stmt_simple() {
    let run = RunStmt {
        agent_name: "Hello".to_string(),
        input: None,
        span: Span::new(0, 9),
    };
    assert_eq!(run.agent_name, "Hello");
    assert!(run.input.is_none());
}

#[test]
fn test_run_stmt_with_input() {
    let run = RunStmt {
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
    let r1 = RunStmt {
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
        span: Span::new(0, 10),
    });
    match stmt {
        Statement::AgentDecl(a) => assert_eq!(a.name, "Hello"),
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_statement_run_stmt() {
    let stmt = Statement::RunStmt(RunStmt {
        agent_name: "Hello".to_string(),
        input: None,
        span: Span::new(0, 9),
    });
    match stmt {
        Statement::RunStmt(r) => assert_eq!(r.agent_name, "Hello"),
        _ => panic!("Expected RunStmt"),
    }
}

#[test]
fn test_statement_equality() {
    let s1 = Statement::RunStmt(RunStmt {
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
        statements: vec![Statement::RunStmt(RunStmt {
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
                span: Span::new(0, 10),
            }),
            Statement::RunStmt(RunStmt {
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
                span: Span::new(0, 43),
            }),
            Statement::RunStmt(RunStmt {
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
        Statement::RunStmt(run) => {
            assert_eq!(run.agent_name, "Hello");
            assert!(run.input.is_none());
        }
        _ => panic!("Expected RunStmt"),
    }
}
