//! Tests for interface declaration parsing

use gent::config::Config;
use gent::parser::{InterfaceMember, Statement, TypeName};

#[test]
fn test_parse_interface() {
    let source = r#"
        interface Tool {
            name: string
            execute(input: string) -> string
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(
        result.is_ok(),
        "Failed to parse interface: {:?}",
        result.err()
    );

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            assert_eq!(decl.name, "Tool");
            assert_eq!(decl.members.len(), 2);

            // Check field
            match &decl.members[0] {
                InterfaceMember::Field(field) => {
                    assert_eq!(field.name, "name");
                    assert_eq!(field.type_name, TypeName::String);
                }
                _ => panic!("Expected field, got method"),
            }

            // Check method
            match &decl.members[1] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "execute");
                    assert_eq!(method.params.len(), 1);
                    assert_eq!(method.params[0].name, "input");
                    assert_eq!(method.params[0].type_name, TypeName::String);
                    assert_eq!(method.return_type, Some(TypeName::String));
                }
                _ => panic!("Expected method, got field"),
            }
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_interface_empty() {
    let source = r#"
        interface Empty {
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            assert_eq!(decl.name, "Empty");
            assert!(decl.members.is_empty());
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_interface_method_no_params() {
    let source = r#"
        interface Runnable {
            run() -> boolean
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            assert_eq!(decl.name, "Runnable");
            match &decl.members[0] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "run");
                    assert!(method.params.is_empty());
                    assert_eq!(method.return_type, Some(TypeName::Boolean));
                }
                _ => panic!("Expected method"),
            }
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_interface_method_no_return_type() {
    let source = r#"
        interface Logger {
            log(message: string)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            match &decl.members[0] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "log");
                    assert_eq!(method.params.len(), 1);
                    assert!(method.return_type.is_none());
                }
                _ => panic!("Expected method"),
            }
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_interface_multiple_methods() {
    let source = r#"
        interface DataStore {
            get(key: string) -> string
            set(key: string, value: string)
            delete(key: string) -> boolean
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            assert_eq!(decl.name, "DataStore");
            assert_eq!(decl.members.len(), 3);

            // get method
            match &decl.members[0] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "get");
                    assert_eq!(method.params.len(), 1);
                    assert_eq!(method.return_type, Some(TypeName::String));
                }
                _ => panic!("Expected method"),
            }

            // set method
            match &decl.members[1] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "set");
                    assert_eq!(method.params.len(), 2);
                    assert!(method.return_type.is_none());
                }
                _ => panic!("Expected method"),
            }

            // delete method
            match &decl.members[2] {
                InterfaceMember::Method(method) => {
                    assert_eq!(method.name, "delete");
                    assert_eq!(method.params.len(), 1);
                    assert_eq!(method.return_type, Some(TypeName::Boolean));
                }
                _ => panic!("Expected method"),
            }
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_interface_fields_only() {
    let source = r#"
        interface Config {
            name: string
            enabled: boolean
            count: number
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => {
            assert_eq!(decl.name, "Config");
            assert_eq!(decl.members.len(), 3);

            for member in &decl.members {
                match member {
                    InterfaceMember::Field(_) => {}
                    _ => panic!("Expected all members to be fields"),
                }
            }
        }
        _ => panic!("Expected InterfaceDecl statement"),
    }
}

#[test]
fn test_parse_multiple_interfaces() {
    let source = r#"
        interface Reader {
            read() -> string
        }

        interface Writer {
            write(data: string)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok());

    let program = result.unwrap();
    assert_eq!(program.statements.len(), 2);

    match &program.statements[0] {
        Statement::InterfaceDecl(decl) => assert_eq!(decl.name, "Reader"),
        _ => panic!("Expected InterfaceDecl"),
    }

    match &program.statements[1] {
        Statement::InterfaceDecl(decl) => assert_eq!(decl.name, "Writer"),
        _ => panic!("Expected InterfaceDecl"),
    }
}

#[test]
fn test_parse_struct_implements() {
    let source = r#"
        interface Tool {
            name: string
        }
        struct MyTool implements Tool {
            name: string
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    if let Statement::StructDecl(s) = &program.statements[1] {
        assert_eq!(s.implements, vec!["Tool".to_string()]);
    } else {
        panic!("Expected StructDecl");
    }
}

#[test]
fn test_parse_struct_implements_multiple() {
    let source = r#"
        struct MyTool implements Tool, Searchable {
            name: string
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    if let Statement::StructDecl(s) = &program.statements[0] {
        assert_eq!(s.implements.len(), 2);
        assert!(s.implements.contains(&"Tool".to_string()));
        assert!(s.implements.contains(&"Searchable".to_string()));
    } else {
        panic!("Expected StructDecl");
    }
}

#[test]
fn test_parse_struct_no_implements() {
    // Test that structs without implements clause still work and have empty implements vec
    let source = r#"
        struct SimpleStruct {
            name: string
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

    let program = result.unwrap();
    if let Statement::StructDecl(s) = &program.statements[0] {
        assert!(s.implements.is_empty());
        assert_eq!(s.name, "SimpleStruct");
    } else {
        panic!("Expected StructDecl");
    }
}

#[tokio::test]
async fn test_interface_definition_registered() {
    let source = r#"
        interface Tool {
            name: string
            execute(input: string) -> string
        }
        println("ok")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok(), "Interface evaluation failed: {:?}", result.err());
}
