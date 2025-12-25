use super::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::path::Path;

pub struct ReadFileTool;

impl ReadFileTool {
    pub fn new() -> Self {
        Self
    }

    fn validate_path(&self, path: &str) -> Result<(), String> {
        // Block path traversal attempts
        if path.contains("..") {
            return Err("Invalid path: path traversal not allowed".to_string());
        }
        Ok(())
    }
}

impl Default for ReadFileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Returns the file content as text."
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, String> {
        let path = args
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required parameter: path".to_string())?;

        self.validate_path(path)?;

        let path = Path::new(path);

        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))
    }
}
