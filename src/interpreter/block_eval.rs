//! Block evaluation module
//!
//! This module provides async block evaluation for executing tool bodies
//! with let bindings, return statements, if/else, and expression statements.

use crate::errors::{GentError, GentResult};
use crate::interpreter::builtins::{call_builtin, is_builtin};
use crate::interpreter::expr_eval::evaluate_expr;
use crate::interpreter::array_methods::{call_array_method, call_array_method_with_callback, is_callback_method};
use crate::interpreter::string_methods::call_string_method;
use crate::interpreter::{Environment, Value};
use crate::parser::ast::{Block, BlockStmt, Expression};
use crate::runtime::tools::ToolRegistry;

/// Control flow signal for break/continue/return propagation
#[derive(Debug, Clone, PartialEq)]
enum ControlFlow {
    /// Normal execution, continue to next statement
    Continue,
    /// Break out of the current loop
    Break,
    /// Skip to next iteration of the current loop
    LoopContinue,
    /// Return from the function with the given value (boxed to reduce enum size)
    Return(Box<Value>),
}

/// Type alias for async block evaluation result with control flow
type BlockInternalFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<(ControlFlow, Value)>> + 'a>>;

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

        let (flow, result) = evaluate_block_internal(block, env, tools).await?;

        // Pop the scope
        env.pop_scope();

        // Handle control flow that escaped the block
        match flow {
            ControlFlow::Return(val) => Ok(*val),
            ControlFlow::Continue => Ok(result),
            // Break/LoopContinue outside of a loop is an error, but we treat it as normal
            // completion for now (the loop handler consumes these signals)
            ControlFlow::Break | ControlFlow::LoopContinue => Ok(result),
        }
    })
}

/// Internal block evaluation that returns control flow signals
fn evaluate_block_internal<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
) -> BlockInternalFuture<'a> {
    Box::pin(async move {
        let mut result = Value::Null;

        for stmt in &block.statements {
            match stmt {
                BlockStmt::Let(let_stmt) => {
                    // Check if the value is a mutating array method call (push/pop)
                    let value = if let Some((arr_var, method_name, args)) = extract_array_method_call(&let_stmt.value) {
                        if method_name == "push" || method_name == "pop" {
                            if let Some(Value::Array(arr)) = env.get(&arr_var).cloned() {
                                let mut arr_mut = arr;

                                // Evaluate arguments
                                let mut arg_values = Vec::new();
                                for arg in args {
                                    let val = evaluate_expr_async(arg, env, tools).await?;
                                    arg_values.push(val);
                                }

                                // Call the method and get result
                                let result = call_array_method(&mut arr_mut, &method_name, &arg_values)?;

                                // Update the array variable with the mutated array
                                env.set(&arr_var, Value::Array(arr_mut));

                                result
                            } else {
                                evaluate_expr_async(&let_stmt.value, env, tools).await?
                            }
                        } else {
                            evaluate_expr_async(&let_stmt.value, env, tools).await?
                        }
                    } else {
                        evaluate_expr_async(&let_stmt.value, env, tools).await?
                    };

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
                    return Ok((ControlFlow::Return(Box::new(result)), Value::Null));
                }

                BlockStmt::If(if_stmt) => {
                    // Evaluate the condition
                    let condition = evaluate_expr_async(&if_stmt.condition, env, tools).await?;

                    // Execute the appropriate block
                    if condition.is_truthy() {
                        // Execute then block (create a new scope)
                        env.push_scope();
                        let (flow, _) = evaluate_block_internal(&if_stmt.then_block, env, tools).await?;
                        env.pop_scope();

                        // Propagate control flow signals
                        match flow {
                            ControlFlow::Continue => {}
                            other => return Ok((other, Value::Null)),
                        }
                    } else if let Some(ref else_block) = if_stmt.else_block {
                        // Execute else block (create a new scope)
                        env.push_scope();
                        let (flow, _) = evaluate_block_internal(else_block, env, tools).await?;
                        env.pop_scope();

                        // Propagate control flow signals
                        match flow {
                            ControlFlow::Continue => {}
                            other => return Ok((other, Value::Null)),
                        }
                    }
                }

                BlockStmt::For(for_stmt) => {
                    // Evaluate the iterable expression
                    let iterable = evaluate_expr(&for_stmt.iterable, env)?;

                    // Convert iterable to a list of items
                    let items: Vec<Value> = match iterable {
                        Value::Array(arr) => arr,
                        Value::String(s) => s.chars().map(|c| Value::String(c.to_string())).collect(),
                        other => {
                            return Err(GentError::TypeError {
                                expected: "Array or String".to_string(),
                                got: other.type_name().to_string(),
                                span: for_stmt.span.clone(),
                            });
                        }
                    };

                    // Iterate over items
                    'outer: for item in items {
                        env.push_scope();
                        env.define(&for_stmt.variable, item);

                        // Execute the loop body using internal evaluation
                        let (flow, _) = evaluate_block_internal(&for_stmt.body, env, tools).await?;

                        env.pop_scope();

                        // Handle control flow from the loop body
                        match flow {
                            ControlFlow::Continue => {
                                // Normal completion, continue to next iteration
                            }
                            ControlFlow::LoopContinue => {
                                // Skip to next iteration (already handled by continuing the loop)
                                continue 'outer;
                            }
                            ControlFlow::Break => {
                                // Exit the loop
                                break 'outer;
                            }
                            ControlFlow::Return(val) => {
                                // Propagate return up
                                return Ok((ControlFlow::Return(val), Value::Null));
                            }
                        }
                    }
                }

                BlockStmt::Expr(expr) => {
                    // Check for mutating array method calls (push/pop) and handle specially
                    if let Some((var_name, method_name, args)) = extract_array_method_call(expr) {
                        if method_name == "push" || method_name == "pop" {
                            // Get the current array value
                            if let Some(Value::Array(arr)) = env.get(&var_name).cloned() {
                                let mut arr_mut = arr;

                                // Evaluate arguments
                                let mut arg_values = Vec::new();
                                for arg in args {
                                    let val = evaluate_expr_async(arg, env, tools).await?;
                                    arg_values.push(val);
                                }

                                // Call the method
                                call_array_method(&mut arr_mut, &method_name, &arg_values)?;

                                // Update the variable with the mutated array
                                env.set(&var_name, Value::Array(arr_mut));
                            }
                            continue;
                        }
                    }

                    // Evaluate the expression for side effects, discarding the result
                    evaluate_expr_async(expr, env, tools).await?;
                }

                BlockStmt::While(while_stmt) => {
                    const MAX_ITERATIONS: usize = 10000; // Prevent infinite loops
                    let mut iterations = 0;

                    'while_loop: loop {
                        iterations += 1;
                        if iterations > MAX_ITERATIONS {
                            return Err(GentError::SyntaxError {
                                message: format!(
                                    "While loop exceeded maximum iterations ({})",
                                    MAX_ITERATIONS
                                ),
                                span: while_stmt.span.clone(),
                            });
                        }

                        // Evaluate condition
                        let condition =
                            evaluate_expr_async(&while_stmt.condition, env, tools).await?;
                        if !condition.is_truthy() {
                            break;
                        }

                        // Execute body statements with a new scope
                        env.push_scope();
                        let (flow, _) =
                            evaluate_block_internal(&while_stmt.body, env, tools).await?;
                        env.pop_scope();

                        // Handle control flow from the loop body
                        match flow {
                            ControlFlow::Continue => {
                                // Normal completion, continue to next iteration
                            }
                            ControlFlow::LoopContinue => {
                                // Skip to next iteration
                                continue 'while_loop;
                            }
                            ControlFlow::Break => {
                                // Exit the loop
                                break 'while_loop;
                            }
                            ControlFlow::Return(val) => {
                                // Propagate return up
                                return Ok((ControlFlow::Return(val), Value::Null));
                            }
                        }
                    }
                }

                BlockStmt::Break(_) => {
                    // Signal break to the enclosing loop
                    return Ok((ControlFlow::Break, Value::Null));
                }

                BlockStmt::Continue(_) => {
                    // Signal continue to the enclosing loop
                    return Ok((ControlFlow::LoopContinue, Value::Null));
                }

                BlockStmt::Try(try_stmt) => {
                    // Execute try block and capture result
                    env.push_scope();
                    let try_result = evaluate_block_internal(&try_stmt.try_block, env, tools).await;
                    env.pop_scope();

                    match try_result {
                        Ok((flow, _value)) => {
                            // Try block succeeded
                            match flow {
                                ControlFlow::Return(val) => {
                                    return Ok((ControlFlow::Return(val), Value::Null));
                                }
                                ControlFlow::Break => {
                                    return Ok((ControlFlow::Break, Value::Null));
                                }
                                ControlFlow::LoopContinue => {
                                    return Ok((ControlFlow::LoopContinue, Value::Null));
                                }
                                ControlFlow::Continue => {
                                    // Normal completion, continue with next statement after try/catch
                                }
                            }
                        }
                        Err(e) => {
                            // Error occurred, execute catch block with error bound
                            env.push_scope();
                            env.define(&try_stmt.error_var, Value::String(e.to_string()));

                            let catch_result =
                                evaluate_block_internal(&try_stmt.catch_block, env, tools).await?;
                            env.pop_scope();

                            match catch_result.0 {
                                ControlFlow::Return(val) => {
                                    return Ok((ControlFlow::Return(val), Value::Null));
                                }
                                ControlFlow::Break => {
                                    return Ok((ControlFlow::Break, Value::Null));
                                }
                                ControlFlow::LoopContinue => {
                                    return Ok((ControlFlow::LoopContinue, Value::Null));
                                }
                                ControlFlow::Continue => {
                                    // Normal completion, continue with next statement after try/catch
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((ControlFlow::Continue, result))
    })
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
                // Check if this is a method call on a string
                if let Expression::Member(obj_expr, method_name, _) = callee_expr.as_ref() {
                    // Evaluate the object expression
                    let obj = evaluate_expr_async(obj_expr, env, tools).await?;

                    // If it's a string, dispatch to string methods
                    if let Value::String(s) = &obj {
                        // Evaluate method arguments
                        let mut arg_values = Vec::new();
                        for arg in args {
                            let val = evaluate_expr_async(arg, env, tools).await?;
                            arg_values.push(val);
                        }
                        return call_string_method(s, method_name, &arg_values);
                    }

                    // If it's an array, dispatch to array methods
                    if let Value::Array(ref arr) = obj {
                        // Evaluate method arguments
                        let mut arg_values = Vec::new();
                        for arg in args {
                            let val = evaluate_expr_async(arg, env, tools).await?;
                            arg_values.push(val);
                        }

                        // Check if this is a callback method (map, filter, reduce, find)
                        if is_callback_method(method_name) {
                            let callback = arg_values.first().ok_or_else(|| GentError::TypeError {
                                expected: "callback function for array method".to_string(),
                                got: "missing argument".to_string(),
                                span: span.clone(),
                            })?;
                            let extra_args = if arg_values.len() > 1 { &arg_values[1..] } else { &[] };
                            return call_array_method_with_callback(
                                arr,
                                method_name,
                                callback,
                                extra_args,
                                env,
                                tools,
                            ).await;
                        }

                        // Non-callback methods
                        let mut arr_clone = arr.clone();
                        let result = call_array_method(
                            &mut arr_clone,
                            method_name,
                            &arg_values,
                        )?;

                        return Ok(result);
                    }

                    // For other types, return an error for now
                    return Err(GentError::TypeError {
                        expected: "String or Array".to_string(),
                        got: obj.type_name().to_string(),
                        span: span.clone(),
                    });
                }

                // Get the callable name
                let callable_name = if let Expression::Identifier(name, _) = callee_expr.as_ref() {
                    name.clone()
                } else {
                    let callee = evaluate_expr(callee_expr, env)?;
                    return Err(GentError::TypeError {
                        expected: "function or tool name".to_string(),
                        got: callee.type_name().to_string(),
                        span: span.clone(),
                    });
                };

                // Evaluate arguments first (needed for both functions and tools)
                let mut arg_values = Vec::new();
                for arg in args {
                    let val = evaluate_expr_async(arg, env, tools).await?;
                    arg_values.push(val);
                }

                // Check if it's a built-in function
                if is_builtin(&callable_name) {
                    return call_builtin(&callable_name, &arg_values, span);
                }

                // Check if it's a function in the environment
                if let Some(Value::Function(fn_val)) = env.get(&callable_name) {
                    // Clone the function value since we need to borrow env mutably later
                    let fn_val = fn_val.clone();

                    // Check argument count
                    if arg_values.len() != fn_val.params.len() {
                        return Err(GentError::SyntaxError {
                            message: format!(
                                "Function '{}' expects {} arguments, got {}",
                                fn_val.name,
                                fn_val.params.len(),
                                arg_values.len()
                            ),
                            span: span.clone(),
                        });
                    }

                    // Create a new environment with function scope
                    let mut fn_env = env.clone();
                    fn_env.push_scope();

                    // Bind parameters to arguments
                    for (param, arg_val) in fn_val.params.iter().zip(arg_values.iter()) {
                        fn_env.define(&param.name, arg_val.clone());
                    }

                    // Evaluate the function body
                    let result = evaluate_block(&fn_val.body, &mut fn_env, tools).await?;
                    return Ok(result);
                }

                // Look up the tool in the registry
                let tool = tools
                    .get(&callable_name)
                    .ok_or_else(|| GentError::UnknownTool {
                        name: callable_name.clone(),
                        span: span.clone(),
                    })?;

                // Convert arguments to JSON for tool execution
                let json_args = args_to_json(&arg_values);

                // Execute the tool
                let result = tool
                    .execute(json_args)
                    .await
                    .map_err(|e| GentError::ToolError {
                        tool: callable_name.clone(),
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
            Value::Function(_) => JsonValue::String("<function>".to_string()),
            Value::Lambda(_) => JsonValue::String("<lambda>".to_string()),
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

/// Extract variable name, method name, and arguments from a method call expression
/// Returns Some((var_name, method_name, args)) if this is a method call on an identifier
fn extract_array_method_call(expr: &Expression) -> Option<(String, String, &Vec<Expression>)> {
    if let Expression::Call(callee_expr, args, _) = expr {
        if let Expression::Member(obj_expr, method_name, _) = callee_expr.as_ref() {
            if let Expression::Identifier(var_name, _) = obj_expr.as_ref() {
                return Some((var_name.clone(), method_name.clone(), args));
            }
        }
    }
    None
}
