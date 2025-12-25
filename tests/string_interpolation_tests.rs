use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::parser::{parse, Expression, Statement, StringPart};
use gent::runtime::{llm::MockLLMClient, ToolRegistry};

async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &llm, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

// ============================================
// String Interpolation Parsing Tests
// ============================================
// These tests verify that string interpolation syntax like "Hello, {name}"
// is properly parsed into the AST. Currently these should FAIL because
// the grammar and AST don't support interpolation yet.
//
// The tests check that strings with `{expr}` are NOT treated as plain strings,
// indicating that interpolation is being parsed properly.

/// Helper to check if an expression is a plain string (not interpolated)
/// A plain string has exactly one Literal part
fn is_plain_string(expr: &Expression) -> bool {
    match expr {
        Expression::String(parts, _) => {
            parts.len() == 1 && matches!(&parts[0], StringPart::Literal(_))
        }
        _ => false,
    }
}

/// Helper to get the string content if it's a plain string (single literal)
fn get_plain_string(expr: &Expression) -> Option<&str> {
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

#[test]
fn test_parse_simple_interpolation() {
    let source = r#"let msg = "Hello, {name}""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            // Currently this is parsed as a plain string with literal "{name}" in it.
            // After implementing interpolation, this should NOT be a plain String anymore.
            // This test will FAIL once interpolation is properly implemented.
            if is_plain_string(&let_stmt.value) {
                let content = get_plain_string(&let_stmt.value).unwrap();
                // If the string contains the literal "{name}" text, interpolation is NOT working
                assert!(
                    !content.contains("{name}"),
                    "String interpolation not implemented: '{{name}}' should be parsed as an expression, \
                     not as literal text. Got plain string: {:?}",
                    content
                );
            }
            // If it's not a plain string, the test passes (interpolation is working)
        }
        _ => panic!("Expected LetStmt"),
    }
}

#[test]
fn test_parse_multiple_interpolations() {
    let source = r#"let msg = "{a} and {b}""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            if is_plain_string(&let_stmt.value) {
                let content = get_plain_string(&let_stmt.value).unwrap();
                // If the string contains literal "{a}" or "{b}", interpolation is NOT working
                assert!(
                    !content.contains("{a}") && !content.contains("{b}"),
                    "String interpolation not implemented: '{{a}}' and '{{b}}' should be parsed as expressions, \
                     not as literal text. Got plain string: {:?}",
                    content
                );
            }
        }
        _ => panic!("Expected LetStmt"),
    }
}

#[test]
fn test_parse_interpolation_with_member_access() {
    let source = r#"let msg = "Name: {user.name}""#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            if is_plain_string(&let_stmt.value) {
                let content = get_plain_string(&let_stmt.value).unwrap();
                // If the string contains literal "{user.name}", interpolation is NOT working
                assert!(
                    !content.contains("{user.name}"),
                    "String interpolation not implemented: '{{user.name}}' should be parsed as a member access expression, \
                     not as literal text. Got plain string: {:?}",
                    content
                );
            }
        }
        _ => panic!("Expected LetStmt"),
    }
}

#[test]
fn test_parse_escaped_braces() {
    // Escaped braces should NOT be treated as interpolation - they should become literal braces
    let source = r#"let msg = "Use \{braces\}""#;
    let result = parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse escaped braces: {:?}. \
         Escape sequences \\{{ and \\}} need to be supported in the grammar.",
        result.err()
    );

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::LetStmt(let_stmt) => {
            if is_plain_string(&let_stmt.value) {
                let content = get_plain_string(&let_stmt.value).unwrap();
                // Escaped braces should become literal braces in the output
                assert_eq!(
                    content, "Use {braces}",
                    "Escaped braces \\{{...\\}} should be converted to literal braces {{...}}"
                );
            }
            // If it's an interpolated string type, it should have a single literal part
        }
        _ => panic!("Expected LetStmt"),
    }
}

// ============================================
// String Interpolation Evaluation Tests
// ============================================
// These tests verify that string interpolation is properly evaluated at runtime.
// Variables and expressions within {braces} should be resolved to their values.

#[tokio::test]
async fn test_eval_simple_interpolation() {
    let source = r#"
        let name = "Alice"
        let msg = "Hello, {name}!"
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_number_interpolation() {
    let source = r#"
        let count = 5
        let msg = "You have {count} items"
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_expression_interpolation() {
    let source = r#"
        let x = 10
        let msg = "Result: {x + 1}"
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_member_access_interpolation() {
    let source = r#"
        let user = { name: "Bob" }
        let msg = "User: {user.name}"
    "#;
    let result = run_program(source).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
