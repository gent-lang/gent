use gent::parser::parse;

#[test]
fn test_parse_for_loop_array() {
    let source = r#"
        tool test_loop() {
            for item in [1, 2, 3] {
                let x = item
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_for_loop_variable() {
    let source = r#"
        tool test_loop() {
            let items = [1, 2, 3]
            for item in items {
                let x = item
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_for_loop_range() {
    let source = r#"
        tool test_loop() {
            for i in 0..5 {
                let x = i
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_nested_for_loops() {
    let source = r#"
        tool test_loop() {
            for i in [1, 2] {
                for j in [3, 4] {
                    let x = i
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
