//! Array method implementations for GENT
//!
//! This module provides built-in methods for array values,
//! including length, push, pop, indexOf, join, slice, concat.
//! Also includes higher-order methods: map, filter, reduce, find.

use crate::errors::{GentError, GentResult};
use crate::interpreter::{Environment, Value};
use crate::parser::ast::LambdaBody;
use crate::runtime::tools::ToolRegistry;
use crate::Span;

/// Call a method on an array value (non-lambda methods only)
///
/// # Arguments
/// * `arr` - The array to call the method on (mutable for push/pop)
/// * `method` - The method name
/// * `args` - Arguments to the method
///
/// # Supported Methods
/// * `length()` - Returns the number of elements
/// * `push(value)` - Adds an element to the end (mutates array)
/// * `pop()` - Removes and returns the last element (mutates array)
/// * `indexOf(value)` - Returns index of first occurrence, or -1
/// * `join(separator)` - Joins elements into a string
/// * `slice(start, end)` - Returns a new array with elements from start to end
/// * `concat(other)` - Returns a new array combining this and other
pub fn call_array_method(arr: &mut Vec<Value>, method: &str, args: &[Value]) -> GentResult<Value> {
    match method {
        "length" => Ok(Value::Number(arr.len() as f64)),

        "push" => {
            let value = args.get(0).cloned().ok_or_else(|| GentError::TypeError {
                expected: "argument for push()".to_string(),
                got: "missing argument".to_string(),
                span: Span::default(),
            })?;
            arr.push(value);
            Ok(Value::Null)
        }

        "pop" => Ok(arr.pop().unwrap_or(Value::Null)),

        "indexOf" => {
            let target = args.get(0).ok_or_else(|| GentError::TypeError {
                expected: "argument for indexOf()".to_string(),
                got: "missing argument".to_string(),
                span: Span::default(),
            })?;
            let index = arr.iter().position(|v| values_equal(v, target));
            Ok(Value::Number(index.map(|i| i as f64).unwrap_or(-1.0)))
        }

        "join" => {
            let sep = get_string_arg(args, 0, "join")?;
            let strings: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
            Ok(Value::String(strings.join(&sep)))
        }

        "slice" => {
            let start = get_number_arg(args, 0, "slice")? as usize;
            let end = get_number_arg(args, 1, "slice")? as usize;
            let end = end.min(arr.len());
            let start = start.min(end);
            let sliced: Vec<Value> = arr[start..end].to_vec();
            Ok(Value::Array(sliced))
        }

        "concat" => {
            let other = get_array_arg(args, 0, "concat")?;
            let mut result = arr.clone();
            result.extend(other);
            Ok(Value::Array(result))
        }

        _ => Err(GentError::UndefinedProperty {
            property: method.to_string(),
            type_name: "Array".to_string(),
            span: Span::default(),
        }),
    }
}

/// Compare two values for equality
fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        (Value::Number(n1), Value::Number(n2)) => (n1 - n2).abs() < f64::EPSILON,
        (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        _ => false,
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

/// Helper function to extract a number argument from the argument list
fn get_number_arg(args: &[Value], index: usize, method: &str) -> GentResult<f64> {
    args.get(index)
        .and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        })
        .ok_or_else(|| {
            let got = args
                .get(index)
                .map(|v| v.type_name())
                .unwrap_or("missing argument");
            GentError::TypeError {
                expected: format!("Number argument for {}()", method),
                got: got.to_string(),
                span: Span::default(),
            }
        })
}

/// Helper function to extract an array argument from the argument list
fn get_array_arg(args: &[Value], index: usize, method: &str) -> GentResult<Vec<Value>> {
    args.get(index)
        .and_then(|v| match v {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        })
        .ok_or_else(|| {
            let got = args
                .get(index)
                .map(|v| v.type_name())
                .unwrap_or("missing argument");
            GentError::TypeError {
                expected: format!("Array argument for {}()", method),
                got: got.to_string(),
                span: Span::default(),
            }
        })
}

// ============================================
// Higher-order array methods (map, filter, reduce, find)
// ============================================

/// Check if method requires a callback (for dispatch routing)
pub fn is_callback_method(method: &str) -> bool {
    matches!(method, "map" | "filter" | "reduce" | "find")
}

/// Call a higher-order array method that takes a lambda/function callback
pub fn call_array_method_with_callback<'a>(
    arr: &'a [Value],
    method: &'a str,
    callback: &'a Value,
    extra_args: &'a [Value],
    env: &'a Environment,
    tools: &'a ToolRegistry,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        match method {
            "map" => {
                let mut results = Vec::new();
                for item in arr {
                    let result = apply_callback(callback, &[item.clone()], env, tools).await?;
                    results.push(result);
                }
                Ok(Value::Array(results))
            }

            "filter" => {
                let mut results = Vec::new();
                for item in arr {
                    let result = apply_callback(callback, &[item.clone()], env, tools).await?;
                    if result.is_truthy() {
                        results.push(item.clone());
                    }
                }
                Ok(Value::Array(results))
            }

            "reduce" => {
                // Validate callback has exactly 2 parameters
                let param_count = match callback {
                    Value::Lambda(lambda) => lambda.params.len(),
                    Value::Function(fn_val) => fn_val.params.len(),
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "function or lambda".to_string(),
                            got: callback.type_name().to_string(),
                            span: Span::default(),
                        });
                    }
                };
                if param_count != 2 {
                    return Err(GentError::TypeError {
                        expected: "reduce callback requires 2 parameters".to_string(),
                        got: format!("{} parameters", param_count),
                        span: Span::default(),
                    });
                }

                let initial = extra_args.first().cloned().ok_or_else(|| GentError::TypeError {
                    expected: "initial value for reduce()".to_string(),
                    got: "missing argument".to_string(),
                    span: Span::default(),
                })?;

                let mut acc = initial;
                for item in arr {
                    acc = apply_callback(callback, &[acc, item.clone()], env, tools).await?;
                }
                Ok(acc)
            }

            "find" => {
                for item in arr {
                    let result = apply_callback(callback, &[item.clone()], env, tools).await?;
                    if result.is_truthy() {
                        return Ok(item.clone());
                    }
                }
                Ok(Value::Null)
            }

            _ => Err(GentError::UndefinedProperty {
                property: method.to_string(),
                type_name: "Array".to_string(),
                span: Span::default(),
            }),
        }
    })
}

/// Apply a callback (lambda or function reference) to arguments
fn apply_callback<'a>(
    callback: &'a Value,
    args: &'a [Value],
    env: &'a Environment,
    tools: &'a ToolRegistry,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        match callback {
            Value::Lambda(lambda) => {
                let mut lambda_env = env.clone();
                lambda_env.push_scope();

                for (param, arg) in lambda.params.iter().zip(args.iter()) {
                    lambda_env.define(param, arg.clone());
                }

                let result = match &lambda.body {
                    LambdaBody::Expression(expr) => {
                        crate::interpreter::expr_eval::evaluate_expr(expr, &lambda_env)?
                    }
                    LambdaBody::Block(block) => {
                        crate::interpreter::block_eval::evaluate_block(block, &mut lambda_env, tools).await?
                    }
                };

                Ok(result)
            }

            Value::Function(fn_val) => {
                let mut fn_env = env.clone();
                fn_env.push_scope();

                for (param, arg) in fn_val.params.iter().zip(args.iter()) {
                    fn_env.define(&param.name, arg.clone());
                }

                let result = crate::interpreter::block_eval::evaluate_block(&fn_val.body, &mut fn_env, tools).await?;
                Ok(result)
            }

            _ => Err(GentError::TypeError {
                expected: "function or lambda".to_string(),
                got: callback.type_name().to_string(),
                span: Span::default(),
            }),
        }
    })
}
