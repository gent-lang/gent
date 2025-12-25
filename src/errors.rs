//! Error types for the GENT programming language

use thiserror::Error;

/// Source location span for error reporting
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create a new span with start and end positions
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

/// All possible errors in the GENT language
#[derive(Debug, Error)]
pub enum GentError {
    /// Syntax error during parsing
    #[error("Syntax error at {span:?}: {message}")]
    SyntaxError { message: String, span: Span },

    /// Unexpected token during parsing
    #[error("Unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    /// Reference to undefined agent
    #[error("Undefined agent: '{name}'")]
    UndefinedAgent { name: String, span: Span },

    /// Agent missing a required field
    #[error("Agent '{agent}' missing required field: '{field}'")]
    MissingAgentField {
        agent: String,
        field: String,
        span: Span,
    },

    /// Type mismatch error
    #[error("Type error: expected {expected}, got {got}")]
    TypeError {
        expected: String,
        got: String,
        span: Span,
    },

    /// LLM communication error
    #[error("LLM error: {message}")]
    LLMError { message: String },

    /// File read error
    #[error("Could not read file '{path}': {source}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// API request failed
    #[error("API error: {message}")]
    ApiError { message: String },

    /// Missing API key
    #[error("Missing API key for {provider}. Set {provider}_API_KEY environment variable.")]
    MissingApiKey { provider: String },

    /// Agent exceeded maximum steps
    #[error("Agent exceeded maximum steps ({limit})")]
    MaxStepsExceeded { limit: u32 },

    /// Unknown tool referenced
    #[error("Unknown tool: '{name}'")]
    UnknownTool { name: String, span: Span },

    /// Tool execution failed
    #[error("Tool '{tool}' failed: {message}")]
    ToolError { tool: String, message: String },

    /// Undefined variable reference
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String, span: Span },

    /// Undefined property access
    #[error("Undefined property: {property} on {type_name}")]
    UndefinedProperty {
        property: String,
        type_name: String,
        span: Span,
    },

    /// Array/List index out of bounds
    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds {
        index: i64,
        length: usize,
        span: Span,
    },

    /// Attempt to index a non-indexable type
    #[error("Cannot index into {type_name}")]
    NotIndexable { type_name: String, span: Span },

    /// Invalid operand types for binary operation
    #[error("Invalid operand types: {left} {op} {right}")]
    InvalidOperands {
        op: String,
        left: String,
        right: String,
        span: Span,
    },

    /// Division by zero error
    #[error("Division by zero")]
    DivisionByZero { span: Span },

    /// Wrong number of arguments to function/tool
    #[error("Expected {expected} arguments, got {got}")]
    WrongArgumentCount {
        expected: usize,
        got: usize,
        span: Span,
    },

    /// Type mismatch for function/tool argument
    #[error("Type mismatch for parameter '{param}': expected {expected}, got {got}")]
    ArgumentTypeMismatch {
        param: String,
        expected: String,
        got: String,
        span: Span,
    },
}

impl GentError {
    /// Get the source span if this error has one
    pub fn span(&self) -> Option<&Span> {
        match self {
            GentError::SyntaxError { span, .. } => Some(span),
            GentError::UnexpectedToken { span, .. } => Some(span),
            GentError::UndefinedAgent { span, .. } => Some(span),
            GentError::MissingAgentField { span, .. } => Some(span),
            GentError::TypeError { span, .. } => Some(span),
            GentError::UnknownTool { span, .. } => Some(span),
            GentError::UndefinedVariable { span, .. } => Some(span),
            GentError::UndefinedProperty { span, .. } => Some(span),
            GentError::IndexOutOfBounds { span, .. } => Some(span),
            GentError::NotIndexable { span, .. } => Some(span),
            GentError::InvalidOperands { span, .. } => Some(span),
            GentError::DivisionByZero { span } => Some(span),
            GentError::WrongArgumentCount { span, .. } => Some(span),
            GentError::ArgumentTypeMismatch { span, .. } => Some(span),
            GentError::LLMError { .. } => None,
            GentError::FileReadError { .. } => None,
            GentError::ApiError { .. } => None,
            GentError::MissingApiKey { .. } => None,
            GentError::MaxStepsExceeded { .. } => None,
            GentError::ToolError { .. } => None,
        }
    }
}

/// Result type alias for GENT operations
pub type GentResult<T> = Result<T, GentError>;
