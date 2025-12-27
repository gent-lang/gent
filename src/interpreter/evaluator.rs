//! Program evaluation for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::block_eval::evaluate_block_with_llm;
use crate::interpreter::builtins::{call_builtin, is_builtin};
use crate::interpreter::expr_eval::evaluate_expr;
use crate::interpreter::imports::collect_imports;
use crate::interpreter::string_methods::call_string_method;
use crate::interpreter::{AgentValue, Environment, FnValue, OutputSchema, ParallelValue, UserToolValue, Value};
use crate::logging::{LogLevel, Logger, NullLogger};
use crate::parser::{AgentDecl, Expression, Program, Statement, StringPart, StructField, ToolDecl};
use crate::runtime::{run_agent_with_tools, LLMClient, ToolRegistry, UserToolWrapper};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Evaluate a GENT program
///
/// # Arguments
/// * `program` - The parsed AST
/// * `llm` - The LLM client to use for agent execution
/// * `tools` - The tool registry for agent execution
/// * `logger` - The logger for debug output
///
/// # Returns
/// Ok(()) on success, Err on failure
pub async fn evaluate(
    program: &Program,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<()> {
    let mut env = Environment::new();
    let mut structs: HashMap<String, Vec<StructField>> = HashMap::new();

    // First pass: collect struct declarations
    for statement in &program.statements {
        if let Statement::StructDecl(decl) = statement {
            structs.insert(decl.name.clone(), decl.fields.clone());
        }
    }

    // Collect enum definitions
    for statement in &program.statements {
        if let Statement::EnumDecl(decl) = statement {
            let def = crate::interpreter::types::EnumDef {
                name: decl.name.clone(),
                variants: decl
                    .variants
                    .iter()
                    .map(|v| crate::interpreter::types::EnumVariantDef {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|f| crate::interpreter::types::EnumFieldDef {
                                name: f.name.clone(),
                                type_name: f.type_name.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            };
            env.define_enum(def);
        }
    }

    // Second pass: evaluate statements
    for statement in &program.statements {
        evaluate_statement(statement, &mut env, llm, tools, logger, &structs).await?;
    }

    Ok(())
}

/// Evaluate a GENT program and capture output (uses null logger)
pub async fn evaluate_with_output(
    program: &Program,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
) -> GentResult<Vec<String>> {
    let logger = NullLogger;
    let mut env = Environment::new();
    let mut outputs = Vec::new();
    let mut structs: HashMap<String, Vec<StructField>> = HashMap::new();

    // First pass: collect struct declarations
    for statement in &program.statements {
        if let Statement::StructDecl(decl) = statement {
            structs.insert(decl.name.clone(), decl.fields.clone());
        }
    }

    // Collect enum definitions
    for statement in &program.statements {
        if let Statement::EnumDecl(decl) = statement {
            let def = crate::interpreter::types::EnumDef {
                name: decl.name.clone(),
                variants: decl
                    .variants
                    .iter()
                    .map(|v| crate::interpreter::types::EnumVariantDef {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|f| crate::interpreter::types::EnumFieldDef {
                                name: f.name.clone(),
                                type_name: f.type_name.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            };
            env.define_enum(def);
        }
    }

    // Second pass: evaluate statements
    for statement in &program.statements {
        if let Some(output) =
            evaluate_statement_with_output(statement, &mut env, llm, tools, &logger, &structs)
                .await?
        {
            outputs.push(output);
        }
    }

    Ok(outputs)
}

/// Evaluate a program with imports resolved from source file
pub async fn evaluate_with_imports(
    program: &Program,
    source_file: Option<&Path>,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<()> {
    let mut env = Environment::new();
    let mut structs: HashMap<String, Vec<StructField>> = HashMap::new();

    // Process imports if source file is provided
    if let Some(file) = source_file {
        let mut visited = std::collections::HashSet::new();
        let imports = collect_imports(program, file, &mut visited)?;

        for (names, imported_program) in imports {
            // Collect structs from imported program
            for stmt in &imported_program.statements {
                if let Statement::StructDecl(decl) = stmt {
                    if names.contains(&decl.name) {
                        structs.insert(decl.name.clone(), decl.fields.clone());
                    }
                }
            }

            // Evaluate imported declarations
            for stmt in &imported_program.statements {
                match stmt {
                    Statement::FnDecl(fn_decl) if names.contains(&fn_decl.name) => {
                        let fn_value = Value::Function(FnValue {
                            name: fn_decl.name.clone(),
                            params: fn_decl.params.clone(),
                            return_type: fn_decl.return_type.clone(),
                            body: fn_decl.body.clone(),
                        });
                        env.define(&fn_decl.name, fn_value);
                    }
                    Statement::AgentDecl(decl) if names.contains(&decl.name) => {
                        evaluate_agent_decl(decl, &mut env, &structs)?;
                    }
                    Statement::ToolDecl(decl) if names.contains(&decl.name) => {
                        evaluate_tool_decl(decl, &mut env, tools)?;
                    }
                    _ => {}
                }
            }
        }
    }

    // First pass: collect struct declarations from main program
    for statement in &program.statements {
        if let Statement::StructDecl(decl) = statement {
            structs.insert(decl.name.clone(), decl.fields.clone());
        }
    }

    // Collect enum definitions
    for statement in &program.statements {
        if let Statement::EnumDecl(decl) = statement {
            let def = crate::interpreter::types::EnumDef {
                name: decl.name.clone(),
                variants: decl
                    .variants
                    .iter()
                    .map(|v| crate::interpreter::types::EnumVariantDef {
                        name: v.name.clone(),
                        fields: v
                            .fields
                            .iter()
                            .map(|f| crate::interpreter::types::EnumFieldDef {
                                name: f.name.clone(),
                                type_name: f.type_name.clone(),
                            })
                            .collect(),
                    })
                    .collect(),
            };
            env.define_enum(def);
        }
    }

    // Second pass: evaluate statements
    for statement in &program.statements {
        evaluate_statement(statement, &mut env, llm, tools, logger, &structs).await?;
    }

    Ok(())
}

async fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
    structs: &HashMap<String, Vec<StructField>>,
) -> GentResult<()> {
    match statement {
        Statement::Import(_) => {
            // Import statements are handled during a separate import resolution phase
            // No runtime action needed here
        }
        Statement::AgentDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring agent '{}'", decl.name),
            );
            evaluate_agent_decl(decl, env, structs)?;
        }
        Statement::ToolDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring tool '{}'", decl.name),
            );
            evaluate_tool_decl(decl, env, tools)?;
        }
        Statement::StructDecl(_) => {
            // Struct declarations are handled during parsing/validation
            // No runtime action needed
        }
        Statement::EnumDecl(_) => {
            // Enum declarations are handled during parsing/validation
            // No runtime action needed
        }
        Statement::InterfaceDecl(_) => {
            // Interface declarations are handled during parsing/validation
            // No runtime action needed
        }
        Statement::ParallelDecl(decl) => {
            let parallel = ParallelValue {
                name: decl.name.clone(),
                agents: decl.agents.clone(),
                timeout_ms: decl.timeout.to_millis(),
            };
            env.define(&decl.name, Value::Parallel(parallel));
        }
        Statement::FnDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring function '{}'", decl.name),
            );
            let fn_value = Value::Function(FnValue {
                name: decl.name.clone(),
                params: decl.params.clone(),
                return_type: decl.return_type.clone(),
                body: decl.body.clone(),
            });
            env.define(&decl.name, fn_value);
        }
        Statement::LetStmt(stmt) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Evaluating let '{}'", stmt.name),
            );
            let value = evaluate_expr_with_env(&stmt.value, env, llm, tools, logger).await?;
            env.define(&stmt.name, value);
        }
        Statement::TopLevelCall(call) => {
            // Evaluate arguments
            let mut arg_values = Vec::new();
            for arg in &call.args {
                let val = evaluate_expr_with_env(arg, env, llm, tools, logger).await?;
                arg_values.push(val);
            }

            // Check if it's a built-in function
            if is_builtin(&call.name) {
                call_builtin(&call.name, &arg_values, &call.span)?;
                return Ok(());
            }

            // Check if it's a user-defined function
            if let Some(Value::Function(fn_val)) = env.get(&call.name) {
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
                        span: call.span.clone(),
                    });
                }

                // Create function scope and bind parameters
                let mut fn_env = env.clone();
                fn_env.push_scope();
                for (param, arg_val) in fn_val.params.iter().zip(arg_values.iter()) {
                    fn_env.define(&param.name, arg_val.clone());
                }

                // Evaluate function body with LLM support for agent calls
                evaluate_block_with_llm(&fn_val.body, &mut fn_env, tools, llm, logger).await?;
                return Ok(());
            }

            return Err(GentError::UnknownTool {
                name: call.name.clone(),
                span: call.span.clone(),
            });
        }
    }
    Ok(())
}

async fn evaluate_statement_with_output(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
    structs: &HashMap<String, Vec<StructField>>,
) -> GentResult<Option<String>> {
    match statement {
        Statement::Import(_) => {
            // Import statements are handled during a separate import resolution phase
            // No runtime action needed here
            Ok(None)
        }
        Statement::AgentDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring agent '{}'", decl.name),
            );
            evaluate_agent_decl(decl, env, structs)?;
            Ok(None)
        }
        Statement::ToolDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring tool '{}'", decl.name),
            );
            evaluate_tool_decl(decl, env, tools)?;
            Ok(None)
        }
        Statement::StructDecl(_) => {
            // Struct declarations are handled during parsing/validation
            // No runtime action needed
            Ok(None)
        }
        Statement::EnumDecl(_) => {
            // Enum declarations are handled during parsing/validation
            // No runtime action needed
            Ok(None)
        }
        Statement::InterfaceDecl(_) => {
            // Interface declarations are handled during parsing/validation
            // No runtime action needed
            Ok(None)
        }
        Statement::ParallelDecl(decl) => {
            let parallel = ParallelValue {
                name: decl.name.clone(),
                agents: decl.agents.clone(),
                timeout_ms: decl.timeout.to_millis(),
            };
            env.define(&decl.name, Value::Parallel(parallel));
            Ok(None)
        }
        Statement::FnDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring function '{}'", decl.name),
            );
            let fn_value = Value::Function(FnValue {
                name: decl.name.clone(),
                params: decl.params.clone(),
                return_type: decl.return_type.clone(),
                body: decl.body.clone(),
            });
            env.define(&decl.name, fn_value);
            Ok(None)
        }
        Statement::LetStmt(stmt) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Evaluating let '{}'", stmt.name),
            );
            let value = evaluate_expr_with_env(&stmt.value, env, llm, tools, logger).await?;
            // Capture string outputs (e.g., from agent invocations)
            let output = if let Value::String(s) = &value {
                Some(s.clone())
            } else {
                None
            };
            env.define(&stmt.name, value);
            Ok(output)
        }
        Statement::TopLevelCall(call) => {
            // Evaluate arguments
            let mut arg_values = Vec::new();
            for arg in &call.args {
                let val = evaluate_expr_with_env(arg, env, llm, tools, logger).await?;
                arg_values.push(val);
            }

            // Check if it's a built-in function
            if is_builtin(&call.name) {
                call_builtin(&call.name, &arg_values, &call.span)?;
                return Ok(None);
            }

            // Check if it's a user-defined function
            if let Some(Value::Function(fn_val)) = env.get(&call.name) {
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
                        span: call.span.clone(),
                    });
                }

                // Create function scope and bind parameters
                let mut fn_env = env.clone();
                fn_env.push_scope();
                for (param, arg_val) in fn_val.params.iter().zip(arg_values.iter()) {
                    fn_env.define(&param.name, arg_val.clone());
                }

                // Evaluate function body with LLM support for agent calls
                evaluate_block_with_llm(&fn_val.body, &mut fn_env, tools, llm, logger).await?;
                return Ok(None);
            }

            return Err(GentError::UnknownTool {
                name: call.name.clone(),
                span: call.span.clone(),
            });
        }
    }
}

fn evaluate_agent_decl(
    decl: &AgentDecl,
    env: &mut Environment,
    structs: &HashMap<String, Vec<StructField>>,
) -> GentResult<()> {
    let mut prompt: Option<String> = None;
    let mut user_prompt: Option<String> = None;
    let mut max_steps: Option<u32> = None;
    let mut model: Option<String> = None;
    let mut output_retries: Option<u32> = None;

    // Extract fields
    for field in &decl.fields {
        match field.name.as_str() {
            "prompt" | "systemPrompt" => {
                let value = evaluate_expr(&field.value, env)?;
                prompt = Some(match value {
                    Value::String(s) => s,
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "String".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
            "max_steps" | "maxSteps" => {
                let value = evaluate_expr(&field.value, env)?;
                max_steps = Some(match value {
                    Value::Number(n) if n >= 0.0 => n as u32,
                    Value::Number(_) => {
                        return Err(GentError::TypeError {
                            expected: "positive number".to_string(),
                            got: "negative number".to_string(),
                            span: field.span.clone(),
                        })
                    }
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "Number".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
            "model" => {
                let value = evaluate_expr(&field.value, env)?;
                model = Some(match value {
                    Value::String(s) => s,
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "String".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
            "output_retries" | "outputRetries" => {
                let value = evaluate_expr(&field.value, env)?;
                output_retries = Some(match value {
                    Value::Number(n) if n >= 0.0 => n as u32,
                    Value::Number(_) => {
                        return Err(GentError::TypeError {
                            expected: "positive number".to_string(),
                            got: "negative number".to_string(),
                            span: field.span.clone(),
                        })
                    }
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "Number".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
            "output_instructions" | "outputInstructions" => {
                // Ignore for now - will be implemented later
            }
            "retry_prompt" | "retryPrompt" => {
                // Ignore for now - will be implemented later
            }
            "userPrompt" => {
                let value = evaluate_expr(&field.value, env)?;
                user_prompt = Some(match value {
                    Value::String(s) => s,
                    _ => {
                        return Err(GentError::TypeError {
                            expected: "String".to_string(),
                            got: value.type_name().to_string(),
                            span: field.span.clone(),
                        })
                    }
                });
            }
            _ => {
                // Ignore unknown fields for forward compatibility
            }
        }
    }

    // Prompt is now optional (default to empty string)
    let prompt = prompt.unwrap_or_default();

    // Model is required
    let model = model.ok_or_else(|| GentError::MissingAgentField {
        agent: decl.name.clone(),
        field: "model".to_string(),
        span: decl.span.clone(),
    })?;

    // Build agent with all fields
    let mut agent = AgentValue::new(&decl.name, prompt)
        .with_tools(decl.tools.clone())
        .with_model(model);

    if let Some(steps) = max_steps {
        agent = agent.with_max_steps(steps);
    }

    // Set output_retries if present
    if let Some(retries) = output_retries {
        agent = agent.with_output_retries(retries);
    }

    // Set user_prompt if present
    if let Some(up) = user_prompt {
        agent = agent.with_user_prompt(up);
    }

    // Convert output type to schema if present
    if let Some(output_type) = &decl.output {
        let schema = OutputSchema::from_output_type(output_type, structs).map_err(|msg| {
            GentError::TypeError {
                expected: "valid output type".to_string(),
                got: msg,
                span: decl.span.clone(),
            }
        })?;
        agent = agent.with_output_schema(schema);
    }

    env.define(&decl.name, Value::Agent(agent));

    Ok(())
}

fn evaluate_tool_decl(
    decl: &ToolDecl,
    env: &mut Environment,
    tools: &mut ToolRegistry,
) -> GentResult<()> {
    let tool_value = UserToolValue {
        name: decl.name.clone(),
        params: decl.params.clone(),
        return_type: decl.return_type.clone(),
        body: decl.body.clone(),
    };

    // Store in environment for potential future use
    env.define(&decl.name, Value::Tool(tool_value.clone()));

    // Register in tool registry so LLM can call it
    let wrapper = UserToolWrapper::new(tool_value, Arc::new(env.clone()));
    tools.register(Box::new(wrapper));

    Ok(())
}

/// Evaluate an expression with environment access and async agent call support
fn evaluate_expr_with_env<'a>(
    expr: &'a Expression,
    env: &'a Environment,
    llm: &'a dyn LLMClient,
    tools: &'a ToolRegistry,
    logger: &'a dyn Logger,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = GentResult<Value>> + 'a>> {
    Box::pin(async move {
        match expr {
            Expression::String(parts, _span) => {
                // Evaluate each part and concatenate
                let mut result = String::new();
                for part in parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Expr(expr) => {
                            let value = evaluate_expr_with_env(expr, env, llm, tools, logger).await?;
                            result.push_str(&value.to_string());
                        }
                    }
                }
                Ok(Value::String(result))
            }
            Expression::Number(n, _) => Ok(Value::Number(*n)),
            Expression::Boolean(b, _) => Ok(Value::Boolean(*b)),
            Expression::Identifier(name, span) => {
                // Look up variable in environment
                env.get(name).cloned().ok_or_else(|| GentError::SyntaxError {
                    message: format!("Undefined variable: {}", name),
                    span: span.clone(),
                })
            }
            Expression::Null(_) => Ok(Value::String("null".to_string())),
            Expression::Call(callee, args, span) => {
                // Check if this is a method call (callee is Member expression)
                if let Expression::Member(obj, method, _) = callee.as_ref() {
                    // Evaluate the object
                    let obj_value = evaluate_expr_with_env(obj, env, llm, tools, logger).await?;

                    match obj_value {
                        Value::Agent(mut agent) => {
                            match method.as_str() {
                                "run" => {
                                    // Execute the agent
                                    let result = run_agent_with_tools(&agent, None, llm, tools, logger).await?;
                                    return Ok(Value::String(result));
                                }
                                "userPrompt" => {
                                    // Set user_prompt and return modified agent
                                    if args.is_empty() {
                                        return Err(GentError::SyntaxError {
                                            message: "userPrompt() requires an argument".to_string(),
                                            span: span.clone(),
                                        });
                                    }
                                    let arg = evaluate_expr_with_env(&args[0], env, llm, tools, logger).await?;
                                    let prompt = match arg {
                                        Value::String(s) => s,
                                        other => format!("{}", other),
                                    };
                                    agent.user_prompt = Some(prompt);
                                    return Ok(Value::Agent(agent));
                                }
                                "systemPrompt" => {
                                    // Set prompt and return modified agent
                                    if args.is_empty() {
                                        return Err(GentError::SyntaxError {
                                            message: "systemPrompt() requires an argument".to_string(),
                                            span: span.clone(),
                                        });
                                    }
                                    let arg = evaluate_expr_with_env(&args[0], env, llm, tools, logger).await?;
                                    let prompt = match arg {
                                        Value::String(s) => s,
                                        other => format!("{}", other),
                                    };
                                    agent.system_prompt = prompt;
                                    return Ok(Value::Agent(agent));
                                }
                                _ => {
                                    return Err(GentError::SyntaxError {
                                        message: format!("Unknown agent method: {}", method),
                                        span: span.clone(),
                                    });
                                }
                            }
                        }
                        Value::String(s) => {
                            // String method call - evaluate arguments and dispatch
                            let mut arg_values = Vec::new();
                            for arg in args {
                                let val = evaluate_expr_with_env(arg, env, llm, tools, logger).await?;
                                arg_values.push(val);
                            }
                            return call_string_method(&s, method, &arg_values);
                        }
                        Value::Array(mut arr) => {
                            // Array method call - evaluate arguments and dispatch
                            let mut arg_values = Vec::new();
                            for arg in args {
                                let val = evaluate_expr_with_env(arg, env, llm, tools, logger).await?;
                                arg_values.push(val);
                            }

                            // Check if this is a callback method (map, filter, reduce, find)
                            if crate::interpreter::array_methods::is_callback_method(method) {
                                let callback = arg_values.first().ok_or_else(|| GentError::TypeError {
                                    expected: "callback function or lambda".to_string(),
                                    got: "missing argument".to_string(),
                                    span: span.clone(),
                                })?;
                                let extra_args = if arg_values.len() > 1 { &arg_values[1..] } else { &[] };
                                return crate::interpreter::array_methods::call_array_method_with_callback(
                                    &arr,
                                    method,
                                    callback,
                                    extra_args,
                                    env,
                                    tools,
                                ).await;
                            }

                            // Otherwise, call non-callback array method
                            return crate::interpreter::array_methods::call_array_method(&mut arr, method, &arg_values);
                        }
                        Value::Parallel(parallel) => {
                            // Parallel block method call
                            if method == "run" {
                                if !args.is_empty() {
                                    return Err(GentError::TypeError {
                                        expected: "no arguments for .run()".to_string(),
                                        got: format!("{} arguments", args.len()),
                                        span: span.clone(),
                                    });
                                }
                                return run_parallel(&parallel, env, llm, tools, logger).await;
                            } else {
                                return Err(GentError::SyntaxError {
                                    message: format!("Unknown parallel method: {}", method),
                                    span: span.clone(),
                                });
                            }
                        }
                        _ => {
                            // Not an agent, string, or array - method calls not yet supported
                            return Err(GentError::SyntaxError {
                                message: format!("Method calls on {} not yet implemented", obj_value.type_name()),
                                span: span.clone(),
                            });
                        }
                    }
                }

                // Check if callee is an identifier
                if let Expression::Identifier(name, _) = callee.as_ref() {
                    // Check if it's an agent (direct call)
                    if let Some(Value::Agent(agent)) = env.get(name) {
                        // This is an agent call - execute it
                        let input = if !args.is_empty() {
                            let arg_value = evaluate_expr_with_env(&args[0], env, llm, tools, logger).await?;
                            match arg_value {
                                Value::String(s) => Some(s),
                                other => Some(format!("{}", other)),
                            }
                        } else {
                            None
                        };
                        let output = run_agent_with_tools(agent, input, llm, tools, logger).await?;
                        return Ok(Value::String(output));
                    }

                    // Check if it's a function call
                    if let Some(Value::Function(fn_val)) = env.get(name) {
                        // Clone the function value since we need to modify the environment
                        let fn_val = fn_val.clone();

                        // Evaluate arguments
                        let mut arg_values = Vec::new();
                        for arg in args {
                            let val = evaluate_expr_with_env(arg, env, llm, tools, logger).await?;
                            arg_values.push(val);
                        }

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

                        // Evaluate the function body with LLM support for agent calls
                        let result = crate::interpreter::evaluate_block_with_llm(&fn_val.body, &mut fn_env, tools, llm, logger).await?;
                        return Ok(result);
                    }
                }
                // Not a known callable type
                Err(GentError::SyntaxError {
                    message: "Unknown function or agent".to_string(),
                    span: span.clone(),
                })
            }
            // Binary operations - need async evaluation for operands that might contain calls
            Expression::Binary(op, left, right, span) => {
                let left_val = evaluate_expr_with_env(left, env, llm, tools, logger).await?;
                let right_val = evaluate_expr_with_env(right, env, llm, tools, logger).await?;
                crate::interpreter::expr_eval::evaluate_binary_op_public(op, left_val, right_val, span)
            }

            // Unary operations - need async evaluation for operand
            Expression::Unary(op, operand, span) => {
                let val = evaluate_expr_with_env(operand, env, llm, tools, logger).await?;
                crate::interpreter::expr_eval::evaluate_unary_op_public(op, val, span)
            }

            // Array literals - need async evaluation for elements
            Expression::Array(elements, _) => {
                let mut values = Vec::new();
                for elem in elements {
                    values.push(evaluate_expr_with_env(elem, env, llm, tools, logger).await?);
                }
                Ok(Value::Array(values))
            }

            // Object literals - need async evaluation for values
            Expression::Object(fields, _) => {
                let mut map = std::collections::HashMap::new();
                for (key, value_expr) in fields {
                    let value = evaluate_expr_with_env(value_expr, env, llm, tools, logger).await?;
                    map.insert(key.clone(), value);
                }
                Ok(Value::Object(map))
            }

            // Member access - need async evaluation for object expression
            Expression::Member(object_expr, property, span) => {
                // Check if this is an enum construction: EnumName.Variant
                if let Expression::Identifier(name, _) = object_expr.as_ref() {
                    if let Some(enum_def) = env.get_enum(name) {
                        use crate::interpreter::types::{EnumConstructor, EnumValue};
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

                let object = evaluate_expr_with_env(object_expr, env, llm, tools, logger).await?;
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

            // Index access - need async evaluation for target and index
            Expression::Index(target_expr, index_expr, span) => {
                let target = evaluate_expr_with_env(target_expr, env, llm, tools, logger).await?;
                let index = evaluate_expr_with_env(index_expr, env, llm, tools, logger).await?;

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

            // For remaining simple expression types, delegate to expr_eval
            other => evaluate_expr(other, env),
        }
    })
}

/// Execute a parallel block - runs all agents concurrently
async fn run_parallel(
    parallel: &ParallelValue,
    env: &Environment,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<Value> {
    use futures::future::try_join_all;

    let timeout = std::time::Duration::from_millis(parallel.timeout_ms);

    // Evaluate each agent expression and collect futures
    let mut agent_values = Vec::new();
    for expr in &parallel.agents {
        // Evaluate the expression to get the configured agent
        let agent_val = evaluate_expr_with_env(expr, env, llm, tools, logger).await?;

        let agent = agent_val.as_agent().ok_or_else(|| GentError::TypeError {
            expected: "agent".to_string(),
            got: agent_val.type_name(),
            span: expr.span().clone(),
        })?;

        agent_values.push(agent.clone());
    }

    // Create futures for all agents
    let futures: Vec<_> = agent_values
        .iter()
        .map(|agent| run_agent_with_tools(agent, None, llm, tools, logger))
        .collect();

    // Wait for all with timeout
    let results = tokio::time::timeout(timeout, try_join_all(futures))
        .await
        .map_err(|_| GentError::ParallelTimeout {
            name: parallel.name.clone(),
            timeout_ms: parallel.timeout_ms,
        })??;

    // Return as array of strings
    Ok(Value::Array(
        results.into_iter().map(Value::String).collect(),
    ))
}
