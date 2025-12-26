use gent::parser::parse;

#[test]
fn test_parse_fn_declaration() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_fn_no_return_type() {
    let source = r#"
        fn greet(name: string) {
            let msg = "Hello, {name}"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_fn_no_params() {
    let source = r#"
        fn sayHello() -> string {
            return "hello"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
