use gent::parser::parse;

#[test]
fn test_parse_system_prompt_field() {
    let source = r#"agent Test { systemPrompt: "Hello" model: "gpt-4o-mini" }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_user_prompt_field() {
    let source = r#"agent Test { userPrompt: "Hello" model: "gpt-4o-mini" }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_max_steps_camel() {
    let source = r#"agent Test { systemPrompt: "Hi" model: "gpt-4o-mini" maxSteps: 5 }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_output_retries_camel() {
    let source = r#"agent Test { systemPrompt: "Hi" model: "gpt-4o-mini" outputRetries: 3 }"#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
