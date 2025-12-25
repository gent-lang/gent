use gent::lexer::{GentParser, Rule};
use pest::Parser;
use std::fs;

#[test]
fn test_parse_hello_gnt_example() {
    let content =
        fs::read_to_string("examples/hello.gnt").expect("Failed to read examples/hello.gnt");

    let parse_result = GentParser::parse(Rule::program, &content);
    assert!(
        parse_result.is_ok(),
        "Failed to parse hello.gnt: {:?}",
        parse_result.err()
    );

    // Verify we got a valid parse tree
    let pairs = parse_result.unwrap();
    assert!(pairs.len() > 0, "Parse tree should not be empty");
}
