//! Value types for the GENT interpreter

use crate::parser::ast::{
    Block, FieldType, OutputType, Param, StructField, TypeName as ParserTypeName,
};
use std::collections::HashMap;
use std::fmt;

/// Runtime representation of an output schema
#[derive(Debug, Clone, PartialEq)]
pub struct OutputSchema {
    pub fields: Vec<StructField>,
}

impl OutputSchema {
    pub fn from_output_type(
        output_type: &OutputType,
        structs: &HashMap<String, Vec<StructField>>,
    ) -> Result<Self, String> {
        match output_type {
            OutputType::Inline(fields) => Ok(OutputSchema {
                fields: fields.clone(),
            }),
            OutputType::Named(name) => structs
                .get(name)
                .map(|fields| OutputSchema {
                    fields: fields.clone(),
                })
                .ok_or_else(|| format!("Unknown struct: {}", name)),
        }
    }

    pub fn to_json_schema(&self) -> serde_json::Value {
        use serde_json::json;

        let properties: serde_json::Map<String, serde_json::Value> = self
            .fields
            .iter()
            .map(|f| (f.name.clone(), field_type_to_json_schema(&f.field_type)))
            .collect();

        let required: Vec<String> = self.fields.iter().map(|f| f.name.clone()).collect();

        json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }
}

fn field_type_to_json_schema(ft: &FieldType) -> serde_json::Value {
    use serde_json::json;
    match ft {
        FieldType::String => json!({"type": "string"}),
        FieldType::Number => json!({"type": "number"}),
        FieldType::Boolean => json!({"type": "boolean"}),
        FieldType::Array(inner) => json!({
            "type": "array",
            "items": field_type_to_json_schema(inner)
        }),
        FieldType::Object(fields) => {
            let properties: serde_json::Map<String, serde_json::Value> = fields
                .iter()
                .map(|f| (f.name.clone(), field_type_to_json_schema(&f.field_type)))
                .collect();
            let required: Vec<String> = fields.iter().map(|f| f.name.clone()).collect();
            json!({
                "type": "object",
                "properties": properties,
                "required": required
            })
        }
        FieldType::Named(name) => json!({"$ref": format!("#/definitions/{}", name)}),
    }
}

/// Represents a user-defined tool at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct UserToolValue {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<ParserTypeName>,
    pub body: Block,
}

/// Represents a user-defined function at runtime (pure, no agent access)
#[derive(Debug, Clone, PartialEq)]
pub struct FnValue {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<ParserTypeName>,
    pub body: Block,
}

/// Represents a lambda/closure at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaValue {
    pub params: Vec<String>,
    pub body: crate::parser::ast::LambdaBody,
}

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
    /// Object value (key-value map)
    Object(HashMap<String, Value>),
    /// User-defined tool
    Tool(UserToolValue),
    /// User-defined function (pure, no agent access)
    Function(FnValue),
    /// Lambda/closure value
    Lambda(LambdaValue),
}

/// Represents a defined agent at runtime
#[derive(Debug, Clone, PartialEq)]
pub struct AgentValue {
    /// Name of the agent
    pub name: String,
    /// System prompt for the agent
    pub system_prompt: String,
    /// User prompt for the agent (optional)
    pub user_prompt: Option<String>,
    /// Tools available to this agent
    pub tools: Vec<String>,
    /// Maximum steps before stopping (None = default 10)
    pub max_steps: Option<u32>,
    /// Model to use (None = default)
    pub model: Option<String>,
    /// Output schema for structured responses
    pub output_schema: Option<OutputSchema>,
    /// Number of retries for output validation
    pub output_retries: u32,
    /// Custom instructions for schema output (None = default)
    pub output_instructions: Option<String>,
    /// Custom prompt for validation retries (None = default)
    pub retry_prompt: Option<String>,
}

impl AgentValue {
    /// Create a new agent value
    pub fn new(name: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            system_prompt: system_prompt.into(),
            user_prompt: None,
            tools: Vec::new(),
            max_steps: None,
            model: None,
            output_schema: None,
            output_retries: 1, // default: retry once
            output_instructions: None,
            retry_prompt: None,
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

    /// Set output schema
    pub fn with_output_schema(mut self, schema: OutputSchema) -> Self {
        self.output_schema = Some(schema);
        self
    }

    /// Set output retries
    pub fn with_output_retries(mut self, retries: u32) -> Self {
        self.output_retries = retries;
        self
    }

    /// Set custom output instructions
    pub fn with_output_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.output_instructions = Some(instructions.into());
        self
    }

    /// Set custom retry prompt
    pub fn with_retry_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.retry_prompt = Some(prompt.into());
        self
    }

    /// Set user prompt
    pub fn with_user_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.user_prompt = Some(prompt.into());
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
            Value::Object(map) => {
                let formatted: Vec<String> =
                    map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", formatted.join(", "))
            }
            Value::Tool(t) => write!(f, "<tool {}>", t.name),
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Lambda(_) => write!(f, "<lambda>"),
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
            Value::Object(map) => !map.is_empty(),
            Value::Tool(_) => true,
            Value::Function(_) => true,
            Value::Lambda(_) => true,
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
            Value::Object(_) => "Object",
            Value::Tool(_) => "Tool",
            Value::Function(_) => "Function",
            Value::Lambda(_) => "Lambda",
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

    /// Try to get as object
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Try to get as tool
    pub fn as_tool(&self) -> Option<&UserToolValue> {
        match self {
            Value::Tool(t) => Some(t),
            _ => None,
        }
    }

    /// Try to get as function
    pub fn as_function(&self) -> Option<&FnValue> {
        match self {
            Value::Function(f) => Some(f),
            _ => None,
        }
    }
}
