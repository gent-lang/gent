use gent::parser::ast::{AgentDecl, AgentField, Expression};
use gent::Span;

#[test]
fn test_agent_decl_with_tools() {
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![],
        tools: vec!["web_fetch".to_string(), "read_file".to_string()],
        span: Span::new(0, 10),
    };
    assert_eq!(decl.tools.len(), 2);
    assert_eq!(decl.tools[0], "web_fetch");
}

#[test]
fn test_agent_decl_without_tools() {
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![],
        tools: vec![],
        span: Span::new(0, 10),
    };
    assert!(decl.tools.is_empty());
}

#[test]
fn test_agent_decl_with_tools_and_fields() {
    let decl = AgentDecl {
        name: "Bot".to_string(),
        fields: vec![AgentField {
            name: "prompt".to_string(),
            value: Expression::String("Hi".to_string(), Span::new(0, 2)),
            span: Span::new(0, 10),
        }],
        tools: vec!["web_fetch".to_string()],
        span: Span::new(0, 50),
    };
    assert_eq!(decl.fields.len(), 1);
    assert_eq!(decl.tools.len(), 1);
}
