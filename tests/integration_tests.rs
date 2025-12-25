use gent::interpreter::evaluate;
use gent::parser::parse;
use gent::runtime::{llm::MockLLMClient, ToolRegistry};

// Helper to run a program and check success
async fn run_program(source: &str) -> Result<(), String> {
    let program = parse(source).map_err(|e| e.to_string())?;
    let llm = MockLLMClient::new();
    let tools = ToolRegistry::new();
    evaluate(&program, &llm, &tools)
        .await
        .map_err(|e| e.to_string())
}

// Helper to run and expect failure
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

// === Happy Path Tests ===

#[tokio::test]
async fn test_hello_world() {
    let result = run_program(
        r#"
        agent Hello { prompt: "You are friendly." model: "gpt-4o-mini" }
        run Hello
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_minimal_agent() {
    let result = run_program(r#"agent A { prompt: "x" model: "gpt-4o-mini" } run A"#).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_many_fields() {
    let result = run_program(
        r#"
        agent Complex {
            prompt: "Be helpful"
            model: "gpt-4"
            max_steps: 10
            verbose: true
        }
        run Complex
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_agents_multiple_runs() {
    let result = run_program(
        r#"
        agent A { prompt: "Agent A" model: "gpt-4o-mini" }
        agent B { prompt: "Agent B" model: "gpt-4o-mini" }
        agent C { prompt: "Agent C" model: "gpt-4o-mini" }
        run A
        run B
        run C
        run A
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_with_string_input() {
    let result = run_program(
        r#"
        agent Echo { prompt: "Echo things" model: "gpt-4o-mini" }
        run Echo with "Hello there"
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_comments_everywhere() {
    let result = run_program(
        r#"
        // Comment at start
        agent Test { // Inline comment
            prompt: "Test" // Field comment
            model: "gpt-4o-mini"
        } // End of agent
        // Before run
        run Test // After run
        // Final comment
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_whitespace_variations() {
    let result = run_program("agent   A   {   prompt  :   \"x\"   model: \"gpt-4o-mini\"  }   run   A").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_newlines_in_various_places() {
    let result = run_program(
        r#"
agent
Test
{
prompt
:
"value"
model: "gpt-4o-mini"
}
run
Test
"#,
    )
    .await;
    assert!(result.is_ok());
}

// === Error Cases ===

#[tokio::test]
async fn test_error_undefined_agent() {
    expect_failure("run Ghost", "Undefined agent").await;
}

#[tokio::test]
async fn test_error_missing_prompt() {
    expect_failure(
        r#"agent NoPrompt { model: "gpt-4" } run NoPrompt"#,
        "missing",
    )
    .await;
}

#[tokio::test]
async fn test_error_syntax_missing_brace() {
    expect_failure(r#"agent Broken { prompt: "x""#, "Syntax error").await;
}

#[tokio::test]
async fn test_error_syntax_missing_name() {
    expect_failure(r#"agent { prompt: "x" }"#, "Syntax error").await;
}

#[tokio::test]
async fn test_error_run_before_define() {
    expect_failure(
        r#"run Later agent Later { prompt: "x" model: "gpt-4o-mini" }"#,
        "Undefined agent",
    )
    .await;
}

// === Edge Cases ===

#[tokio::test]
async fn test_empty_program() {
    let result = run_program("").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_only_comments() {
    let result = run_program(
        r#"
        // Just comments
        // Nothing else
        // More comments
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_empty_body() {
    // Empty body means no prompt - should fail at runtime
    expect_failure("agent Empty { } run Empty", "missing").await;
}

#[tokio::test]
async fn test_agent_name_with_numbers() {
    let result = run_program(
        r#"
        agent Agent123 { prompt: "Test" model: "gpt-4o-mini" }
        run Agent123
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_name_with_underscore() {
    let result = run_program(
        r#"
        agent my_agent { prompt: "Test" model: "gpt-4o-mini" }
        run my_agent
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_long_prompt() {
    let long_prompt = "x".repeat(5000);
    let source = format!(r#"agent Long {{ prompt: "{}" model: "gpt-4o-mini" }} run Long"#, long_prompt);
    let result = run_program(&source).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_special_chars_in_prompt() {
    let result = run_program(
        r#"
        agent Special { prompt: "Hello! How are you? I'm fine. @#$%^&*()" model: "gpt-4o-mini" }
        run Special
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_redefine_agent() {
    let result = run_program(
        r#"
        agent Bot { prompt: "First version" model: "gpt-4o-mini" }
        agent Bot { prompt: "Second version" model: "gpt-4o-mini" }
        run Bot
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_case_sensitive_names() {
    expect_failure(
        r#"
            agent Hello { prompt: "Hi" model: "gpt-4o-mini" }
            run hello
        "#,
        "Undefined agent",
    )
    .await;
}

#[tokio::test]
async fn test_numeric_fields() {
    let result = run_program(
        r#"
        agent Bot {
            prompt: "Test"
            model: "gpt-4o-mini"
            max_steps: 100
            temperature: 0.7
            timeout: -1
        }
        run Bot
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_boolean_fields() {
    let result = run_program(
        r#"
        agent Bot {
            prompt: "Test"
            model: "gpt-4o-mini"
            verbose: true
            debug: false
        }
        run Bot
    "#,
    )
    .await;
    assert!(result.is_ok());
}
