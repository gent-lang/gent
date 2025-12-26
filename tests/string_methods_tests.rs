//! Tests for string methods in GENT
//!
//! These tests verify the functionality of built-in string methods.

use gent::interpreter::string_methods::call_string_method;
use gent::interpreter::Value;

// ============================================
// length() Tests
// ============================================

#[test]
fn test_string_length_basic() {
    let result = call_string_method("hello", "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(5.0));
}

#[test]
fn test_string_length_empty() {
    let result = call_string_method("", "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_string_length_unicode() {
    // Unicode characters should count as individual characters
    let result = call_string_method("hello world", "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(11.0));
}

#[test]
fn test_string_length_emoji() {
    // Emoji should count correctly
    let result = call_string_method("hi!", "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));
}

// ============================================
// trim() Tests
// ============================================

#[test]
fn test_string_trim_basic() {
    let result = call_string_method("  hello  ", "trim", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_left_only() {
    let result = call_string_method("  hello", "trim", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_right_only() {
    let result = call_string_method("hello  ", "trim", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_no_whitespace() {
    let result = call_string_method("hello", "trim", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_trim_tabs_and_newlines() {
    let result = call_string_method("\t\nhello\r\n", "trim", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

// ============================================
// contains() Tests
// ============================================

#[test]
fn test_string_contains_true() {
    let result = call_string_method(
        "hello world",
        "contains",
        &[Value::String("world".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_string_contains_false() {
    let result = call_string_method(
        "hello world",
        "contains",
        &[Value::String("xyz".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_string_contains_empty_substring() {
    let result = call_string_method("hello", "contains", &[Value::String("".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_string_contains_entire_string() {
    let result = call_string_method("hello", "contains", &[Value::String("hello".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

// ============================================
// startsWith() Tests
// ============================================

#[test]
fn test_string_starts_with_true() {
    let result = call_string_method(
        "hello world",
        "startsWith",
        &[Value::String("hello".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_string_starts_with_false() {
    let result = call_string_method(
        "hello world",
        "startsWith",
        &[Value::String("world".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_string_starts_with_empty() {
    let result = call_string_method("hello", "startsWith", &[Value::String("".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

// ============================================
// endsWith() Tests
// ============================================

#[test]
fn test_string_ends_with_true() {
    let result = call_string_method(
        "hello world",
        "endsWith",
        &[Value::String("world".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

#[test]
fn test_string_ends_with_false() {
    let result = call_string_method(
        "hello world",
        "endsWith",
        &[Value::String("hello".to_string())],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(false));
}

#[test]
fn test_string_ends_with_empty() {
    let result = call_string_method("hello", "endsWith", &[Value::String("".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Boolean(true));
}

// ============================================
// split() Tests
// ============================================

#[test]
fn test_string_split_basic() {
    let result = call_string_method("a,b,c", "split", &[Value::String(",".to_string())]);
    assert!(result.is_ok());
    let expected = Value::Array(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_string_split_no_match() {
    let result = call_string_method("hello", "split", &[Value::String(",".to_string())]);
    assert!(result.is_ok());
    let expected = Value::Array(vec![Value::String("hello".to_string())]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_string_split_empty_separator() {
    // Splitting by empty string returns each character
    let result = call_string_method("abc", "split", &[Value::String("".to_string())]);
    assert!(result.is_ok());
    let expected = Value::Array(vec![
        Value::String("".to_string()),
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
        Value::String("".to_string()),
    ]);
    assert_eq!(result.unwrap(), expected);
}

#[test]
fn test_string_split_multi_char_separator() {
    let result = call_string_method("a::b::c", "split", &[Value::String("::".to_string())]);
    assert!(result.is_ok());
    let expected = Value::Array(vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ]);
    assert_eq!(result.unwrap(), expected);
}

// ============================================
// replace() Tests
// ============================================

#[test]
fn test_string_replace_basic() {
    let result = call_string_method(
        "hello world",
        "replace",
        &[
            Value::String("world".to_string()),
            Value::String("there".to_string()),
        ],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello there".to_string()));
}

#[test]
fn test_string_replace_first_occurrence_only() {
    let result = call_string_method(
        "hello hello",
        "replace",
        &[
            Value::String("hello".to_string()),
            Value::String("hi".to_string()),
        ],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hi hello".to_string()));
}

#[test]
fn test_string_replace_no_match() {
    let result = call_string_method(
        "hello world",
        "replace",
        &[
            Value::String("xyz".to_string()),
            Value::String("abc".to_string()),
        ],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
}

#[test]
fn test_string_replace_with_empty() {
    let result = call_string_method(
        "hello world",
        "replace",
        &[
            Value::String("world".to_string()),
            Value::String("".to_string()),
        ],
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello ".to_string()));
}

// ============================================
// toLowerCase() Tests
// ============================================

#[test]
fn test_string_to_lower_case_basic() {
    let result = call_string_method("Hello World", "toLowerCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello world".to_string()));
}

#[test]
fn test_string_to_lower_case_all_caps() {
    let result = call_string_method("HELLO", "toLowerCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_to_lower_case_already_lower() {
    let result = call_string_method("hello", "toLowerCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello".to_string()));
}

#[test]
fn test_string_to_lower_case_mixed() {
    let result = call_string_method("HeLLo123", "toLowerCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("hello123".to_string()));
}

// ============================================
// toUpperCase() Tests
// ============================================

#[test]
fn test_string_to_upper_case_basic() {
    let result = call_string_method("Hello World", "toUpperCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("HELLO WORLD".to_string()));
}

#[test]
fn test_string_to_upper_case_all_lower() {
    let result = call_string_method("hello", "toUpperCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
}

#[test]
fn test_string_to_upper_case_already_upper() {
    let result = call_string_method("HELLO", "toUpperCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("HELLO".to_string()));
}

#[test]
fn test_string_to_upper_case_mixed() {
    let result = call_string_method("HeLLo123", "toUpperCase", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("HELLO123".to_string()));
}

// ============================================
// Error Cases
// ============================================

#[test]
fn test_unknown_method() {
    let result = call_string_method("hello", "unknownMethod", &[]);
    assert!(result.is_err());
}

#[test]
fn test_contains_wrong_arg_type() {
    let result = call_string_method("hello", "contains", &[Value::Number(42.0)]);
    assert!(result.is_err());
}

#[test]
fn test_contains_missing_arg() {
    let result = call_string_method("hello", "contains", &[]);
    assert!(result.is_err());
}

#[test]
fn test_replace_missing_args() {
    let result = call_string_method("hello", "replace", &[Value::String("x".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_replace_wrong_arg_types() {
    let result = call_string_method(
        "hello",
        "replace",
        &[Value::Number(1.0), Value::Number(2.0)],
    );
    assert!(result.is_err());
}
