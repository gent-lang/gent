//! Embedding providers for RAG

use async_trait::async_trait;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

mod openai;
pub use openai::OpenAIEmbeddings;

/// Trait for embedding text into vectors
#[async_trait]
pub trait EmbeddingProvider: Send + Sync + std::fmt::Debug {
    /// Embed a single text into a vector
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String>;

    /// Embed multiple texts into vectors (more efficient for batches)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String>;

    /// Get the dimensionality of embeddings
    fn dimensions(&self) -> usize;
}

/// Mock embedding provider for testing
/// Generates deterministic normalized embeddings based on text hash
#[derive(Debug)]
pub struct MockEmbeddings {
    dimensions: usize,
}

impl MockEmbeddings {
    /// Create a new mock embeddings provider with default dimensions (384)
    pub fn new() -> Self {
        Self { dimensions: 384 }
    }

    /// Create a mock embeddings provider with custom dimensions
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self { dimensions }
    }

    /// Generate a deterministic embedding from text
    fn generate_embedding(&self, text: &str) -> Vec<f32> {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // Use the hash as a seed to generate deterministic values
        let mut embedding = Vec::with_capacity(self.dimensions);
        let mut seed = hash;

        for _ in 0..self.dimensions {
            // Simple LCG-style pseudo-random number generation
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            // Convert to float in range [-1, 1]
            let value = ((seed >> 32) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            embedding.push(value);
        }

        // Normalize to unit length
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for x in &mut embedding {
                *x /= magnitude;
            }
        }

        embedding
    }
}

impl Default for MockEmbeddings {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddings {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        Ok(self.generate_embedding(text))
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        Ok(texts.iter().map(|text| self.generate_embedding(text)).collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}
