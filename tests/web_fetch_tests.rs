use gent::runtime::tools::{Tool, WebFetchTool};
use serde_json::json;

#[test]
fn test_web_fetch_name() {
    let tool = WebFetchTool::new();
    assert_eq!(tool.name(), "web_fetch");
}

#[test]
fn test_web_fetch_schema() {
    let tool = WebFetchTool::new();
    let schema = tool.parameters_schema();
    assert!(schema["properties"]["url"].is_object());
}

#[tokio::test]
async fn test_web_fetch_missing_url() {
    let tool = WebFetchTool::new();
    let result = tool.execute(json!({})).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("url"));
}

#[tokio::test]
async fn test_web_fetch_invalid_url() {
    let tool = WebFetchTool::new();
    let result = tool.execute(json!({"url": "not-a-url"})).await;
    assert!(result.is_err());
}

// Note: This test makes a real HTTP request
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_web_fetch_real_request() {
    let tool = WebFetchTool::new();
    let result = tool
        .execute(json!({"url": "https://httpbin.org/get"}))
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap().contains("httpbin"));
}
