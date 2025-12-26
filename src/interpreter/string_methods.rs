//! String method implementations for GENT
//!
//! This module provides built-in methods for string values,
//! including length, trim, split, contains, and more.

use crate::errors::{GentError, GentResult};
use crate::interpreter::Value;
use crate::Span;

/// Call a method on a string value
///
/// # Arguments
/// * `s` - The string to call the method on
/// * `method` - The method name
/// * `args` - Arguments to the method
///
/// # Supported Methods
/// * `length()` - Returns the character count
/// * `trim()` - Removes leading/trailing whitespace
/// * `toLowerCase()` - Converts to lowercase
/// * `toUpperCase()` - Converts to uppercase
/// * `contains(substr)` - Checks if substring exists
/// * `startsWith(prefix)` - Checks if string starts with prefix
/// * `endsWith(suffix)` - Checks if string ends with suffix
/// * `split(separator)` - Splits string by separator
/// * `replace(old, new)` - Replaces first occurrence
pub fn call_string_method(s: &str, method: &str, args: &[Value]) -> GentResult<Value> {
    match method {
        "length" => Ok(Value::Number(s.chars().count() as f64)),

        "trim" => Ok(Value::String(s.trim().to_string())),

        "toLowerCase" => Ok(Value::String(s.to_lowercase())),

        "toUpperCase" => Ok(Value::String(s.to_uppercase())),

        "contains" => {
            let substr = get_string_arg(args, 0, "contains")?;
            Ok(Value::Boolean(s.contains(&substr)))
        }

        "startsWith" => {
            let prefix = get_string_arg(args, 0, "startsWith")?;
            Ok(Value::Boolean(s.starts_with(&prefix)))
        }

        "endsWith" => {
            let suffix = get_string_arg(args, 0, "endsWith")?;
            Ok(Value::Boolean(s.ends_with(&suffix)))
        }

        "split" => {
            let sep = get_string_arg(args, 0, "split")?;
            let parts: Vec<Value> = s
                .split(&sep)
                .map(|p| Value::String(p.to_string()))
                .collect();
            Ok(Value::Array(parts))
        }

        "replace" => {
            let old = get_string_arg(args, 0, "replace")?;
            let new = get_string_arg(args, 1, "replace")?;
            Ok(Value::String(s.replacen(&old, &new, 1)))
        }

        _ => Err(GentError::UndefinedProperty {
            property: method.to_string(),
            type_name: "String".to_string(),
            span: Span::default(),
        }),
    }
}

/// Helper function to extract a string argument from the argument list
fn get_string_arg(args: &[Value], index: usize, method: &str) -> GentResult<String> {
    args.get(index)
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            let got = args
                .get(index)
                .map(|v| v.type_name())
                .unwrap_or("missing argument");
            GentError::TypeError {
                expected: format!("String argument for {}()", method),
                got: got.to_string(),
                span: Span::default(),
            }
        })
}
