//! Tests for KnowledgeBase in GENT

use gent::config::Config;

#[tokio::test]
async fn test_create_knowledge_base() {
    let source = r#"
        let kb = KnowledgeBase("./examples")
        println("created")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_base_no_args_error() {
    let source = r#"let kb = KnowledgeBase()"#;
    let program = gent::parser::parse(source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;
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
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;
    assert!(result.is_err(), "Expected error for too many arguments");
}

#[tokio::test]
async fn test_knowledge_base_wrong_type_error() {
    let source = r#"let kb = KnowledgeBase(42)"#;
    let program = gent::parser::parse(source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;
    assert!(result.is_err(), "Expected error for wrong argument type");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("String") || err.contains("type"),
        "Error should mention expected type: {}",
        err
    );
}

#[tokio::test]
async fn test_knowledge_base_index_and_search() {
    // Create a temp directory with test files
    let temp_dir = std::env::temp_dir().join("gent_test_kb");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("test.md"),
        "# Hello\n\nThis is about authentication and JWT tokens.",
    )
    .unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let count = kb.index({{}})
        println("indexed {{count}} chunks")
        let results = kb.search("authentication")
        let numResults = results.length()
        println("found {{numResults}} results")
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );

    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_base_search_before_index_error() {
    // Create temp dir that exists but hasn't been indexed
    let temp_dir = std::env::temp_dir().join("gent_test_kb_unindexed");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("test.md"), "# Test content").unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let results = kb.search("query")
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );
    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_err(), "Expected error when searching before indexing");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not indexed") || err.contains("index"),
        "Error should mention indexing: {}",
        err
    );
}

#[tokio::test]
async fn test_knowledge_base_is_indexed() {
    let temp_dir = std::env::temp_dir().join("gent_test_kb_indexed_check");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(temp_dir.join("test.md"), "# Test content").unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let beforeIndex = kb.isIndexed()
        println("before: {{beforeIndex}}")
        let indexCount = kb.index({{}})
        let afterIndex = kb.isIndexed()
        println("after: {{afterIndex}}")
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );

    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_base_search_with_limit() {
    let temp_dir = std::env::temp_dir().join("gent_test_kb_limit");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("test.md"),
        "# Test\n\nFirst paragraph about testing.\n\nSecond paragraph about testing.\n\nThird paragraph about testing.",
    )
    .unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let indexCount = kb.index({{}})
        let results = kb.search("testing", {{ limit: 2 }})
        let numResults = results.length()
        println("found {{numResults}} results")
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );

    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_base_unknown_method_error() {
    let temp_dir = std::env::temp_dir().join("gent_test_kb_unknown_method");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let result = kb.unknownMethod()
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );

    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_err(), "Expected error for unknown method");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("no method") || err.contains("unknownMethod"),
        "Error should mention missing method: {}",
        err
    );
}

#[tokio::test]
async fn test_knowledge_base_as_tool() {
    use gent::runtime::rag::KnowledgeBaseTool;
    use gent::runtime::tools::Tool;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Create a temp directory with test files
    let temp_dir = std::env::temp_dir().join("gent_test_kb_tool");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("doc.md"),
        "# API\n\nThis is about REST API endpoints.",
    )
    .unwrap();

    // Create and index knowledge base
    let mut kb = gent::runtime::rag::KnowledgeBase::new(&temp_dir);
    kb.index(gent::runtime::rag::IndexOptions::default())
        .await
        .unwrap();

    // Wrap as tool
    let kb_arc = Arc::new(RwLock::new(kb));
    let tool = KnowledgeBaseTool::new(kb_arc, "docs".to_string());

    // Verify tool properties
    assert_eq!(tool.name(), "docs");
    assert!(tool.description().contains("knowledge base"));

    // Execute search
    let result = tool
        .execute(serde_json::json!({"query": "API", "limit": 3}))
        .await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("API") || output.contains("doc.md"));

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[tokio::test]
async fn test_agent_with_knowledge_base_tool() {
    // Create a temp directory with test files
    let temp_dir = std::env::temp_dir().join("gent_test_kb_agent");
    std::fs::create_dir_all(&temp_dir).unwrap();
    std::fs::write(
        temp_dir.join("info.md"),
        "# Info\n\nThis document explains the system.",
    )
    .unwrap();

    let source = format!(
        r#"
        let kb = KnowledgeBase("{}")
        let indexCount = kb.index({{}})

        agent Helper {{
            tools: [kb]
            model: "gpt-4o"
            systemPrompt: "You help users"
        }}

        println("Agent created with KB tool")
    "#,
        temp_dir.display().to_string().replace("\\", "/")
    );

    let program = gent::parser::parse(&source).unwrap();
    let config = gent::config::Config::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &config, &mut tools, &logger).await;

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).ok();

    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
