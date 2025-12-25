//! Integration tests for structured output feature

use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

#[tokio::test]
async fn test_agent_with_inline_structured_output() {
    let source = r#"
        agent Classifier {
            prompt: "Classify the input"
            model: "gpt-4o"
            output: { category: string, confidence: number }
        }
        Classifier("test input")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"category": "test", "confidence": 0.95}"#);
    let mut tools = ToolRegistry::new();

    let outputs = evaluate_with_output(&program, &mock, &mut tools)
        .await
        .unwrap();
    assert_eq!(outputs.len(), 1);

    let json: serde_json::Value = serde_json::from_str(&outputs[0]).unwrap();
    assert_eq!(json["category"], "test");
    assert_eq!(json["confidence"], 0.95);
}

#[tokio::test]
async fn test_agent_with_named_struct_output() {
    let source = r#"
        struct Classification {
            category: string
            confidence: number
        }

        agent Classifier {
            prompt: "Classify the input"
            model: "gpt-4o"
            output: Classification
        }
        Classifier("test input")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"category": "billing", "confidence": 0.87}"#);
    let mut tools = ToolRegistry::new();

    let outputs = evaluate_with_output(&program, &mock, &mut tools)
        .await
        .unwrap();
    assert_eq!(outputs.len(), 1);

    let json: serde_json::Value = serde_json::from_str(&outputs[0]).unwrap();
    assert_eq!(json["category"], "billing");
    assert_eq!(json["confidence"], 0.87);
}

#[tokio::test]
async fn test_agent_without_output_schema() {
    // Agent without output field should work normally
    let source = r#"
        agent Simple {
            prompt: "Just respond"
            model: "gpt-4o"
        }
        Simple("hello")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response("Hello! How can I help you?");
    let mut tools = ToolRegistry::new();

    let outputs = evaluate_with_output(&program, &mock, &mut tools)
        .await
        .unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0], "Hello! How can I help you?");
}

#[tokio::test]
async fn test_struct_with_nested_output() {
    let source = r#"
        struct Metadata {
            created: string
            updated: string
        }

        struct DataOutput {
            name: string
            metadata: Metadata
        }

        agent Extractor {
            prompt: "Extract data"
            model: "gpt-4o"
            output: DataOutput
        }
        Extractor("extract from this")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(
        r#"{"name": "test", "metadata": {"created": "2024-01-01", "updated": "2024-01-02"}}"#,
    );
    let mut tools = ToolRegistry::new();

    let outputs = evaluate_with_output(&program, &mock, &mut tools)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&outputs[0]).unwrap();

    assert_eq!(json["name"], "test");
    assert_eq!(json["metadata"]["created"], "2024-01-01");
}

#[tokio::test]
async fn test_struct_with_array_output() {
    let source = r#"
        struct TagList {
            tags: string[]
            count: number
        }

        agent TagExtractor {
            prompt: "Extract tags"
            model: "gpt-4o"
            output: TagList
        }
        TagExtractor("find tags")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"tags": ["rust", "gent", "ai"], "count": 3}"#);
    let mut tools = ToolRegistry::new();

    let outputs = evaluate_with_output(&program, &mock, &mut tools)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_str(&outputs[0]).unwrap();

    assert_eq!(json["tags"].as_array().unwrap().len(), 3);
    assert_eq!(json["count"], 3);
}
