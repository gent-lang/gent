//! Agent execution for GENT

use crate::errors::GentResult;
use crate::interpreter::AgentValue;
use crate::runtime::{LLMClient, LLMResponse, Message};

/// Run an agent with the given input
///
/// # Arguments
/// * `agent` - The agent to run
/// * `input` - Optional user input (defaults to "Hello!")
/// * `llm` - The LLM client to use
///
/// # Returns
/// The LLM response content as a string
pub async fn run_agent(
    agent: &AgentValue,
    input: Option<String>,
    llm: &dyn LLMClient,
) -> GentResult<String> {
    // Build messages
    let mut messages = Vec::new();

    // Add system message with agent prompt
    messages.push(Message::system(&agent.prompt));

    // Add user message
    let user_input = input.unwrap_or_else(|| "Hello!".to_string());
    messages.push(Message::user(user_input));

    // Call LLM
    let response = llm.chat(messages, vec![]).await?;

    Ok(response.content.unwrap_or_default())
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

    llm.chat(messages, vec![]).await
}
