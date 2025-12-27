//! RAG (Retrieval-Augmented Generation) system for GENT

pub mod chunker;
pub mod embeddings;
pub mod kb_tool;
pub mod knowledge_base;
pub mod store;

pub use chunker::{Chunk, ChunkConfig, ChunkStrategy, Chunker, FixedChunker, SemanticChunker};
pub use embeddings::EmbeddingProvider;
pub use kb_tool::KnowledgeBaseTool;
pub use knowledge_base::{IndexOptions, KnowledgeBase};
pub use store::{Metadata, SearchResult, VectorStore};
