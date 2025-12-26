//! Tests for top-level function calls

use gent::parser::parse;

#[test]
fn test_parse_top_level_call() {
    let source = r#"
        fn greet() {
            return "hello"
        }
        greet()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_top_level_call_with_args() {
    let source = r#"
        println("Hello", "World")
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_top_level_call_no_args() {
    let source = r#"
        println()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
