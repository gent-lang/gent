//! Tests for enum types in GENT

// ============================================
// Parsing Tests
// ============================================

#[test]
fn test_parse_simple_enum() {
    let source = r#"
        enum Status {
            Pending
            Active
            Completed
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse simple enum: {:?}", result.err());
}

#[test]
fn test_parse_enum_with_data() {
    let source = r#"
        enum Result {
            Ok(value)
            Err(string)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse enum with data: {:?}", result.err());
}

#[test]
fn test_parse_enum_with_multiple_fields() {
    let source = r#"
        enum Color {
            RGB(r: number, g: number, b: number)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse enum with multiple fields: {:?}", result.err());
}

// ============================================
// Environment Registration Tests
// ============================================

#[tokio::test]
async fn test_enum_definition_registered() {
    let source = r#"
        enum Status {
            Pending
            Active
        }
        println("ok")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok());
}
