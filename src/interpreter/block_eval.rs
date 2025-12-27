//! Block evaluation module
//!
//! This module provides async block evaluation for executing tool bodies
//! with let bindings, return statements, if/else, and expression statements.

use crate::errors::{GentError, GentResult};
use crate::interpreter::builtins::{call_builtin, is_builtin};
use crate::interpreter::expr_eval::evaluate_expr;
use crate::interpreter::array_methods::{call_array_method, call_array_method_with_callback, is_callback_method};
use crate::interpreter::string_methods::call_string_method;
use crate::interpreter::types::EnumValue;
use crate::interpreter::{Environment, Value};
use crate::logging::{Logger, NullLogger};
use crate::parser::ast::{Block, BlockStmt, Expression, MatchBody, MatchPattern};
use crate::runtime::tools::ToolRegistry;
use crate::runtime::{run_agent_with_tools, LLMClient};
use std::collections::HashMap;

/// Context for block evaluation that includes optional LLM client for agent execution
pub struct BlockEvalContext<'a> {
    pub llm: Option<&'a dyn LLMClient>,
    pub logger: &'a dyn Logger,
}

impl<'a> BlockEvalContext<'a> {
    /// Create a new context with LLM client
    pub fn with_llm(llm: &'a dyn LLMClient, logger: &'a dyn Logger) -> Self {
        Self { llm: Some(llm), logger }
    }

    /// Create an empty context (no agent execution support)
    pub fn empty() -> Self {
        // Use a static NullLogger for the empty context
        static NULL_LOGGER: NullLogger = NullLogger;
        Self { llm: None, logger: &NULL_LOGGER }
    }
}

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
///
/// Note: This version does not support agent execution. Use `evaluate_block_with_llm`
/// if you need to call agent methods like `.run()`.
pub fn evaluate_block<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        // Create a new scope for this block
        env.push_scope();

        let ctx = BlockEvalContext::empty();
        let (flow, result) = evaluate_block_internal(block, env, tools, &ctx).await?;

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

/// Evaluate a block with LLM support for agent execution
///
/// This version supports calling agent methods like `.run()` within the block.
pub fn evaluate_block_with_llm<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
    llm: &'a dyn LLMClient,
    logger: &'a dyn Logger,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        // Create a new scope for this block
        env.push_scope();

        let ctx = BlockEvalContext::with_llm(llm, logger);
        let (flow, result) = evaluate_block_internal(block, env, tools, &ctx).await?;

        // Pop the scope
        env.pop_scope();

        // Handle control flow that escaped the block
        match flow {
            ControlFlow::Return(val) => Ok(*val),
            ControlFlow::Continue => Ok(result),
            ControlFlow::Break | ControlFlow::LoopContinue => Ok(result),
        }
    })
}

/// Evaluate a block with an existing context
///
/// This is used internally when calling functions to preserve the LLM context.
fn evaluate_block_with_ctx<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
    ctx: &'a BlockEvalContext<'a>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        // Create a new scope for this block
        env.push_scope();

        let (flow, result) = evaluate_block_internal(block, env, tools, ctx).await?;

        // Pop the scope
        env.pop_scope();

        // Handle control flow that escaped the block
        match flow {
            ControlFlow::Return(val) => Ok(*val),
            ControlFlow::Continue => Ok(result),
            ControlFlow::Break | ControlFlow::LoopContinue => Ok(result),
        }
    })
}

/// Internal block evaluation that returns control flow signals
fn evaluate_block_internal<'a>(
    block: &'a Block,
    env: &'a mut Environment,
    tools: &'a ToolRegistry,
    ctx: &'a BlockEvalContext<'a>,
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
                                    let val = evaluate_expr_async(arg, env, tools, ctx).await?;
                                    arg_values.push(val);
                                }

                                // Call the method and get result
                                let result = call_array_method(&mut arr_mut, &method_name, &arg_values)?;

                                // Update the array variable with the mutated array
                                env.set(&arr_var, Value::Array(arr_mut));

                                result
                            } else {
                                evaluate_expr_async(&let_stmt.value, env, tools, ctx).await?
                            }
                        } else {
                            evaluate_expr_async(&let_stmt.value, env, tools, ctx).await?
                        }
                    } else {
                        evaluate_expr_async(&let_stmt.value, env, tools, ctx).await?
                    };

                    // Define the variable in the current scope
                    env.define(&let_stmt.name, value);
                }

                BlockStmt::Assignment(assign_stmt) => {
                    // Evaluate the right-hand side expression
                    let value = evaluate_expr_async(&assign_stmt.value, env, tools, ctx).await?;

                    // Update the variable in the environment
                    if !env.set(&assign_stmt.name, value) {
                        return Err(GentError::SyntaxError {
                            message: format!("Undefined variable: '{}'", assign_stmt.name),
                            span: assign_stmt.span.clone(),
                        });
                    }
                }

                BlockStmt::Return(return_stmt) => {
                    // Evaluate the return value (if any)
                    result = if let Some(ref expr) = return_stmt.value {
                        evaluate_expr_async(expr, env, tools, ctx).await?
                    } else {
                        Value::Null
                    };
                    return Ok((ControlFlow::Return(Box::new(result)), Value::Null));
                }

                BlockStmt::If(if_stmt) => {
                    // Evaluate the condition
                    let condition = evaluate_expr_async(&if_stmt.condition, env, tools, ctx).await?;

                    // Execute the appropriate block
                    if condition.is_truthy() {
                        // Execute then block (create a new scope)
                        env.push_scope();
                        let (flow, _) = evaluate_block_internal(&if_stmt.then_block, env, tools, ctx).await?;
                        env.pop_scope();

                        // Propagate control flow signals
                        match flow {
                            ControlFlow::Continue => {}
                            other => return Ok((other, Value::Null)),
                        }
                    } else if let Some(ref else_block) = if_stmt.else_block {
                        // Execute else block (create a new scope)
                        env.push_scope();
                        let (flow, _) = evaluate_block_internal(else_block, env, tools, ctx).await?;
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
                        let (flow, _) = evaluate_block_internal(&for_stmt.body, env, tools, ctx).await?;

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
                                    let val = evaluate_expr_async(arg, env, tools, ctx).await?;
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
                    evaluate_expr_async(expr, env, tools, ctx).await?;
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
                            evaluate_expr_async(&while_stmt.condition, env, tools, ctx).await?;
                        if !condition.is_truthy() {
                            break;
                        }

                        // Execute body statements with a new scope
                        env.push_scope();
                        let (flow, _) =
                            evaluate_block_internal(&while_stmt.body, env, tools, ctx).await?;
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
                    let try_result = evaluate_block_internal(&try_stmt.try_block, env, tools, ctx).await;
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
                                evaluate_block_internal(&try_stmt.catch_block, env, tools, ctx).await?;
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
/// The `ctx` parameter provides optional LLM client for agent execution.
pub fn evaluate_expr_async<'a>(
    expr: &'a Expression,
    env: &'a Environment,
    tools: &'a ToolRegistry,
    ctx: &'a BlockEvalContext<'a>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        match expr {
            // Function/tool calls require async context
            Expression::Call(callee_expr, args, span) => {
                // Check if this is a method call on a string, array, or enum constructor
                if let Expression::Member(obj_expr, method_name, _) = callee_expr.as_ref() {
                    // First check if this could be an enum constructor call: EnumName.Variant(args)
                    if let Expression::Identifier(name, _) = obj_expr.as_ref() {
                        if let Some(enum_def) = env.get_enum(name) {
                            // Find the variant
                            if let Some(v) = enum_def.variants.iter().find(|v| v.name == *method_name) {
                                // Evaluate arguments
                                let mut arg_values = Vec::new();
                                for arg in args {
                                    let val = evaluate_expr_async(arg, env, tools, ctx).await?;
                                    arg_values.push(val);
                                }

                                if arg_values.len() != v.fields.len() {
                                    return Err(GentError::TypeError {
                                        expected: format!(
                                            "Variant '{}' expects {} arguments",
                                            method_name, v.fields.len()
                                        ),
                                        got: format!("{} arguments", arg_values.len()),
                                        span: span.clone(),
                                    });
                                }

                                return Ok(Value::Enum(EnumValue {
                                    enum_name: name.clone(),
                                    variant: method_name.clone(),
                                    data: arg_values,
                                }));
                            } else {
                                return Err(GentError::TypeError {
                                    expected: format!("valid variant of enum '{}'", name),
                                    got: method_name.clone(),
                                    span: span.clone(),
                                });
                            }
                        }
                    }

                    // Evaluate the object expression
                    let obj = evaluate_expr_async(obj_expr, env, tools, ctx).await?;

                    // If it's a string, dispatch to string methods
                    if let Value::String(s) = &obj {
                        // Evaluate method arguments
                        let mut arg_values = Vec::new();
                        for arg in args {
                            let val = evaluate_expr_async(arg, env, tools, ctx).await?;
                            arg_values.push(val);
                        }
                        return call_string_method(s, method_name, &arg_values);
                    }

                    // Check if this is an enum .is() or .data() call
                    if let Value::Enum(ref enum_val) = obj {
                        if method_name == "is" {
                            // Evaluate the argument (should be an EnumValue or EnumConstructor)
                            if args.len() != 1 {
                                return Err(GentError::TypeError {
                                    expected: "1 argument for .is()".to_string(),
                                    got: format!("{} arguments", args.len()),
                                    span: span.clone(),
                                });
                            }

                            let arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;
                            let matches = match arg {
                                Value::Enum(other) => {
                                    enum_val.enum_name == other.enum_name && enum_val.variant == other.variant
                                }
                                Value::EnumConstructor(ctor) => {
                                    enum_val.enum_name == ctor.enum_name && enum_val.variant == ctor.variant
                                }
                                _ => false,
                            };

                            return Ok(Value::Boolean(matches));
                        }

                        if method_name == "data" {
                            if args.len() != 1 {
                                return Err(GentError::TypeError {
                                    expected: "1 argument for .data()".to_string(),
                                    got: format!("{} arguments", args.len()),
                                    span: span.clone(),
                                });
                            }

                            let arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;

                            match arg {
                                Value::Number(n) => {
                                    let idx = n as usize;
                                    return Ok(enum_val.data.get(idx).cloned().unwrap_or(Value::Null));
                                }
                                Value::String(_) => {
                                    // Named access not yet implemented
                                    return Err(GentError::TypeError {
                                        expected: "number index for .data()".to_string(),
                                        got: "string (named access not yet implemented)".to_string(),
                                        span: span.clone(),
                                    });
                                }
                                _ => {
                                    return Err(GentError::TypeError {
                                        expected: "number index".to_string(),
                                        got: arg.type_name(),
                                        span: span.clone(),
                                    });
                                }
                            }
                        }
                    }

                    // If it's an array, dispatch to array methods
                    if let Value::Array(ref arr) = obj {
                        // Evaluate method arguments
                        let mut arg_values = Vec::new();
                        for arg in args {
                            let val = evaluate_expr_async(arg, env, tools, ctx).await?;
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

                    // Handle Agent method calls (userPrompt, systemPrompt, run)
                    if let Value::Agent(mut agent) = obj {
                        match method_name.as_str() {
                            "userPrompt" => {
                                // Set user_prompt and return modified agent
                                if args.is_empty() {
                                    return Err(GentError::SyntaxError {
                                        message: "userPrompt() requires an argument".to_string(),
                                        span: span.clone(),
                                    });
                                }
                                let arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;
                                let prompt = match arg {
                                    Value::String(s) => s,
                                    other => format!("{}", other),
                                };
                                agent.user_prompt = Some(prompt);
                                return Ok(Value::Agent(agent));
                            }
                            "systemPrompt" => {
                                // Set system_prompt and return modified agent
                                if args.is_empty() {
                                    return Err(GentError::SyntaxError {
                                        message: "systemPrompt() requires an argument".to_string(),
                                        span: span.clone(),
                                    });
                                }
                                let arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;
                                let prompt = match arg {
                                    Value::String(s) => s,
                                    other => format!("{}", other),
                                };
                                agent.system_prompt = prompt;
                                return Ok(Value::Agent(agent));
                            }
                            "run" => {
                                // Execute the agent - requires LLM client
                                if let Some(llm) = ctx.llm {
                                    let result = run_agent_with_tools(&agent, None, llm, tools, ctx.logger).await?;
                                    // If agent has structured output, parse as JSON
                                    if agent.output_schema.is_some() {
                                        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
                                            return Ok(json_to_value(&json_val));
                                        }
                                    }
                                    return Ok(Value::String(result));
                                } else {
                                    return Err(GentError::SyntaxError {
                                        message: "Cannot call .run() on agent in this context (no LLM client available)".to_string(),
                                        span: span.clone(),
                                    });
                                }
                            }
                            _ => {
                                return Err(GentError::SyntaxError {
                                    message: format!("Unknown agent method: {}", method_name),
                                    span: span.clone(),
                                });
                            }
                        }
                    }

                    // Handle KnowledgeBase method calls (index, search, isIndexed)
                    if let Value::KnowledgeBase(kb) = obj {
                        match method_name.as_str() {
                            "index" => {
                                let options = if args.is_empty() {
                                    crate::runtime::rag::IndexOptions::default()
                                } else {
                                    // Evaluate options argument
                                    let arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;
                                    parse_index_options(&arg)?
                                };

                                let mut kb = kb.write().await;
                                let count = kb.index(options).await
                                    .map_err(|e| GentError::SyntaxError { message: e, span: span.clone() })?;

                                return Ok(Value::Number(count as f64));
                            }
                            "search" => {
                                // Get query string
                                if args.is_empty() {
                                    return Err(GentError::SyntaxError {
                                        message: "search requires a query string".to_string(),
                                        span: span.clone(),
                                    });
                                }

                                let query_arg = evaluate_expr_async(&args[0], env, tools, ctx).await?;
                                let query = match query_arg {
                                    Value::String(s) => s,
                                    other => return Err(GentError::TypeError {
                                        expected: "String".to_string(),
                                        got: other.type_name(),
                                        span: span.clone(),
                                    }),
                                };

                                // Get limit from optional second argument (object with 'limit' field)
                                let limit = if args.len() > 1 {
                                    let options_arg = evaluate_expr_async(&args[1], env, tools, ctx).await?;
                                    if let Value::Object(o) = options_arg {
                                        o.get("limit")
                                            .and_then(|v| if let Value::Number(n) = v { Some(*n as usize) } else { None })
                                            .unwrap_or(5)
                                    } else if let Value::Number(n) = options_arg {
                                        n as usize
                                    } else {
                                        5
                                    }
                                } else {
                                    5
                                };

                                let kb = kb.read().await;
                                let results = kb.search(&query, limit).await
                                    .map_err(|e| GentError::SyntaxError { message: e, span: span.clone() })?;

                                // Convert to array of objects
                                let result_values: Vec<Value> = results.iter().map(|r| {
                                    let mut map = std::collections::HashMap::new();
                                    map.insert("source".to_string(), Value::String(r.metadata.source.clone()));
                                    map.insert("score".to_string(), Value::Number(r.score as f64));
                                    map.insert("content".to_string(), Value::String(r.metadata.content.clone()));
                                    map.insert("startLine".to_string(), Value::Number(r.metadata.start_line as f64));
                                    map.insert("endLine".to_string(), Value::Number(r.metadata.end_line as f64));
                                    Value::Object(map)
                                }).collect();

                                return Ok(Value::Array(result_values));
                            }
                            "isIndexed" => {
                                let kb = kb.read().await;
                                return Ok(Value::Boolean(kb.is_indexed()));
                            }
                            _ => {
                                return Err(GentError::SyntaxError {
                                    message: format!("KnowledgeBase has no method '{}'", method_name),
                                    span: span.clone(),
                                });
                            }
                        }
                    }

                    // For other types, return an error for now
                    return Err(GentError::TypeError {
                        expected: "String, Array, Agent, or KnowledgeBase".to_string(),
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
                    let val = evaluate_expr_async(arg, env, tools, ctx).await?;
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
                    let result = evaluate_block_with_ctx(&fn_val.body, &mut fn_env, tools, ctx).await?;
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

            // Match expression
            Expression::Match(match_expr) => {
                let subject = evaluate_expr_async(&match_expr.subject, env, tools, ctx).await?;

                for arm in &match_expr.arms {
                    if let Some(bindings) = match_pattern(&subject, &arm.pattern) {
                        // Create new scope with bindings
                        let mut match_env = env.clone();
                        match_env.push_scope();
                        for (name, value) in bindings {
                            match_env.define(&name, value);
                        }

                        // Evaluate arm body
                        let result = match &arm.body {
                            MatchBody::Expression(expr) => {
                                evaluate_expr_async(expr, &match_env, tools, ctx).await?
                            }
                            MatchBody::Block(block) => {
                                evaluate_block_with_ctx(block, &mut match_env, tools, ctx).await?
                            }
                        };

                        return Ok(result);
                    }
                }

                // No match found
                Err(GentError::SyntaxError {
                    message: "Non-exhaustive match: no pattern matched".to_string(),
                    span: match_expr.span.clone(),
                })
            }

            // Binary operations - need async for operands that might contain calls
            Expression::Binary(op, left, right, span) => {
                let left_val = evaluate_expr_async(left, env, tools, ctx).await?;
                let right_val = evaluate_expr_async(right, env, tools, ctx).await?;
                crate::interpreter::expr_eval::evaluate_binary_op_public(op, left_val, right_val, span)
            }

            // Unary operations - need async for operand that might contain calls
            Expression::Unary(op, operand, span) => {
                let val = evaluate_expr_async(operand, env, tools, ctx).await?;
                crate::interpreter::expr_eval::evaluate_unary_op_public(op, val, span)
            }

            // Array literals - need async for elements that might contain calls
            Expression::Array(elements, _) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(evaluate_expr_async(elem, env, tools, ctx).await?);
                }
                Ok(Value::Array(values))
            }

            // Object literals - need async for values that might contain calls
            Expression::Object(fields, _) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in fields {
                    let value = evaluate_expr_async(value_expr, env, tools, ctx).await?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }

            // String with interpolation - need async for embedded expressions
            Expression::String(parts, _) => {
                use crate::parser::ast::StringPart;
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expr(expr) => {
                            let value = evaluate_expr_async(expr, env, tools, ctx).await?;
                            result.push_str(&value.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }

            // Member access - need async for object expression that might contain calls
            Expression::Member(object_expr, property, span) => {
                // Check if this is an enum construction: EnumName.Variant
                if let Expression::Identifier(name, _) = object_expr.as_ref() {
                    if let Some(enum_def) = env.get_enum(name) {
                        use crate::interpreter::types::{EnumConstructor, EnumValue};
                        // Find the variant
                        let variant = enum_def.variants.iter().find(|v| v.name == *property);
                        if let Some(v) = variant {
                            if v.fields.is_empty() {
                                return Ok(Value::Enum(EnumValue {
                                    enum_name: name.clone(),
                                    variant: property.clone(),
                                    data: vec![],
                                }));
                            } else {
                                return Ok(Value::EnumConstructor(EnumConstructor {
                                    enum_name: name.clone(),
                                    variant: property.clone(),
                                    expected_fields: v.fields.len(),
                                }));
                            }
                        } else {
                            return Err(GentError::TypeError {
                                expected: format!("valid variant of enum '{}'", name),
                                got: property.clone(),
                                span: span.clone(),
                            });
                        }
                    }
                }

                // Regular member access
                let object = evaluate_expr_async(object_expr, env, tools, ctx).await?;
                match object {
                    Value::Object(map) => {
                        map.get(property)
                            .cloned()
                            .ok_or_else(|| GentError::UndefinedProperty {
                                property: property.clone(),
                                type_name: "Object".to_string(),
                                span: span.clone(),
                            })
                    }
                    _ => Err(GentError::UndefinedProperty {
                        property: property.clone(),
                        type_name: object.type_name().to_string(),
                        span: span.clone(),
                    }),
                }
            }

            // Index access - need async for target and index expressions
            Expression::Index(target_expr, index_expr, span) => {
                let target = evaluate_expr_async(target_expr, env, tools, ctx).await?;
                let index = evaluate_expr_async(index_expr, env, tools, ctx).await?;

                match target {
                    Value::Array(ref items) => {
                        if let Value::Number(n) = index {
                            let idx = n as i64;
                            if idx < 0 || idx >= items.len() as i64 {
                                return Err(GentError::IndexOutOfBounds {
                                    index: idx,
                                    length: items.len(),
                                    span: span.clone(),
                                });
                            }
                            Ok(items[idx as usize].clone())
                        } else {
                            Err(GentError::NotIndexable {
                                type_name: format!("Array with {} index", index.type_name()),
                                span: span.clone(),
                            })
                        }
                    }
                    Value::Object(ref map) => {
                        if let Value::String(key) = index {
                            map.get(&key)
                                .cloned()
                                .ok_or_else(|| GentError::UndefinedProperty {
                                    property: key.clone(),
                                    type_name: "Object".to_string(),
                                    span: span.clone(),
                                })
                        } else {
                            Err(GentError::NotIndexable {
                                type_name: format!("Object with {} index", index.type_name()),
                                span: span.clone(),
                            })
                        }
                    }
                    _ => Err(GentError::NotIndexable {
                        type_name: target.type_name().to_string(),
                        span: span.clone(),
                    }),
                }
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
            Value::Enum(e) => {
                if e.data.is_empty() {
                    JsonValue::String(format!("{}.{}", e.enum_name, e.variant))
                } else {
                    let mut map = Map::new();
                    map.insert("enum".to_string(), JsonValue::String(e.enum_name.clone()));
                    map.insert(
                        "variant".to_string(),
                        JsonValue::String(e.variant.clone()),
                    );
                    let data: Vec<JsonValue> = e.data.iter().map(value_to_json).collect();
                    map.insert("data".to_string(), JsonValue::Array(data));
                    JsonValue::Object(map)
                }
            }
            Value::EnumConstructor(c) => {
                JsonValue::String(format!("<enum constructor {}.{}>", c.enum_name, c.variant))
            }
            Value::Parallel(p) => JsonValue::String(format!("<parallel {}>", p.name)),
            Value::KnowledgeBase(_) => JsonValue::String("<KnowledgeBase>".to_string()),
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

/// Match a value against a pattern, returning bindings if successful
fn match_pattern(value: &Value, pattern: &MatchPattern) -> Option<Vec<(String, Value)>> {
    match pattern {
        MatchPattern::Wildcard => Some(vec![]),
        MatchPattern::EnumVariant { enum_name, variant_name, bindings } => {
            if let Value::Enum(enum_val) = value {
                if enum_val.enum_name == *enum_name && enum_val.variant == *variant_name {
                    // Bind data to pattern variables
                    let mut result = Vec::new();
                    for (i, binding) in bindings.iter().enumerate() {
                        if let Some(data) = enum_val.data.get(i) {
                            result.push((binding.clone(), data.clone()));
                        }
                    }
                    return Some(result);
                }
            }
            None
        }
    }
}

/// Convert a JSON value to a GENT Value
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let items = arr.iter().map(json_to_value).collect();
            Value::Array(items)
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Object(map)
        }
    }
}

/// Parse IndexOptions from a GENT Value (typically an object)
fn parse_index_options(value: &Value) -> GentResult<crate::runtime::rag::IndexOptions> {
    let mut options = crate::runtime::rag::IndexOptions::default();

    if let Value::Object(map) = value {
        // Parse extensions
        if let Some(Value::Array(exts)) = map.get("extensions") {
            options.extensions = exts.iter()
                .filter_map(|v| if let Value::String(s) = v { Some(s.clone()) } else { None })
                .collect();
        }

        // Parse recursive
        if let Some(Value::Boolean(b)) = map.get("recursive") {
            options.recursive = *b;
        }

        // Parse chunk_size
        if let Some(Value::Number(n)) = map.get("chunkSize") {
            options.chunk_size = *n as usize;
        }

        // Parse chunk_overlap
        if let Some(Value::Number(n)) = map.get("chunkOverlap") {
            options.chunk_overlap = *n as usize;
        }

        // Parse strategy
        if let Some(Value::String(s)) = map.get("strategy") {
            options.strategy = s.clone();
        }
    }

    Ok(options)
}
