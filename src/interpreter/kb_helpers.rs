//! Helper functions for KnowledgeBase evaluation

use crate::errors::GentResult;
use crate::interpreter::types::Value;
use crate::runtime::rag::IndexOptions;

/// Parse IndexOptions from a GENT Value (typically an object)
pub fn parse_index_options(value: &Value) -> GentResult<IndexOptions> {
    let mut options = IndexOptions::default();

    if let Value::Object(map) = value {
        // Parse extensions
        if let Some(Value::Array(exts)) = map.get("extensions") {
            options.extensions = exts
                .iter()
                .filter_map(|v| {
                    if let Value::String(s) = v {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
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
