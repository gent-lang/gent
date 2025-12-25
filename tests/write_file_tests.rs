use gent::runtime::tools::{Tool, WriteFileTool};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_write_file_name() {
    let tool = WriteFileTool::new();
    assert_eq!(tool.name(), "write_file");
}

#[tokio::test]
async fn test_write_file_missing_path() {
    let tool = WriteFileTool::new();
    let result = tool.execute(json!({"content": "hello"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("path"));
}

#[tokio::test]
async fn test_write_file_missing_content() {
    let tool = WriteFileTool::new();
    let result = tool.execute(json!({"path": "/tmp/test.txt"})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("content"));
}

#[tokio::test]
async fn test_write_file_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("output.txt");

    let tool = WriteFileTool::new();
    let result = tool
        .execute(json!({
            "path": file_path.to_str().unwrap(),
            "content": "Hello, World!"
        }))
        .await;

    assert!(result.is_ok());

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "Hello, World!");
}

#[tokio::test]
async fn test_write_file_path_traversal_blocked() {
    let tool = WriteFileTool::new();
    let result = tool
        .execute(json!({
            "path": "../../../tmp/evil.txt",
            "content": "bad"
        }))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_write_file_overwrite() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("overwrite.txt");
    fs::write(&file_path, "original").unwrap();

    let tool = WriteFileTool::new();
    let result = tool
        .execute(json!({
            "path": file_path.to_str().unwrap(),
            "content": "updated"
        }))
        .await;

    assert!(result.is_ok());

    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "updated");
}
