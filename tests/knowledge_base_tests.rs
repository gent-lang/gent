//! Tests for KnowledgeBase in GENT

#[tokio::test]
async fn test_create_knowledge_base() {
    let source = r#"
        let kb = KnowledgeBase("./examples")
        println("created")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_base_no_args_error() {
    let source = r#"let kb = KnowledgeBase()"#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_err(), "Expected error for no arguments");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("1 argument") || err.contains("expects 1"),
        "Error should mention expected argument count: {}",
        err
    );
}

#[tokio::test]
async fn test_knowledge_base_too_many_args_error() {
    let source = r#"let kb = KnowledgeBase("./a", "./b")"#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_err(), "Expected error for too many arguments");
}

#[tokio::test]
async fn test_knowledge_base_wrong_type_error() {
    let source = r#"let kb = KnowledgeBase(42)"#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_err(), "Expected error for wrong argument type");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("String") || err.contains("type"),
        "Error should mention expected type: {}",
        err
    );
}
