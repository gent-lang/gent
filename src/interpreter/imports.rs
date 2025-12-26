//! Import resolution and loading for GENT

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::errors::{GentError, GentResult, Span};
use crate::parser::{parse, Program, Statement};

/// Resolve an import path relative to the current file
pub fn resolve_import_path(current_file: &Path, import_path: &str) -> PathBuf {
    let dir = current_file.parent().unwrap_or(Path::new("."));
    dir.join(import_path)
}

/// Load and parse an imported file
pub fn load_import(path: &Path) -> GentResult<Program> {
    let source = std::fs::read_to_string(path).map_err(|e| GentError::FileReadError {
        path: path.display().to_string(),
        source: e,
    })?;

    parse(&source)
}

/// Collect all imports from a program, checking for circular dependencies
pub fn collect_imports(
    program: &Program,
    current_file: &Path,
    visited: &mut HashSet<PathBuf>,
) -> GentResult<Vec<(Vec<String>, Program)>> {
    let canonical = current_file
        .canonicalize()
        .unwrap_or_else(|_| current_file.to_path_buf());

    if visited.contains(&canonical) {
        return Err(GentError::SyntaxError {
            message: format!("Circular import detected: {}", current_file.display()),
            span: Span::default(),
        });
    }

    visited.insert(canonical.clone());

    let mut imports = Vec::new();

    for stmt in &program.statements {
        if let Statement::Import(import_stmt) = stmt {
            let import_path = resolve_import_path(current_file, &import_stmt.path);
            let imported_program = load_import(&import_path)?;

            // Recursively process imports
            let nested = collect_imports(&imported_program, &import_path, visited)?;
            imports.extend(nested);

            imports.push((import_stmt.names.clone(), imported_program));
        }
    }

    Ok(imports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_relative_path() {
        let base = PathBuf::from("/home/user/project/main.gnt");
        let import = "./helpers.gnt";
        let resolved = resolve_import_path(&base, import);
        assert_eq!(resolved, PathBuf::from("/home/user/project/./helpers.gnt"));
    }

    #[test]
    fn test_resolve_with_subdir() {
        let base = PathBuf::from("/home/user/project/main.gnt");
        let import = "./lib/utils.gnt";
        let resolved = resolve_import_path(&base, import);
        assert_eq!(
            resolved,
            PathBuf::from("/home/user/project/./lib/utils.gnt")
        );
    }
}
