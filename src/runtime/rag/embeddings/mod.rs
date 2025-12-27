//! Embedding providers for RAG

use async_trait::async_trait;

// Note: openai module will be added in next task
// mod openai;
// pub use openai::OpenAIEmbeddings;

/// Trait for embedding text into vectors
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Embed a single text into a vector
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

    /// Embed multiple texts into vectors (more efficient for batches)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>;

    /// Get the dimensionality of embeddings
    fn dimensions(&self) -> usize;
}
