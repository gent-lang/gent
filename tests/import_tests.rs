use gent::parser::parse;

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
