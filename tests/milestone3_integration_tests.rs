//! Milestone 3 Integration Tests
//! End-to-end tests for user-defined tools feature

use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, ToolRegistry};

// ============================================
// Basic Tool Tests
// ============================================

#[tokio::test]
async fn test_simple_user_tool() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name + "!"
        }

        agent Bot {
            prompt: "You are a bot"
            model: "gpt-4o-mini"
        }

        run Bot
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Program should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_with_arithmetic() {
    let source = r#"
        tool add(a: number, b: number) -> number {
            return a + b
        }

        agent Calculator {
            prompt: "You are a calculator"
            model: "gpt-4o-mini"
        }

        run Calculator
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Arithmetic tool should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_with_conditional() {
    let source = r#"
        tool classify(temp: number) -> string {
            if temp > 30 {
                return "hot"
            } else {
                return "cold"
            }
        }

        agent WeatherBot {
            prompt: "You classify weather"
            model: "gpt-4o-mini"
        }

        run WeatherBot
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with conditional should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_with_let_binding() {
    let source = r#"
        tool format_name(first: string, last: string) -> string {
            let full = first + " " + last
            return full
        }

        agent Formatter {
            prompt: "You format names"
            model: "gpt-4o-mini"
        }

        run Formatter
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with let binding should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_multiple_tools() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }

        tool add(a: number, b: number) -> number {
            return a + b
        }

        tool subtract(a: number, b: number) -> number {
            return a - b
        }

        agent MultiTool {
            prompt: "You have multiple tools"
            model: "gpt-4o-mini"
        }

        run MultiTool
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Multiple tools should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_parse_full_expression() {
    let source = r#"
        tool complex_calc(x: number, y: number, name: string) -> string {
            let sum = x + y
            let product = x * y
            let difference = x - y

            if sum > 10 {
                return name + ": large result " + sum
            } else {
                return name + ": small result " + product
            }
        }

        agent ComplexBot {
            prompt: "You do complex calculations"
            model: "gpt-4o-mini"
        }

        run ComplexBot
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Complex tool with all features should parse and evaluate without errors"
    );
}

// ============================================
// Integration with Agent Features
// ============================================

#[tokio::test]
async fn test_tool_with_agent_fields() {
    let source = r#"
        tool divide(a: number, b: number) -> number {
            return a / b
        }

        agent MathBot {
            prompt: "You are a math expert"
            max_steps: 10
            model: "gpt-4o"
            use web_fetch
        }

        run MathBot
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with fully configured agent should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_with_run_input() {
    let source = r#"
        tool process(data: string) -> string {
            return "Processed: " + data
        }

        agent Processor {
            prompt: "You process data"
            model: "gpt-4o-mini"
        }

        run Processor("test input")
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with run input should parse and evaluate without errors"
    );
}

// ============================================
// Advanced Tool Body Features
// ============================================

#[tokio::test]
async fn test_tool_nested_conditionals() {
    let source = r#"
        tool grade(score: number) -> string {
            if score >= 90 {
                return "A"
            } else {
                if score >= 80 {
                    return "B"
                } else {
                    return "C"
                }
            }
        }

        agent Grader {
            prompt: "You grade students"
            model: "gpt-4o-mini"
        }

        run Grader
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with nested conditionals should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_multiple_let_bindings() {
    let source = r#"
        tool build_message(first: string, last: string, age: number) -> string {
            let full_name = first + " " + last
            let age_str = "Age: " + age
            let message = full_name + " - " + age_str
            return message
        }

        agent MessageBuilder {
            prompt: "You build messages"
            model: "gpt-4o-mini"
        }

        run MessageBuilder
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with multiple let bindings should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_all_arithmetic_operators() {
    let source = r#"
        tool calc(a: number, b: number) -> number {
            let sum = a + b
            let diff = a - b
            let prod = a * b
            let quot = a / b
            let mod = a % b
            return sum + diff + prod + quot + mod
        }

        agent AllMath {
            prompt: "You do all math operations"
            model: "gpt-4o-mini"
        }

        run AllMath
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with all arithmetic operators should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_comparison_operators() {
    let source = r#"
        tool compare(a: number, b: number) -> string {
            if a > b {
                return "greater"
            } else {
                if a < b {
                    return "less"
                } else {
                    return "equal"
                }
            }
        }

        agent Comparator {
            prompt: "You compare numbers"
            model: "gpt-4o-mini"
        }

        run Comparator
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with comparison operators should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_string_concatenation() {
    let source = r#"
        tool make_url(protocol: string, domain: string, path: string) -> string {
            let base = protocol + "://" + domain
            let full = base + "/" + path
            return full
        }

        agent URLBuilder {
            prompt: "You build URLs"
            model: "gpt-4o-mini"
        }

        run URLBuilder
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with string concatenation should parse and evaluate without errors"
    );
}

// ============================================
// Edge Cases and Special Scenarios
// ============================================

#[tokio::test]
async fn test_tool_no_return_type() {
    let source = r#"
        tool log(message: string) {
            return message
        }

        agent Logger {
            prompt: "You log messages"
            model: "gpt-4o-mini"
        }

        run Logger
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool without return type should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_tool_no_parameters() {
    let source = r#"
        tool get_greeting() -> string {
            return "Hello, World!"
        }

        agent SimpleBot {
            prompt: "You greet people"
            model: "gpt-4o-mini"
        }

        run SimpleBot
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Tool with no parameters should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_multiple_agents_with_tools() {
    let source = r#"
        tool helper(msg: string) -> string {
            return "Help: " + msg
        }

        agent Bot1 {
            prompt: "You are Bot1"
            model: "gpt-4o-mini"
        }

        agent Bot2 {
            prompt: "You are Bot2"
            model: "gpt-4o-mini"
            max_steps: 5
        }

        run Bot1
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Multiple agents with tools should parse and evaluate without errors"
    );
}

#[tokio::test]
async fn test_full_milestone3_program() {
    let source = r#"
        tool calculate_discount(price: number, discount_percent: number) -> number {
            let discount = price * discount_percent / 100
            let final_price = price - discount
            return final_price
        }

        tool format_price(amount: number, currency: string) -> string {
            return currency + " " + amount
        }

        tool validate_discount(percent: number) -> string {
            if percent > 100 {
                return "Invalid: too high"
            } else {
                if percent < 0 {
                    return "Invalid: negative"
                } else {
                    return "Valid"
                }
            }
        }

        agent ShoppingAssistant {
            prompt: "You help customers with shopping"
            use web_fetch, read_file
            max_steps: 20
            model: "gpt-4o-mini"
        }

        run ShoppingAssistant("Calculate my savings")
    "#;

    let program = parse(source).unwrap();
    let llm = MockLLMClient::with_response("I'll help you calculate your savings!");
    let mut tools = ToolRegistry::with_builtins();
    let result = evaluate_with_output(&program, &llm, &mut tools).await;

    assert!(
        result.is_ok(),
        "Full Milestone 3 program should parse and evaluate without errors"
    );
    let outputs = result.unwrap();
    assert!(!outputs.is_empty(), "Program should produce output");
}
