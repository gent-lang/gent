//! Tests for array methods in GENT
//!
//! These tests verify the functionality of built-in array methods.
//! Note: These tests test the array_methods module directly.
//! Integration tests that test the methods through the interpreter
//! will be added in Task 5.

use gent::interpreter::array_methods::call_array_method;
use gent::interpreter::Value;

// ============================================
// length() Tests
// ============================================

#[test]
fn test_array_length_basic() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(&mut arr, "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));
}

#[test]
fn test_array_length_empty() {
    let mut arr: Vec<Value> = vec![];
    let result = call_array_method(&mut arr, "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_array_length_single_element() {
    let mut arr = vec![Value::String("hello".to_string())];
    let result = call_array_method(&mut arr, "length", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

// ============================================
// push() Tests
// ============================================

#[test]
fn test_array_push_basic() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0)];
    let result = call_array_method(&mut arr, "push", &[Value::Number(3.0)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[2], Value::Number(3.0));
}

#[test]
fn test_array_push_to_empty() {
    let mut arr: Vec<Value> = vec![];
    let result = call_array_method(&mut arr, "push", &[Value::String("first".to_string())]);
    assert!(result.is_ok());
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0], Value::String("first".to_string()));
}

#[test]
fn test_array_push_missing_arg() {
    let mut arr = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr, "push", &[]);
    assert!(result.is_err());
}

// ============================================
// pop() Tests
// ============================================

#[test]
fn test_array_pop_basic() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(&mut arr, "pop", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(3.0));
    assert_eq!(arr.len(), 2);
}

#[test]
fn test_array_pop_empty() {
    let mut arr: Vec<Value> = vec![];
    let result = call_array_method(&mut arr, "pop", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Null);
}

#[test]
fn test_array_pop_single_element() {
    let mut arr = vec![Value::String("only".to_string())];
    let result = call_array_method(&mut arr, "pop", &[]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("only".to_string()));
    assert!(arr.is_empty());
}

// ============================================
// indexOf() Tests
// ============================================

#[test]
fn test_array_index_of_found() {
    let mut arr = vec![Value::Number(10.0), Value::Number(20.0), Value::Number(30.0)];
    let result = call_array_method(&mut arr, "indexOf", &[Value::Number(20.0)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_array_index_of_not_found() {
    let mut arr = vec![Value::Number(10.0), Value::Number(20.0), Value::Number(30.0)];
    let result = call_array_method(&mut arr, "indexOf", &[Value::Number(99.0)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(-1.0));
}

#[test]
fn test_array_index_of_string() {
    let mut arr = vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ];
    let result = call_array_method(&mut arr, "indexOf", &[Value::String("b".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(1.0));
}

#[test]
fn test_array_index_of_first_occurrence() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(1.0)];
    let result = call_array_method(&mut arr, "indexOf", &[Value::Number(1.0)]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::Number(0.0));
}

#[test]
fn test_array_index_of_missing_arg() {
    let mut arr = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr, "indexOf", &[]);
    assert!(result.is_err());
}

// ============================================
// join() Tests
// ============================================

#[test]
fn test_array_join_basic() {
    let mut arr = vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ];
    let result = call_array_method(&mut arr, "join", &[Value::String("-".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("a-b-c".to_string()));
}

#[test]
fn test_array_join_empty_separator() {
    let mut arr = vec![
        Value::String("a".to_string()),
        Value::String("b".to_string()),
        Value::String("c".to_string()),
    ];
    let result = call_array_method(&mut arr, "join", &[Value::String("".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("abc".to_string()));
}

#[test]
fn test_array_join_numbers() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(&mut arr, "join", &[Value::String(", ".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("1, 2, 3".to_string()));
}

#[test]
fn test_array_join_single_element() {
    let mut arr = vec![Value::String("only".to_string())];
    let result = call_array_method(&mut arr, "join", &[Value::String("-".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("only".to_string()));
}

#[test]
fn test_array_join_empty_array() {
    let mut arr: Vec<Value> = vec![];
    let result = call_array_method(&mut arr, "join", &[Value::String("-".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), Value::String("".to_string()));
}

#[test]
fn test_array_join_missing_arg() {
    let mut arr = vec![Value::String("a".to_string())];
    let result = call_array_method(&mut arr, "join", &[]);
    assert!(result.is_err());
}

#[test]
fn test_array_join_wrong_type() {
    let mut arr = vec![Value::String("a".to_string())];
    let result = call_array_method(&mut arr, "join", &[Value::Number(1.0)]);
    assert!(result.is_err());
}

// ============================================
// slice() Tests
// ============================================

#[test]
fn test_array_slice_basic() {
    let mut arr = vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
        Value::Number(4.0),
        Value::Number(5.0),
    ];
    let result = call_array_method(
        &mut arr,
        "slice",
        &[Value::Number(1.0), Value::Number(4.0)],
    );
    assert!(result.is_ok());
    let sliced = result.unwrap();
    match sliced {
        Value::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(2.0));
            assert_eq!(items[1], Value::Number(3.0));
            assert_eq!(items[2], Value::Number(4.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_slice_from_start() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(
        &mut arr,
        "slice",
        &[Value::Number(0.0), Value::Number(2.0)],
    );
    assert!(result.is_ok());
    let sliced = result.unwrap();
    match sliced {
        Value::Array(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_slice_to_end() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(
        &mut arr,
        "slice",
        &[Value::Number(1.0), Value::Number(3.0)],
    );
    assert!(result.is_ok());
    let sliced = result.unwrap();
    match sliced {
        Value::Array(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0], Value::Number(2.0));
            assert_eq!(items[1], Value::Number(3.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_slice_empty_result() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)];
    let result = call_array_method(
        &mut arr,
        "slice",
        &[Value::Number(2.0), Value::Number(2.0)],
    );
    assert!(result.is_ok());
    let sliced = result.unwrap();
    match sliced {
        Value::Array(items) => {
            assert!(items.is_empty());
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_slice_out_of_bounds() {
    let mut arr = vec![Value::Number(1.0), Value::Number(2.0)];
    // End index beyond array length should be clamped
    let result = call_array_method(
        &mut arr,
        "slice",
        &[Value::Number(0.0), Value::Number(10.0)],
    );
    assert!(result.is_ok());
    let sliced = result.unwrap();
    match sliced {
        Value::Array(items) => {
            assert_eq!(items.len(), 2);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_slice_missing_args() {
    let mut arr = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr, "slice", &[Value::Number(0.0)]);
    assert!(result.is_err());
}

// ============================================
// concat() Tests
// ============================================

#[test]
fn test_array_concat_basic() {
    let mut arr1 = vec![Value::Number(1.0), Value::Number(2.0)];
    let arr2 = Value::Array(vec![Value::Number(3.0), Value::Number(4.0)]);
    let result = call_array_method(&mut arr1, "concat", &[arr2]);
    assert!(result.is_ok());
    let combined = result.unwrap();
    match combined {
        Value::Array(items) => {
            assert_eq!(items.len(), 4);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
            assert_eq!(items[3], Value::Number(4.0));
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_concat_empty_arrays() {
    let mut arr1: Vec<Value> = vec![];
    let arr2 = Value::Array(vec![]);
    let result = call_array_method(&mut arr1, "concat", &[arr2]);
    assert!(result.is_ok());
    let combined = result.unwrap();
    match combined {
        Value::Array(items) => {
            assert!(items.is_empty());
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_concat_one_empty() {
    let mut arr1 = vec![Value::Number(1.0)];
    let arr2 = Value::Array(vec![]);
    let result = call_array_method(&mut arr1, "concat", &[arr2]);
    assert!(result.is_ok());
    let combined = result.unwrap();
    match combined {
        Value::Array(items) => {
            assert_eq!(items.len(), 1);
        }
        _ => panic!("Expected Array"),
    }
}

#[test]
fn test_array_concat_does_not_modify_original() {
    let mut arr1 = vec![Value::Number(1.0)];
    let arr2 = Value::Array(vec![Value::Number(2.0)]);
    let _ = call_array_method(&mut arr1, "concat", &[arr2]);
    // Original array should not be modified
    assert_eq!(arr1.len(), 1);
}

#[test]
fn test_array_concat_missing_arg() {
    let mut arr1 = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr1, "concat", &[]);
    assert!(result.is_err());
}

#[test]
fn test_array_concat_wrong_type() {
    let mut arr1 = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr1, "concat", &[Value::Number(2.0)]);
    assert!(result.is_err());
}

// ============================================
// Error Cases
// ============================================

#[test]
fn test_unknown_method() {
    let mut arr = vec![Value::Number(1.0)];
    let result = call_array_method(&mut arr, "unknownMethod", &[]);
    assert!(result.is_err());
}

// ============================================
// Integration Tests (through the interpreter)
// ============================================

#[tokio::test]
async fn test_array_length_integration() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_join_integration() {
    let source = r#"
        fn test() {
            let arr = ["a", "b", "c"]
            return arr.join("-")
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let llm = gent::runtime::llm::MockLLMClient::new();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &llm, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}
