//! Built-in functions for GENT
//!
//! This module provides global built-in functions like print and println.

use std::io::{self, Write};

use crate::errors::{GentError, GentResult};
use crate::interpreter::Value;
use crate::Span;

/// Check if a function name is a built-in
pub fn is_builtin(name: &str) -> bool {
    matches!(name, "print" | "println")
}

/// Call a built-in function
///
/// # Arguments
/// * `name` - The built-in function name
/// * `args` - Arguments to the function (converted to strings automatically)
/// * `span` - Source span for error reporting
///
/// # Returns
/// * `Ok(Value::Null)` on success
pub fn call_builtin(name: &str, args: &[Value], span: &Span) -> GentResult<Value> {
    // Convert all arguments to their string representation
    let strings: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();

    // Join with spaces
    let output = strings.join(" ");

    match name {
        "print" => {
            print!("{}", output);
            io::stdout().flush().unwrap();
        }
        "println" => {
            println!("{}", output);
        }
        _ => {
            return Err(GentError::UnknownTool {
                name: name.to_string(),
                span: span.clone(),
            });
        }
    }

    Ok(Value::Null)
}
