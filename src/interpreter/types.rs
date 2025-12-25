//! Value types for the GENT interpreter

use std::fmt;

/// Runtime values in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// String value
    String(String),
    /// Numeric value (f64)
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Null/none value
    Null,
    /// Agent value
    Agent(AgentValue),
    /// Array value
    Array(Vec<Value>),
}

/// Represents a defined agent at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct AgentValue {
    /// Name of the agent
    pub name: String,
    /// System prompt for the agent
    pub prompt: String,
    /// Tools available to this agent
    pub tools: Vec<String>,
    /// Maximum steps before stopping (None = default 10)
    pub max_steps: Option<u32>,
    /// Model to use (None = default)
    pub model: Option<String>,
}

impl AgentValue {
    /// Create a new agent value
    pub fn new(name: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prompt: prompt.into(),
            tools: Vec::new(),
            max_steps: None,
            model: None,
        }
    }

    /// Add tools to the agent
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    /// Set max steps
    pub fn with_max_steps(mut self, steps: u32) -> Self {
        self.max_steps = Some(steps);
        self
    }

    /// Set model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Agent(agent) => write!(f, "<agent {}>", agent.name),
            Value::Array(items) => {
                let formatted: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", formatted.join(", "))
            }
        }
    }
}

impl fmt::Display for AgentValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<agent {}>", self.name)
    }
}

impl Value {
    /// Check if value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Agent(_) => true,
            Value::Array(items) => !items.is_empty(),
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::String(_) => "String",
            Value::Number(_) => "Number",
            Value::Boolean(_) => "Boolean",
            Value::Null => "Null",
            Value::Agent(_) => "Agent",
            Value::Array(_) => "Array",
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<&String> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as agent
    pub fn as_agent(&self) -> Option<&AgentValue> {
        match self {
            Value::Agent(a) => Some(a),
            _ => None,
        }
    }

    /// Try to get as array
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }
}
