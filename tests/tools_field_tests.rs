//! Tests for tools: field in agents

use gent::parser::ast::{Expression, Statement};

#[test]
fn test_parse_agent_with_tools_array() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }
        agent Helper {
            tools: [greet]
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    // Find the agent declaration
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some(), "Expected to find agent declaration");

    let agent = agent.unwrap();
    assert_eq!(agent.name, "Helper");
    assert!(agent.tools_expr.is_some(), "Expected tools_expr to be set");

    // Verify it's an array expression
    if let Some(Expression::Array(elements, _)) = &agent.tools_expr {
        assert_eq!(elements.len(), 1, "Expected one tool in the array");
        if let Expression::Identifier(name, _) = &elements[0] {
            assert_eq!(name, "greet");
        } else {
            panic!("Expected identifier in tools array");
        }
    } else {
        panic!("Expected tools_expr to be an array expression");
    }
}

#[test]
fn test_parse_agent_with_dynamic_tools() {
    let source = r#"
        agent Helper {
            tools: baseTools + [customTool]
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some(), "Expected to find agent declaration");

    let agent = agent.unwrap();
    assert!(agent.tools_expr.is_some(), "Expected tools_expr to be set");

    // Verify it's a binary expression (addition)
    if let Some(Expression::Binary(op, _, _, _)) = &agent.tools_expr {
        assert_eq!(*op, gent::parser::ast::BinaryOp::Add);
    } else {
        panic!("Expected tools_expr to be a binary expression");
    }
}

#[test]
fn test_parse_agent_with_multiple_tools() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }
        tool search(query: string) -> string {
            return "Results for: " + query
        }
        agent Helper {
            tools: [greet, search]
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some());

    let agent = agent.unwrap();
    if let Some(Expression::Array(elements, _)) = &agent.tools_expr {
        assert_eq!(elements.len(), 2, "Expected two tools in the array");
    } else {
        panic!("Expected tools_expr to be an array expression");
    }
}

#[test]
fn test_parse_agent_without_tools_field() {
    let source = r#"
        agent Helper {
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some());

    let agent = agent.unwrap();
    assert!(
        agent.tools_expr.is_none(),
        "Expected tools_expr to be None when not specified"
    );
}

#[test]
fn test_parse_agent_with_identifier_tools() {
    // Test using a variable reference for tools
    let source = r#"
        agent Helper {
            tools: myToolsArray
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some());

    let agent = agent.unwrap();
    if let Some(Expression::Identifier(name, _)) = &agent.tools_expr {
        assert_eq!(name, "myToolsArray");
    } else {
        panic!("Expected tools_expr to be an identifier");
    }
}

#[test]
fn test_parse_agent_with_both_use_and_tools() {
    // Test that both use statements and tools field can coexist
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }
        tool search(query: string) -> string {
            return "Results for: " + query
        }
        agent Helper {
            use greet
            tools: [search]
            model: "gpt-4o"
            systemPrompt: "You help people"
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    let agent = program.statements.iter().find_map(|s| {
        if let Statement::AgentDecl(a) = s {
            Some(a)
        } else {
            None
        }
    });
    assert!(agent.is_some());

    let agent = agent.unwrap();
    // Should have both use-declared tools and tools_expr
    assert_eq!(agent.tools.len(), 1, "Expected one tool from use statement");
    assert_eq!(agent.tools[0], "greet");
    assert!(
        agent.tools_expr.is_some(),
        "Expected tools_expr from tools field"
    );
}

#[tokio::test]
async fn test_eval_agent_with_tools_field() {
    let source = r#"
        tool greet(name: string) -> string {
            return "Hello, " + name
        }

        agent Helper {
            tools: [greet]
            model: "gpt-4o"
            systemPrompt: "You help"
        }

        println("ok")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
