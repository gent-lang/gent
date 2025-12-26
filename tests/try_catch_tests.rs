use gent::parser::parse;

#[test]
fn test_parse_try_catch() {
    let source = r#"
        tool test() {
            try {
                let x = 1
            } catch error {
                let msg = error
            }
            return "done"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_function_call() {
    let source = r#"
        tool test() {
            try {
                let result = riskyOperation()
            } catch err {
                let fallback = "error occurred"
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_nested_try_catch() {
    let source = r#"
        tool test() {
            try {
                try {
                    let x = 1
                } catch innerErr {
                    let y = 2
                }
            } catch outerErr {
                let z = 3
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_return() {
    let source = r#"
        tool test() {
            try {
                return "success"
            } catch error {
                return "failure"
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_in_loop() {
    let source = r#"
        tool test() {
            for i in 1..5 {
                try {
                    let x = i
                } catch error {
                    break
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_try_catch_with_if() {
    let source = r#"
        tool test() {
            try {
                if true {
                    let x = 1
                }
            } catch error {
                if error == "fatal" {
                    return "abort"
                }
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
