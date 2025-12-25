use gent::lexer::{GentParser, Rule};
use pest::Parser;

// ============================================
// Helper Functions
// ============================================

fn parse_rule(rule: Rule, input: &str) -> bool {
    GentParser::parse(rule, input).is_ok()
}

// ============================================
// Identifier Tests
// ============================================

#[test]
fn test_identifier_simple() {
    assert!(parse_rule(Rule::identifier, "hello"));
}

#[test]
fn test_identifier_with_underscore() {
    assert!(parse_rule(Rule::identifier, "my_agent"));
}

#[test]
fn test_identifier_with_numbers() {
    assert!(parse_rule(Rule::identifier, "agent1"));
}

#[test]
fn test_identifier_camel_case() {
    assert!(parse_rule(Rule::identifier, "myAgent"));
}

#[test]
fn test_identifier_pascal_case() {
    assert!(parse_rule(Rule::identifier, "MyAgent"));
}

#[test]
fn test_identifier_cannot_start_with_number() {
    assert!(!parse_rule(Rule::identifier, "1agent"));
}

#[test]
fn test_identifier_cannot_start_with_underscore() {
    assert!(!parse_rule(Rule::identifier, "_agent"));
}

// ============================================
// String Literal Tests
// ============================================

#[test]
fn test_string_simple() {
    assert!(parse_rule(Rule::string_literal, "\"hello\""));
}

#[test]
fn test_string_with_spaces() {
    assert!(parse_rule(Rule::string_literal, "\"hello world\""));
}

#[test]
fn test_string_empty() {
    assert!(parse_rule(Rule::string_literal, "\"\""));
}

#[test]
fn test_string_with_escape_quote() {
    assert!(parse_rule(Rule::string_literal, "\"say \\\"hi\\\"\""));
}

#[test]
fn test_string_with_escape_newline() {
    assert!(parse_rule(Rule::string_literal, "\"line1\\nline2\""));
}

#[test]
fn test_string_with_escape_tab() {
    assert!(parse_rule(Rule::string_literal, "\"col1\\tcol2\""));
}

#[test]
fn test_string_with_escape_backslash() {
    assert!(parse_rule(Rule::string_literal, "\"path\\\\file\""));
}

#[test]
fn test_string_unclosed_fails() {
    assert!(!parse_rule(Rule::string_literal, "\"unclosed"));
}

// ============================================
// Number Literal Tests
// ============================================

#[test]
fn test_number_integer() {
    assert!(parse_rule(Rule::number_literal, "42"));
}

#[test]
fn test_number_float() {
    assert!(parse_rule(Rule::number_literal, "3.14"));
}

#[test]
fn test_number_negative_integer() {
    assert!(parse_rule(Rule::number_literal, "-42"));
}

#[test]
fn test_number_negative_float() {
    assert!(parse_rule(Rule::number_literal, "-3.14"));
}

#[test]
fn test_number_zero() {
    assert!(parse_rule(Rule::number_literal, "0"));
}

#[test]
fn test_number_large() {
    assert!(parse_rule(Rule::number_literal, "1234567890"));
}

// ============================================
// Boolean Literal Tests
// ============================================

#[test]
fn test_boolean_true() {
    assert!(parse_rule(Rule::boolean_literal, "true"));
}

#[test]
fn test_boolean_false() {
    assert!(parse_rule(Rule::boolean_literal, "false"));
}

#[test]
fn test_boolean_not_truthy() {
    assert!(!parse_rule(Rule::boolean_literal, "True"));
}

// ============================================
// Expression Tests
// ============================================

#[test]
fn test_expression_string() {
    assert!(parse_rule(Rule::expression, "\"hello\""));
}

#[test]
fn test_expression_number() {
    assert!(parse_rule(Rule::expression, "42"));
}

#[test]
fn test_expression_boolean() {
    assert!(parse_rule(Rule::expression, "true"));
}

#[test]
fn test_expression_identifier() {
    assert!(parse_rule(Rule::expression, "myVar"));
}

// ============================================
// Agent Field Tests
// ============================================

#[test]
fn test_agent_field_string() {
    assert!(parse_rule(
        Rule::agent_field,
        "prompt: \"You are helpful.\""
    ));
}

#[test]
fn test_agent_field_identifier() {
    assert!(parse_rule(Rule::agent_field, "model: gpt4"));
}

#[test]
fn test_agent_field_boolean() {
    assert!(parse_rule(Rule::agent_field, "verbose: true"));
}

#[test]
fn test_agent_field_number() {
    assert!(parse_rule(Rule::agent_field, "timeout: 30"));
}

// ============================================
// Agent Body Tests
// ============================================

#[test]
fn test_agent_body_empty() {
    assert!(parse_rule(Rule::agent_body, ""));
}

#[test]
fn test_agent_body_single_field() {
    assert!(parse_rule(Rule::agent_body, "prompt: \"test\""));
}

#[test]
fn test_agent_body_multiple_fields() {
    assert!(parse_rule(Rule::agent_body, "prompt: \"test\" model: gpt4"));
}

// ============================================
// Agent Declaration Tests
// ============================================

#[test]
fn test_agent_decl_minimal() {
    assert!(parse_rule(Rule::agent_decl, "agent Hello { }"));
}

#[test]
fn test_agent_decl_with_prompt() {
    assert!(parse_rule(
        Rule::agent_decl,
        "agent Hello { prompt: \"You are friendly.\" }"
    ));
}

#[test]
fn test_agent_decl_multiple_fields() {
    assert!(parse_rule(
        Rule::agent_decl,
        "agent Bot { prompt: \"Help.\" model: gpt4 }"
    ));
}

#[test]
fn test_agent_decl_with_newlines() {
    let input = r#"agent Hello {
        prompt: "You are friendly."
    }"#;
    assert!(parse_rule(Rule::agent_decl, input));
}

// ============================================
// Run Statement Tests
// ============================================

#[test]
fn test_run_simple() {
    assert!(parse_rule(Rule::run_stmt, "run Hello"));
}

#[test]
fn test_run_with_input() {
    assert!(parse_rule(Rule::run_stmt, "run Hello with \"Hi there!\""));
}

#[test]
fn test_run_with_identifier_input() {
    assert!(parse_rule(Rule::run_stmt, "run Hello with userInput"));
}

// ============================================
// Program Tests
// ============================================

#[test]
fn test_program_empty() {
    assert!(parse_rule(Rule::program, ""));
}

#[test]
fn test_program_single_agent() {
    assert!(parse_rule(Rule::program, "agent Hello { }"));
}

#[test]
fn test_program_single_run() {
    assert!(parse_rule(Rule::program, "run Hello"));
}

#[test]
fn test_program_agent_and_run() {
    let input = r#"agent Hello { prompt: "You are friendly." }
run Hello"#;
    assert!(parse_rule(Rule::program, input));
}

#[test]
fn test_program_with_comments() {
    let input = r#"// This is a comment
agent Hello { prompt: "Hi" }
// Run the agent
run Hello"#;
    assert!(parse_rule(Rule::program, input));
}

#[test]
fn test_program_hello_world() {
    let input = r#"agent Hello { prompt: "You are friendly." }
run Hello"#;
    assert!(parse_rule(Rule::program, input));
}

// ============================================
// Whitespace Handling Tests
// ============================================

#[test]
fn test_whitespace_tabs() {
    assert!(parse_rule(Rule::program, "agent\tHello\t{\t}"));
}

#[test]
fn test_whitespace_mixed() {
    assert!(parse_rule(Rule::program, "  agent  Hello  {  }  "));
}

// ============================================
// Tool Declaration Tests
// ============================================

#[test]
fn test_parse_tool_declaration() {
    let input = r#"tool greet(name: string) -> string {
        return "Hello"
    }"#;
    let result = GentParser::parse(Rule::tool_decl, input);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
}

#[test]
fn test_parse_tool_no_return_type() {
    let input = r#"tool log(msg: string) {
        return msg
    }"#;
    let result = GentParser::parse(Rule::tool_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_tool_multiple_params() {
    let input = r#"tool add(a: number, b: number) -> number {
        return a
    }"#;
    let result = GentParser::parse(Rule::tool_decl, input);
    assert!(result.is_ok());
}

#[test]
fn test_parse_tool_no_params() {
    let input = r#"tool get_time() -> string {
        return "now"
    }"#;
    let result = GentParser::parse(Rule::tool_decl, input);
    assert!(result.is_ok());
}

// ============================================
// Error Cases
// ============================================

#[test]
fn test_error_missing_agent_name() {
    assert!(!parse_rule(Rule::agent_decl, "agent { }"));
}

#[test]
fn test_error_missing_braces() {
    assert!(!parse_rule(Rule::agent_decl, "agent Hello"));
}

#[test]
fn test_error_unclosed_brace() {
    assert!(!parse_rule(Rule::agent_decl, "agent Hello {"));
}
