//! Tests for agent tools field lexing

use gent::lexer::{GentParser, Rule};
use pest::Parser;

#[test]
fn test_parse_tools_field_single() {
    let result = GentParser::parse(Rule::tools_field, "tools: [web_fetch]");
    assert!(result.is_ok());
}

#[test]
fn test_parse_tools_field_multiple() {
    let result = GentParser::parse(Rule::tools_field, "tools: [web_fetch, read_file, write_file]");
    assert!(result.is_ok());
}

#[test]
fn test_parse_agent_with_tools() {
    let input = r#"agent Bot {
        systemPrompt: "Hello"
        tools: [web_fetch]
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_agent_with_tools_and_fields() {
    let input = r#"agent Bot {
        systemPrompt: "Hello"
        tools: [web_fetch, read_file]
        maxSteps: 5
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_tools_before_prompt() {
    let input = r#"agent Bot {
        tools: [web_fetch]
        systemPrompt: "Hello"
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}
