use super::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct ReadFileTool;

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read contents of a file"
    }
    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": { "path": { "type": "string", "description": "Path to the file" } },
            "required": ["path"]
        })
    }
    async fn execute(&self, _args: Value) -> Result<String, String> {
        Ok("stub".to_string()) // Stub - will implement in Task 11
    }
}
