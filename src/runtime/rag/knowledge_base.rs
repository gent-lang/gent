//! KnowledgeBase implementation for RAG

use super::chunker::{ChunkConfig, ChunkStrategy, Chunker, SemanticChunker};
use super::embeddings::{EmbeddingProvider, MockEmbeddings, OpenAIEmbeddings};
use super::store::{LocalVectorStore, Metadata, SearchResult, VectorStore};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for KnowledgeBase indexing
#[derive(Debug, Clone)]
pub struct IndexOptions {
    pub extensions: Vec<String>,
    pub recursive: bool,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub strategy: String,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self {
            extensions: vec![
                ".md".to_string(),
                ".txt".to_string(),
                ".rs".to_string(),
                ".py".to_string(),
                ".js".to_string(),
                ".ts".to_string(),
            ],
            recursive: true,
            chunk_size: 500,
            chunk_overlap: 50,
            strategy: "semantic".to_string(),
        }
    }
}

/// A searchable collection of documents
#[derive(Debug)]
pub struct KnowledgeBase {
    path: PathBuf,
    store: Arc<RwLock<Box<dyn VectorStore>>>,
    embeddings: Arc<dyn EmbeddingProvider>,
    indexed: Arc<AtomicBool>,
}

impl KnowledgeBase {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let index_path = path.join(".gent_index").join("vectors.json");
        let store = LocalVectorStore::with_path(index_path);
        let has_existing = store.len() > 0;

        Self {
            path,
            store: Arc::new(RwLock::new(Box::new(store))),
            embeddings: Arc::new(MockEmbeddings::new()),
            indexed: Arc::new(AtomicBool::new(has_existing)),
        }
    }

    pub fn with_openai(path: impl Into<PathBuf>, api_key: String) -> Self {
        let path = path.into();
        let index_path = path.join(".gent_index").join("vectors.json");
        let store = LocalVectorStore::with_path(index_path);
        let has_existing = store.len() > 0;

        Self {
            path,
            store: Arc::new(RwLock::new(Box::new(store))),
            embeddings: Arc::new(OpenAIEmbeddings::new(api_key)),
            indexed: Arc::new(AtomicBool::new(has_existing)),
        }
    }

    pub async fn index(&mut self, options: IndexOptions) -> Result<usize, String> {
        let mut store = self.store.write().await;
        store.clear().await?;

        let files = self.collect_files(&options)?;
        let chunker = SemanticChunker::new(ChunkConfig {
            chunk_size: options.chunk_size,
            chunk_overlap: options.chunk_overlap,
            strategy: if options.strategy == "fixed" {
                ChunkStrategy::Fixed
            } else {
                ChunkStrategy::Semantic
            },
        });

        let mut total_chunks = 0;

        for file_path in files {
            let content = std::fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

            let chunks = chunker.chunk(&content, &file_path);

            for (i, chunk) in chunks.iter().enumerate() {
                let embedding = self.embeddings.embed(&chunk.content).await?;
                let id = format!("{}:{}", file_path.display(), i);

                let relative_path = file_path
                    .strip_prefix(&self.path)
                    .unwrap_or(&file_path)
                    .to_string_lossy()
                    .to_string();

                let metadata = Metadata {
                    source: relative_path,
                    chunk_index: i,
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                    content: chunk.content.clone(),
                };

                store.add(&id, embedding, metadata).await?;
                total_chunks += 1;
            }
        }

        drop(store);
        self.indexed.store(true, Ordering::SeqCst);
        Ok(total_chunks)
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
        if !self.indexed.load(Ordering::SeqCst) {
            return Err("KnowledgeBase not indexed. Call .index() first.".to_string());
        }

        let query_embedding = self.embeddings.embed(query).await?;
        let store = self.store.read().await;
        store.search(query_embedding, limit).await
    }

    fn collect_files(&self, options: &IndexOptions) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        self.collect_files_recursive(&self.path, options, &mut files)?;
        Ok(files)
    }

    fn collect_files_recursive(
        &self,
        dir: &Path,
        options: &IndexOptions,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        if !dir.is_dir() {
            return Err(format!("Directory not found: {}", dir.display()));
        }

        for entry in std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                if options.recursive && !path.file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
                {
                    self.collect_files_recursive(&path, options, files)?;
                }
            } else if let Some(ext) = path.extension() {
                let ext_str = format!(".{}", ext.to_string_lossy());
                if options.extensions.contains(&ext_str) {
                    files.push(path);
                }
            }
        }

        Ok(())
    }

    pub fn is_indexed(&self) -> bool {
        self.indexed.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_index_options_default() {
        let options = IndexOptions::default();
        assert!(options.extensions.contains(&".md".to_string()));
        assert!(options.extensions.contains(&".rs".to_string()));
        assert!(options.recursive);
        assert_eq!(options.chunk_size, 500);
        assert_eq!(options.chunk_overlap, 50);
        assert_eq!(options.strategy, "semantic");
    }

    #[test]
    fn test_knowledge_base_creation() {
        let kb = KnowledgeBase::new("/tmp/test");
        assert!(!kb.is_indexed());
    }

    #[tokio::test]
    async fn test_search_before_index() {
        let kb = KnowledgeBase::new("/tmp/test");
        let result = kb.search("query", 5).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not indexed"));
    }

    #[tokio::test]
    async fn test_index_and_search() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.md");
        fs::write(&test_file, "# Test\nThis is test content about Rust programming.").unwrap();

        let mut kb = KnowledgeBase::new(temp_dir.path());
        let count = kb.index(IndexOptions::default()).await.unwrap();
        assert!(count > 0);
        assert!(kb.is_indexed());

        let results = kb.search("Rust programming", 5).await.unwrap();
        assert!(!results.is_empty());
        assert!(results[0].metadata.content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_collect_files_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create files in root
        fs::write(temp_dir.path().join("readme.md"), "# Readme").unwrap();

        // Create files in subdirectory
        let sub_dir = temp_dir.path().join("src");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("main.rs"), "fn main() {}").unwrap();

        // Create hidden directory (should be skipped)
        let hidden_dir = temp_dir.path().join(".hidden");
        fs::create_dir(&hidden_dir).unwrap();
        fs::write(hidden_dir.join("secret.md"), "secret").unwrap();

        let kb = KnowledgeBase::new(temp_dir.path());
        let options = IndexOptions::default();
        let files = kb.collect_files(&options).unwrap();

        assert_eq!(files.len(), 2);
        let file_names: Vec<_> = files.iter().filter_map(|p| p.file_name()).collect();
        assert!(file_names.iter().any(|n| n.to_string_lossy() == "readme.md"));
        assert!(file_names.iter().any(|n| n.to_string_lossy() == "main.rs"));
    }

    #[tokio::test]
    async fn test_non_recursive_collection() {
        let temp_dir = TempDir::new().unwrap();

        // Create file in root
        fs::write(temp_dir.path().join("readme.md"), "# Readme").unwrap();

        // Create file in subdirectory
        let sub_dir = temp_dir.path().join("src");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(sub_dir.join("main.rs"), "fn main() {}").unwrap();

        let kb = KnowledgeBase::new(temp_dir.path());
        let options = IndexOptions {
            recursive: false,
            ..Default::default()
        };
        let files = kb.collect_files(&options).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_string_lossy() == "readme.md");
    }
}
