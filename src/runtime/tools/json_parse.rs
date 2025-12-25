use super::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct JsonParseTool;

impl JsonParseTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonParseTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for JsonParseTool {
    fn name(&self) -> &str {
        "json_parse"
    }

    fn description(&self) -> &str {
        "Parse a JSON string into an object or array"
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The JSON string to parse"
                }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, args: Value) -> Result<String, String> {
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing required parameter: text".to_string())?;

        // Parse the JSON string to validate it
        let parsed: Value = serde_json::from_str(text)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Return the JSON string (the interpreter will convert it to a Value)
        serde_json::to_string(&parsed)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }
}
