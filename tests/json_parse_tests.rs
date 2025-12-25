use gent::runtime::tools::{JsonParseTool, Tool};
use serde_json::json;

#[tokio::test]
async fn test_json_parse_object() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": r#"{"name": "Tokyo"}"#})).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    // The result should be valid JSON that can be parsed back
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(value["name"], "Tokyo");
}

#[tokio::test]
async fn test_json_parse_array() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": "[1, 2, 3]"})).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 3);
    assert_eq!(value[0], 1);
    assert_eq!(value[1], 2);
    assert_eq!(value[2], 3);
}

#[tokio::test]
async fn test_json_parse_invalid() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": "not valid json"})).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("parse") || err.contains("invalid") || err.contains("JSON"));
}

#[tokio::test]
async fn test_json_parse_nested() {
    let tool = JsonParseTool::new();
    let result = tool
        .execute(json!({"text": r#"{"main": {"temp": 22}}"#}))
        .await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(value["main"]["temp"], 22);
}

#[test]
fn test_json_parse_tool_info() {
    let tool = JsonParseTool::new();
    assert_eq!(tool.name(), "json_parse");
    assert!(tool.description().contains("JSON") || tool.description().contains("json"));
}

#[tokio::test]
async fn test_json_parse_missing_text() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({})).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("text") || err.contains("parameter"));
}

#[tokio::test]
async fn test_json_parse_boolean() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": "true"})).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(value, true);
}

#[tokio::test]
async fn test_json_parse_null() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": "null"})).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert!(value.is_null());
}

#[tokio::test]
async fn test_json_parse_number() {
    let tool = JsonParseTool::new();
    let result = tool.execute(json!({"text": "42.5"})).await;

    assert!(result.is_ok());
    let parsed = result.unwrap();
    let value: serde_json::Value = serde_json::from_str(&parsed).unwrap();
    assert_eq!(value, 42.5);
}
