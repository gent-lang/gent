use gent::interpreter::{AgentValue, Value};

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
