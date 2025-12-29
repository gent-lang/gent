use gent::runtime::providers::{detect_provider, Provider};

#[test]
fn test_detect_claude_models() {
    assert!(matches!(detect_provider("claude-3-5-sonnet-20241022"), Ok(Provider::Anthropic)));
    assert!(matches!(detect_provider("claude-3-opus"), Ok(Provider::Anthropic)));
    assert!(matches!(detect_provider("claude-3-haiku"), Ok(Provider::Anthropic)));
}

#[test]
fn test_detect_openai_models() {
    assert!(matches!(detect_provider("gpt-4o"), Ok(Provider::OpenAI)));
    assert!(matches!(detect_provider("gpt-4o-mini"), Ok(Provider::OpenAI)));
    assert!(matches!(detect_provider("gpt-3.5-turbo"), Ok(Provider::OpenAI)));
    assert!(matches!(detect_provider("o1-preview"), Ok(Provider::OpenAI)));
    assert!(matches!(detect_provider("o1-mini"), Ok(Provider::OpenAI)));
    assert!(matches!(detect_provider("o3-mini"), Ok(Provider::OpenAI)));
}

#[test]
fn test_detect_unknown_model() {
    assert!(detect_provider("llama-3").is_err());
    assert!(detect_provider("mistral-7b").is_err());
    assert!(detect_provider("unknown").is_err());
}
