//! Program evaluation for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::{AgentValue, Environment, OutputSchema, UserToolValue, Value};
use crate::logging::{LogLevel, Logger, NullLogger};
use crate::parser::{AgentDecl, Expression, Program, Statement, StructField, ToolDecl};
use crate::runtime::{run_agent_with_tools, LLMClient, ToolRegistry, UserToolWrapper};
use std::collections::HashMap;
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

async fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
    tools: &mut ToolRegistry,
    logger: &dyn Logger,
    structs: &HashMap<String, Vec<StructField>>,
) -> GentResult<()> {
    match statement {
        Statement::AgentDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring agent '{}'", decl.name),
            );
            evaluate_agent_decl(decl, env, structs)?;
        }
        Statement::AgentCall(call) => {
            logger.log(
                LogLevel::Info,
                "eval",
                &format!("Calling agent '{}'", call.agent_name),
            );
            let output = evaluate_agent_call(call, env, llm, tools, logger).await?;
            println!("{}", output);
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
        Statement::LetStmt(stmt) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Evaluating let '{}'", stmt.name),
            );
            let value = evaluate_expr_with_env(&stmt.value, env, llm, tools, logger).await?;
            env.define(&stmt.name, value);
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
        Statement::AgentDecl(decl) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Declaring agent '{}'", decl.name),
            );
            evaluate_agent_decl(decl, env, structs)?;
            Ok(None)
        }
        Statement::AgentCall(call) => {
            logger.log(
                LogLevel::Info,
                "eval",
                &format!("Calling agent '{}'", call.agent_name),
            );
            let output = evaluate_agent_call(call, env, llm, tools, logger).await?;
            Ok(Some(output))
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
        Statement::LetStmt(stmt) => {
            logger.log(
                LogLevel::Debug,
                "eval",
                &format!("Evaluating let '{}'", stmt.name),
            );
            let value = evaluate_expr_with_env(&stmt.value, env, llm, tools, logger).await?;
            env.define(&stmt.name, value);
            Ok(None)
        }
    }
}

fn evaluate_agent_decl(
    decl: &AgentDecl,
    env: &mut Environment,
    structs: &HashMap<String, Vec<StructField>>,
) -> GentResult<()> {
    let mut prompt: Option<String> = None;
    let mut max_steps: Option<u32> = None;
    let mut model: Option<String> = None;
    let mut output_retries: Option<u32> = None;

    // Extract fields
    for field in &decl.fields {
        match field.name.as_str() {
            "prompt" | "systemPrompt" => {
                let value = evaluate_expression(&field.value)?;
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
                let value = evaluate_expression(&field.value)?;
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
                let value = evaluate_expression(&field.value)?;
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
                let value = evaluate_expression(&field.value)?;
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
            _ => {
                // Ignore unknown fields for forward compatibility
            }
        }
    }

    // Prompt is required
    let prompt = prompt.ok_or_else(|| GentError::MissingAgentField {
        agent: decl.name.clone(),
        field: "prompt".to_string(),
        span: decl.span.clone(),
    })?;

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

async fn evaluate_agent_call(
    call: &crate::parser::AgentCall,
    env: &Environment,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<String> {
    // Look up the agent
    let agent_value = env
        .get(&call.agent_name)
        .ok_or_else(|| GentError::UndefinedAgent {
            name: call.agent_name.clone(),
            span: call.span.clone(),
        })?;

    let agent = match agent_value {
        Value::Agent(a) => a,
        other => {
            return Err(GentError::TypeError {
                expected: "Agent".to_string(),
                got: other.type_name().to_string(),
                span: call.span.clone(),
            })
        }
    };

    // Evaluate input expression if present (using env for variable lookups)
    let input = if let Some(expr) = &call.input {
        let value = evaluate_expr_with_env(expr, env, llm, tools, logger).await?;
        match value {
            Value::String(s) => Some(s),
            other => Some(format!("{}", other)),
        }
    } else {
        None
    };

    // Run the agent with tools
    run_agent_with_tools(agent, input, llm, tools, logger).await
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
            Expression::String(s, _) => Ok(Value::String(s.clone())),
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
                // Check if callee is an agent
                if let Expression::Identifier(name, _) = callee.as_ref() {
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
                }
                // Not an agent call - not yet supported
                Err(GentError::SyntaxError {
                    message: "Function calls not yet implemented".to_string(),
                    span: span.clone(),
                })
            }
            // For other expression types, delegate to the simple evaluator
            other => evaluate_expression(other),
        }
    })
}

fn evaluate_expression(expr: &Expression) -> GentResult<Value> {
    match expr {
        Expression::String(s, _) => Ok(Value::String(s.clone())),
        Expression::Number(n, _) => Ok(Value::Number(*n)),
        Expression::Boolean(b, _) => Ok(Value::Boolean(*b)),
        Expression::Identifier(name, span) => {
            // For now, identifiers in expressions are treated as undefined
            // In the future, this could look up variables
            Err(GentError::SyntaxError {
                message: format!("Undefined variable: {}", name),
                span: span.clone(),
            })
        }
        Expression::Null(_) => {
            // TODO: Implement null evaluation
            Ok(Value::String("null".to_string()))
        }
        Expression::Array(_, span) => {
            // TODO: Implement array evaluation
            Err(GentError::SyntaxError {
                message: "Array expressions not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Object(_, span) => {
            // TODO: Implement object evaluation
            Err(GentError::SyntaxError {
                message: "Object expressions not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Binary(_, _, _, span) => {
            // TODO: Implement binary operation evaluation
            Err(GentError::SyntaxError {
                message: "Binary operations not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Unary(_, _, span) => {
            // TODO: Implement unary operation evaluation
            Err(GentError::SyntaxError {
                message: "Unary operations not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Call(_, _, span) => {
            // TODO: Implement function call evaluation
            Err(GentError::SyntaxError {
                message: "Function calls not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Member(_, _, span) => {
            // TODO: Implement member access evaluation
            Err(GentError::SyntaxError {
                message: "Member access not yet implemented".to_string(),
                span: span.clone(),
            })
        }
        Expression::Index(_, _, span) => {
            // TODO: Implement index access evaluation
            Err(GentError::SyntaxError {
                message: "Index access not yet implemented".to_string(),
                span: span.clone(),
            })
        }
    }
}
