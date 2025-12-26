//! Tests for built-in functions (print, println)

use gent::interpreter::builtins::{call_builtin, is_builtin};
use gent::interpreter::Value;
use gent::Span;

#[test]
fn test_is_builtin() {
    assert!(is_builtin("print"));
    assert!(is_builtin("println"));
    assert!(!is_builtin("not_a_builtin"));
}

#[test]
fn test_print_single_arg() {
    let args = vec![Value::String("hello".to_string())];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_println_single_arg() {
    let args = vec![Value::String("hello".to_string())];
    let result = call_builtin("println", &args, &Span::default());
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_print_multiple_args() {
    let args = vec![
        Value::String("hello".to_string()),
        Value::String("world".to_string()),
    ];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_ok());
}

#[test]
fn test_println_no_args() {
    let args = vec![];
    let result = call_builtin("println", &args, &Span::default());
    assert!(result.is_ok());
}

#[test]
fn test_print_type_error() {
    let args = vec![Value::Number(42.0)];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_err());
}

#[test]
fn test_print_mixed_type_error() {
    let args = vec![
        Value::String("hello".to_string()),
        Value::Number(42.0),
    ];
    let result = call_builtin("print", &args, &Span::default());
    assert!(result.is_err());
}
