//! Interpreter module for GENT

pub mod block_eval;
pub mod environment;
pub mod evaluator;
pub mod expr_eval;
pub mod types;

pub use block_eval::{evaluate_block, evaluate_expr_async};
pub use environment::Environment;
pub use evaluator::*;
pub use expr_eval::evaluate_expr;
pub use types::*;
