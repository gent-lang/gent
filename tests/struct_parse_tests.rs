use gent::parser::{parse, FieldType, OutputType, Statement};

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
            assert!(
                matches!(&decl.fields[1].field_type, FieldType::Array(inner) if matches!(inner.as_ref(), FieldType::Named(n) if n == "Item"))
            );
            // metadata is an inline object
            assert!(matches!(&decl.fields[2].field_type, FieldType::Object(_)));
        }
        _ => panic!("Expected StructDecl"),
    }
}

#[test]
fn test_parse_agent_with_inline_output() {
    let source = r#"
        agent Classifier {
            prompt: "Classify input"
            model: "gpt-4o"
            output: { category: string, confidence: number }
        }
    "#;

    let program = parse(source).unwrap();
    match &program.statements[0] {
        Statement::AgentDecl(decl) => {
            assert_eq!(decl.name, "Classifier");
            assert!(decl.output.is_some());
            match decl.output.as_ref().unwrap() {
                OutputType::Inline(fields) => {
                    assert_eq!(fields.len(), 2);
                    assert_eq!(fields[0].name, "category");
                    assert_eq!(fields[1].name, "confidence");
                }
                _ => panic!("Expected inline output type"),
            }
        }
        _ => panic!("Expected AgentDecl"),
    }
}

#[test]
fn test_parse_agent_with_named_output() {
    let source = r#"
        struct Result { value: string }
        agent Extractor {
            prompt: "Extract"
            model: "gpt-4o"
            output: Result
        }
    "#;

    let program = parse(source).unwrap();
    match &program.statements[1] {
        Statement::AgentDecl(decl) => {
            assert!(matches!(decl.output.as_ref().unwrap(), OutputType::Named(n) if n == "Result"));
        }
        _ => panic!("Expected AgentDecl"),
    }
}
