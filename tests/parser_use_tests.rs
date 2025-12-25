use gent::parser::parse;

#[test]
fn test_parse_agent_with_single_tool() {
    let source = r#"agent Bot { prompt: "Hi" use web_fetch }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert_eq!(decl.tools, vec!["web_fetch"]);
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_with_multiple_tools() {
    let source = r#"agent Bot { prompt: "Hi" use web_fetch, read_file, write_file }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert_eq!(decl.tools.len(), 3);
        assert_eq!(decl.tools[0], "web_fetch");
        assert_eq!(decl.tools[1], "read_file");
        assert_eq!(decl.tools[2], "write_file");
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_use_before_fields() {
    let source = r#"agent Bot { use web_fetch prompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert_eq!(decl.tools, vec!["web_fetch"]);
        assert_eq!(decl.fields.len(), 1);
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_no_tools() {
    let source = r#"agent Bot { prompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        assert!(decl.tools.is_empty());
    } else {
        panic!("Expected AgentDecl");
    }
}

#[test]
fn test_parse_agent_multiple_use_statements() {
    let source = r#"agent Bot { use web_fetch use read_file prompt: "Hi" }"#;
    let program = parse(source).unwrap();

    let stmt = &program.statements[0];
    if let gent::parser::ast::Statement::AgentDecl(decl) = stmt {
        // Multiple use statements should accumulate
        assert_eq!(decl.tools.len(), 2);
    } else {
        panic!("Expected AgentDecl");
    }
}
