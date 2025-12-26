//! Expression evaluation module
//!
//! This module provides synchronous expression evaluation without tool calls.

use crate::errors::{GentError, GentResult};
use crate::interpreter::types::{EnumConstructor, EnumValue};
use crate::interpreter::{Environment, Value};
use crate::parser::ast::{BinaryOp, Expression, StringPart, UnaryOp};
use std::collections::HashMap;

/// Evaluate an expression in the given environment
pub fn evaluate_expr(expr: &Expression, env: &Environment) -> GentResult<Value> {
    match expr {
        // Literals
        Expression::String(parts, _span) => {
            // Evaluate each part and concatenate
            let mut result = String::new();
            for part in parts {
                match part {
                    StringPart::Literal(s) => result.push_str(s),
                    StringPart::Expr(expr) => {
                        let value = evaluate_expr(expr, env)?;
                        result.push_str(&value.to_string());
                    }
                }
            }
            Ok(Value::String(result))
        }
        Expression::Number(n, _) => Ok(Value::Number(*n)),
        Expression::Boolean(b, _) => Ok(Value::Boolean(*b)),
        Expression::Null(_) => Ok(Value::Null),

        // Identifier
        Expression::Identifier(name, span) => {
            env.get(name)
                .cloned()
                .ok_or_else(|| GentError::UndefinedVariable {
                    name: name.clone(),
                    span: span.clone(),
                })
        }

        // Array literal
        Expression::Array(elements, _) => {
            let mut values = Vec::new();
            for elem in elements {
                values.push(evaluate_expr(elem, env)?);
            }
            Ok(Value::Array(values))
        }

        // Object literal
        Expression::Object(fields, _) => {
            let mut map = HashMap::new();
            for (key, value_expr) in fields {
                let value = evaluate_expr(value_expr, env)?;
                map.insert(key.clone(), value);
            }
            Ok(Value::Object(map))
        }

        // Binary operations
        Expression::Binary(op, left, right, span) => {
            let left_val = evaluate_expr(left, env)?;
            let right_val = evaluate_expr(right, env)?;
            evaluate_binary_op(op, left_val, right_val, span)
        }

        // Unary operations
        Expression::Unary(op, operand, span) => {
            let val = evaluate_expr(operand, env)?;
            evaluate_unary_op(op, val, span)
        }

        // Member access: obj.prop or EnumName.Variant
        Expression::Member(object_expr, property, span) => {
            // Check if this is an enum construction: EnumName.Variant
            if let Expression::Identifier(name, _) = object_expr.as_ref() {
                if let Some(enum_def) = env.get_enum(name) {
                    // Find the variant
                    let variant = enum_def.variants.iter().find(|v| v.name == *property);
                    if let Some(v) = variant {
                        if v.fields.is_empty() {
                            // Unit variant - return EnumValue directly
                            return Ok(Value::Enum(EnumValue {
                                enum_name: name.clone(),
                                variant: property.clone(),
                                data: vec![],
                            }));
                        } else {
                            // Variant with fields - return a callable constructor
                            return Ok(Value::EnumConstructor(EnumConstructor {
                                enum_name: name.clone(),
                                variant: property.clone(),
                                expected_fields: v.fields.len(),
                            }));
                        }
                    } else {
                        return Err(GentError::TypeError {
                            expected: format!("valid variant of enum '{}'", name),
                            got: property.clone(),
                            span: span.clone(),
                        });
                    }
                }
            }

            // Regular member access
            let object = evaluate_expr(object_expr, env)?;
            match object {
                Value::Object(map) => {
                    map.get(property)
                        .cloned()
                        .ok_or_else(|| GentError::UndefinedProperty {
                            property: property.clone(),
                            type_name: "Object".to_string(),
                            span: span.clone(),
                        })
                }
                _ => Err(GentError::UndefinedProperty {
                    property: property.clone(),
                    type_name: object.type_name().to_string(),
                    span: span.clone(),
                }),
            }
        }

        // Index access: arr[i] or obj["key"]
        Expression::Index(target_expr, index_expr, span) => {
            let target = evaluate_expr(target_expr, env)?;
            let index = evaluate_expr(index_expr, env)?;

            match target {
                Value::Array(ref items) => {
                    // Array indexing requires a number
                    if let Value::Number(n) = index {
                        let idx = n as i64;
                        if idx < 0 || idx >= items.len() as i64 {
                            return Err(GentError::IndexOutOfBounds {
                                index: idx,
                                length: items.len(),
                                span: span.clone(),
                            });
                        }
                        Ok(items[idx as usize].clone())
                    } else {
                        Err(GentError::NotIndexable {
                            type_name: format!("Array with {} index", index.type_name()),
                            span: span.clone(),
                        })
                    }
                }
                Value::Object(ref map) => {
                    // Object indexing requires a string
                    if let Value::String(key) = index {
                        map.get(&key)
                            .cloned()
                            .ok_or_else(|| GentError::UndefinedProperty {
                                property: key.clone(),
                                type_name: "Object".to_string(),
                                span: span.clone(),
                            })
                    } else {
                        Err(GentError::NotIndexable {
                            type_name: format!("Object with {} index", index.type_name()),
                            span: span.clone(),
                        })
                    }
                }
                _ => Err(GentError::NotIndexable {
                    type_name: target.type_name().to_string(),
                    span: span.clone(),
                }),
            }
        }

        // Function calls require async context
        Expression::Call(_, _, span) => Err(GentError::TypeError {
            expected: "synchronous expression".to_string(),
            got: "function call (requires async context)".to_string(),
            span: span.clone(),
        }),

        // Range expressions - produce an array of numbers
        Expression::Range(start, end, span) => {
            let start_val = evaluate_expr(start, env)?;
            let end_val = evaluate_expr(end, env)?;

            match (start_val, end_val) {
                (Value::Number(s), Value::Number(e)) => {
                    let range: Vec<Value> = (s as i64..e as i64)
                        .map(|n| Value::Number(n as f64))
                        .collect();
                    Ok(Value::Array(range))
                }
                _ => Err(GentError::TypeError {
                    expected: "Number".to_string(),
                    got: "non-number".to_string(),
                    span: span.clone(),
                }),
            }
        }

        // Lambda expressions - evaluate to LambdaValue
        Expression::Lambda(lambda) => {
            Ok(Value::Lambda(crate::interpreter::types::LambdaValue {
                params: lambda.params.clone(),
                body: lambda.body.clone(),
            }))
        }

        // Match expressions require async context for body evaluation
        Expression::Match(match_expr) => Err(GentError::TypeError {
            expected: "synchronous expression".to_string(),
            got: "match expression (requires async context)".to_string(),
            span: match_expr.span.clone(),
        }),
    }
}

/// Evaluate a binary operation
fn evaluate_binary_op(
    op: &BinaryOp,
    left: Value,
    right: Value,
    span: &crate::Span,
) -> GentResult<Value> {
    match op {
        // Arithmetic
        BinaryOp::Add => {
            match (&left, &right) {
                // String concatenation
                (Value::String(s1), Value::String(s2)) => {
                    Ok(Value::String(format!("{}{}", s1, s2)))
                }
                // Number addition
                (Value::Number(n1), Value::Number(n2)) => Ok(Value::Number(n1 + n2)),
                // String + Any = concatenate
                (Value::String(s), other) => Ok(Value::String(format!("{}{}", s, other))),
                (other, Value::String(s)) => Ok(Value::String(format!("{}{}", other, s))),
                _ => Err(GentError::InvalidOperands {
                    op: "+".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                    span: span.clone(),
                }),
            }
        }
        BinaryOp::Sub => binary_arithmetic_op(left, right, span, "-", |a, b| a - b),
        BinaryOp::Mul => binary_arithmetic_op(left, right, span, "*", |a, b| a * b),
        BinaryOp::Div => {
            if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                if *b == 0.0 {
                    return Err(GentError::DivisionByZero { span: span.clone() });
                }
                Ok(Value::Number(a / b))
            } else {
                Err(GentError::InvalidOperands {
                    op: "/".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                    span: span.clone(),
                })
            }
        }
        BinaryOp::Mod => {
            if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
                if *b == 0.0 {
                    return Err(GentError::DivisionByZero { span: span.clone() });
                }
                Ok(Value::Number(a % b))
            } else {
                Err(GentError::InvalidOperands {
                    op: "%".to_string(),
                    left: left.type_name().to_string(),
                    right: right.type_name().to_string(),
                    span: span.clone(),
                })
            }
        }

        // Comparison
        BinaryOp::Eq => Ok(Value::Boolean(values_equal(&left, &right))),
        BinaryOp::Ne => Ok(Value::Boolean(!values_equal(&left, &right))),
        BinaryOp::Lt => binary_comparison_op(left, right, span, "<", |a, b| a < b),
        BinaryOp::Le => binary_comparison_op(left, right, span, "<=", |a, b| a <= b),
        BinaryOp::Gt => binary_comparison_op(left, right, span, ">", |a, b| a > b),
        BinaryOp::Ge => binary_comparison_op(left, right, span, ">=", |a, b| a >= b),

        // Logical
        BinaryOp::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
        BinaryOp::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
    }
}

/// Helper for arithmetic operations
fn binary_arithmetic_op<F>(
    left: Value,
    right: Value,
    span: &crate::Span,
    op_str: &str,
    op: F,
) -> GentResult<Value>
where
    F: FnOnce(f64, f64) -> f64,
{
    if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
        Ok(Value::Number(op(*a, *b)))
    } else {
        Err(GentError::InvalidOperands {
            op: op_str.to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
            span: span.clone(),
        })
    }
}

/// Helper for comparison operations
fn binary_comparison_op<F>(
    left: Value,
    right: Value,
    span: &crate::Span,
    op_str: &str,
    op: F,
) -> GentResult<Value>
where
    F: FnOnce(f64, f64) -> bool,
{
    if let (Value::Number(a), Value::Number(b)) = (&left, &right) {
        Ok(Value::Boolean(op(*a, *b)))
    } else {
        Err(GentError::InvalidOperands {
            op: op_str.to_string(),
            left: left.type_name().to_string(),
            right: right.type_name().to_string(),
            span: span.clone(),
        })
    }
}

/// Check if two values are equal
fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Number(a), Value::Number(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return false;
            }
            a.iter()
                .all(|(k, v)| b.get(k).is_some_and(|v2| values_equal(v, v2)))
        }
        _ => false, // Different types are never equal
    }
}

/// Evaluate a unary operation
fn evaluate_unary_op(op: &UnaryOp, operand: Value, span: &crate::Span) -> GentResult<Value> {
    match op {
        UnaryOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
        UnaryOp::Neg => {
            if let Value::Number(n) = operand {
                Ok(Value::Number(-n))
            } else {
                Err(GentError::InvalidOperands {
                    op: "-".to_string(),
                    left: operand.type_name().to_string(),
                    right: "".to_string(),
                    span: span.clone(),
                })
            }
        }
    }
}
