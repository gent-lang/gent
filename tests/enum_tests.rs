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
