//! Agent execution for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::AgentValue;
use crate::logging::{LogLevel, Logger, NullLogger};
use crate::runtime::{LLMClient, LLMResponse, Message, ToolRegistry, ToolResult};

const DEFAULT_MAX_STEPS: u32 = 10;

/// Run an agent with the given input (simple, no tools)
pub async fn run_agent(
    agent: &AgentValue,
    input: Option<String>,
    llm: &dyn LLMClient,
) -> GentResult<String> {
    let registry = ToolRegistry::new();
    let logger = NullLogger;
    run_agent_with_tools(agent, input, llm, &registry, &logger).await
}

/// Run an agent with tools
pub async fn run_agent_with_tools(
    agent: &AgentValue,
    input: Option<String>,
    llm: &dyn LLMClient,
    tools: &ToolRegistry,
    logger: &dyn Logger,
) -> GentResult<String> {
    let max_steps = agent.max_steps.unwrap_or(DEFAULT_MAX_STEPS);
    let tool_defs = tools.definitions_for(&agent.tools);
    let model = agent.model.as_deref();

    logger.log(
        LogLevel::Debug,
        "agent",
        &format!("Agent '{}' requested tools: {:?}", agent.name, agent.tools),
    );
    logger.log(
        LogLevel::Debug,
        "agent",
        &format!("Tool definitions provided to LLM: {}", tool_defs.len()),
    );
    for def in &tool_defs {
        logger.log(
            LogLevel::Trace,
            "agent",
            &format!("  - {} : {}", def.name, def.description),
        );
    }

    let mut messages = vec![
        Message::system(&agent.prompt),
        Message::user(input.unwrap_or_else(|| "Hello!".to_string())),
    ];

    for step in 0..max_steps {
        logger.log(
            LogLevel::Debug,
            "agent",
            &format!("Step {}/{}", step + 1, max_steps),
        );
        let response = llm.chat(messages.clone(), tool_defs.clone(), model, false).await?;

        // If no tool calls, return the response content
        if response.tool_calls.is_empty() {
            logger.log(
                LogLevel::Debug,
                "agent",
                "No tool calls, returning response",
            );
            return Ok(response.content.unwrap_or_default());
        }

        logger.log(
            LogLevel::Debug,
            "agent",
            &format!("LLM made {} tool call(s)", response.tool_calls.len()),
        );
        for call in &response.tool_calls {
            logger.log(
                LogLevel::Trace,
                "agent",
                &format!("  - {}({})", call.name, call.arguments),
            );
        }

        // Add assistant message with tool calls
        messages.push(Message::assistant_with_tool_calls(
            response.tool_calls.clone(),
        ));

        // Execute each tool call
        for call in &response.tool_calls {
            let result = match tools.get(&call.name) {
                Some(tool) => match tool.execute(call.arguments.clone()).await {
                    Ok(output) => {
                        logger.log(
                            LogLevel::Debug,
                            "agent",
                            &format!("Tool '{}' returned: {}", call.name, output),
                        );
                        ToolResult {
                            call_id: call.id.clone(),
                            content: output,
                            is_error: false,
                        }
                    }
                    Err(error) => {
                        logger.log(
                            LogLevel::Warn,
                            "agent",
                            &format!("Tool '{}' error: {}", call.name, error),
                        );
                        ToolResult {
                            call_id: call.id.clone(),
                            content: error,
                            is_error: true,
                        }
                    }
                },
                None => {
                    logger.log(
                        LogLevel::Warn,
                        "agent",
                        &format!("Unknown tool: {}", call.name),
                    );
                    ToolResult {
                        call_id: call.id.clone(),
                        content: format!("Unknown tool: {}", call.name),
                        is_error: true,
                    }
                }
            };

            messages.push(Message::tool_result(result));
        }
    }

    Err(GentError::MaxStepsExceeded { limit: max_steps })
}

/// Run an agent and return the full LLM response
pub async fn run_agent_full(
    agent: &AgentValue,
    input: Option<String>,
    llm: &dyn LLMClient,
) -> GentResult<LLMResponse> {
    let mut messages = Vec::new();
    messages.push(Message::system(&agent.prompt));

    let user_input = input.unwrap_or_else(|| "Hello!".to_string());
    messages.push(Message::user(user_input));

    let model = agent.model.as_deref();
    llm.chat(messages, vec![], model, false).await
}
