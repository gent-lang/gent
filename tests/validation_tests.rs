use gent::errors::Span;
use gent::interpreter::OutputSchema;
use gent::parser::{FieldType, StructField};
use gent::runtime::validation::validate_output;
use serde_json::json;

fn make_schema(fields: Vec<(&str, FieldType)>) -> OutputSchema {
    OutputSchema {
        fields: fields
            .into_iter()
            .map(|(name, ft)| StructField {
                name: name.to_string(),
                field_type: ft,
                span: Span::new(0, 0),
            })
            .collect(),
    }
}

#[test]
fn test_validate_valid_output() {
    let schema = make_schema(vec![
        ("name", FieldType::String),
        ("age", FieldType::Number),
    ]);
    let json = json!({"name": "Alice", "age": 30});
    assert!(validate_output(&json, &schema).is_ok());
}

#[test]
fn test_validate_missing_field() {
    let schema = make_schema(vec![
        ("name", FieldType::String),
        ("age", FieldType::Number),
    ]);
    let json = json!({"name": "Alice"});
    let err = validate_output(&json, &schema).unwrap_err();
    assert!(err.contains("missing"));
    assert!(err.contains("age"));
}

#[test]
fn test_validate_wrong_type() {
    let schema = make_schema(vec![("age", FieldType::Number)]);
    let json = json!({"age": "thirty"});
    let err = validate_output(&json, &schema).unwrap_err();
    assert!(err.contains("expected number"));
}

#[test]
fn test_validate_nested_object() {
    let schema = make_schema(vec![(
        "meta",
        FieldType::Object(vec![StructField {
            name: "id".to_string(),
            field_type: FieldType::String,
            span: Span::new(0, 0),
        }]),
    )]);
    let json = json!({"meta": {"id": "abc"}});
    assert!(validate_output(&json, &schema).is_ok());
}

#[test]
fn test_validate_array() {
    let schema = make_schema(vec![(
        "tags",
        FieldType::Array(Box::new(FieldType::String)),
    )]);
    let json = json!({"tags": ["a", "b", "c"]});
    assert!(validate_output(&json, &schema).is_ok());

    let bad_json = json!({"tags": [1, 2, 3]});
    assert!(validate_output(&bad_json, &schema).is_err());
}
