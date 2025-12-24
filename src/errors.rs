//! Error types for GENT

use thiserror::Error;

/// Placeholder error type - will be expanded in Task 2
#[derive(Debug, Error)]
pub enum GentError {
    #[error("Not yet implemented")]
    NotImplemented,
}

pub type GentResult<T> = Result<T, GentError>;
