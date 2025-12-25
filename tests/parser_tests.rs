use gent::parser::{parse, Expression, Statement, StringPart, TypeName};

/// Helper to extract string content from an Expression::String with a single Literal part
fn get_string_content(expr: &Expression) -> Option<&str> {
    match expr {
        Expression::String(parts, _) => {
            if parts.len() == 1 {
                if let StringPart::Literal(s) = &parts[0] {
                    return Some(s.as_str());
                }
            }
            None
        }
        _ => None,
    }
}

// ============================================
// Basic Parsing Tests
// ============================================

#[test]
fn test_parse_empty_program() {
    let result = parse("");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert!(program.statements.is_empty());
}

#[test]
fn test_parse_agent_minimal() {
    let result = parse("agent Hello { }");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.name, "Hello");
            assert!(agent.fields.is_empty());
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_agent_with_prompt() {
    let result = parse(r#"agent Hello { prompt: "You are friendly." }"#);
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.name, "Hello");
            assert_eq!(agent.fields.len(), 1);
            assert_eq!(agent.fields[0].name, "prompt");
            assert_eq!(get_string_content(&agent.fields[0].value), Some("You are friendly."));
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_agent_multiple_fields() {
    let result = parse(r#"agent Bot { prompt: "Help." model: gpt4 verbose: true }"#);
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.fields.len(), 3);
            assert_eq!(agent.fields[0].name, "prompt");
            assert_eq!(agent.fields[1].name, "model");
            assert_eq!(agent.fields[2].name, "verbose");
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_top_level_let() {
    let result = parse("let result = Hello.invoke()");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            assert_eq!(let_stmt.name, "result");
            // Value should be a method call expression
            match &let_stmt.value {
                Expression::Call(callee, _, _) => {
                    // Callee should be Hello.invoke member expression
                    match callee.as_ref() {
                        Expression::Member(base, method, _) => {
                            assert_eq!(method, "invoke");
                            match base.as_ref() {
                                Expression::Identifier(name, _) => assert_eq!(name, "Hello"),
                                _ => panic!("Expected Identifier base"),
                            }
                        }
                        _ => panic!("Expected Member expression"),
                    }
                }
                _ => panic!("Expected Call expression"),
            }
        }
        _ => panic!("Expected LetStmt"),
    }
}

#[test]
fn test_parse_top_level_let_with_user_prompt() {
    let result = parse(r#"let result = Hello.userPrompt("Hi there!").invoke()"#);
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            assert_eq!(let_stmt.name, "result");
        }
        _ => panic!("Expected LetStmt"),
    }
}

// ============================================
// Hello World Test
// ============================================

#[test]
fn test_parse_hello_world() {
    let source = r#"agent Hello { systemPrompt: "You are friendly." }
let result = Hello.invoke()"#;
    let result = parse(source);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);

    // First statement: agent decl
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.name, "Hello");
            assert_eq!(agent.fields.len(), 1);
        }
        _ => panic!("Expected AgentDecl"),
    }

    // Second statement: let statement with agent invoke
    match &program.statements[1] {
        Statement::LetStmt(let_stmt) => {
            assert_eq!(let_stmt.name, "result");
        }
        _ => panic!("Expected LetStmt"),
    }
}

// ============================================
// Expression Parsing Tests
// ============================================

#[test]
fn test_parse_string_simple() {
    let result = parse(r#"agent A { x: "hello" }"#);
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(get_string_content(&agent.fields[0].value), Some("hello"));
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_string_with_escapes() {
    let result = parse(r#"agent A { x: "say \"hi\"" }"#);
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(get_string_content(&agent.fields[0].value), Some("say \"hi\""));
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_string_with_newline() {
    let result = parse(r#"agent A { x: "line1\nline2" }"#);
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(get_string_content(&agent.fields[0].value), Some("line1\nline2"));
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_number_integer() {
    let result = parse("agent A { x: 42 }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Number(n, _) => assert_eq!(*n, 42.0),
            _ => panic!("Expected Number"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_number_float() {
    let result = parse("agent A { x: 3.14 }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Number(n, _) => assert!((n - 3.14).abs() < 0.001),
            _ => panic!("Expected Number"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_number_negative() {
    let result = parse("agent A { x: -42 }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Number(n, _) => assert_eq!(*n, -42.0),
            _ => panic!("Expected Number"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_boolean_true() {
    let result = parse("agent A { x: true }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Boolean(b, _) => assert!(*b),
            _ => panic!("Expected Boolean"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_boolean_false() {
    let result = parse("agent A { x: false }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Boolean(b, _) => assert!(!*b),
            _ => panic!("Expected Boolean"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_identifier_value() {
    let result = parse("agent A { x: myVar }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => match &agent.fields[0].value {
            Expression::Identifier(name, _) => assert_eq!(name, "myVar"),
            _ => panic!("Expected Identifier"),
        },
        _ => panic!("Expected AgentDecl"),
    }
}

// ============================================
// Span Tests
// ============================================

#[test]
fn test_parse_span_agent() {
    let result = parse("agent Hello { }");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(agent) => {
            assert_eq!(agent.span.start, 0);
            assert_eq!(agent.span.end, 15);
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_span_let_stmt() {
    let result = parse("let x = Hello.invoke()");
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            assert_eq!(let_stmt.span.start, 0);
            assert_eq!(let_stmt.span.end, 22);
        }
        _ => panic!("Expected LetStmt"),
    }
}

// ============================================
// Error Cases
// ============================================

#[test]
fn test_parse_error_invalid_syntax() {
    let result = parse("agent { }");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unclosed_brace() {
    let result = parse("agent Hello {");
    assert!(result.is_err());
}

#[test]
fn test_parse_error_unclosed_string() {
    let result = parse(r#"agent A { x: "unclosed }"#);
    assert!(result.is_err());
}

// ============================================
// Whitespace and Comments
// ============================================

#[test]
fn test_parse_with_comments() {
    let source = r#"// Comment
agent Hello { systemPrompt: "Hi" }
// Another comment
let result = Hello.invoke()"#;
    let result = parse(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().statements.len(), 2);
}

#[test]
fn test_parse_with_extra_whitespace() {
    let result = parse("  agent   Hello   {   }  ");
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiline() {
    let source = r#"agent Hello {
    systemPrompt: "You are friendly."
}
let result = Hello.invoke()"#;
    let result = parse(source);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().statements.len(), 2);
}

// ============================================
// Tool Declaration Tests
// ============================================

#[test]
fn test_parse_simple_tool() {
    let source = r#"
        tool greet(name: string) -> string {
            return "hello"
        }
    "#;
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    match &program.statements[0] {
        Statement::ToolDecl(tool) => {
            assert_eq!(tool.name, "greet");
            assert_eq!(tool.params.len(), 1);
            assert_eq!(tool.params[0].name, "name");
            assert_eq!(tool.params[0].type_name, TypeName::String);
            assert_eq!(tool.return_type, Some(TypeName::String));
        }
        _ => panic!("Expected ToolDecl"),
    }
}

#[test]
fn test_parse_tool_no_params() {
    let source = r#"
        tool noop() {
            return null
        }
    "#;
    let program = parse(source).unwrap();
    match &program.statements[0] {
        Statement::ToolDecl(tool) => {
            assert_eq!(tool.params.len(), 0);
            assert_eq!(tool.return_type, None);
        }
        _ => panic!("Expected ToolDecl"),
    }
}

#[test]
fn test_parse_tool_multiple_params() {
    let source = r#"
        tool add(a: number, b: number) -> number {
            return a
        }
    "#;
    let program = parse(source).unwrap();
    match &program.statements[0] {
        Statement::ToolDecl(tool) => {
            assert_eq!(tool.params.len(), 2);
            assert_eq!(tool.params[0].type_name, TypeName::Number);
            assert_eq!(tool.params[1].type_name, TypeName::Number);
        }
        _ => panic!("Expected ToolDecl"),
    }
}

// ============================================
// Full Expression Parsing Tests
// ============================================

#[test]
fn test_parse_binary_expression() {
    let source = r#"
        tool calc() -> number {
            return 1 + 2 * 3
        }
    "#;
    let program = parse(source).unwrap();
    match &program.statements[0] {
        Statement::ToolDecl(tool) => {
            assert_eq!(tool.body.statements.len(), 1);
        }
        _ => panic!("Expected ToolDecl"),
    }
}

#[test]
fn test_parse_member_expression() {
    let source = r#"
        tool get_temp() -> number {
            return data.main.temp
        }
    "#;
    let program = parse(source).unwrap();
    assert!(program.statements.len() > 0);
}

#[test]
fn test_parse_call_expression() {
    let source = r#"
        tool wrapper() -> string {
            return foo(1, 2)
        }
    "#;
    let program = parse(source).unwrap();
    assert!(program.statements.len() > 0);
}

#[test]
fn test_parse_array_literal_in_tool() {
    let source = r#"
        tool make_arr() -> array {
            return [1, 2, 3]
        }
    "#;
    let program = parse(source).unwrap();
    assert!(program.statements.len() > 0);
}

#[test]
fn test_parse_object_literal_in_tool() {
    let source = r#"
        tool make_obj() -> object {
            return {name: "test", value: 42}
        }
    "#;
    let program = parse(source).unwrap();
    assert!(program.statements.len() > 0);
}

#[test]
fn test_parse_index_expression() {
    let source = r#"
        tool first() -> any {
            return items[0]
        }
    "#;
    let program = parse(source).unwrap();
    assert!(program.statements.len() > 0);
}
