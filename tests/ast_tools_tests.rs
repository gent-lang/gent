//! Tests for AgentDecl tools_expr field

use gent::parser::ast::{AgentDecl, AgentField, Expression, StringPart};
use gent::Span;

#[test]
fn test_agent_decl_with_tools_expr() {
    // Test with a tools expression (e.g., tools: [web_fetch, read_file])
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![],
        tools_expr: Some(Expression::Array(
            vec![
                Expression::Identifier("web_fetch".to_string(), Span::new(0, 9)),
                Expression::Identifier("read_file".to_string(), Span::new(11, 20)),
            ],
            Span::new(0, 21),
        )),
        output: None,
        span: Span::new(0, 10),
    };
    assert!(decl.tools_expr.is_some());
    if let Some(Expression::Array(items, _)) = &decl.tools_expr {
        assert_eq!(items.len(), 2);
    }
}

#[test]
fn test_agent_decl_without_tools() {
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![],
        tools_expr: None,
        output: None,
        span: Span::new(0, 10),
    };
    assert!(decl.tools_expr.is_none());
}

#[test]
fn test_agent_decl_with_tools_and_fields() {
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![AgentField {
            name: "systemPrompt".to_string(),
            value: Expression::String(vec![StringPart::Literal("Hi".to_string())], Span::new(0, 2)),
            span: Span::new(0, 10),
        }],
        tools_expr: Some(Expression::Array(
            vec![Expression::Identifier("web_fetch".to_string(), Span::new(0, 9))],
            Span::new(0, 10),
        )),
        output: None,
        span: Span::new(0, 50),
    };
    assert_eq!(decl.fields.len(), 1);
    assert!(decl.tools_expr.is_some());
}

#[test]
fn test_agent_decl_with_identifier_tools_expr() {
    // Test with a variable reference for tools (e.g., tools: myTools)
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![],
        tools_expr: Some(Expression::Identifier("myTools".to_string(), Span::new(0, 7))),
        output: None,
        span: Span::new(0, 50),
    };
    assert!(decl.tools_expr.is_some());
    if let Some(Expression::Identifier(name, _)) = &decl.tools_expr {
        assert_eq!(name, "myTools");
    }
}
