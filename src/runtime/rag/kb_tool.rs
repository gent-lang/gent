//! Tool wrapper for KnowledgeBase

use super::KnowledgeBase;
use crate::runtime::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tool wrapper that allows KnowledgeBase to be used by agents
pub struct KnowledgeBaseTool {
    kb: Arc<RwLock<KnowledgeBase>>,
    name: String,
}

impl KnowledgeBaseTool {
    pub fn new(kb: Arc<RwLock<KnowledgeBase>>, name: String) -> Self {
        Self { kb, name }
    }
}

#[async_trait]
impl Tool for KnowledgeBaseTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Search the knowledge base for relevant documents"
    }

    fn parameters_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to find relevant documents"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, String> {
        let query = args
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or("Missing required 'query' parameter")?;

        let limit = args
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(5) as usize;

        let kb = self.kb.read().await;
        let results = kb.search(query, limit).await?;

        // Format results for LLM consumption
        let mut output = String::new();
        for result in results {
            output.push_str(&format!(
                "[source: {}, lines {}-{}, score: {:.2}]\n{}\n\n",
                result.metadata.source,
                result.metadata.start_line,
                result.metadata.end_line,
                result.score,
                result.metadata.content
            ));
        }

        if output.is_empty() {
            output = "No matching documents found.".to_string();
        }

        Ok(output)
    }
}
