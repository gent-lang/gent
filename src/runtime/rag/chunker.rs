//! Document chunking strategies for RAG

use std::path::Path;

/// A chunk of document content with location information
#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    /// The text content of this chunk
    pub content: String,
    /// Starting line number (1-indexed)
    pub start_line: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,
}

impl Chunk {
    /// Create a new chunk
    pub fn new(content: String, start_line: usize, end_line: usize) -> Self {
        Self {
            content,
            start_line,
            end_line,
        }
    }
}

/// Chunking strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChunkStrategy {
    /// Semantic chunking based on file type (headers for markdown, blank lines for code)
    #[default]
    Semantic,
    /// Fixed-size chunking with overlap
    Fixed,
}

/// Configuration for chunking
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Target chunk size (in characters for SemanticChunker, in lines for FixedChunker)
    pub chunk_size: usize,
    /// Overlap between chunks (in characters for SemanticChunker, in lines for FixedChunker)
    pub chunk_overlap: usize,
    /// Chunking strategy
    pub strategy: ChunkStrategy,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: 500,
            chunk_overlap: 50,
            strategy: ChunkStrategy::Semantic,
        }
    }
}

/// Trait for document chunkers
pub trait Chunker: Send + Sync {
    /// Split content into chunks
    fn chunk(&self, content: &str, file_path: &Path) -> Vec<Chunk>;
}

/// Semantic chunker that adapts to file type
#[derive(Debug, Clone)]
pub struct SemanticChunker {
    config: ChunkConfig,
}

impl SemanticChunker {
    /// Create a new semantic chunker with custom config
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::default()
    }
}

impl Default for SemanticChunker {
    fn default() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }
}

impl Chunker for SemanticChunker {
    fn chunk(&self, content: &str, file_path: &Path) -> Vec<Chunk> {
        let extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "md" | "markdown" => chunk_markdown(content, self.config.chunk_size),
            "py" | "rs" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h"
            | "hpp" | "gnt" => chunk_code(content, self.config.chunk_size),
            _ => chunk_fixed(
                content,
                self.config.chunk_size,
                self.config.chunk_overlap,
            ),
        }
    }
}

/// Fixed-size chunker with overlap
#[derive(Debug, Clone)]
pub struct FixedChunker {
    config: ChunkConfig,
}

impl FixedChunker {
    /// Create a new fixed chunker with custom config
    pub fn new(config: ChunkConfig) -> Self {
        Self { config }
    }
}

impl Default for FixedChunker {
    fn default() -> Self {
        Self {
            config: ChunkConfig {
                chunk_size: 20,     // 20 lines per chunk
                chunk_overlap: 5,   // 5 lines overlap
                strategy: ChunkStrategy::Fixed,
            },
        }
    }
}

impl Chunker for FixedChunker {
    fn chunk(&self, content: &str, _file_path: &Path) -> Vec<Chunk> {
        chunk_fixed(content, self.config.chunk_size, self.config.chunk_overlap)
    }
}

/// Chunk markdown content by headers and size
pub fn chunk_markdown(content: &str, max_size: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let mut chunks = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut current_start = 1usize;
    let mut current_size = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let is_header = line.starts_with('#');

        // Start a new chunk if:
        // 1. We hit a header and have content, OR
        // 2. Current chunk is getting too large
        let should_split = !current_lines.is_empty()
            && (is_header || current_size + line.len() > max_size);

        if should_split {
            let chunk_content = current_lines.join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk::new(
                    chunk_content,
                    current_start,
                    line_num - 1,
                ));
            }
            current_lines.clear();
            current_start = line_num;
            current_size = 0;
        }

        current_lines.push(line);
        current_size += line.len() + 1; // +1 for newline
    }

    // Don't forget the last chunk
    if !current_lines.is_empty() {
        let chunk_content = current_lines.join("\n");
        if !chunk_content.trim().is_empty() {
            chunks.push(Chunk::new(
                chunk_content,
                current_start,
                lines.len(),
            ));
        }
    }

    chunks
}

/// Chunk code content by blank lines and size
pub fn chunk_code(content: &str, max_size: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let mut chunks = Vec::new();
    let mut current_lines: Vec<&str> = Vec::new();
    let mut current_start = 1usize;
    let mut current_size = 0usize;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        let is_blank = line.trim().is_empty();

        // Split on blank lines when chunk is getting large
        let should_split = !current_lines.is_empty()
            && is_blank
            && current_size > max_size / 2;

        if should_split {
            let chunk_content = current_lines.join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk::new(
                    chunk_content,
                    current_start,
                    line_num - 1,
                ));
            }
            current_lines.clear();
            current_start = line_num + 1; // Skip the blank line
            current_size = 0;
            continue; // Don't add the blank line
        }

        // Force split if way over size limit
        if !current_lines.is_empty() && current_size + line.len() > max_size * 2 {
            let chunk_content = current_lines.join("\n");
            if !chunk_content.trim().is_empty() {
                chunks.push(Chunk::new(
                    chunk_content,
                    current_start,
                    line_num - 1,
                ));
            }
            current_lines.clear();
            current_start = line_num;
            current_size = 0;
        }

        current_lines.push(line);
        current_size += line.len() + 1;
    }

    // Don't forget the last chunk
    if !current_lines.is_empty() {
        let chunk_content = current_lines.join("\n");
        if !chunk_content.trim().is_empty() {
            chunks.push(Chunk::new(
                chunk_content,
                current_start,
                lines.len(),
            ));
        }
    }

    chunks
}

/// Chunk content by fixed line count with overlap
///
/// # Arguments
/// * `content` - The text content to chunk
/// * `lines_per_chunk` - Number of lines per chunk
/// * `overlap_lines` - Number of overlapping lines between chunks
pub fn chunk_fixed(content: &str, lines_per_chunk: usize, overlap_lines: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    // Ensure we have at least 1 line per chunk
    let lines_per_chunk = lines_per_chunk.max(1);
    // Overlap must be less than chunk size to make progress
    let overlap_lines = overlap_lines.min(lines_per_chunk.saturating_sub(1));

    let mut chunks = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let start = i;
        let end = (i + lines_per_chunk).min(lines.len());

        let chunk_content = lines[start..end].join("\n");
        if !chunk_content.trim().is_empty() {
            chunks.push(Chunk::new(
                chunk_content,
                start + 1, // 1-indexed
                end,       // 1-indexed (end is exclusive in slice, so this is correct)
            ));
        }

        // Move forward, accounting for overlap
        let step = lines_per_chunk.saturating_sub(overlap_lines);
        i += step.max(1); // Ensure we always make progress
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = Chunk::new("test content".to_string(), 1, 5);
        assert_eq!(chunk.content, "test content");
        assert_eq!(chunk.start_line, 1);
        assert_eq!(chunk.end_line, 5);
    }

    #[test]
    fn test_default_config() {
        let config = ChunkConfig::default();
        assert_eq!(config.chunk_size, 500);
        assert_eq!(config.chunk_overlap, 50);
        assert_eq!(config.strategy, ChunkStrategy::Semantic);
    }

    #[test]
    fn test_chunk_markdown_by_headers() {
        let content = "# Header 1\nContent 1\n\n# Header 2\nContent 2";
        let chunks = chunk_markdown(content, 500);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].content.contains("Header 1"));
        assert!(chunks[1].content.contains("Header 2"));
    }

    #[test]
    fn test_chunk_code_by_blank_lines() {
        let content = "fn foo() {\n    println!(\"hello\");\n}\n\nfn bar() {\n    println!(\"world\");\n}";
        // Using small max_size to force split
        let chunks = chunk_code(content, 30);
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_chunk_fixed() {
        let content = (0..20)
            .map(|i| format!("Line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        // chunk_size=5 means 5 lines per chunk, overlap=2 means 2 overlapping lines
        let chunks = chunk_fixed(&content, 5, 2);
        assert!(!chunks.is_empty());
        // First chunk should start at line 1
        assert_eq!(chunks[0].start_line, 1);
        // First chunk should end at line 5 (5 lines)
        assert_eq!(chunks[0].end_line, 5);
        // With 20 lines, 5 lines per chunk, 2 overlap (step=3), we expect ~7 chunks
        // Lines: 1-5, 4-8, 7-11, 10-14, 13-17, 16-20
        assert!(chunks.len() >= 6);
    }

    #[test]
    fn test_empty_content() {
        let chunks = chunk_markdown("", 500);
        assert!(chunks.is_empty());

        let chunks = chunk_code("", 500);
        assert!(chunks.is_empty());

        let chunks = chunk_fixed("", 10, 2);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_semantic_chunker_selects_strategy() {
        let chunker = SemanticChunker::default();

        // Markdown file
        let md_chunks = chunker.chunk("# Title\nContent", Path::new("test.md"));
        assert!(!md_chunks.is_empty());

        // Rust file
        let rs_chunks = chunker.chunk("fn main() {}", Path::new("test.rs"));
        assert!(!rs_chunks.is_empty());

        // Unknown file type falls back to fixed
        let txt_chunks = chunker.chunk("Some text", Path::new("test.xyz"));
        assert!(!txt_chunks.is_empty());
    }
}
