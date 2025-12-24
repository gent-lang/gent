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
            GentError::LLMError { .. } => None,
            GentError::FileReadError { .. } => None,
        }
    }
}

/// Result type alias for GENT operations
pub type GentResult<T> = Result<T, GentError>;
