//! Tests for expression evaluation

use gent::interpreter::{evaluate_expr, Environment, Value};
use gent::parser::ast::{BinaryOp, Expression, StringPart, UnaryOp};
use gent::Span;
use std::collections::HashMap;

/// Helper to create a simple string expression (single literal)
fn string_expr(s: &str, span: Span) -> Expression {
    Expression::String(vec![StringPart::Literal(s.to_string())], span)
}

#[test]
fn test_eval_number() {
    let env = Environment::new();
    let expr = Expression::Number(42.0, Span::new(0, 2));
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_eval_string() {
    let env = Environment::new();
    let expr = string_expr("hello", Span::new(0, 7));
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::String("hello".to_string()));
}

#[test]
fn test_eval_boolean() {
    let env = Environment::new();
    let expr = Expression::Boolean(true, Span::new(0, 4));
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_null() {
    let env = Environment::new();
    let expr = Expression::Null(Span::new(0, 4));
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Null);
}

#[test]
fn test_eval_identifier() {
    let mut env = Environment::new();
    env.define("x", Value::Number(10.0));

    let expr = Expression::Identifier("x".to_string(), Span::new(0, 1));
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(10.0));
}

#[test]
fn test_eval_undefined_variable() {
    let env = Environment::new();
    let expr = Expression::Identifier("unknown".to_string(), Span::new(0, 7));
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());

    // Check it's the right error type
    let err = result.unwrap_err();
    assert!(matches!(err, gent::GentError::UndefinedVariable { .. }));
}

#[test]
fn test_eval_array_literal() {
    let env = Environment::new();
    let expr = Expression::Array(
        vec![
            Expression::Number(1.0, Span::new(1, 2)),
            Expression::Number(2.0, Span::new(4, 5)),
            Expression::Number(3.0, Span::new(7, 8)),
        ],
        Span::new(0, 9),
    );
    let result = evaluate_expr(&expr, &env).unwrap();

    match result {
        Value::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0], Value::Number(1.0));
            assert_eq!(items[1], Value::Number(2.0));
            assert_eq!(items[2], Value::Number(3.0));
        }
        _ => panic!("Expected Array value"),
    }
}

#[test]
fn test_eval_object_literal() {
    let env = Environment::new();
    let expr = Expression::Object(
        vec![
            (
                "name".to_string(),
                string_expr("John", Span::new(7, 13)),
            ),
            (
                "age".to_string(),
                Expression::Number(30.0, Span::new(20, 22)),
            ),
        ],
        Span::new(0, 24),
    );
    let result = evaluate_expr(&expr, &env).unwrap();

    match result {
        Value::Object(map) => {
            assert_eq!(map.len(), 2);
            assert_eq!(map.get("name"), Some(&Value::String("John".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Number(30.0)));
        }
        _ => panic!("Expected Object value"),
    }
}

// Binary operators - Arithmetic
#[test]
fn test_eval_add_numbers() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Add,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(5.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_eval_add_strings() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Add,
        Box::new(string_expr("hello", Span::new(0, 7))),
        Box::new(string_expr(" world", Span::new(10, 18))),
        Span::new(0, 18),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::String("hello world".to_string()));
}

#[test]
fn test_eval_add_string_number() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Add,
        Box::new(string_expr("value: ", Span::new(0, 9))),
        Box::new(Expression::Number(42.0, Span::new(12, 14))),
        Span::new(0, 14),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::String("value: 42".to_string()));
}

#[test]
fn test_eval_sub() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Sub,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(3.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(7.0));
}

#[test]
fn test_eval_mul() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Mul,
        Box::new(Expression::Number(4.0, Span::new(0, 1))),
        Box::new(Expression::Number(5.0, Span::new(4, 5))),
        Span::new(0, 5),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn test_eval_div() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Div,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(2.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_eval_div_by_zero() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Div,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(0.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::DivisionByZero { .. }
    ));
}

#[test]
fn test_eval_mod() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Mod,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(3.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_eval_mod_by_zero() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Mod,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(0.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::DivisionByZero { .. }
    ));
}

// Binary operators - Comparison
#[test]
fn test_eval_eq_same_type() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Eq,
        Box::new(Expression::Number(5.0, Span::new(0, 1))),
        Box::new(Expression::Number(5.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_eq_different_values() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Eq,
        Box::new(Expression::Number(5.0, Span::new(0, 1))),
        Box::new(Expression::Number(10.0, Span::new(5, 7))),
        Span::new(0, 7),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_eval_ne() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Ne,
        Box::new(Expression::Number(5.0, Span::new(0, 1))),
        Box::new(Expression::Number(10.0, Span::new(5, 7))),
        Span::new(0, 7),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_lt() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Lt,
        Box::new(Expression::Number(3.0, Span::new(0, 1))),
        Box::new(Expression::Number(5.0, Span::new(4, 5))),
        Span::new(0, 5),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_le() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Le,
        Box::new(Expression::Number(5.0, Span::new(0, 1))),
        Box::new(Expression::Number(5.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_gt() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Gt,
        Box::new(Expression::Number(10.0, Span::new(0, 2))),
        Box::new(Expression::Number(5.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_ge() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Ge,
        Box::new(Expression::Number(5.0, Span::new(0, 1))),
        Box::new(Expression::Number(5.0, Span::new(5, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

// Binary operators - Logical
#[test]
fn test_eval_and_true() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::And,
        Box::new(Expression::Boolean(true, Span::new(0, 4))),
        Box::new(Expression::Boolean(true, Span::new(8, 12))),
        Span::new(0, 12),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_and_false() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::And,
        Box::new(Expression::Boolean(true, Span::new(0, 4))),
        Box::new(Expression::Boolean(false, Span::new(8, 13))),
        Span::new(0, 13),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_eval_or_true() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Or,
        Box::new(Expression::Boolean(false, Span::new(0, 5))),
        Box::new(Expression::Boolean(true, Span::new(9, 13))),
        Span::new(0, 13),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_or_false() {
    let env = Environment::new();
    let expr = Expression::Binary(
        BinaryOp::Or,
        Box::new(Expression::Boolean(false, Span::new(0, 5))),
        Box::new(Expression::Boolean(false, Span::new(9, 14))),
        Span::new(0, 14),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// Unary operators
#[test]
fn test_eval_not_true() {
    let env = Environment::new();
    let expr = Expression::Unary(
        UnaryOp::Not,
        Box::new(Expression::Boolean(true, Span::new(1, 5))),
        Span::new(0, 5),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_eval_not_false() {
    let env = Environment::new();
    let expr = Expression::Unary(
        UnaryOp::Not,
        Box::new(Expression::Boolean(false, Span::new(1, 6))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_eval_neg_number() {
    let env = Environment::new();
    let expr = Expression::Unary(
        UnaryOp::Neg,
        Box::new(Expression::Number(5.0, Span::new(1, 2))),
        Span::new(0, 2),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(-5.0));
}

// Member access
#[test]
fn test_eval_member_access() {
    let mut obj = HashMap::new();
    obj.insert("name".to_string(), Value::String("John".to_string()));
    obj.insert("age".to_string(), Value::Number(30.0));

    let mut env = Environment::new();
    env.define("person", Value::Object(obj));

    let expr = Expression::Member(
        Box::new(Expression::Identifier(
            "person".to_string(),
            Span::new(0, 6),
        )),
        "name".to_string(),
        Span::new(0, 11),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::String("John".to_string()));
}

#[test]
fn test_eval_member_access_undefined_property() {
    let obj = HashMap::new();

    let mut env = Environment::new();
    env.define("person", Value::Object(obj));

    let expr = Expression::Member(
        Box::new(Expression::Identifier(
            "person".to_string(),
            Span::new(0, 6),
        )),
        "name".to_string(),
        Span::new(0, 11),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::UndefinedProperty { .. }
    ));
}

#[test]
fn test_eval_member_access_not_object() {
    let mut env = Environment::new();
    env.define("x", Value::Number(42.0));

    let expr = Expression::Member(
        Box::new(Expression::Identifier("x".to_string(), Span::new(0, 1))),
        "foo".to_string(),
        Span::new(0, 5),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::UndefinedProperty { .. }
    ));
}

// Index access
#[test]
fn test_eval_index_access_array() {
    let mut env = Environment::new();
    env.define(
        "arr",
        Value::Array(vec![
            Value::Number(10.0),
            Value::Number(20.0),
            Value::Number(30.0),
        ]),
    );

    let expr = Expression::Index(
        Box::new(Expression::Identifier("arr".to_string(), Span::new(0, 3))),
        Box::new(Expression::Number(1.0, Span::new(4, 5))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::Number(20.0));
}

#[test]
fn test_eval_index_access_object() {
    let mut obj = HashMap::new();
    obj.insert("key".to_string(), Value::String("value".to_string()));

    let mut env = Environment::new();
    env.define("obj", Value::Object(obj));

    let expr = Expression::Index(
        Box::new(Expression::Identifier("obj".to_string(), Span::new(0, 3))),
        Box::new(string_expr("key", Span::new(4, 9))),
        Span::new(0, 10),
    );
    let result = evaluate_expr(&expr, &env).unwrap();
    assert_eq!(result, Value::String("value".to_string()));
}

#[test]
fn test_eval_index_out_of_bounds() {
    let mut env = Environment::new();
    env.define("arr", Value::Array(vec![Value::Number(10.0)]));

    let expr = Expression::Index(
        Box::new(Expression::Identifier("arr".to_string(), Span::new(0, 3))),
        Box::new(Expression::Number(5.0, Span::new(4, 5))),
        Span::new(0, 6),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::IndexOutOfBounds { .. }
    ));
}

#[test]
fn test_eval_index_not_indexable() {
    let mut env = Environment::new();
    env.define("x", Value::Number(42.0));

    let expr = Expression::Index(
        Box::new(Expression::Identifier("x".to_string(), Span::new(0, 1))),
        Box::new(Expression::Number(0.0, Span::new(2, 3))),
        Span::new(0, 4),
    );
    let result = evaluate_expr(&expr, &env);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        gent::GentError::NotIndexable { .. }
    ));
}

// Complex expressions
#[test]
fn test_eval_nested_arithmetic() {
    let _env = Environment::new();
    // (10 + 5) * 2 = 30
    let expr = Expression::Binary(
        BinaryOp::Mul,
        Box::new(Expression::Binary(
            BinaryOp::Add,
            Box::new(Expression::Number(10.0, Span::new(1, 3))),
            Box::new(Expression::Number(5.0, Span::new(6, 7))),
            Span::new(1, 7),
        )),
        Box::new(Expression::Number(2.0, Span::new(11, 12))),
        Span::new(0, 12),
    );
    let result = evaluate_expr(&expr, &_env).unwrap();
    assert_eq!(result, Value::Number(30.0));
}

#[test]
fn test_eval_call_requires_async() {
    let _env = Environment::new();
    let expr = Expression::Call(
        Box::new(Expression::Identifier(
            "someFunc".to_string(),
            Span::new(0, 8),
        )),
        vec![],
        Span::new(0, 10),
    );
    let result = evaluate_expr(&expr, &_env);
    assert!(result.is_err());
    // Should return error indicating function calls require async context
}
