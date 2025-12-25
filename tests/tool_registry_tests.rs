use async_trait::async_trait;
use gent::runtime::tools::{Tool, ToolRegistry};
use serde_json::{json, Value};

struct MockTool {
    name: String,
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "A mock tool"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: Value) -> Result<String, String> {
        Ok("mock result".to_string())
    }
}

#[test]
fn test_registry_new() {
    let registry = ToolRegistry::new();
    assert!(registry.get("anything").is_none());
}

#[test]
fn test_registry_register_and_get() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(MockTool {
        name: "mock".to_string(),
    }));

    assert!(registry.get("mock").is_some());
    assert!(registry.get("other").is_none());
}

#[test]
fn test_registry_definitions_for() {
    let mut registry = ToolRegistry::new();
    registry.register(Box::new(MockTool {
        name: "tool_a".to_string(),
    }));
    registry.register(Box::new(MockTool {
        name: "tool_b".to_string(),
    }));
    registry.register(Box::new(MockTool {
        name: "tool_c".to_string(),
    }));

    let defs = registry.definitions_for(&["tool_a".to_string(), "tool_c".to_string()]);
    assert_eq!(defs.len(), 2);
}

#[test]
fn test_registry_with_builtins() {
    let registry = ToolRegistry::with_builtins();

    assert!(registry.get("web_fetch").is_some());
    assert!(registry.get("read_file").is_some());
    assert!(registry.get("write_file").is_some());
}
