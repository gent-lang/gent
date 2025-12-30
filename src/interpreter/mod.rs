//! Interpreter module for GENT

pub mod array_methods;
pub mod block_eval;
pub mod builtins;
pub mod environment;
pub mod evaluator;
pub mod expr_eval;
pub mod imports;
mod kb_helpers;
pub mod string_methods;
pub mod types;

pub(crate) use kb_helpers::parse_index_options;

pub use array_methods::{call_array_method, call_array_method_with_callback, is_callback_method};
pub use block_eval::{evaluate_block, evaluate_block_with_provider_factory, evaluate_expr_async, BlockEvalContext};
pub use builtins::{call_builtin, is_builtin};
pub use environment::Environment;
pub use evaluator::*;
pub use expr_eval::evaluate_expr;
pub use imports::{collect_imports, load_import, resolve_import_path};
pub use types::*;
