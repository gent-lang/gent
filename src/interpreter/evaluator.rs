//! Program evaluation for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::{AgentValue, Environment, Value};
use crate::parser::{AgentDecl, Expression, Program, Statement};
use crate::runtime::{run_agent_with_tools, LLMClient, ToolRegistry};

/// Evaluate a GENT program
///
/// # Arguments
/// * `program` - The parsed AST
/// * `llm` - The LLM client to use for agent execution
/// * `tools` - The tool registry for agent execution
///
/// # Returns
/// Ok(()) on success, Err on failure
pub async fn evaluate(
    program: &Program,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
) -> GentResult<()> {
    let mut env = Environment::new();

    for statement in &program.statements {
        evaluate_statement(statement, &mut env, llm, tools).await?;
    }

    Ok(())
}

/// Evaluate a GENT program and capture output
pub async fn evaluate_with_output(
    program: &Program,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
) -> GentResult<Vec<String>> {
    let mut env = Environment::new();
    let mut outputs = Vec::new();

    for statement in &program.statements {
        if let Some(output) =
            evaluate_statement_with_output(statement, &mut env, llm, tools).await?
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
    tools: &ToolRegistry,
) -> GentResult<()> {
    match statement {
        Statement::AgentDecl(decl) => {
            evaluate_agent_decl(decl, env)?;
        }
        Statement::RunStmt(run) => {
            let output = evaluate_run_stmt(run, env, llm, tools).await?;
            println!("{}", output);
        }
        Statement::ToolDecl(_) => {
            // TODO: Implement tool declaration evaluation
        }
    }
    Ok(())
}

async fn evaluate_statement_with_output(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
) -> GentResult<Option<String>> {
    match statement {
        Statement::AgentDecl(decl) => {
            evaluate_agent_decl(decl, env)?;
            Ok(None)
        }
        Statement::RunStmt(run) => {
            let output = evaluate_run_stmt(run, env, llm, tools).await?;
            Ok(Some(output))
        }
        Statement::ToolDecl(_) => {
            // TODO: Implement tool declaration evaluation
            Ok(None)
        }
    }
}

fn evaluate_agent_decl(decl: &AgentDecl, env: &mut Environment) -> GentResult<()> {
    let mut prompt: Option<String> = None;
    let mut max_steps: Option<u32> = None;
    let mut model: Option<String> = None;

    // Extract fields
    for field in &decl.fields {
        match field.name.as_str() {
            "prompt" => {
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
            "max_steps" => {
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

    // Build agent with all fields
    let mut agent = AgentValue::new(&decl.name, prompt).with_tools(decl.tools.clone());

    if let Some(steps) = max_steps {
        agent = agent.with_max_steps(steps);
    }

    if let Some(m) = model {
        agent = agent.with_model(m);
    }

    env.define(&decl.name, Value::Agent(agent));

    Ok(())
}

async fn evaluate_run_stmt(
    run: &crate::parser::RunStmt,
    env: &Environment,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
) -> GentResult<String> {
    // Look up the agent
    let agent_value = env
        .get(&run.agent_name)
        .ok_or_else(|| GentError::UndefinedAgent {
            name: run.agent_name.clone(),
            span: run.span.clone(),
        })?;

    let agent = match agent_value {
        Value::Agent(a) => a,
        other => {
            return Err(GentError::TypeError {
                expected: "Agent".to_string(),
                got: other.type_name().to_string(),
                span: run.span.clone(),
            })
        }
    };

    // Evaluate input expression if present
    let input = if let Some(expr) = &run.input {
        let value = evaluate_expression(expr)?;
        match value {
            Value::String(s) => Some(s),
            other => Some(format!("{}", other)),
        }
    } else {
        None
    };

    // Run the agent with tools
    run_agent_with_tools(agent, input, llm, tools).await
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
