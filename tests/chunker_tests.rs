//! Tests for document chunking

use gent::runtime::rag::chunker::{
    Chunk, ChunkConfig, ChunkStrategy, Chunker, FixedChunker, SemanticChunker,
};
use std::path::Path;

#[test]
fn test_semantic_chunker_markdown() {
    let content = "# Header 1\nSome content here.\n\n# Header 2\nMore content.";
    let chunker = SemanticChunker::default();
    let chunks = chunker.chunk(content, Path::new("test.md"));
    assert!(chunks.len() >= 2);
}

#[test]
fn test_fixed_chunker() {
    let content = (0..20)
        .map(|i| format!("Line {}", i))
        .collect::<Vec<_>>()
        .join("\n");
    let config = ChunkConfig {
        chunk_size: 100,
        chunk_overlap: 20,
        strategy: ChunkStrategy::Fixed,
    };
    let chunker = FixedChunker::new(config);
    let chunks = chunker.chunk(&content, Path::new("test.txt"));
    assert!(!chunks.is_empty());
}

#[test]
fn test_chunk_has_line_numbers() {
    let content = "Line 1\nLine 2\nLine 3";
    let chunker = SemanticChunker::default();
    let chunks = chunker.chunk(content, Path::new("test.txt"));
    assert!(!chunks.is_empty());
    assert!(chunks[0].start_line >= 1);
}

#[test]
fn test_semantic_chunker_code_files() {
    let rust_code = r#"fn main() {
    println!("Hello");
}

fn helper() {
    // do something
}"#;

    let chunker = SemanticChunker::new(ChunkConfig {
        chunk_size: 30,
        ..ChunkConfig::default()
    });

    // Test Rust file
    let chunks = chunker.chunk(rust_code, Path::new("main.rs"));
    assert!(!chunks.is_empty());

    // Test Python file
    let chunks = chunker.chunk("def foo():\n    pass", Path::new("main.py"));
    assert!(!chunks.is_empty());

    // Test JavaScript file
    let chunks = chunker.chunk("function foo() {}", Path::new("main.js"));
    assert!(!chunks.is_empty());
}

#[test]
fn test_fixed_chunker_overlap() {
    // Create content with enough lines to span multiple chunks
    let content = (0..30)
        .map(|i| format!("This is line number {}", i))
        .collect::<Vec<_>>()
        .join("\n");

    let config = ChunkConfig {
        chunk_size: 200,
        chunk_overlap: 40,
        strategy: ChunkStrategy::Fixed,
    };
    let chunker = FixedChunker::new(config);
    let chunks = chunker.chunk(&content, Path::new("test.txt"));

    // Should have multiple chunks due to content length
    assert!(chunks.len() >= 2);

    // Verify line numbers are 1-indexed
    assert_eq!(chunks[0].start_line, 1);
}

#[test]
fn test_chunk_struct() {
    let chunk = Chunk::new("test content".to_string(), 5, 10);
    assert_eq!(chunk.content, "test content");
    assert_eq!(chunk.start_line, 5);
    assert_eq!(chunk.end_line, 10);
}

#[test]
fn test_chunk_config_default() {
    let config = ChunkConfig::default();
    assert_eq!(config.chunk_size, 500);
    assert_eq!(config.chunk_overlap, 50);
    assert_eq!(config.strategy, ChunkStrategy::Semantic);
}

#[test]
fn test_empty_content_returns_empty_chunks() {
    let chunker = SemanticChunker::default();
    let chunks = chunker.chunk("", Path::new("test.md"));
    assert!(chunks.is_empty());
}

#[test]
fn test_whitespace_only_content() {
    let chunker = SemanticChunker::default();
    let chunks = chunker.chunk("   \n\n   \n", Path::new("test.md"));
    assert!(chunks.is_empty());
}

#[test]
fn test_gnt_file_uses_code_chunking() {
    let gnt_code = r#"agent MyAgent {
    system "Hello"
}

fn helper() {
    return 42
}"#;

    let chunker = SemanticChunker::new(ChunkConfig {
        chunk_size: 30,
        ..ChunkConfig::default()
    });

    let chunks = chunker.chunk(gnt_code, Path::new("main.gnt"));
    assert!(!chunks.is_empty());
}
