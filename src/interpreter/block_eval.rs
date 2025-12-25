//! Block evaluation module
//!
//! This module provides async block evaluation for executing tool bodies
//! with let bindings, return statements, if/else, and expression statements.

use crate::errors::{GentError, GentResult};
use crate::interpreter::expr_eval::evaluate_expr;
use crate::interpreter::{Environment, Value};
use crate::parser::ast::{Block, BlockStmt, Expression};
use crate::runtime::tools::ToolRegistry;

/// Evaluate a block of statements in the given environment
///
/// Returns the value of the first return statement encountered,
/// or Value::Null if the block completes without returning.
pub fn evaluate_block<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        // Create a new scope for this block
        env.push_scope();

        let mut result = Value::Null;

        for stmt in &block.statements {
            match stmt {
                BlockStmt::Let(let_stmt) => {
                    // Evaluate the value expression
                    let value = evaluate_expr_async(&let_stmt.value, env, tools).await?;
                    // Define the variable in the current scope
                    env.define(&let_stmt.name, value);
                }

                BlockStmt::Return(return_stmt) => {
                    // Evaluate the return value (if any)
                    result = if let Some(ref expr) = return_stmt.value {
                        evaluate_expr_async(expr, env, tools).await?
                    } else {
                        Value::Null
                    };
                    // Pop the scope before returning
                    env.pop_scope();
                    return Ok(result);
                }

                BlockStmt::If(if_stmt) => {
                    // Evaluate the condition
                    let condition = evaluate_expr_async(&if_stmt.condition, env, tools).await?;

                    // Execute the appropriate block
                    if condition.is_truthy() {
                        // Execute then block
                        let then_result = evaluate_block(&if_stmt.then_block, env, tools).await?;
                        // If the then block returned, propagate that return
                        if !matches!(then_result, Value::Null) || has_return(&if_stmt.then_block)
                        {
                            env.pop_scope();
                            return Ok(then_result);
                        }
                    } else if let Some(ref else_block) = if_stmt.else_block {
                        // Execute else block
                        let else_result = evaluate_block(else_block, env, tools).await?;
                        // If the else block returned, propagate that return
                        if !matches!(else_result, Value::Null) || has_return(else_block) {
                            env.pop_scope();
                            return Ok(else_result);
                        }
                    }
                }

                BlockStmt::Expr(expr) => {
                    // Evaluate the expression for side effects, discarding the result
                    evaluate_expr_async(expr, env, tools).await?;
                }
            }
        }

        // Pop the scope when the block ends naturally
        env.pop_scope();

        Ok(result)
    })
}

/// Check if a block contains a return statement at the top level
fn has_return(block: &Block) -> bool {
    block
        .statements
        .iter()
        .any(|stmt| matches!(stmt, BlockStmt::Return(_)))
}

/// Evaluate an expression in an async context, handling function calls
///
/// This function is similar to evaluate_expr but supports async tool calls.
pub fn evaluate_expr_async<'a>(
    expr: &'a Expression,
    env: &'a Environment,
    tools: &'a ToolRegistry,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        match expr {
            // Function/tool calls require async context
            Expression::Call(callee_expr, args, span) => {
                // Evaluate the callee
                let callee = evaluate_expr(callee_expr, env)?;

                // Get the tool name
                let tool_name = if let Expression::Identifier(name, _) = callee_expr.as_ref() {
                    name.clone()
                } else {
                    return Err(GentError::TypeError {
                        expected: "tool name".to_string(),
                        got: callee.type_name().to_string(),
                        span: span.clone(),
                    });
                };

                // Look up the tool in the registry
                let tool = tools.get(&tool_name).ok_or_else(|| GentError::UnknownTool {
                    name: tool_name.clone(),
                    span: span.clone(),
                })?;

                // Evaluate arguments recursively
                let mut arg_values = Vec::new();
                for arg in args {
                    let val = evaluate_expr_async(arg, env, tools).await?;
                    arg_values.push(val);
                }

                // Convert arguments to JSON for tool execution
                let json_args = args_to_json(&arg_values);

                // Execute the tool
                let result = tool
                    .execute(json_args)
                    .await
                    .map_err(|e| GentError::ToolError {
                        tool: tool_name.clone(),
                        message: e,
                    })?;

                // For now, return the result as a string
                // TODO: Parse JSON results in the future
                Ok(Value::String(result))
            }

            // All other expressions can be evaluated synchronously
            _ => evaluate_expr(expr, env),
        }
    })
}

/// Convert a vector of Values to a JSON value for tool execution
fn args_to_json(args: &[Value]) -> serde_json::Value {
    use serde_json::{json, Map, Value as JsonValue};

    fn value_to_json(val: &Value) -> JsonValue {
        match val {
            Value::String(s) => JsonValue::String(s.clone()),
            Value::Number(n) => json!(n),
            Value::Boolean(b) => JsonValue::Bool(*b),
            Value::Null => JsonValue::Null,
            Value::Array(items) => {
                let json_items: Vec<JsonValue> = items.iter().map(value_to_json).collect();
                JsonValue::Array(json_items)
            }
            Value::Object(map) => {
                let mut json_map = Map::new();
                for (k, v) in map {
                    json_map.insert(k.clone(), value_to_json(v));
                }
                JsonValue::Object(json_map)
            }
            Value::Agent(_) => JsonValue::String("<agent>".to_string()),
            Value::Tool(_) => JsonValue::String("<tool>".to_string()),
        }
    }

    // If there's a single object argument, use it directly
    // Otherwise, wrap arguments in an array
    if args.len() == 1 {
        if let Value::Object(_) = &args[0] {
            return value_to_json(&args[0]);
        }
    }

    // For multiple args or non-object single arg, create an array
    JsonValue::Array(args.iter().map(value_to_json).collect())
}
