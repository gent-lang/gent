//! Tests for enum types in GENT

// ============================================
// Parsing Tests
// ============================================

#[test]
fn test_parse_simple_enum() {
    let source = r#"
        enum Status {
            Pending
            Active
            Completed
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse simple enum: {:?}", result.err());
}

#[test]
fn test_parse_enum_with_data() {
    let source = r#"
        enum Result {
            Ok(value)
            Err(string)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse enum with data: {:?}", result.err());
}

#[test]
fn test_parse_enum_with_multiple_fields() {
    let source = r#"
        enum Color {
            RGB(r: number, g: number, b: number)
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse enum with multiple fields: {:?}", result.err());
}

// ============================================
// Environment Registration Tests
// ============================================

#[tokio::test]
async fn test_enum_definition_registered() {
    let source = r#"
        enum Status {
            Pending
            Active
        }
        println("ok")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

// ============================================
// Construction Tests
// ============================================

#[tokio::test]
async fn test_enum_construct_unit_variant() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Pending
            println("{s}")
            return s
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_construct_with_data() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Ok(42)
            println("{r}")
            return r
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_construct_with_multiple_fields() {
    let source = r#"
        enum Color { RGB(r: number, g: number, b: number) }
        fn test() {
            let c = Color.RGB(255, 128, 0)
            println("{c}")
            return c
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_invalid_variant() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Unknown
            return s
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    // Should fail because Unknown is not a valid variant
    assert!(result.is_err());
}

#[tokio::test]
async fn test_enum_wrong_arg_count() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Ok(1, 2)
            return r
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    // Should fail because Ok expects 1 argument but got 2
    assert!(result.is_err());
}

// ============================================
// Match Expression Parsing Tests
// ============================================

#[test]
fn test_parse_match_expression() {
    let source = r#"
        enum Status { Pending, Failed(msg) }
        fn test() {
            let s = Status.Pending
            let result = match s {
                Status.Pending => "waiting"
                Status.Failed(m) => m
                _ => "other"
            }
            return result
        }
    "#;
    let result = gent::parser::parse(source);
    assert!(result.is_ok(), "Failed to parse match: {:?}", result.err());
}

// ============================================
// .is() Method Tests
// ============================================

#[tokio::test]
async fn test_enum_is_method_true() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Pending
            if s.is(Status.Pending) {
                return true
            }
            return false
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_is_method_false() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Pending
            if s.is(Status.Active) {
                return true
            }
            return false
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

// ============================================
// Match Expression Evaluation Tests
// ============================================

#[tokio::test]
async fn test_match_unit_variant() {
    let source = r#"
        enum Status { Pending, Active, Completed }
        fn test() {
            let s = Status.Active
            let result = match s {
                Status.Pending => "waiting"
                Status.Active => "running"
                Status.Completed => "done"
            }
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_match_with_binding() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Err("oops")
            let result = match r {
                Result.Ok(v) => v
                Result.Err(m) => m
            }
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_match_wildcard() {
    let source = r#"
        enum Status { Pending, Active, Completed }
        fn test() {
            let s = Status.Completed
            let result = match s {
                Status.Pending => "waiting"
                _ => "other"
            }
            return result
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

// ============================================
// .data() Method Tests
// ============================================

#[tokio::test]
async fn test_enum_data_method() {
    let source = r#"
        enum Result { Ok(value), Err(msg) }
        fn test() {
            let r = Result.Ok(42)
            let val = r.data(0)
            return val
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_enum_data_out_of_bounds() {
    let source = r#"
        enum Result { Ok(value) }
        fn test() {
            let r = Result.Ok(42)
            let val = r.data(5)
            return val
        }
        println("{test()}")
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_ok()); // Returns null for out of bounds
}

// ============================================
// Edge Case Tests
// ============================================

#[tokio::test]
async fn test_enum_unknown_variant_error() {
    let source = r#"
        enum Status { Pending }
        fn test() {
            let s = Status.Unknown
            return s
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_match_non_exhaustive() {
    let source = r#"
        enum Status { Pending, Active }
        fn test() {
            let s = Status.Active
            let result = match s {
                Status.Pending => "waiting"
            }
            return result
        }
        test()
    "#;
    let program = gent::parser::parse(source).unwrap();
    let factory = gent::runtime::ProviderFactory::mock();
    let mut tools = gent::runtime::ToolRegistry::new();
    let logger = gent::logging::NullLogger;
    let result = gent::interpreter::evaluate(&program, &factory, &mut tools, &logger).await;
    assert!(result.is_err()); // Non-exhaustive match error
}
