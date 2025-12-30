//! Value types for the GENT interpreter

use crate::parser::ast::{
    Block, FieldType, OutputType, Param, StructField, TypeName as ParserTypeName,
};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

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
            OutputType::Inline(fields) => {
                // Resolve any Named types in the inline fields
                let resolved_fields = resolve_named_types(fields, structs)?;
                Ok(OutputSchema {
                    fields: resolved_fields,
                })
            }
            OutputType::Named(name) => {
                let fields = structs
                    .get(name)
                    .ok_or_else(|| format!("Unknown struct: {}", name))?;
                // Resolve any Named types in the struct fields
                let resolved_fields = resolve_named_types(fields, structs)?;
                Ok(OutputSchema {
                    fields: resolved_fields,
                })
            }
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

/// Recursively resolve FieldType::Named references to FieldType::Object
fn resolve_named_types(
    fields: &[StructField],
    structs: &HashMap<String, Vec<StructField>>,
) -> Result<Vec<StructField>, String> {
    fields
        .iter()
        .map(|field| {
            let resolved_type = resolve_field_type(&field.field_type, structs)?;
            Ok(StructField {
                name: field.name.clone(),
                field_type: resolved_type,
                span: field.span.clone(),
            })
        })
        .collect()
}

/// Resolve a single FieldType, converting Named to Object
fn resolve_field_type(
    ft: &FieldType,
    structs: &HashMap<String, Vec<StructField>>,
) -> Result<FieldType, String> {
    match ft {
        FieldType::String => Ok(FieldType::String),
        FieldType::Number => Ok(FieldType::Number),
        FieldType::Boolean => Ok(FieldType::Boolean),
        FieldType::Array(inner) => {
            let resolved_inner = resolve_field_type(inner, structs)?;
            Ok(FieldType::Array(Box::new(resolved_inner)))
        }
        FieldType::Object(fields) => {
            let resolved_fields = resolve_named_types(fields, structs)?;
            Ok(FieldType::Object(resolved_fields))
        }
        FieldType::Named(name) => {
            let struct_fields = structs
                .get(name)
                .ok_or_else(|| format!("Unknown struct: {}", name))?;
            // Recursively resolve the struct's fields
            let resolved_fields = resolve_named_types(struct_fields, structs)?;
            Ok(FieldType::Object(resolved_fields))
        }
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

/// Runtime value for a parallel execution block
#[derive(Debug, Clone, PartialEq)]
pub struct ParallelValue {
    pub name: String,
    pub agents: Vec<crate::parser::ast::Expression>,
    pub timeout_ms: u64,
}

/// Definition of an enum type (stored in environment)
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariantDef>,
}

/// Definition of an enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantDef {
    pub name: String,
    pub fields: Vec<EnumFieldDef>,
}

/// Definition of a field in an enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumFieldDef {
    pub name: Option<String>,
    pub type_name: String,
}

/// Runtime value of an enum instance
#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub enum_name: String,
    pub variant: String,
    pub data: Vec<Value>,
}

/// Intermediate value for enum variant with data (before being called)
#[derive(Debug, Clone, PartialEq)]
pub struct EnumConstructor {
    pub enum_name: String,
    pub variant: String,
    pub expected_fields: usize,
}

/// Definition of an interface type
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDef {
    pub name: String,
    pub members: Vec<InterfaceMemberDef>,
}

/// Definition of an interface member
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMemberDef {
    Field {
        name: String,
        type_name: crate::parser::ast::TypeName,
    },
    Method {
        name: String,
        params: Vec<crate::parser::ast::Param>,
        return_type: Option<crate::parser::ast::TypeName>,
    },
}

/// Runtime values in GENT
#[derive(Debug, Clone)]
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
    /// Enum value
    Enum(EnumValue),
    /// Enum constructor (intermediate value before calling with args)
    EnumConstructor(EnumConstructor),
    /// Parallel execution block
    Parallel(ParallelValue),
    /// Knowledge base for RAG
    KnowledgeBase(Arc<RwLock<crate::runtime::rag::KnowledgeBase>>),
    /// Built-in tool reference (name only, actual tool in registry)
    BuiltinTool(String),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Agent(a), Value::Agent(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Tool(a), Value::Tool(b)) => a == b,
            (Value::Function(a), Value::Function(b)) => a == b,
            (Value::Lambda(a), Value::Lambda(b)) => a == b,
            (Value::Enum(a), Value::Enum(b)) => a == b,
            (Value::EnumConstructor(a), Value::EnumConstructor(b)) => a == b,
            (Value::Parallel(a), Value::Parallel(b)) => a == b,
            // KnowledgeBase uses Arc pointer equality
            (Value::KnowledgeBase(a), Value::KnowledgeBase(b)) => Arc::ptr_eq(a, b),
            (Value::BuiltinTool(a), Value::BuiltinTool(b)) => a == b,
            _ => false,
        }
    }
}

/// Configuration for auto-RAG (knowledge base integration)
#[derive(Debug, Clone)]
pub struct KnowledgeConfig {
    /// Reference to the knowledge base
    pub source: Arc<RwLock<crate::runtime::rag::KnowledgeBase>>,
    /// Maximum number of chunks to inject (default: 3)
    pub chunk_limit: usize,
    /// Minimum relevance score threshold (default: 0.5)
    pub score_threshold: f64,
}

impl PartialEq for KnowledgeConfig {
    fn eq(&self, other: &Self) -> bool {
        // Compare by Arc pointer equality and config values
        Arc::ptr_eq(&self.source, &other.source)
            && self.chunk_limit == other.chunk_limit
            && self.score_threshold == other.score_threshold
    }
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
    /// Knowledge base configuration for auto-RAG (optional)
    pub knowledge_config: Option<KnowledgeConfig>,
    /// Maximum steps before stopping (None = default 10)
    pub max_steps: Option<u32>,
    /// Model to use (None = default)
    pub model: Option<String>,
    /// Provider to use (openai, claude-code)
    pub provider: Option<String>,
    /// Output schema for structured responses
    pub output_schema: Option<OutputSchema>,
    /// Number of retries for output validation
    pub output_retries: u32,
    /// Custom instructions for schema output (None = default)
    pub output_instructions: Option<String>,
    /// Custom prompt for validation retries (None = default)
    pub retry_prompt: Option<String>,
    /// Skip permission prompts for claude-code provider (dangerous!)
    pub dangerously_skip_permissions: bool,
}

impl AgentValue {
    /// Create a new agent value
    pub fn new(name: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            system_prompt: system_prompt.into(),
            user_prompt: None,
            tools: Vec::new(),
            knowledge_config: None,
            max_steps: None,
            model: None,
            provider: None,
            output_schema: None,
            output_retries: 1, // default: retry once
            output_instructions: None,
            retry_prompt: None,
            dangerously_skip_permissions: false,
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

    /// Set provider
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
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
            Value::Enum(e) => {
                if e.data.is_empty() {
                    write!(f, "{}.{}", e.enum_name, e.variant)
                } else {
                    let data_str: Vec<String> = e.data.iter().map(|v| v.to_string()).collect();
                    write!(f, "{}.{}({})", e.enum_name, e.variant, data_str.join(", "))
                }
            }
            Value::EnumConstructor(c) => {
                write!(f, "<enum constructor {}.{}>", c.enum_name, c.variant)
            }
            Value::Parallel(p) => write!(f, "<parallel {}>", p.name),
            Value::KnowledgeBase(_) => write!(f, "<KnowledgeBase>"),
            Value::BuiltinTool(name) => write!(f, "<builtin tool {}>", name),
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
            Value::Enum(_) => true,
            Value::EnumConstructor(_) => true,
            Value::Parallel(_) => true,
            Value::KnowledgeBase(_) => true,
            Value::BuiltinTool(_) => true,
        }
    }

    /// Get type name for error messages
    pub fn type_name(&self) -> String {
        match self {
            Value::String(_) => "String".to_string(),
            Value::Number(_) => "Number".to_string(),
            Value::Boolean(_) => "Boolean".to_string(),
            Value::Null => "Null".to_string(),
            Value::Agent(_) => "Agent".to_string(),
            Value::Array(_) => "Array".to_string(),
            Value::Object(_) => "Object".to_string(),
            Value::Tool(_) => "Tool".to_string(),
            Value::Function(_) => "Function".to_string(),
            Value::Lambda(_) => "Lambda".to_string(),
            Value::Enum(e) => format!("{}.{}", e.enum_name, e.variant),
            Value::EnumConstructor(c) => format!("EnumConstructor({}.{})", c.enum_name, c.variant),
            Value::Parallel(_) => "parallel".to_string(),
            Value::KnowledgeBase(_) => "KnowledgeBase".to_string(),
            Value::BuiltinTool(_) => "BuiltinTool".to_string(),
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
