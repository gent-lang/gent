//! Tests for agent tools: field parsing

use gent::parser::parse;

#[test]
fn test_parse_agent_with_single_tool() {
    let source = r#"agent Bot { systemPrompt: "Hi" tools: [web_fetch] }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools_expr.is_some());
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_with_multiple_tools() {
    let source = r#"agent Bot { systemPrompt: "Hi" tools: [web_fetch, read_file, write_file] }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools_expr.is_some());
        // Verify it's an array expression
        if let Some(gent::parser::ast::Expression::Array(items, _)) = &decl.tools_expr {
            assert_eq!(items.len(), 3);
        } else {
            panic!("Expected Array expression for tools");
        }
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_tools_before_fields() {
    let source = r#"agent Bot { tools: [web_fetch] systemPrompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools_expr.is_some());
        assert_eq!(decl.fields.len(), 1);
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_no_tools() {
    let source = r#"agent Bot { systemPrompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools_expr.is_none());
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_empty_tools_array() {
    let source = r#"agent Bot { tools: [] systemPrompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools_expr.is_some());
        if let Some(gent::parser::ast::Expression::Array(items, _)) = &decl.tools_expr {
            assert!(items.is_empty());
        } else {
            panic!("Expected Array expression for tools");
        }
    } else {
        panic!("Expected AgentDecl");
    }
}
