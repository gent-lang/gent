//! Output validation for structured outputs

use crate::interpreter::OutputSchema;
use crate::parser::ast::FieldType;
use serde_json::Value as JsonValue;

/// Validate JSON output against a schema
pub fn validate_output(json: &JsonValue, schema: &OutputSchema) -> Result<(), String> {
    let obj = json.as_object().ok_or("Expected JSON object")?;

    for field in &schema.fields {
        let value = obj
            .get(&field.name)
            .ok_or_else(|| format!("missing required field: '{}'", field.name))?;

        validate_field_type(value, &field.field_type, &field.name)?;
    }

    Ok(())
}

fn validate_field_type(value: &JsonValue, expected: &FieldType, path: &str) -> Result<(), String> {
    match expected {
        FieldType::String => {
            if !value.is_string() {
                return Err(format!(
                    "'{}': expected string, got {}",
                    path,
                    json_type_name(value)
                ));
            }
        }
        FieldType::Number => {
            if !value.is_number() {
                return Err(format!(
                    "'{}': expected number, got {}",
                    path,
                    json_type_name(value)
                ));
            }
        }
        FieldType::Boolean => {
            if !value.is_boolean() {
                return Err(format!(
                    "'{}': expected boolean, got {}",
                    path,
                    json_type_name(value)
                ));
            }
        }
        FieldType::Array(inner) => {
            let arr = value.as_array().ok_or_else(|| {
                format!("'{}': expected array, got {}", path, json_type_name(value))
            })?;
            for (i, item) in arr.iter().enumerate() {
                validate_field_type(item, inner, &format!("{}[{}]", path, i))?;
            }
        }
        FieldType::Object(fields) => {
            let obj = value.as_object().ok_or_else(|| {
                format!("'{}': expected object, got {}", path, json_type_name(value))
            })?;
            for field in fields {
                let field_value = obj
                    .get(&field.name)
                    .ok_or_else(|| format!("'{}.{}': missing required field", path, field.name))?;
                validate_field_type(
                    field_value,
                    &field.field_type,
                    &format!("{}.{}", path, field.name),
                )?;
            }
        }
        FieldType::Named(_) => {
            // Named types should be resolved before validation
            // For now, accept any object
            if !value.is_object() {
                return Err(format!(
                    "'{}': expected object, got {}",
                    path,
                    json_type_name(value)
                ));
            }
        }
    }
    Ok(())
}

fn json_type_name(value: &JsonValue) -> &'static str {
    match value {
        JsonValue::Null => "null",
        JsonValue::Bool(_) => "boolean",
        JsonValue::Number(_) => "number",
        JsonValue::String(_) => "string",
        JsonValue::Array(_) => "array",
        JsonValue::Object(_) => "object",
    }
}
