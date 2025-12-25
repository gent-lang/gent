use gent::interpreter::{AgentValue, Value};
use gent::interpreter::types::UserToolValue;
use gent::parser::ast::{Block, Param, TypeName};
use gent::Span;
use std::collections::HashMap;

// ============================================
// Value Creation Tests
// ============================================

#[test]
fn test_value_string() {
    let val = Value::String("hello".to_string());
    match val {
        Value::String(s) => assert_eq!(s, "hello"),
        _ => panic!("Expected String"),
    }
}

#[test]
fn test_value_number_integer() {
    let val = Value::Number(42.0);
    match val {
        Value::Number(n) => assert_eq!(n, 42.0),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_value_number_float() {
    let val = Value::Number(3.14);
    match val {
        Value::Number(n) => assert!((n - 3.14).abs() < 0.001),
        _ => panic!("Expected Number"),
    }
}

#[test]
fn test_value_boolean_true() {
    let val = Value::Boolean(true);
    match val {
        Value::Boolean(b) => assert!(b),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_value_boolean_false() {
    let val = Value::Boolean(false);
    match val {
        Value::Boolean(b) => assert!(!b),
        _ => panic!("Expected Boolean"),
    }
}

#[test]
fn test_value_null() {
    let val = Value::Null;
    assert!(matches!(val, Value::Null));
}

#[test]
fn test_value_agent() {
    let agent = AgentValue::new("Hello", "You are friendly.");
    let val = Value::Agent(agent);
    match val {
        Value::Agent(a) => {
            assert_eq!(a.name, "Hello");
            assert_eq!(a.prompt, "You are friendly.");
        }
        _ => panic!("Expected Agent"),
    }
}

// ============================================
// AgentValue Tests
// ============================================

#[test]
fn test_agent_value_new() {
    let agent = AgentValue::new("Bot", "Help users.");
    assert_eq!(agent.name, "Bot");
    assert_eq!(agent.prompt, "Help users.");
}

#[test]
fn test_agent_value_new_string_conversion() {
    let agent = AgentValue::new(String::from("Bot"), String::from("Help."));
    assert_eq!(agent.name, "Bot");
    assert_eq!(agent.prompt, "Help.");
}

#[test]
fn test_agent_value_clone() {
    let a1 = AgentValue::new("Test", "Prompt");
    let a2 = a1.clone();
    assert_eq!(a1, a2);
}

#[test]
fn test_agent_value_equality() {
    let a1 = AgentValue::new("Test", "Prompt");
    let a2 = AgentValue::new("Test", "Prompt");
    let a3 = AgentValue::new("Other", "Prompt");
    assert_eq!(a1, a2);
    assert_ne!(a1, a3);
}

// ============================================
// Display Tests
// ============================================

#[test]
fn test_display_string() {
    let val = Value::String("hello world".to_string());
    assert_eq!(format!("{}", val), "hello world");
}

#[test]
fn test_display_number_integer() {
    let val = Value::Number(42.0);
    assert_eq!(format!("{}", val), "42");
}

#[test]
fn test_display_number_float() {
    let val = Value::Number(3.14);
    assert_eq!(format!("{}", val), "3.14");
}

#[test]
fn test_display_boolean_true() {
    let val = Value::Boolean(true);
    assert_eq!(format!("{}", val), "true");
}

#[test]
fn test_display_boolean_false() {
    let val = Value::Boolean(false);
    assert_eq!(format!("{}", val), "false");
}

#[test]
fn test_display_null() {
    let val = Value::Null;
    assert_eq!(format!("{}", val), "null");
}

#[test]
fn test_display_agent() {
    let val = Value::Agent(AgentValue::new("Hello", "prompt"));
    assert_eq!(format!("{}", val), "<agent Hello>");
}

#[test]
fn test_display_agent_value() {
    let agent = AgentValue::new("MyBot", "prompt");
    assert_eq!(format!("{}", agent), "<agent MyBot>");
}

// ============================================
// Truthy Tests
// ============================================

#[test]
fn test_truthy_boolean_true() {
    assert!(Value::Boolean(true).is_truthy());
}

#[test]
fn test_truthy_boolean_false() {
    assert!(!Value::Boolean(false).is_truthy());
}

#[test]
fn test_truthy_null() {
    assert!(!Value::Null.is_truthy());
}

#[test]
fn test_truthy_string_empty() {
    assert!(!Value::String("".to_string()).is_truthy());
}

#[test]
fn test_truthy_string_nonempty() {
    assert!(Value::String("hello".to_string()).is_truthy());
}

#[test]
fn test_truthy_number_zero() {
    assert!(!Value::Number(0.0).is_truthy());
}

#[test]
fn test_truthy_number_nonzero() {
    assert!(Value::Number(42.0).is_truthy());
}

#[test]
fn test_truthy_number_negative() {
    assert!(Value::Number(-1.0).is_truthy());
}

#[test]
fn test_truthy_agent() {
    assert!(Value::Agent(AgentValue::new("A", "p")).is_truthy());
}

// ============================================
// Type Name Tests
// ============================================

#[test]
fn test_type_name_string() {
    assert_eq!(Value::String("x".to_string()).type_name(), "String");
}

#[test]
fn test_type_name_number() {
    assert_eq!(Value::Number(1.0).type_name(), "Number");
}

#[test]
fn test_type_name_boolean() {
    assert_eq!(Value::Boolean(true).type_name(), "Boolean");
}

#[test]
fn test_type_name_null() {
    assert_eq!(Value::Null.type_name(), "Null");
}

#[test]
fn test_type_name_agent() {
    assert_eq!(Value::Agent(AgentValue::new("A", "p")).type_name(), "Agent");
}

// ============================================
// Accessor Tests
// ============================================

#[test]
fn test_as_string_success() {
    let val = Value::String("test".to_string());
    assert_eq!(val.as_string(), Some(&"test".to_string()));
}

#[test]
fn test_as_string_failure() {
    let val = Value::Number(42.0);
    assert_eq!(val.as_string(), None);
}

#[test]
fn test_as_agent_success() {
    let agent = AgentValue::new("Bot", "prompt");
    let val = Value::Agent(agent.clone());
    assert_eq!(val.as_agent(), Some(&agent));
}

#[test]
fn test_as_agent_failure() {
    let val = Value::String("test".to_string());
    assert_eq!(val.as_agent(), None);
}

#[test]
fn test_as_array_success() {
    let arr = vec![Value::Number(1.0), Value::Number(2.0)];
    let val = Value::Array(arr.clone());
    assert_eq!(val.as_array(), Some(&arr));
}

#[test]
fn test_as_array_failure() {
    let val = Value::String("test".to_string());
    assert_eq!(val.as_array(), None);
}

#[test]
fn test_as_object_success() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Number(42.0));
    let val = Value::Object(map.clone());
    assert_eq!(val.as_object(), Some(&map));
}

#[test]
fn test_as_object_failure() {
    let val = Value::String("test".to_string());
    assert_eq!(val.as_object(), None);
}

// ============================================
// Clone and Equality Tests
// ============================================

#[test]
fn test_value_clone() {
    let v1 = Value::String("test".to_string());
    let v2 = v1.clone();
    assert_eq!(v1, v2);
}

#[test]
fn test_value_equality() {
    let v1 = Value::Number(42.0);
    let v2 = Value::Number(42.0);
    let v3 = Value::Number(43.0);
    assert_eq!(v1, v2);
    assert_ne!(v1, v3);
}

#[test]
fn test_value_debug() {
    let val = Value::String("test".to_string());
    let debug = format!("{:?}", val);
    assert!(debug.contains("String"));
    assert!(debug.contains("test"));
}

// ============================================
// Array Tests
// ============================================

#[test]
fn test_array_value_creation() {
    let arr = Value::Array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::String("three".to_string()),
    ]);
    assert!(matches!(arr, Value::Array(_)));
}

#[test]
fn test_array_display() {
    let arr = Value::Array(vec![Value::Number(1.0), Value::Number(2.0)]);
    assert_eq!(format!("{}", arr), "[1, 2]");
}

#[test]
fn test_array_display_empty() {
    let arr = Value::Array(vec![]);
    assert_eq!(format!("{}", arr), "[]");
}

#[test]
fn test_array_type_name() {
    let arr = Value::Array(vec![]);
    assert_eq!(arr.type_name(), "Array");
}

#[test]
fn test_array_is_truthy() {
    let empty = Value::Array(vec![]);
    let non_empty = Value::Array(vec![Value::Null]);
    assert!(!empty.is_truthy());
    assert!(non_empty.is_truthy());
}

// ============================================
// Object Tests
// ============================================

#[test]
fn test_object_value_creation() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("Tokyo".to_string()));
    map.insert("temp".to_string(), Value::Number(22.0));
    let obj = Value::Object(map);
    assert!(matches!(obj, Value::Object(_)));
}

#[test]
fn test_object_display() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), Value::Number(1.0));
    let obj = Value::Object(map);
    // Object display shows {key: value} format
    let s = format!("{}", obj);
    assert!(s.contains("a: 1"));
}

#[test]
fn test_object_display_empty() {
    let obj = Value::Object(HashMap::new());
    assert_eq!(format!("{}", obj), "{}");
}

#[test]
fn test_object_type_name() {
    let obj = Value::Object(HashMap::new());
    assert_eq!(obj.type_name(), "Object");
}

#[test]
fn test_object_is_truthy() {
    let empty = Value::Object(HashMap::new());
    let mut map = HashMap::new();
    map.insert("x".to_string(), Value::Null);
    let non_empty = Value::Object(map);
    assert!(!empty.is_truthy());
    assert!(non_empty.is_truthy());
}

// ============================================
// Tool Tests
// ============================================

#[test]
fn test_tool_value_creation() {
    let tool = UserToolValue {
        name: "greet".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![],
            span: Span::new(0, 0),
        },
    };
    let val = Value::Tool(tool);
    assert!(matches!(val, Value::Tool(_)));
}

#[test]
fn test_tool_display() {
    let tool = UserToolValue {
        name: "greet".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![],
            span: Span::new(0, 0),
        },
    };
    let val = Value::Tool(tool);
    assert_eq!(format!("{}", val), "<tool greet>");
}

#[test]
fn test_tool_type_name() {
    let tool = UserToolValue {
        name: "greet".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![],
            span: Span::new(0, 0),
        },
    };
    let val = Value::Tool(tool);
    assert_eq!(val.type_name(), "Tool");
}

#[test]
fn test_tool_is_truthy() {
    let tool = UserToolValue {
        name: "greet".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: vec![],
            span: Span::new(0, 0),
        },
    };
    let val = Value::Tool(tool);
    assert!(val.is_truthy());
}

#[test]
fn test_tool_with_params() {
    let tool = UserToolValue {
        name: "add".to_string(),
        params: vec![
            Param {
                name: "a".to_string(),
                type_name: TypeName::Number,
                span: Span::new(0, 0),
            },
            Param {
                name: "b".to_string(),
                type_name: TypeName::Number,
                span: Span::new(0, 0),
            },
        ],
        return_type: Some(TypeName::Number),
        body: Block {
            statements: vec![],
            span: Span::new(0, 0),
        },
    };
    let val = Value::Tool(tool.clone());
    assert_eq!(val.as_tool().unwrap().params.len(), 2);
}

#[test]
fn test_tool_with_ast_block() {
    use gent::parser::ast::{BlockStmt, Expression, LetStmt, ReturnStmt};

    // Create a tool with a real AST block containing statements
    let tool = UserToolValue {
        name: "calculate".to_string(),
        params: vec![
            Param {
                name: "x".to_string(),
                type_name: TypeName::Number,
                span: Span::new(0, 1),
            },
        ],
        return_type: Some(TypeName::Number),
        body: Block {
            statements: vec![
                // let result = x + 10
                BlockStmt::Let(LetStmt {
                    name: "result".to_string(),
                    value: Expression::Number(42.0, Span::new(10, 12)),
                    span: Span::new(5, 12),
                }),
                // return result
                BlockStmt::Return(ReturnStmt {
                    value: Some(Expression::Identifier("result".to_string(), Span::new(20, 26))),
                    span: Span::new(15, 26),
                }),
            ],
            span: Span::new(0, 30),
        },
    };

    let val = Value::Tool(tool.clone());

    // Verify the tool has the correct structure
    assert_eq!(val.as_tool().unwrap().name, "calculate");
    assert_eq!(val.as_tool().unwrap().params.len(), 1);
    assert_eq!(val.as_tool().unwrap().params[0].name, "x");
    assert_eq!(val.as_tool().unwrap().body.statements.len(), 2);
    assert!(matches!(val.as_tool().unwrap().body.statements[0], BlockStmt::Let(_)));
    assert!(matches!(val.as_tool().unwrap().body.statements[1], BlockStmt::Return(_)));
}
