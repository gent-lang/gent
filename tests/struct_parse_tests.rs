use gent::parser::{parse, Statement, FieldType};

#[test]
fn test_parse_simple_struct() {
    let source = r#"
        struct Person {
            name: string,
            age: number
        }
    "#;

    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::StructDecl(decl) => {
            assert_eq!(decl.name, "Person");
            assert_eq!(decl.fields.len(), 2);
            assert_eq!(decl.fields[0].name, "name");
            assert!(matches!(decl.fields[0].field_type, FieldType::String));
            assert_eq!(decl.fields[1].name, "age");
            assert!(matches!(decl.fields[1].field_type, FieldType::Number));
        }
        _ => panic!("Expected StructDecl"),
    }
}

#[test]
fn test_parse_nested_struct() {
    let source = r#"
        struct Order {
            id: string,
            items: Item[],
            metadata: { created: string, updated: string }
        }
    "#;

    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::StructDecl(decl) => {
            assert_eq!(decl.name, "Order");
            assert_eq!(decl.fields.len(), 3);
            // items is an array of Named("Item")
            assert!(matches!(&decl.fields[1].field_type, FieldType::Array(inner) if matches!(inner.as_ref(), FieldType::Named(n) if n == "Item")));
            // metadata is an inline object
            assert!(matches!(&decl.fields[2].field_type, FieldType::Object(_)));
        }
        _ => panic!("Expected StructDecl"),
    }
}
