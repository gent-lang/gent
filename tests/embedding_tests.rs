//! Tests for embedding providers

use gent::runtime::rag::embeddings::{EmbeddingProvider, MockEmbeddings};

#[tokio::test]
async fn test_mock_embeddings() {
    let provider = MockEmbeddings::new();
    let embedding = provider.embed("hello world").await.unwrap();

    assert_eq!(embedding.len(), provider.dimensions());

    // Check normalized (magnitude ~= 1)
    let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((magnitude - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_mock_embeddings_deterministic() {
    let provider = MockEmbeddings::new();
    let e1 = provider.embed("hello").await.unwrap();
    let e2 = provider.embed("hello").await.unwrap();
    assert_eq!(e1, e2);
}

#[tokio::test]
async fn test_mock_embeddings_batch() {
    let provider = MockEmbeddings::new();
    let embeddings = provider.embed_batch(&["hello", "world"]).await.unwrap();
    assert_eq!(embeddings.len(), 2);
}

#[tokio::test]
async fn test_mock_embeddings_different_texts_produce_different_embeddings() {
    let provider = MockEmbeddings::new();
    let e1 = provider.embed("hello").await.unwrap();
    let e2 = provider.embed("world").await.unwrap();
    assert_ne!(e1, e2);
}

#[tokio::test]
async fn test_mock_embeddings_custom_dimensions() {
    let provider = MockEmbeddings::with_dimensions(128);
    let embedding = provider.embed("test").await.unwrap();

    assert_eq!(embedding.len(), 128);
    assert_eq!(provider.dimensions(), 128);
}

#[tokio::test]
async fn test_mock_embeddings_default_dimensions() {
    let provider = MockEmbeddings::new();
    assert_eq!(provider.dimensions(), 384);
}

#[tokio::test]
async fn test_mock_embeddings_batch_matches_individual() {
    let provider = MockEmbeddings::new();

    let individual1 = provider.embed("hello").await.unwrap();
    let individual2 = provider.embed("world").await.unwrap();

    let batch = provider.embed_batch(&["hello", "world"]).await.unwrap();

    assert_eq!(batch[0], individual1);
    assert_eq!(batch[1], individual2);
}
