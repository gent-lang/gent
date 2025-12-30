//! Tests for lambda parsing

use gent::config::Config;
use gent::parser::parse;

#[test]
fn test_parse_lambda_single_param() {
    let source = r#"
        let doubled = [1, 2, 3].map((x) => x * 2)
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_lambda_two_params() {
    let source = r#"
        let sum = [1, 2, 3].reduce((acc, x) => acc + x, 0)
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_lambda_no_params() {
    let source = r#"
        let items = [1, 2, 3].map(() => 42)
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[tokio::test]
async fn test_lambda_in_variable() {
    let source = r#"
        fn test() {
            let double = (x) => x * 2
            return 1
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
