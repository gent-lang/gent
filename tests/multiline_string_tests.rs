use gent::parser::{parse, Statement, Expression, StringPart};

#[test]
fn test_parse_multiline_string() {
    let source = r#"
        let prompt = """
        Hello
        World
        """
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_multiline_preserves_content() {
    let source = r#"let x = """
line1
line2
""""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    let program = result.unwrap();
    if let Statement::LetStmt(let_stmt) = &program.statements[0] {
        if let Expression::String(parts, _) = &let_stmt.value {
            assert!(!parts.is_empty(), "Should have string parts");
            if let StringPart::Literal(content) = &parts[0] {
                assert!(content.contains("line1"), "Should contain line1");
                assert!(content.contains("line2"), "Should contain line2");
            }
        } else {
            panic!("Expected String expression");
        }
    } else {
        panic!("Expected LetStmt");
    }
}

#[test]
fn test_multiline_with_interpolation() {
    let source = r#"
        let name = "World"
        let msg = """
        Hello {name}!
        """
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_multiline_empty_string() {
    let source = r#"let x = """""" "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse empty multiline: {:?}", result.err());
}

#[test]
fn test_multiline_with_special_chars() {
    // Multiline strings allow double quotes inside without escaping
    let source = r#"let x = """
    Line with "quotes" inside
    And regular text here
    """"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_multiline_preserves_newlines() {
    let source = r#"let x = """
first
second
third
""""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    let program = result.unwrap();
    if let Statement::LetStmt(let_stmt) = &program.statements[0] {
        if let Expression::String(parts, _) = &let_stmt.value {
            // Should have the newline character in the content
            let mut full_content = String::new();
            for part in parts {
                if let StringPart::Literal(s) = part {
                    full_content.push_str(s);
                }
            }
            assert!(full_content.contains('\n'), "Should contain newline characters");
        }
    }
}

#[test]
fn test_multiline_in_agent() {
    let source = r#"
        agent TestAgent {
            prompt: """
            You are a helpful assistant.
            Please respond professionally.
            """
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse multiline in agent: {:?}", result.err());
}
