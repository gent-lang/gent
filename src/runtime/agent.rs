//! Agent execution for GENT

use crate::errors::{GentError, GentResult};
use crate::interpreter::{AgentValue, OutputSchema};
use crate::logging::{LogLevel, Logger, NullLogger};
use crate::runtime::validation::validate_output;
use crate::runtime::{LLMClient, LLMResponse, Message, ToolDefinition, ToolRegistry, ToolResult};

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
    let json_mode = agent.output_schema.is_some();

    logger.log(
        LogLevel::Debug,
        "agent",
        &format!(
            "Agent '{}' - knowledge_config: {}, user_prompt: {:?}",
            agent.name,
            agent.knowledge_config.is_some(),
            agent.user_prompt
        ),
    );
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

    // Build messages based on which prompts are present
    let mut messages = Vec::new();

    // Determine user query for RAG (if applicable)
    let user_query = agent
        .user_prompt
        .as_ref()
        .or(input.as_ref())
        .cloned();

    // Perform RAG search if knowledge config is present
    let rag_context = if let Some(ref knowledge_config) = agent.knowledge_config {
        if let Some(ref query) = user_query {
            logger.log(
                LogLevel::Debug,
                "agent",
                &format!("Searching knowledge base with query: '{}'", query),
            );

            // Search the knowledge base
            let kb = knowledge_config.source.read().await;
            match kb.search(query, knowledge_config.chunk_limit).await {
                Ok(results) => {
                    // Filter by score threshold
                    let filtered: Vec<_> = results
                        .into_iter()
                        .filter(|r| r.score >= knowledge_config.score_threshold as f32)
                        .collect();

                    if filtered.is_empty() {
                        logger.log(
                            LogLevel::Debug,
                            "agent",
                            "No relevant context found above threshold",
                        );
                        None
                    } else {
                        logger.log(
                            LogLevel::Debug,
                            "agent",
                            &format!("Found {} relevant chunks for context", filtered.len()),
                        );

                        // Format results for injection
                        let mut context = String::from("\n---\nRelevant context from knowledge base:\n\n");
                        for (i, result) in filtered.iter().enumerate() {
                            context.push_str(&format!(
                                "[{}] (source: {}, lines {}-{}, score: {:.2})\n{}\n\n",
                                i + 1,
                                result.metadata.source,
                                result.metadata.start_line,
                                result.metadata.end_line,
                                result.score,
                                result.metadata.content
                            ));
                        }
                        context.push_str("---");
                        Some(context)
                    }
                }
                Err(e) => {
                    logger.log(
                        LogLevel::Warn,
                        "agent",
                        &format!("Knowledge base search failed: {}", e),
                    );
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Add system message if prompt is not empty
    if !agent.system_prompt.is_empty() {
        let mut system_prompt = if let Some(schema) = &agent.output_schema {
            logger.log(
                LogLevel::Debug,
                "agent",
                "Agent has output schema, enabling JSON mode",
            );
            let default_instructions = "You must respond with JSON matching this schema:";
            let instructions = agent
                .output_instructions
                .as_deref()
                .unwrap_or(default_instructions);
            format!(
                "{}\n\n{}\n{}",
                agent.system_prompt,
                instructions,
                serde_json::to_string_pretty(&schema.to_json_schema())
                    .unwrap_or_else(|_| "<schema>".to_string())
            )
        } else {
            agent.system_prompt.clone()
        };

        // Append RAG context if available
        if let Some(context) = rag_context {
            system_prompt.push_str(&context);
        }

        messages.push(Message::system(&system_prompt));
    }

    // Add user message from agent's user_prompt or from input parameter
    if let Some(user_prompt) = &agent.user_prompt {
        messages.push(Message::user(user_prompt.clone()));
    } else if let Some(user_input) = input {
        messages.push(Message::user(user_input));
    }

    // If no messages at all, return empty result
    if messages.is_empty() {
        logger.log(
            LogLevel::Debug,
            "agent",
            "No prompts provided, returning empty result",
        );
        return Ok(String::new());
    }

    for step in 0..max_steps {
        logger.log(
            LogLevel::Debug,
            "agent",
            &format!("Step {}/{}", step + 1, max_steps),
        );
        let response = llm
            .chat(messages.clone(), tool_defs.clone(), model, json_mode)
            .await?;

        // If no tool calls, validate and return the response content
        if response.tool_calls.is_empty() {
            logger.log(
                LogLevel::Debug,
                "agent",
                "No tool calls, returning response",
            );
            let content = response.content.unwrap_or_default();

            // Validate output if schema exists
            if let Some(schema) = &agent.output_schema {
                return validate_and_retry_output(
                    &content, schema, agent, &messages, llm, &tool_defs, model, logger,
                )
                .await;
            }

            return Ok(content);
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
    // Build messages based on which prompts are present
    let mut messages = Vec::new();

    // Add system message if prompt is not empty
    if !agent.system_prompt.is_empty() {
        messages.push(Message::system(&agent.system_prompt));
    }

    // Add user message from agent's user_prompt or from input parameter
    if let Some(user_prompt) = &agent.user_prompt {
        messages.push(Message::user(user_prompt.clone()));
    } else if let Some(user_input) = input {
        messages.push(Message::user(user_input));
    }

    // If no messages at all, return empty response
    if messages.is_empty() {
        return Ok(LLMResponse {
            content: Some(String::new()),
            tool_calls: vec![],
        });
    }

    let model = agent.model.as_deref();
    llm.chat(messages, vec![], model, false).await
}

/// Validate output and retry on failure
#[allow(clippy::too_many_arguments)]
async fn validate_and_retry_output(
    content: &str,
    schema: &OutputSchema,
    agent: &AgentValue,
    messages: &[Message],
    llm: &dyn LLMClient,
    tools: &[ToolDefinition],
    model: Option<&str>,
    logger: &dyn Logger,
) -> GentResult<String> {
    let mut last_content = content.to_string();
    let mut retry_messages = messages.to_vec();

    for retry in 0..=agent.output_retries {
        // Try to parse as JSON
        let json: serde_json::Value = match serde_json::from_str(&last_content) {
            Ok(j) => j,
            Err(e) => {
                if retry >= agent.output_retries {
                    return Err(GentError::OutputValidationError {
                        message: format!("Invalid JSON: {}", e),
                        expected: serde_json::to_string(&schema.to_json_schema())
                            .unwrap_or_else(|_| "<schema>".to_string()),
                        got: last_content,
                    });
                }
                logger.log(
                    LogLevel::Debug,
                    "agent",
                    &format!("Retry {}: invalid JSON", retry + 1),
                );
                let default_retry = "Please respond with valid JSON.";
                let retry_msg = agent.retry_prompt.as_deref().unwrap_or(default_retry);
                retry_messages.push(Message::assistant(&last_content));
                retry_messages.push(Message::user(retry_msg));
                let response = llm
                    .chat(retry_messages.clone(), tools.to_vec(), model, true)
                    .await?;
                last_content = response.content.unwrap_or_default();
                continue;
            }
        };

        // Validate against schema
        match validate_output(&json, schema) {
            Ok(()) => {
                logger.log(LogLevel::Debug, "agent", "Output validation successful");
                return Ok(last_content);
            }
            Err(e) => {
                if retry >= agent.output_retries {
                    return Err(GentError::OutputValidationError {
                        message: e,
                        expected: serde_json::to_string(&schema.to_json_schema())
                            .unwrap_or_else(|_| "<schema>".to_string()),
                        got: last_content,
                    });
                }
                logger.log(
                    LogLevel::Debug,
                    "agent",
                    &format!("Retry {}: {}", retry + 1, e),
                );
                let default_retry = format!(
                    "Invalid response: {}. Please respond with JSON matching the schema.",
                    e
                );
                let retry_msg = agent
                    .retry_prompt
                    .clone()
                    .unwrap_or(default_retry);
                retry_messages.push(Message::assistant(&last_content));
                retry_messages.push(Message::user(retry_msg));
                let response = llm
                    .chat(retry_messages.clone(), tools.to_vec(), model, true)
                    .await?;
                last_content = response.content.unwrap_or_default();
            }
        }
    }

    Ok(last_content)
}
