use gent::interpreter::evaluate;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

#[tokio::test]
async fn test_agent_with_tools_parses() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            use web_fetch, read_file
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::with_builtins();

    let result = evaluate(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_max_steps() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            max_steps: 3
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();

    let result = evaluate(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_model() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            model: "gpt-4o"
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();

    let result = evaluate(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_full_researcher_example() {
    let source = r#"
        agent Researcher {
            prompt: "You help research topics."
            use web_fetch
            max_steps: 5
        }
        run Researcher with "Tell me about Rust"
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("Rust is a systems programming language.");
    let tools = ToolRegistry::with_builtins();

    let result = evaluate(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_agents_with_different_tools() {
    let source = r#"
        agent Reader {
            prompt: "Read files"
            use read_file
        }
        agent Writer {
            prompt: "Write files"
            use write_file
        }
        run Reader
        run Writer
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::with_builtins();

    let result = evaluate(&program, &llm, &tools).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cli_mock_mode() {
    use std::process::Command;

    let output = Command::new("cargo")
        .args(["run", "--", "--mock", "examples/hello.gnt"])
        .output()
        .expect("Failed to run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("friendly"));
}
