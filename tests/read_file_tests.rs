use gent::runtime::tools::{ReadFileTool, Tool};
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_read_file_name() {
    let tool = ReadFileTool::new();
    assert_eq!(tool.name(), "read_file");
}

#[tokio::test]
async fn test_read_file_missing_path() {
    let tool = ReadFileTool::new();
    let result = tool.execute(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("path"));
}

#[tokio::test]
async fn test_read_file_not_found() {
    let tool = ReadFileTool::new();
    let result = tool.execute(json!({"path": "/nonexistent/file.txt"})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_file_success() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "Hello, World!").unwrap();

    let tool = ReadFileTool::new();
    let result = tool
        .execute(json!({"path": file_path.to_str().unwrap()}))
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, World!");
}

#[tokio::test]
async fn test_read_file_path_traversal_blocked() {
    let tool = ReadFileTool::new();
    let result = tool.execute(json!({"path": "../../../etc/passwd"})).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("traversal") || err.contains("invalid"));
}
