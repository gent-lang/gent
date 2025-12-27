//! Tests for vector stores

use gent::runtime::rag::store::{LocalVectorStore, Metadata, VectorStore};

fn make_metadata(source: &str) -> Metadata {
    Metadata {
        source: source.to_string(),
        chunk_index: 0,
        start_line: 1,
        end_line: 10,
        content: "test content".to_string(),
    }
}

#[tokio::test]
async fn test_local_store_add_and_search() {
    let mut store = LocalVectorStore::new();
    store
        .add("doc1", vec![1.0, 0.0, 0.0], make_metadata("doc1.md"))
        .await
        .unwrap();
    store
        .add("doc2", vec![0.0, 1.0, 0.0], make_metadata("doc2.md"))
        .await
        .unwrap();
    store
        .add("doc3", vec![0.7, 0.7, 0.0], make_metadata("doc3.md"))
        .await
        .unwrap();

    let results = store.search(vec![1.0, 0.0, 0.0], 2).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].id, "doc1");
    assert!((results[0].score - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_local_store_delete() {
    let mut store = LocalVectorStore::new();
    store
        .add("doc1", vec![1.0, 0.0], make_metadata("doc1.md"))
        .await
        .unwrap();
    assert_eq!(store.len(), 1);
    store.delete("doc1").await.unwrap();
    assert_eq!(store.len(), 0);
}

#[tokio::test]
async fn test_local_store_clear() {
    let mut store = LocalVectorStore::new();
    store
        .add("doc1", vec![1.0, 0.0], make_metadata("doc1.md"))
        .await
        .unwrap();
    store
        .add("doc2", vec![0.0, 1.0], make_metadata("doc2.md"))
        .await
        .unwrap();
    store.clear().await.unwrap();
    assert!(store.is_empty());
}

#[tokio::test]
async fn test_local_store_cosine_similarity_ranking() {
    let mut store = LocalVectorStore::new();

    // Add vectors at different angles
    store
        .add("exact_match", vec![1.0, 0.0], make_metadata("exact.md"))
        .await
        .unwrap();
    store
        .add("similar", vec![0.9, 0.1], make_metadata("similar.md"))
        .await
        .unwrap();
    store
        .add("orthogonal", vec![0.0, 1.0], make_metadata("orthogonal.md"))
        .await
        .unwrap();
    store
        .add("opposite", vec![-1.0, 0.0], make_metadata("opposite.md"))
        .await
        .unwrap();

    let results = store.search(vec![1.0, 0.0], 4).await.unwrap();

    // Results should be ordered by similarity
    assert_eq!(results[0].id, "exact_match");
    assert_eq!(results[1].id, "similar");
    assert_eq!(results[2].id, "orthogonal");
    assert_eq!(results[3].id, "opposite");

    // Check score values
    assert!((results[0].score - 1.0).abs() < 0.01); // exact match = 1.0
    assert!(results[1].score > 0.9); // similar should be high
    assert!(results[2].score.abs() < 0.01); // orthogonal = 0.0
    assert!((results[3].score - (-1.0)).abs() < 0.01); // opposite = -1.0
}

#[tokio::test]
async fn test_local_store_empty_search() {
    let store = LocalVectorStore::new();
    let results = store.search(vec![1.0, 0.0], 10).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_local_store_limit() {
    let mut store = LocalVectorStore::new();
    for i in 0..10 {
        store
            .add(
                &format!("doc{}", i),
                vec![1.0, i as f32 * 0.1],
                make_metadata(&format!("doc{}.md", i)),
            )
            .await
            .unwrap();
    }

    let results = store.search(vec![1.0, 0.0], 3).await.unwrap();
    assert_eq!(results.len(), 3);
}

#[tokio::test]
async fn test_local_store_update_existing() {
    let mut store = LocalVectorStore::new();
    store
        .add("doc1", vec![1.0, 0.0], make_metadata("original.md"))
        .await
        .unwrap();
    store
        .add("doc1", vec![0.0, 1.0], make_metadata("updated.md"))
        .await
        .unwrap();

    assert_eq!(store.len(), 1); // Should overwrite, not duplicate

    let results = store.search(vec![0.0, 1.0], 1).await.unwrap();
    assert_eq!(results[0].id, "doc1");
    assert_eq!(results[0].metadata.source, "updated.md");
}
