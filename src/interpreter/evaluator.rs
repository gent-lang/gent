//! Program evaluation for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::{AgentValue, Environment, Value};
use crate::parser::{AgentDecl, Expression, Program, Statement};
use crate::runtime::{run_agent, LLMClient};

/// Evaluate a GENT program
///
/// # Arguments
/// * `program` - The parsed AST
/// * `llm` - The LLM client to use for agent execution
///
/// # Returns
/// Ok(()) on success, Err on failure
pub fn evaluate(program: &Program, llm: &dyn LLMClient) -> GentResult<()> {
    let mut env = Environment::new();

    for statement in &program.statements {
        evaluate_statement(statement, &mut env, llm)?;
    }

    Ok(())
}

/// Evaluate a GENT program and capture output
pub fn evaluate_with_output(program: &Program, llm: &dyn LLMClient) -> GentResult<Vec<String>> {
    let mut env = Environment::new();
    let mut outputs = Vec::new();

    for statement in &program.statements {
        if let Some(output) = evaluate_statement_with_output(statement, &mut env, llm)? {
            outputs.push(output);
        }
    }

    Ok(outputs)
}

fn evaluate_statement(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
) -> GentResult<()> {
    match statement {
        Statement::AgentDecl(decl) => {
            evaluate_agent_decl(decl, env)?;
        }
        Statement::RunStmt(run) => {
            let output = evaluate_run_stmt(run, env, llm)?;
            println!("{}", output);
        }
    }
    Ok(())
}

fn evaluate_statement_with_output(
    statement: &Statement,
    env: &mut Environment,
    llm: &dyn LLMClient,
) -> GentResult<Option<String>> {
    match statement {
        Statement::AgentDecl(decl) => {
            evaluate_agent_decl(decl, env)?;
            Ok(None)
        }
        Statement::RunStmt(run) => {
            let output = evaluate_run_stmt(run, env, llm)?;
            Ok(Some(output))
        }
    }
}

fn evaluate_agent_decl(decl: &AgentDecl, env: &mut Environment) -> GentResult<()> {
    // Extract prompt from fields
    let prompt = decl
        .fields
        .iter()
        .find(|f| f.name == "prompt")
        .map(|f| evaluate_expression(&f.value))
        .transpose()?
        .and_then(|v| match v {
            Value::String(s) => Some(s),
            _ => None,
        })
        .ok_or_else(|| GentError::MissingAgentField {
            agent: decl.name.clone(),
            field: "prompt".to_string(),
            span: decl.span.clone(),
        })?;

    let agent = AgentValue::new(&decl.name, prompt);
    env.define(&decl.name, Value::Agent(agent));

    Ok(())
}

fn evaluate_run_stmt(
    run: &crate::parser::RunStmt,
    env: &Environment,
    llm: &dyn LLMClient,
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

    // Run the agent
    run_agent(agent, input, llm)
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
    }
}
