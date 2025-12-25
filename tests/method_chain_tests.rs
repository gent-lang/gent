use gent::parser::parse;

// ============================================
// Method Chain Syntax Parsing Tests
// ============================================

#[test]
fn test_parse_invoke_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_user_prompt_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.userPrompt("Hello").invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_chained_methods() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.systemPrompt("Be helpful").userPrompt("Hi").invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_system_prompt_method() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let result = Test.systemPrompt("You are a helpful assistant").invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_full_method_chain() {
    let source = r#"
        agent Translator { model: "gpt-4o-mini" }
        let result = Translator.systemPrompt("You are a translator").userPrompt("Translate to French: Hello").invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_method_chain_with_variable_arg() {
    let source = r#"
        agent Test { model: "gpt-4o-mini" }
        let message = "Hello world"
        let result = Test.userPrompt(message).invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_invoke_without_prompt() {
    let source = r#"
        agent Test {
            model: "gpt-4o-mini"
            prompt: "Default prompt"
        }
        let result = Test.invoke()
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}
