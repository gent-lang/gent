use gent::parser::parse;

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
