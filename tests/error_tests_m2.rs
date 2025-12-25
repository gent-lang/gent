use gent::errors::GentError;

#[test]
fn test_api_error_display() {
    let err = GentError::ApiError {
        message: "rate limited".to_string(),
    };
    assert!(err.to_string().contains("rate limited"));
}

#[test]
fn test_missing_api_key_display() {
    let err = GentError::MissingApiKey {
        provider: "openai".to_string(),
    };
    assert!(err.to_string().contains("openai"));
}

#[test]
fn test_max_steps_exceeded_display() {
    let err = GentError::MaxStepsExceeded { limit: 10 };
    assert!(err.to_string().contains("10"));
}

#[test]
fn test_unknown_tool_display() {
    let err = GentError::UnknownTool {
        name: "bad_tool".to_string(),
        span: gent::errors::Span::new(0, 8),
    };
    assert!(err.to_string().contains("bad_tool"));
}

#[test]
fn test_tool_error_display() {
    let err = GentError::ToolError {
        tool: "web_fetch".to_string(),
        message: "connection refused".to_string(),
    };
    assert!(err.to_string().contains("web_fetch"));
    assert!(err.to_string().contains("connection refused"));
}
