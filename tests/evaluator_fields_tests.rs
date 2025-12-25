use gent::interpreter::evaluate_with_output;
use gent::parser::parse;
use gent::runtime::MockLLMClient;

#[test]
fn test_evaluate_agent_with_max_steps() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            max_steps: 5
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let result = evaluate_with_output(&program, &llm);
    assert!(result.is_ok());
}

#[test]
fn test_evaluate_agent_with_model() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            model: "gpt-4o"
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let result = evaluate_with_output(&program, &llm);
    assert!(result.is_ok());
}

#[test]
fn test_evaluate_agent_with_tools() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            use web_fetch, read_file
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let result = evaluate_with_output(&program, &llm);
    assert!(result.is_ok());
}

#[test]
fn test_evaluate_agent_all_fields() {
    let source = r#"
        agent Bot {
            prompt: "Hello"
            use web_fetch
            max_steps: 20
            model: "gpt-4o-mini"
        }
        run Bot
    "#;
    let program = parse(source).unwrap();
    let llm = MockLLMClient::new();
    let result = evaluate_with_output(&program, &llm);
    assert!(result.is_ok());
}
