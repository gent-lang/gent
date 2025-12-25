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
        agent Hello { systemPrompt: "You are friendly." model: "gpt-4o-mini" }
        let result = Hello.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_minimal_agent() {
    let result = run_program(r#"agent A { systemPrompt: "x" model: "gpt-4o-mini" } let r = A.invoke()"#).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_with_many_fields() {
    let result = run_program(
        r#"
        agent Complex {
            systemPrompt: "Be helpful"
            model: "gpt-4"
            maxSteps: 10
            verbose: true
        }
        let result = Complex.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_agents_multiple_runs() {
    let result = run_program(
        r#"
        agent A { systemPrompt: "Agent A" model: "gpt-4o-mini" }
        agent B { systemPrompt: "Agent B" model: "gpt-4o-mini" }
        agent C { systemPrompt: "Agent C" model: "gpt-4o-mini" }
        let r1 = A.invoke()
        let r2 = B.invoke()
        let r3 = C.invoke()
        let r4 = A.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_run_with_string_input() {
    let result = run_program(
        r#"
        agent Echo { systemPrompt: "Echo things" model: "gpt-4o-mini" }
        let result = Echo.userPrompt("Hello there").invoke()
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
            systemPrompt: "Test" // Field comment
            model: "gpt-4o-mini"
        } // End of agent
        // Before run
        let result = Test.invoke() // After run
        // Final comment
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_whitespace_variations() {
    let result =
        run_program("agent   A   {   systemPrompt  :   \"x\"   model: \"gpt-4o-mini\"  }   let r = A.invoke()")
            .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_newlines_in_various_places() {
    let result = run_program(
        r#"
agent
Test
{
systemPrompt
:
"value"
model: "gpt-4o-mini"
}
let result = Test.invoke()
"#,
    )
    .await;
    assert!(result.is_ok());
}

// === Error Cases ===

#[tokio::test]
async fn test_error_undefined_agent() {
    expect_failure("let r = Ghost.invoke()", "Undefined").await;
}

#[tokio::test]
async fn test_error_missing_model() {
    // Prompt is now optional, but model is still required
    expect_failure(
        r#"agent NoModel { systemPrompt: "Hello" } let r = NoModel.invoke()"#,
        "model",
    )
    .await;
}

#[tokio::test]
async fn test_error_syntax_missing_brace() {
    expect_failure(r#"agent Broken { systemPrompt: "x""#, "Syntax error").await;
}

#[tokio::test]
async fn test_error_syntax_missing_name() {
    expect_failure(r#"agent { systemPrompt: "x" }"#, "Syntax error").await;
}

#[tokio::test]
async fn test_error_run_before_define() {
    expect_failure(
        r#"let r = Later.invoke() agent Later { systemPrompt: "x" model: "gpt-4o-mini" }"#,
        "Undefined",
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
    expect_failure("agent Empty { } let r = Empty.invoke()", "missing").await;
}

#[tokio::test]
async fn test_agent_name_with_numbers() {
    let result = run_program(
        r#"
        agent Agent123 { systemPrompt: "Test" model: "gpt-4o-mini" }
        let r = Agent123.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_agent_name_with_underscore() {
    let result = run_program(
        r#"
        agent my_agent { systemPrompt: "Test" model: "gpt-4o-mini" }
        let r = my_agent.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_long_prompt() {
    let long_prompt = "x".repeat(5000);
    let source = format!(
        r#"agent Long {{ systemPrompt: "{}" model: "gpt-4o-mini" }} let r = Long.invoke()"#,
        long_prompt
    );
    let result = run_program(&source).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_special_chars_in_prompt() {
    let result = run_program(
        r#"
        agent Special { systemPrompt: "Hello! How are you? I'm fine. @#$%^&*()" model: "gpt-4o-mini" }
        let r = Special.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_redefine_agent() {
    let result = run_program(
        r#"
        agent Bot { systemPrompt: "First version" model: "gpt-4o-mini" }
        agent Bot { systemPrompt: "Second version" model: "gpt-4o-mini" }
        let r = Bot.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_case_sensitive_names() {
    expect_failure(
        r#"
            agent Hello { systemPrompt: "Hi" model: "gpt-4o-mini" }
            let r = hello.invoke()
        "#,
        "Undefined",
    )
    .await;
}

#[tokio::test]
async fn test_numeric_fields() {
    let result = run_program(
        r#"
        agent Bot {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            maxSteps: 100
            temperature: 0.7
            timeout: -1
        }
        let r = Bot.invoke()
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
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            verbose: true
            debug: false
        }
        let r = Bot.invoke()
    "#,
    )
    .await;
    assert!(result.is_ok());
}
