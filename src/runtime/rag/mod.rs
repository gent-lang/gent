//! RAG (Retrieval-Augmented Generation) system for GENT

pub mod embeddings;
pub mod store;

pub use embeddings::EmbeddingProvider;
pub use store::{Metadata, SearchResult, VectorStore};
