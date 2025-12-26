//! Interpreter module for GENT

pub mod block_eval;
pub mod builtins;
pub mod environment;
pub mod evaluator;
pub mod expr_eval;
pub mod imports;
pub mod string_methods;
pub mod types;

pub use block_eval::{evaluate_block, evaluate_expr_async};
pub use builtins::{call_builtin, is_builtin};
pub use environment::Environment;
pub use evaluator::*;
pub use expr_eval::evaluate_expr;
pub use imports::{collect_imports, load_import, resolve_import_path};
pub use types::*;
