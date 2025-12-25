use super::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct WebFetchTool;

impl Default for WebFetchTool {
    fn default() -> Self {
        Self::new()
    }
}

impl WebFetchTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }
    fn description(&self) -> &str {
        "Fetch content from a URL"
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "url": { "type": "string", "description": "The URL to fetch" } },
            "required": ["url"]
        })
    }
    async fn execute(&self, _args: Value) -> Result<String, String> {
        Ok("stub".to_string()) // Stub - will implement in Task 10
    }
}
