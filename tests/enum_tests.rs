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

// ============================================
// Construction Tests
// ============================================

#[tokio::test]
async fn test_enum_construct_unit_variant() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Pending
            println("{s}")
            return s
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_construct_with_data() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Ok(42)
            println("{r}")
            return r
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_construct_with_multiple_fields() {
    let source = r#"
        enum Color { RGB(r: number, g: number, b: number) }
        fn test() {
            let c = Color.RGB(255, 128, 0)
            println("{c}")
            return c
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_invalid_variant() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Unknown
            return s
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    // Should fail because Unknown is not a valid variant
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enum_wrong_arg_count() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Ok(1, 2)
            return r
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    // Should fail because Ok expects 1 argument but got 2
    assert!(result.is_err());
}
