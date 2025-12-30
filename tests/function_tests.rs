use gent::config::Config;
use gent::interpreter::evaluate_with_output;
use gent::logging::NullLogger;
use gent::parser::parse;
use gent::runtime::{ProviderFactory, ToolRegistry};

// ============================================
// Parser Tests
// ============================================

#[test]
fn test_parse_fn_declaration() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_fn_no_return_type() {
    let source = r#"
        fn greet(name: string) {
            let msg = "Hello, {name}"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_fn_no_params() {
    let source = r#"
        fn sayHello() -> string {
            return "hello"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

// ============================================
// Evaluation Tests - Function Calls
// ============================================

#[tokio::test]
async fn test_eval_fn_call_simple() {
    let source = r#"
        fn double(x: number) -> number {
            return x + x
        }

        tool test() {
            let result = double(5)
            return "done"
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_multiple_params() {
    let source = r#"
        fn add(a: number, b: number) -> number {
            return a + b
        }

        tool test() {
            let sum = add(3, 7)
            return "done"
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_string_return() {
    let source = r#"
        fn greet(name: string) -> string {
            return "Hello, " + name
        }

        tool test() {
            let message = greet("World")
            return message
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_no_params() {
    let source = r#"
        fn getGreeting() -> string {
            return "Hello!"
        }

        tool test() {
            let msg = getGreeting()
            return msg
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_nested_call() {
    let source = r#"
        fn double(x: number) -> number {
            return x + x
        }

        fn quadruple(x: number) -> number {
            return double(double(x))
        }

        tool test() {
            let result = quadruple(3)
            return "done"
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_with_conditionals() {
    let source = r#"
        fn max(a: number, b: number) -> number {
            if a > b {
                return a
            }
            return b
        }

        tool test() {
            let result = max(10, 5)
            return "done"
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_wrong_arg_count() {
    use gent::interpreter::block_eval::evaluate_block;
    use gent::interpreter::Environment;
    use gent::interpreter::FnValue;
    use gent::interpreter::Value;
    use gent::parser::ast::{Block, BlockStmt, Expression, Param, ReturnStmt, TypeName};
    use gent::Span;

    // Create a function value manually
    let fn_val = FnValue {
        name: "add".to_string(),
        params: vec![
            Param { name: "a".to_string(), type_name: TypeName::Number, span: Span::new(0, 0) },
            Param { name: "b".to_string(), type_name: TypeName::Number, span: Span::new(0, 0) },
        ],
        return_type: Some(TypeName::Number),
        body: Block {
            statements: vec![
                BlockStmt::Return(ReturnStmt {
                    value: Some(Expression::Number(0.0, Span::new(0, 0))),
                    span: Span::new(0, 0),
                }),
            ],
            span: Span::new(0, 0),
        },
    };

    // Create environment and add function
    let mut env = Environment::new();
    env.define("add", Value::Function(fn_val));

    // Create a block that calls the function with wrong number of args
    let block = Block {
        statements: vec![
            BlockStmt::Expr(Expression::Call(
                Box::new(Expression::Identifier("add".to_string(), Span::new(0, 0))),
                vec![Expression::Number(5.0, Span::new(0, 0))], // Only 1 arg instead of 2
                Span::new(0, 0),
            )),
        ],
        span: Span::new(0, 0),
    };

    let tools = ToolRegistry::new();
    let result = evaluate_block(&block, &mut env, &tools).await;
    // This should fail because we're passing 1 arg to a 2-param function
    assert!(result.is_err(), "Should have failed with wrong argument count: {:?}", result);
}

#[tokio::test]
async fn test_eval_fn_declaration_only() {
    // Just declaring a function should not cause any issues
    let source = r#"
        fn unused(x: number) -> number {
            return x * 2
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock_with_response("Hello!");
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_eval_fn_with_local_vars() {
    let source = r#"
        fn calculate(x: number) -> number {
            let doubled = x + x
            let tripled = doubled + x
            return tripled
        }

        tool test() {
            let result = calculate(4)
            return "done"
        }

        agent TestAgent {
            systemPrompt: "Test"
            model: "gpt-4o-mini"
            tools: [test]
        }

        let output = TestAgent.run()
    "#;
    let program = parse(source).unwrap();
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let result = evaluate_with_output(&program, &factory, &mut tools, &NullLogger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
