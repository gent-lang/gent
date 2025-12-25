use gent::interpreter::types::AgentValue;

#[test]
fn test_agent_value_with_tools() {
    let agent = AgentValue::new("Bot", "Hello").with_tools(vec!["web_fetch".to_string()]);

    assert_eq!(agent.tools, vec!["web_fetch"]);
}

#[test]
fn test_agent_value_with_max_steps() {
    let agent = AgentValue::new("Bot", "Hello").with_max_steps(5);

    assert_eq!(agent.max_steps, Some(5));
}

#[test]
fn test_agent_value_with_model() {
    let agent = AgentValue::new("Bot", "Hello").with_model("gpt-4o");

    assert_eq!(agent.model, Some("gpt-4o".to_string()));
}

#[test]
fn test_agent_value_defaults() {
    let agent = AgentValue::new("Bot", "Hello");

    assert!(agent.tools.is_empty());
    assert!(agent.max_steps.is_none());
    assert!(agent.model.is_none());
}

#[test]
fn test_agent_value_builder_chain() {
    let agent = AgentValue::new("Bot", "Hello")
        .with_tools(vec!["web_fetch".to_string(), "read_file".to_string()])
        .with_max_steps(10)
        .with_model("gpt-4o-mini");

    assert_eq!(agent.tools.len(), 2);
    assert_eq!(agent.max_steps, Some(10));
    assert_eq!(agent.model, Some("gpt-4o-mini".to_string()));
}
