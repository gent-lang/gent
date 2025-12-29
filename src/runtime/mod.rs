//! Runtime components for GENT

pub mod agent;
mod client_factory;
pub mod llm;
pub mod providers;
pub mod rag;
pub mod tools;
pub mod validation;

pub use agent::{run_agent, run_agent_full, run_agent_with_tools};
pub use client_factory::create_llm_client;
pub use llm::{
    LLMClient, LLMResponse, Message, MockLLMClient, Role, ToolCall, ToolDefinition, ToolResult,
};
pub use providers::{detect_provider, AnthropicClient, OpenAIClient, Provider};
pub use tools::{Tool, ToolRegistry, UserToolWrapper};
pub use validation::validate_output;
