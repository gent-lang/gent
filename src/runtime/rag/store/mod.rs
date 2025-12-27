//! Vector storage backends for RAG

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod local;
pub use local::LocalVectorStore;

/// Metadata associated with a stored embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub source: String,
    pub chunk_index: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
}

/// A search result with score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub metadata: Metadata,
}

/// Trait for vector storage backends
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add a vector with metadata
    async fn add(&mut self, id: &str, embedding: Vec<f32>, metadata: Metadata) -> Result<(), String>;

    /// Search for similar vectors
    async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, String>;

    /// Delete a vector by ID
    async fn delete(&mut self, id: &str) -> Result<(), String>;

    /// Clear all vectors
    async fn clear(&mut self) -> Result<(), String>;

    /// Get number of stored vectors
    fn len(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
