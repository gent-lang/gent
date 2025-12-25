//! User-defined tool wrapper for the tool registry

use super::Tool;
use crate::interpreter::block_eval::evaluate_block;
use crate::interpreter::{Environment, UserToolValue, Value};
use crate::parser::ast::TypeName;
use async_trait::async_trait;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;

/// Wrapper that makes UserToolValue implement the Tool trait
pub struct UserToolWrapper {
    tool: UserToolValue,
    env: Arc<Environment>,
}

impl UserToolWrapper {
    /// Create a new UserToolWrapper
    pub fn new(tool: UserToolValue, env: Arc<Environment>) -> Self {
        Self { tool, env }
    }
}

#[async_trait]
impl Tool for UserToolWrapper {
    fn name(&self) -> &str {
        &self.tool.name
    }

    fn description(&self) -> &str {
        "User-defined tool"
    }

    fn parameters_schema(&self) -> JsonValue {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.tool.params {
            required.push(param.name.clone());

            let type_str = match param.type_name {
                TypeName::String => "string",
                TypeName::Number => "number",
                TypeName::Boolean => "boolean",
                TypeName::Array => "array",
                TypeName::Object => "object",
                TypeName::Any => "string", // Default to string for Any
            };

            properties.insert(
                param.name.clone(),
                json!({
                    "type": type_str,
                    "description": format!("Parameter {}", param.name)
                }),
            );
        }

        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, String> {
        // Clone all the data we need to own it in the async block
        let tool_body = self.tool.body.clone();
        let params = self.tool.params.clone();
        let base_env = self.env.clone();

        // Use spawn_blocking to run the non-Send future in a blocking context
        tokio::task::spawn_blocking(move || {
            // Create a new runtime for the blocking task
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                // Clone the environment to create an isolated execution context
                let mut exec_env = (*base_env).clone();

                // Bind parameters from JSON args to the environment
                for param in &params {
                    let arg_value = args
                        .get(&param.name)
                        .ok_or_else(|| format!("Missing required parameter: {}", param.name))?;

                    // Convert JSON value to GENT Value
                    let gent_value = json_to_value(arg_value);

                    // Define the parameter in the environment
                    exec_env.define(&param.name, gent_value);
                }

                // Create an empty tool registry for executing the tool body
                // User tools cannot call other tools during their execution
                let tools = super::ToolRegistry::new();

                // Execute the tool body
                let result = evaluate_block(&tool_body, &mut exec_env, &tools)
                    .await
                    .map_err(|e| format!("Tool execution failed: {}", e))?;

                // Convert the result to a string
                Ok::<String, String>(result.to_string())
            })
        })
        .await
        .map_err(|e| format!("Task panicked: {}", e))?
    }
}

/// Convert a JSON value to a GENT Value
fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Null
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            let items = arr.iter().map(json_to_value).collect();
            Value::Array(items)
        }
        JsonValue::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Object(map)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_to_value_primitives() {
        assert_eq!(json_to_value(&json!(null)), Value::Null);
        assert_eq!(json_to_value(&json!(true)), Value::Boolean(true));
        assert_eq!(json_to_value(&json!(42)), Value::Number(42.0));
        assert_eq!(
            json_to_value(&json!("hello")),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_json_to_value_array() {
        let json_arr = json!([1, 2, 3]);
        let result = json_to_value(&json_arr);

        if let Value::Array(items) = result {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        } else {
            panic!("Expected Array value");
        }
    }

    #[test]
    fn test_json_to_value_object() {
        let json_obj = json!({"key": "value", "num": 42});
        let result = json_to_value(&json_obj);

        if let Value::Object(map) = result {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("key"), Some(&Value::String("value".to_string())));
            assert_eq!(map.get("num"), Some(&Value::Number(42.0)));
        } else {
            panic!("Expected Object value");
        }
    }
}
