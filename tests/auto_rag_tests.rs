//! Tests for Auto-RAG functionality (knowledge field on agents)

use gent::interpreter::evaluate;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{llm::MockLLMClient, ToolRegistry};

// Helper to run a program and check success
async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;
    evaluate(&program, &llm, &mut tools, &logger)
        .await
        .map_err(|e| e.to_string())
}

// Helper to run and expect failure with substring
async fn expect_failure(source: &str, expected_substring: &str) {
    let result = run_program(source).await;
    assert!(result.is_err(), "Expected failure but got success");
    assert!(
        result.as_ref().unwrap_err().contains(expected_substring),
        "Error '{}' doesn't contain '{}'",
        result.unwrap_err(),
        expected_substring
    );
}

// ============================================
// Parsing Tests
// ============================================

#[test]
fn test_parse_knowledge_field_basic() {
    let source = r#"
        let kb = KnowledgeBase("./docs")
        agent Helper {
            knowledge: {
                source: kb,
                chunkLimit: 5,
                scoreThreshold: 0.7
            }
            systemPrompt: "You are helpful."
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_knowledge_field_minimal() {
    // Only source is required
    let source = r#"
        let kb = KnowledgeBase("./docs")
        agent Helper {
            knowledge: { source: kb }
            systemPrompt: "You are helpful."
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_knowledge_with_tools() {
    // Both knowledge and tools fields
    let source = r#"
        let kb = KnowledgeBase("./docs")
        agent Helper {
            knowledge: { source: kb, chunkLimit: 3 }
            tools: [kb]
            systemPrompt: "You are helpful."
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_knowledge_with_output() {
    // Knowledge with structured output
    let source = r#"
        let kb = KnowledgeBase("./docs")
        agent Helper {
            knowledge: { source: kb }
            systemPrompt: "Extract data."
            model: "gpt-4o-mini"
            output: { answer: string }
        }
    "#;

    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

// ============================================
// Evaluation Tests
// ============================================

#[tokio::test]
async fn test_evaluate_knowledge_config() {
    let source = r#"
        let kb = KnowledgeBase("./examples")
        let count = kb.index({ extensions: [".md"] })
        agent Helper {
            knowledge: {
                source: kb,
                chunkLimit: 5,
                scoreThreshold: 0.7
            }
            systemPrompt: "You are helpful."
            model: "gpt-4o-mini"
        }
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[tokio::test]
async fn test_evaluate_knowledge_config_defaults() {
    let source = r#"
        let kb = KnowledgeBase("./examples")
        let count = kb.index({ extensions: [".md"] })
        agent Helper {
            knowledge: { source: kb }
            systemPrompt: "You are helpful."
            model: "gpt-4o-mini"
        }
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_without_knowledge() {
    let source = r#"
        agent Simple {
            systemPrompt: "You are helpful."
            model: "gpt-4o-mini"
        }
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

// ============================================
// Error Tests
// ============================================

#[tokio::test]
async fn test_knowledge_missing_source_error() {
    expect_failure(
        r#"
            agent Helper {
                knowledge: { chunkLimit: 5 }
                systemPrompt: "You are helpful."
                model: "gpt-4o-mini"
            }
        "#,
        "source",
    )
    .await;
}

#[tokio::test]
async fn test_knowledge_invalid_source_type_error() {
    expect_failure(
        r#"
            agent Helper {
                knowledge: { source: "not a kb" }
                systemPrompt: "You are helpful."
                model: "gpt-4o-mini"
            }
        "#,
        "KnowledgeBase",
    )
    .await;
}

#[tokio::test]
async fn test_knowledge_wrong_type_error() {
    expect_failure(
        r#"
            agent Helper {
                knowledge: "not an object"
                systemPrompt: "You are helpful."
                model: "gpt-4o-mini"
            }
        "#,
        "Object",
    )
    .await;
}

// ============================================
// Integration Tests
// ============================================

#[tokio::test]
async fn test_knowledge_with_all_fields() {
    let source = r#"
        let kb = KnowledgeBase("./examples")
        let count = kb.index({ extensions: [".md"] })
        agent Helper {
            knowledge: {
                source: kb,
                chunkLimit: 2,
                scoreThreshold: 0.3
            }
            systemPrompt: "Answer questions."
            model: "gpt-4o-mini"
        }
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_agent_with_knowledge() {
    // Full integration test - run an agent with knowledge configured
    let source = r#"
        let kb = KnowledgeBase("./examples")
        let count = kb.index({ extensions: [".md", ".gnt"] })

        agent DocHelper {
            knowledge: {
                source: kb,
                chunkLimit: 3,
                scoreThreshold: 0.5
            }
            model: "gpt-4o-mini"
            systemPrompt: "Answer questions about the codebase."
        }

        let result = DocHelper.userPrompt("What examples are available?").run()
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Run with knowledge failed: {:?}", result.err());
}

#[tokio::test]
async fn test_knowledge_and_tools_together() {
    // Both knowledge (auto-inject) and tools (manual access)
    let source = r#"
        let kb = KnowledgeBase("./examples")
        let count = kb.index({ extensions: [".md"] })

        agent Helper {
            knowledge: {
                source: kb,
                chunkLimit: 2
            }
            tools: [kb]
            model: "gpt-4o-mini"
            systemPrompt: "Answer questions. Use tools for additional searches."
        }
    "#;

    let result = run_program(source).await;
    assert!(result.is_ok(), "Knowledge + tools failed: {:?}", result.err());
}
