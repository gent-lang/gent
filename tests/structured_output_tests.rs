//! Integration tests for structured output feature

use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

#[tokio::test]
async fn test_agent_with_inline_structured_output() {
    let source = r#"
        agent Classifier {
            systemPrompt: "Classify the input"
            model: "gpt-4o"
            output: { category: string, confidence: number }
        }
        let result = Classifier.userPrompt("test input").run()
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
            systemPrompt: "Classify the input"
            model: "gpt-4o"
            output: Classification
        }
        let result = Classifier.userPrompt("test input").run()
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
            systemPrompt: "Just respond"
            model: "gpt-4o"
        }
        let result = Simple.userPrompt("hello").run()
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
            systemPrompt: "Extract data"
            model: "gpt-4o"
            output: DataOutput
        }
        let result = Extractor.userPrompt("extract from this").run()
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
            systemPrompt: "Extract tags"
            model: "gpt-4o"
            output: TagList
        }
        let result = TagExtractor.userPrompt("find tags").run()
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

// ============================================
// Bug Reproduction: Agent with structured output called from function
// Issue: "Type error: expected String or Array, got Agent"
// ============================================

#[tokio::test]
async fn test_agent_with_structured_output_called_from_function() {
    // This reproduces the bug from puzzle_ideation.gnt:
    // fn generateIdeas(prompt: string) -> object {
    //     let session = PuzzleIdeator.userPrompt("...{prompt}...").run()
    //     return session
    // }
    let source = r#"
        struct IdeaList {
            ideas: string[]
            count: number
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeaList
        }

        fn generateIdeas(prompt: string) -> object {
            let result = Ideator.userPrompt("Generate ideas for: {prompt}").run()
            return result
        }

        fn main() {
            let ideas = generateIdeas("puzzle games")
            println("{ideas}")
        }

        main()
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"ideas": ["puzzle1", "puzzle2"], "count": 2}"#);
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_with_structured_output_interpolation_in_function() {
    // Simpler version: just test interpolation works inside function
    let source = r#"
        struct Result {
            value: string
        }

        agent Processor {
            systemPrompt: "Process input"
            model: "gpt-4o"
            output: Result
        }

        fn process(input: string) -> object {
            let r = Processor.userPrompt("Process: {input}").run()
            return r
        }

        let output = process("test input")
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"value": "processed"}"#);
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_agent_run_result_can_be_iterated() {
    // Test that agent result with array field can be iterated
    let source = r#"
        struct IdeaSession {
            ideas: string[]
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeaSession
        }

        fn main() {
            let session = Ideator.userPrompt("generate").run()
            for idea in session.ideas {
                println("{idea}")
            }
        }

        main()
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(r#"{"ideas": ["idea1", "idea2", "idea3"]}"#);
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_nested_object_property_access_in_loop() {
    // This reproduces the bug from puzzle_ideation.gnt:
    // for idea in ideas.ideas {
    //     println("{idea.coreMechanic}")  // ERROR: Undefined property: coreMechanic on Object
    // }
    let source = r#"
        struct GameIdea {
            title: string
            coreMechanic: string
        }

        struct IdeationSession {
            ideas: GameIdea[]
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeationSession
        }

        fn main() {
            let session = Ideator.userPrompt("generate").run()
            for idea in session.ideas {
                println("Title: {idea.title}")
                println("Mechanic: {idea.coreMechanic}")
            }
        }

        main()
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(
        r#"{"ideas": [{"title": "Puzzle Quest", "coreMechanic": "Match tiles"}, {"title": "Logic Land", "coreMechanic": "Solve equations"}]}"#,
    );
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_nested_struct_with_wrong_property_names_should_fail_validation() {
    // When LLM returns different property names than the struct defines,
    // validation should catch this and request a retry
    let source = r#"
        struct GameIdea {
            title: string
            coreMechanic: string
        }

        struct IdeationSession {
            ideas: GameIdea[]
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeationSession
        }

        fn main() {
            let session = Ideator.userPrompt("generate").run()
            for idea in session.ideas {
                println("Title: {idea.title}")
                println("Mechanic: {idea.coreMechanic}")
            }
        }

        main()
    "#;

    let program = parse(source).unwrap();
    // LLM returns snake_case instead of camelCase - validation should catch this
    let mock = MockLLMClient::with_response(
        r#"{"ideas": [{"title": "Puzzle Quest", "core_mechanic": "Match tiles"}]}"#,
    );
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    // This should fail because the property names don't match
    // Currently this passes validation (bug!) and fails at runtime with UndefinedProperty
    assert!(result.is_err(), "Should have failed validation due to wrong property names");

    // The error should be a validation error, not a runtime UndefinedProperty error
    let err = result.unwrap_err();
    let err_msg = format!("{:?}", err);
    assert!(
        err_msg.contains("validation") || err_msg.contains("missing required field"),
        "Error should be from validation, not runtime. Got: {}", err_msg
    );
}

#[tokio::test]
async fn test_array_length_in_if_condition() {
    // This reproduces the bug from puzzle_ideation.gnt:102
    // if ideas.ideas.length() > 0 { ... }
    // Error: "expected synchronous expression, got function call (requires async context)"
    let source = r#"
        struct GameIdea {
            title: string
        }

        struct IdeationSession {
            ideas: GameIdea[]
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeationSession
        }

        fn main() {
            let session = Ideator.userPrompt("generate").run()
            if session.ideas.length() > 0 {
                let first = session.ideas[0]
                println("First idea: {first.title}")
            }
        }

        main()
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(
        r#"{"ideas": [{"title": "Puzzle Quest"}, {"title": "Logic Land"}]}"#,
    );
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_length_in_if_condition_with_ideas_variable() {
    // Exact pattern from user's puzzle_ideation.gnt:102
    // let ideas = generateIdeas(prompt)
    // if ideas.ideas.length() > 0 { ... }
    let source = r#"
        struct GameIdea {
            title: string
            coreMechanic: string
        }

        struct IdeationSession {
            ideas: GameIdea[]
        }

        agent Ideator {
            systemPrompt: "Generate ideas"
            model: "gpt-4o"
            output: IdeationSession
        }

        fn generateIdeas(prompt: string) -> object {
            let result = Ideator.userPrompt("Generate ideas for: {prompt}").run()
            return result
        }

        fn main() {
            let ideas = generateIdeas("puzzle games")
            if ideas.ideas.length() > 0 {
                for idea in ideas.ideas {
                    println("Title: {idea.title}")
                    println("Mechanic: {idea.coreMechanic}")
                }
            }
        }

        main()
    "#;

    let program = parse(source).unwrap();
    let mock = MockLLMClient::with_response(
        r#"{"ideas": [{"title": "Puzzle Quest", "coreMechanic": "Match tiles"}, {"title": "Logic Land", "coreMechanic": "Solve equations"}]}"#,
    );
    let mut tools = ToolRegistry::new();

    let result = evaluate_with_output(&program, &mock, &mut tools).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
