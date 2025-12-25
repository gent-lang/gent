use gent::interpreter::{AgentValue, Environment, Value};

// ============================================
// Basic Creation Tests
// ============================================

#[test]
fn test_environment_new() {
    let env = Environment::new();
    assert_eq!(env.depth(), 1);
}

#[test]
fn test_environment_default() {
    let env = Environment::default();
    assert_eq!(env.depth(), 1);
}

// ============================================
// Define and Get Tests
// ============================================

#[test]
fn test_define_and_get_string() {
    let mut env = Environment::new();
    env.define("name", Value::String("hello".to_string()));
    assert_eq!(env.get("name"), Some(&Value::String("hello".to_string())));
}

#[test]
fn test_define_and_get_number() {
    let mut env = Environment::new();
    env.define("x", Value::Number(42.0));
    assert_eq!(env.get("x"), Some(&Value::Number(42.0)));
}

#[test]
fn test_define_and_get_boolean() {
    let mut env = Environment::new();
    env.define("flag", Value::Boolean(true));
    assert_eq!(env.get("flag"), Some(&Value::Boolean(true)));
}

#[test]
fn test_define_and_get_null() {
    let mut env = Environment::new();
    env.define("nothing", Value::Null);
    assert_eq!(env.get("nothing"), Some(&Value::Null));
}

#[test]
fn test_define_and_get_agent() {
    let mut env = Environment::new();
    let agent = AgentValue::new("Bot", "You help.");
    env.define("myAgent", Value::Agent(agent.clone()));
    match env.get("myAgent") {
        Some(Value::Agent(a)) => {
            assert_eq!(a.name, "Bot");
            assert_eq!(a.prompt, "You help.");
        }
        _ => panic!("Expected Agent"),
    }
}

#[test]
fn test_get_undefined_returns_none() {
    let env = Environment::new();
    assert_eq!(env.get("undefined"), None);
}

#[test]
fn test_define_with_string_conversion() {
    let mut env = Environment::new();
    env.define(String::from("key"), Value::Number(1.0));
    assert!(env.get("key").is_some());
}

// ============================================
// Overwrite Tests
// ============================================

#[test]
fn test_define_overwrites_in_same_scope() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    env.define("x", Value::Number(2.0));
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
}

// ============================================
// Set Tests
// ============================================

#[test]
fn test_set_existing_variable() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    let result = env.set("x", Value::Number(2.0));
    assert!(result);
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
}

#[test]
fn test_set_undefined_returns_false() {
    let mut env = Environment::new();
    let result = env.set("undefined", Value::Number(1.0));
    assert!(!result);
}

#[test]
fn test_set_updates_correct_scope() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    env.push_scope();
    env.set("x", Value::Number(2.0));
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
    env.pop_scope();
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
}

// ============================================
// Scope Tests
// ============================================

#[test]
fn test_push_scope_increases_depth() {
    let mut env = Environment::new();
    assert_eq!(env.depth(), 1);
    env.push_scope();
    assert_eq!(env.depth(), 2);
    env.push_scope();
    assert_eq!(env.depth(), 3);
}

#[test]
fn test_pop_scope_decreases_depth() {
    let mut env = Environment::new();
    env.push_scope();
    env.push_scope();
    assert_eq!(env.depth(), 3);
    env.pop_scope();
    assert_eq!(env.depth(), 2);
    env.pop_scope();
    assert_eq!(env.depth(), 1);
}

#[test]
fn test_pop_scope_cannot_remove_global() {
    let mut env = Environment::new();
    env.pop_scope();
    assert_eq!(env.depth(), 1);
    env.pop_scope();
    assert_eq!(env.depth(), 1);
}

#[test]
fn test_inner_scope_shadows_outer() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    env.push_scope();
    env.define("x", Value::Number(2.0));
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
}

#[test]
fn test_inner_scope_can_access_outer() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    env.push_scope();
    assert_eq!(env.get("x"), Some(&Value::Number(1.0)));
}

#[test]
fn test_pop_scope_removes_inner_variables() {
    let mut env = Environment::new();
    env.push_scope();
    env.define("inner", Value::Number(1.0));
    assert!(env.get("inner").is_some());
    env.pop_scope();
    assert!(env.get("inner").is_none());
}

#[test]
fn test_pop_scope_restores_shadowed_variable() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    env.push_scope();
    env.define("x", Value::Number(2.0));
    assert_eq!(env.get("x"), Some(&Value::Number(2.0)));
    env.pop_scope();
    assert_eq!(env.get("x"), Some(&Value::Number(1.0)));
}

// ============================================
// Contains Tests
// ============================================

#[test]
fn test_contains_existing() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    assert!(env.contains("x"));
}

#[test]
fn test_contains_undefined() {
    let env = Environment::new();
    assert!(!env.contains("undefined"));
}

#[test]
fn test_contains_in_outer_scope() {
    let mut env = Environment::new();
    env.define("outer", Value::Number(1.0));
    env.push_scope();
    assert!(env.contains("outer"));
}

// ============================================
// Clone Tests
// ============================================

#[test]
fn test_environment_clone() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    let env2 = env.clone();
    assert_eq!(env2.get("x"), Some(&Value::Number(1.0)));
}

#[test]
fn test_cloned_environment_independent() {
    let mut env = Environment::new();
    env.define("x", Value::Number(1.0));
    let mut env2 = env.clone();
    env2.define("x", Value::Number(2.0));
    assert_eq!(env.get("x"), Some(&Value::Number(1.0)));
    assert_eq!(env2.get("x"), Some(&Value::Number(2.0)));
}

// ============================================
// Multiple Variables Tests
// ============================================

#[test]
fn test_multiple_variables_same_scope() {
    let mut env = Environment::new();
    env.define("a", Value::Number(1.0));
    env.define("b", Value::Number(2.0));
    env.define("c", Value::Number(3.0));
    assert_eq!(env.get("a"), Some(&Value::Number(1.0)));
    assert_eq!(env.get("b"), Some(&Value::Number(2.0)));
    assert_eq!(env.get("c"), Some(&Value::Number(3.0)));
}

#[test]
fn test_multiple_scopes() {
    let mut env = Environment::new();
    env.define("global", Value::String("g".to_string()));
    env.push_scope();
    env.define("local1", Value::String("l1".to_string()));
    env.push_scope();
    env.define("local2", Value::String("l2".to_string()));

    // All should be visible
    assert!(env.contains("global"));
    assert!(env.contains("local1"));
    assert!(env.contains("local2"));

    env.pop_scope();
    assert!(env.contains("global"));
    assert!(env.contains("local1"));
    assert!(!env.contains("local2"));

    env.pop_scope();
    assert!(env.contains("global"));
    assert!(!env.contains("local1"));
    assert!(!env.contains("local2"));
}

// ============================================
// Debug Tests
// ============================================

#[test]
fn test_environment_debug() {
    let mut env = Environment::new();
    env.define("x", Value::Number(42.0));
    let debug = format!("{:?}", env);
    assert!(debug.contains("Environment"));
}
