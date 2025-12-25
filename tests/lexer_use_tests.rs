use gent::lexer::{GentParser, Rule};
use pest::Parser;

#[test]
fn test_parse_use_single_tool() {
    let result = GentParser::parse(Rule::use_stmt, "use web_fetch");
    assert!(result.is_ok());
}

#[test]
fn test_parse_use_multiple_tools() {
    let result = GentParser::parse(Rule::use_stmt, "use web_fetch, read_file, write_file");
    assert!(result.is_ok());
}

#[test]
fn test_parse_agent_with_use() {
    let input = r#"agent Bot {
        prompt: "Hello"
        use web_fetch
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_agent_with_use_and_fields() {
    let input = r#"agent Bot {
        prompt: "Hello"
        use web_fetch, read_file
        max_steps: 5
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_use_before_prompt() {
    let input = r#"agent Bot {
        use web_fetch
        prompt: "Hello"
    }"#;
    let result = GentParser::parse(Rule::agent_decl, input);
    assert!(result.is_ok());
}
