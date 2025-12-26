//! Environment for scoped variable storage

use crate::interpreter::types::EnumDef;
use crate::interpreter::Value;
use std::collections::HashMap;

/// Scoped environment for storing variables
#[derive(Debug, Clone)]
pub struct Environment {
    /// Stack of scopes (innermost scope is last)
    scopes: Vec<HashMap<String, Value>>,
    /// Enum type definitions
    enums: HashMap<String, EnumDef>,
}

impl Environment {
    /// Create a new environment with a global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            enums: HashMap::new(),
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: impl Into<String>, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.into(), value);
        }
    }

    /// Get a variable from any scope (innermost first)
    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Set an existing variable in the nearest scope where it exists
    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    /// Push a new scope onto the stack
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pop the current scope from the stack
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Check if a variable exists in any scope
    pub fn contains(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    /// Get the current scope depth
    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    /// Define an enum type
    pub fn define_enum(&mut self, def: EnumDef) {
        self.enums.insert(def.name.clone(), def);
    }

    /// Get an enum definition
    pub fn get_enum(&self, name: &str) -> Option<&EnumDef> {
        self.enums.get(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
