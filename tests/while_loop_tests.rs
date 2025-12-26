use gent::parser::parse;

#[test]
fn test_parse_while_loop() {
    // Basic while loop with condition and block body
    let source = r#"
        tool test() {
            while true {
                let x = 1
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_loop_comparison_condition() {
    // While loop with comparison expression as condition
    let source = r#"
        tool test() {
            let x = 0
            while x < 3 {
                let y = x + 1
            }
            return x
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_loop_identifier_condition() {
    // While loop with identifier as condition
    let source = r#"
        tool test() {
            let running = true
            while running {
                let x = 1
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_nested_while_loops() {
    // Nested while loops
    let source = r#"
        tool test() {
            while true {
                while false {
                    let x = 1
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_with_break() {
    // While loop with break statement
    let source = r#"
        tool test() {
            while true {
                break
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_with_continue() {
    // While loop with continue statement
    let source = r#"
        tool test() {
            while true {
                continue
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_with_if() {
    // While loop containing an if statement
    let source = r#"
        tool test() {
            let x = 0
            while x < 5 {
                if x > 2 {
                    break
                }
                let y = x
            }
            return x
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_with_return() {
    // While loop with return statement
    let source = r#"
        tool test() {
            while true {
                return 42
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_while_complex_condition() {
    // While loop with complex boolean condition
    let source = r#"
        tool test() {
            let x = 0
            let y = 10
            while x < 5 && y > 0 {
                let z = x + y
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
