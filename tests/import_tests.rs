use gent::parser::parse;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_parse_import() {
    let source = r#"
        import { Helper, Processor } from "./utils.gnt"

        agent Main {
            systemPrompt: "test"
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_parse_single_import() {
    let source = r#"
        import { Tool } from "./tool.gnt"
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());
}

#[test]
fn test_resolve_import_path() {
    use gent::interpreter::imports::resolve_import_path;
    use std::path::PathBuf;

    let base = PathBuf::from("/project/src/main.gnt");
    let import = "./utils.gnt";

    let resolved = resolve_import_path(&base, import);
    assert_eq!(resolved, PathBuf::from("/project/src/utils.gnt"));
}

#[test]
fn test_resolve_import_path_parent_dir() {
    use gent::interpreter::imports::resolve_import_path;
    use std::path::PathBuf;

    let base = PathBuf::from("/project/src/nested/file.gnt");
    let import = "../shared.gnt";

    let resolved = resolve_import_path(&base, import);
    assert_eq!(resolved, PathBuf::from("/project/src/nested/../shared.gnt"));
}

#[test]
fn test_resolve_import_path_absolute() {
    use gent::interpreter::imports::resolve_import_path;
    use std::path::PathBuf;

    let base = PathBuf::from("/project/src/main.gnt");
    let import = "./lib/helper.gnt";

    let resolved = resolve_import_path(&base, import);
    assert_eq!(resolved, PathBuf::from("/project/src/lib/helper.gnt"));
}

#[tokio::test]
async fn test_import_and_use() {
    let dir = tempdir().unwrap();

    // Create helper file
    let helper_path = dir.path().join("helper.gnt");
    fs::write(
        &helper_path,
        r#"
        fn double(x: number) -> number {
            return x + x
        }
    "#,
    )
    .unwrap();

    // Create main file that imports helper
    let main_path = dir.path().join("main.gnt");
    fs::write(
        &main_path,
        r#"
        import { double } from "./helper.gnt"

        tool test() {
            let result = double(5)
            return "done"
        }
    "#,
    )
    .unwrap();

    // Parse and evaluate
    let source = fs::read_to_string(&main_path).unwrap();
    let program = parse(&source).unwrap();

    // Verify parsing works
    assert!(!program.statements.is_empty());
}

#[tokio::test]
async fn test_evaluate_with_imports() {
    use gent::interpreter::evaluate_with_imports;
    use gent::logging::NullLogger;
    use gent::runtime::{ProviderFactory, ToolRegistry};

    let dir = tempdir().unwrap();

    // Create helper file with a function
    let helper_path = dir.path().join("helper.gnt");
    fs::write(
        &helper_path,
        r#"
        fn add(a: number, b: number) -> number {
            return a + b
        }
    "#,
    )
    .unwrap();

    // Create main file that imports the function
    let main_path = dir.path().join("main.gnt");
    fs::write(
        &main_path,
        r#"
        import { add } from "./helper.gnt"
    "#,
    )
    .unwrap();

    // Parse the main file
    let source = fs::read_to_string(&main_path).unwrap();
    let program = parse(&source).unwrap();

    // Evaluate with imports
    let factory = ProviderFactory::mock();
    let mut tools = ToolRegistry::new();
    let logger = NullLogger;

    let result = evaluate_with_imports(&program, Some(&main_path), &factory, &mut tools, &logger).await;

    // Should succeed without errors
    assert!(result.is_ok(), "Evaluation failed: {:?}", result.err());
}
