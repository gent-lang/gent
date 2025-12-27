//! Local file-based vector store

use super::{Metadata, SearchResult, VectorStore};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Local vector store using cosine similarity
pub struct LocalVectorStore {
    vectors: HashMap<String, StoredVector>,
    index_path: Option<PathBuf>,
}

#[derive(Clone, Serialize, Deserialize)]
struct StoredVector {
    embedding: Vec<f32>,
    metadata: Metadata,
}

impl LocalVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            vectors: HashMap::new(),
            index_path: None,
        }
    }

    /// Create a vector store with file persistence
    pub fn with_path(path: PathBuf) -> Self {
        let mut store = Self::new();
        store.index_path = Some(path);
        store.load().ok();
        store
    }

    /// Load vectors from disk
    fn load(&mut self) -> Result<(), String> {
        if let Some(ref path) = self.index_path {
            if path.exists() {
                let data = std::fs::read_to_string(path)
                    .map_err(|e| format!("Failed to read index: {}", e))?;
                self.vectors = serde_json::from_str(&data)
                    .map_err(|e| format!("Failed to parse index: {}", e))?;
            }
        }
        Ok(())
    }

    /// Save vectors to disk
    fn save(&self) -> Result<(), String> {
        if let Some(ref path) = self.index_path {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create index directory: {}", e))?;
            }
            let data = serde_json::to_string(&self.vectors)
                .map_err(|e| format!("Failed to serialize index: {}", e))?;
            std::fs::write(path, data)
                .map_err(|e| format!("Failed to write index: {}", e))?;
        }
        Ok(())
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }
}

impl Default for LocalVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for LocalVectorStore {
    async fn add(
        &mut self,
        id: &str,
        embedding: Vec<f32>,
        metadata: Metadata,
    ) -> Result<(), String> {
        self.vectors
            .insert(id.to_string(), StoredVector { embedding, metadata });
        self.save()
    }

    async fn search(&self, query: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>, String> {
        let mut scores: Vec<_> = self
            .vectors
            .iter()
            .map(|(id, stored)| {
                let score = Self::cosine_similarity(&query, &stored.embedding);
                (id.clone(), score, stored.metadata.clone())
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(scores
            .into_iter()
            .take(limit)
            .map(|(id, score, metadata)| SearchResult { id, score, metadata })
            .collect())
    }

    async fn delete(&mut self, id: &str) -> Result<(), String> {
        self.vectors.remove(id);
        self.save()
    }

    async fn clear(&mut self) -> Result<(), String> {
        self.vectors.clear();
        self.save()
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }
}
