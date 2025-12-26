use gent::parser::{parse, BlockStmt, Statement};

#[test]
fn test_parse_break() {
    let source = r#"
        tool test() {
            for i in [1, 2, 3] {
                break
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    // Verify break is parsed as BlockStmt::Break, not as an identifier expression
    let program = result.unwrap();
    let tool = match &program.statements[0] {
        Statement::ToolDecl(t) => t,
        _ => panic!("Expected ToolDecl"),
    };

    let for_stmt = match &tool.body.statements[0] {
        BlockStmt::For(f) => f,
        _ => panic!("Expected ForStmt"),
    };

    match &for_stmt.body.statements[0] {
        BlockStmt::Break(_) => {} // Success!
        other => panic!("Expected Break, got {:?}", other),
    }
}

#[test]
fn test_parse_continue() {
    let source = r#"
        tool test() {
            for i in [1, 2, 3] {
                continue
            }
        }
    "#;
    let result = parse(source);
    assert!(result.is_ok(), "Failed: {:?}", result.err());

    // Verify continue is parsed as BlockStmt::Continue, not as an identifier expression
    let program = result.unwrap();
    let tool = match &program.statements[0] {
        Statement::ToolDecl(t) => t,
        _ => panic!("Expected ToolDecl"),
    };

    let for_stmt = match &tool.body.statements[0] {
        BlockStmt::For(f) => f,
        _ => panic!("Expected ForStmt"),
    };

    match &for_stmt.body.statements[0] {
        BlockStmt::Continue(_) => {} // Success!
        other => panic!("Expected Continue, got {:?}", other),
    }
}
