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
