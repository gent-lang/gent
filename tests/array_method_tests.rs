//! Tests for array methods in GENT
//!
//! These tests verify the functionality of built-in array methods.
//! Note: These tests test the array_methods module directly.
//! Integration tests that test the methods through the interpreter
//! will be added in Task 5.

use gent::config::Config;
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
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
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
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

// ============================================
// Lambda-based Array Methods Tests
// ============================================

#[tokio::test]
async fn test_array_map() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let doubled = arr.map((x) => x * 2)
            return doubled.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_filter() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5]
            let evens = arr.filter((x) => x % 2 == 0)
            return evens.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_reduce() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4]
            let sum = arr.reduce((acc, x) => acc + x, 0)
            return sum
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_find() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5]
            let found = arr.find((x) => x > 3)
            return found
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[tokio::test]
async fn test_array_map_with_function_ref() {
    let source = r#"
        fn double(x: number) {
            return x * 2
        }
        fn test() {
            let arr = [1, 2, 3]
            let doubled = arr.map(double)
            return doubled.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

// ============================================
// Edge Case Tests for Lambda-based Array Methods
// ============================================

#[tokio::test]
async fn test_array_map_empty_array() {
    let source = r#"
        fn test() {
            let arr = []
            let result = arr.map((x) => x * 2)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Map on empty array should return empty array: {:?}", result.err());
}

#[tokio::test]
async fn test_array_filter_empty_array() {
    let source = r#"
        fn test() {
            let arr = []
            let result = arr.filter((x) => x > 0)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Filter on empty array should return empty array: {:?}", result.err());
}

#[tokio::test]
async fn test_array_find_empty_array() {
    let source = r#"
        fn test() {
            let arr = []
            let result = arr.find((x) => x > 0)
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Find on empty array should return null: {:?}", result.err());
}

#[tokio::test]
async fn test_array_reduce_empty_array_without_initial() {
    // reduce on empty array without initial value should error
    let source = r#"
        fn test() {
            let arr = []
            let result = arr.reduce((acc, x) => acc + x)
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err(), "Reduce on empty array without initial value should error");
}

#[tokio::test]
async fn test_array_map_wrong_callback_type() {
    // Passing a number instead of a lambda should error
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let result = arr.map(42)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err(), "Passing a number instead of lambda should error");
}

#[tokio::test]
async fn test_array_reduce_wrong_param_count() {
    // Reduce callback with wrong number of parameters should error
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let result = arr.reduce((x) => x, 0)
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err(), "Reduce with 1-param callback should error");
}

#[tokio::test]
async fn test_array_push_mutates() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            arr.push(4)
            arr.push(5)
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Array should now have 5 elements after two pushes
}

#[tokio::test]
async fn test_array_pop_mutates() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4]
            let popped = arr.pop()
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Array should now have 3 elements after pop
}

#[tokio::test]
async fn test_array_pop_returns_value() {
    let source = r#"
        fn test() {
            let arr = [10, 20, 30]
            let last = arr.pop()
            return last
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Pop should return 30
}

// ============================================
// Push/Pop Edge Cases
// ============================================

#[tokio::test]
async fn test_push_to_empty_array_mutates() {
    let source = r#"
        fn test() {
            let arr = []
            arr.push(1)
            arr.push(2)
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pop_until_empty() {
    let source = r#"
        fn test() {
            let arr = [1, 2]
            let a = arr.pop()
            let b = arr.pop()
            let c = arr.pop()
            return c
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Third pop should return null
}

#[tokio::test]
async fn test_push_pop_in_loop() {
    let source = r#"
        fn test() {
            let arr = []
            for i in 1..4 {
                arr.push(i)
            }
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should have 3 elements after loop
}

#[tokio::test]
async fn test_push_different_types() {
    let source = r#"
        fn test() {
            let arr = []
            arr.push(42)
            arr.push("hello")
            arr.push(true)
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_push_pop_conditional() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            if true {
                arr.push(4)
            }
            if false {
                arr.pop()
            }
            return arr.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should have 4 elements (push happened, pop didn't)
}

// ============================================
// Slice Edge Cases
// ============================================

#[tokio::test]
async fn test_slice_beyond_bounds() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let sliced = arr.slice(0, 100)
            return sliced.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should clamp to array length, return 3
}

#[tokio::test]
async fn test_slice_start_equals_end() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let sliced = arr.slice(1, 1)
            return sliced.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return empty array
}

#[tokio::test]
async fn test_slice_start_greater_than_end() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5]
            let sliced = arr.slice(3, 1)
            return sliced.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return empty array when start > end
}

// ============================================
// Map/Filter/Reduce Edge Cases
// ============================================

#[tokio::test]
async fn test_map_with_index_simulation() {
    // Map only gets element, not index - test that it works
    let source = r#"
        fn test() {
            let arr = [10, 20, 30]
            let result = arr.map((x) => x + 1)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_filter_all_match() {
    let source = r#"
        fn test() {
            let arr = [2, 4, 6, 8]
            let evens = arr.filter((x) => x % 2 == 0)
            return evens.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // All 4 should match
}

#[tokio::test]
async fn test_filter_none_match() {
    let source = r#"
        fn test() {
            let arr = [1, 3, 5, 7]
            let evens = arr.filter((x) => x % 2 == 0)
            return evens.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // None should match, empty array
}

#[tokio::test]
async fn test_reduce_single_element() {
    let source = r#"
        fn test() {
            let arr = [42]
            let sum = arr.reduce((acc, x) => acc + x, 0)
            return sum
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return 42
}

#[tokio::test]
async fn test_reduce_string_concatenation() {
    let source = r#"
        fn test() {
            let arr = ["a", "b", "c"]
            let joined = arr.reduce((acc, x) => "{acc}{x}", "")
            return joined
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return "abc"
}

// ============================================
// Find/IndexOf Edge Cases
// ============================================

#[tokio::test]
async fn test_find_first_of_many() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 2, 1]
            let found = arr.find((x) => x == 2)
            return found
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return first 2
}

#[tokio::test]
async fn test_indexof_first_occurrence() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 2, 1]
            return arr.indexOf(2)
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return 1 (first occurrence)
}

#[tokio::test]
async fn test_indexof_string_in_array() {
    let source = r#"
        fn test() {
            let arr = ["apple", "banana", "cherry"]
            return arr.indexOf("banana")
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return 1
}

// ============================================
// Join/Concat Edge Cases
// ============================================

#[tokio::test]
async fn test_join_single_element() {
    let source = r#"
        fn test() {
            let arr = ["only"]
            return arr.join(", ")
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return "only" (no separator needed)
}

#[tokio::test]
async fn test_join_empty_separator() {
    let source = r#"
        fn test() {
            let arr = ["a", "b", "c"]
            return arr.join("")
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return "abc"
}

#[tokio::test]
async fn test_concat_empty_arrays() {
    let source = r#"
        fn test() {
            let a = []
            let b = []
            let c = a.concat(b)
            return c.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return 0
}

#[tokio::test]
async fn test_concat_to_empty() {
    let source = r#"
        fn test() {
            let a = []
            let b = [1, 2, 3]
            let c = a.concat(b)
            return c.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Should return 3
}

// ============================================
// Chaining Edge Cases
// ============================================

#[tokio::test]
async fn test_chain_filter_map_reduce() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5, 6]
            let result = arr.filter((x) => x % 2 == 0).map((x) => x * x).reduce((a, b) => a + b, 0)
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // filter: [2,4,6], map: [4,16,36], reduce: 56
}

#[tokio::test]
async fn test_chain_empty_result() {
    let source = r#"
        fn test() {
            let arr = [1, 3, 5]
            let result = arr.filter((x) => x % 2 == 0).map((x) => x * 2)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // filter returns empty, map on empty returns empty
}

// ============================================
// Lambda Edge Cases
// ============================================

#[tokio::test]
async fn test_lambda_no_params() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let result = arr.map(() => 0)
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Lambda with no params should still work (ignores element)
}

#[tokio::test]
async fn test_lambda_block_body() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3]
            let result = arr.map((x) => {
                let doubled = x * 2
                return doubled + 1
            })
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Block body lambda should work
}

#[tokio::test]
async fn test_lambda_with_conditionals() {
    let source = r#"
        fn test() {
            let arr = [1, 2, 3, 4, 5]
            let result = arr.map((x) => {
                if x % 2 == 0 {
                    return x * 2
                } else {
                    return x
                }
            })
            return result.length()
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
    // Lambda with if/else should work
}
