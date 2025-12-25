//! GENT - A programming language for AI agents

pub mod errors;
pub mod lexer;
pub mod parser;

pub use errors::{GentError, GentResult, Span};
