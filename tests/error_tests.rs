use gent::errors::{GentError, GentResult, Span};

// ============================================
// Span Tests
// ============================================

#[test]
fn test_span_creation() {
    let span = Span::new(0, 10);
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 10);
}

#[test]
fn test_span_default() {
    let span = Span::default();
    assert_eq!(span.start, 0);
    assert_eq!(span.end, 0);
}

#[test]
fn test_span_debug() {
    let span = Span::new(5, 15);
    let debug = format!("{:?}", span);
    assert!(debug.contains("5"));
    assert!(debug.contains("15"));
}

#[test]
fn test_span_clone() {
    let span1 = Span::new(1, 2);
    let span2 = span1.clone();
    assert_eq!(span1, span2);
}

#[test]
fn test_span_equality() {
    let span1 = Span::new(0, 10);
    let span2 = Span::new(0, 10);
    let span3 = Span::new(0, 11);
    assert_eq!(span1, span2);
    assert_ne!(span1, span3);
}

// ============================================
// GentError Display Tests
// ============================================

#[test]
fn test_syntax_error_display() {
    let err = GentError::SyntaxError {
        message: "unexpected token".to_string(),
        span: Span::new(0, 5),
    };
    let msg = err.to_string();
    assert!(msg.contains("Syntax error"));
    assert!(msg.contains("unexpected token"));
}

#[test]
fn test_unexpected_token_display() {
    let err = GentError::UnexpectedToken {
        expected: "identifier".to_string(),
        found: "number".to_string(),
        span: Span::new(10, 15),
    };
    let msg = err.to_string();
    assert!(msg.contains("identifier"));
    assert!(msg.contains("number"));
}

#[test]
fn test_undefined_agent_display() {
    let err = GentError::UndefinedAgent {
        name: "MyAgent".to_string(),
        span: Span::new(0, 7),
    };
    let msg = err.to_string();
    assert!(msg.contains("MyAgent"));
    assert!(msg.contains("Undefined"));
}

#[test]
fn test_missing_agent_field_display() {
    let err = GentError::MissingAgentField {
        agent: "Helper".to_string(),
        field: "prompt".to_string(),
        span: Span::new(0, 10),
    };
    let msg = err.to_string();
    assert!(msg.contains("Helper"));
    assert!(msg.contains("prompt"));
}

#[test]
fn test_type_error_display() {
    let err = GentError::TypeError {
        expected: "String".to_string(),
        got: "Number".to_string(),
        span: Span::new(0, 5),
    };
    let msg = err.to_string();
    assert!(msg.contains("String"));
    assert!(msg.contains("Number"));
}

#[test]
fn test_llm_error_display() {
    let err = GentError::LLMError {
        message: "connection timeout".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("connection timeout"));
}

#[test]
fn test_file_read_error_display() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err = GentError::FileReadError {
        path: "/path/to/file.gnt".to_string(),
        source: io_err,
    };
    let msg = err.to_string();
    assert!(msg.contains("/path/to/file.gnt"));
}

// ============================================
// GentError Properties Tests
// ============================================

#[test]
fn test_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<GentError>();
}

#[test]
fn test_error_debug() {
    let err = GentError::SyntaxError {
        message: "test".to_string(),
        span: Span::new(0, 4),
    };
    let debug = format!("{:?}", err);
    assert!(debug.contains("SyntaxError"));
}

// ============================================
// GentResult Tests
// ============================================

#[test]
fn test_gent_result_ok() {
    let result: GentResult<i32> = Ok(42);
    assert!(result.is_ok());
    if let Ok(value) = result {
        assert_eq!(value, 42);
    }
}

#[test]
fn test_gent_result_err() {
    let result: GentResult<i32> = Err(GentError::LLMError {
        message: "fail".to_string(),
    });
    assert!(result.is_err());
}

// ============================================
// Error Span Extraction Tests
// ============================================

#[test]
fn test_syntax_error_has_span() {
    let span = Span::new(10, 20);
    let err = GentError::SyntaxError {
        message: "test".to_string(),
        span: span.clone(),
    };
    assert_eq!(err.span(), Some(&span));
}

#[test]
fn test_llm_error_no_span() {
    let err = GentError::LLMError {
        message: "test".to_string(),
    };
    assert_eq!(err.span(), None);
}

#[test]
fn test_file_read_error_no_span() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
    let err = GentError::FileReadError {
        path: "test.gnt".to_string(),
        source: io_err,
    };
    assert_eq!(err.span(), None);
}

// ============================================
// Milestone 3: New Error Types Tests
// ============================================

#[test]
fn test_undefined_variable_error() {
    let err = GentError::UndefinedVariable {
        name: "foo".to_string(),
        span: Span::new(0, 3),
    };
    assert!(err.to_string().contains("Undefined variable"));
    assert!(err.to_string().contains("foo"));
}

#[test]
fn test_undefined_property_error() {
    let err = GentError::UndefinedProperty {
        property: "bar".to_string(),
        type_name: "Object".to_string(),
        span: Span::new(0, 3),
    };
    assert!(err.to_string().contains("Undefined property"));
}

#[test]
fn test_index_out_of_bounds_error() {
    let err = GentError::IndexOutOfBounds {
        index: 5,
        length: 3,
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("out of bounds"));
}

#[test]
fn test_not_indexable_error() {
    let err = GentError::NotIndexable {
        type_name: "Boolean".to_string(),
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("Cannot index"));
}

#[test]
fn test_invalid_operands_error() {
    let err = GentError::InvalidOperands {
        op: "+".to_string(),
        left: "String".to_string(),
        right: "Boolean".to_string(),
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("Invalid operand"));
}

#[test]
fn test_division_by_zero_error() {
    let err = GentError::DivisionByZero {
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("Division by zero"));
}

#[test]
fn test_wrong_argument_count_error() {
    let err = GentError::WrongArgumentCount {
        expected: 2,
        got: 1,
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("2"));
    assert!(err.to_string().contains("1"));
}

#[test]
fn test_argument_type_mismatch_error() {
    let err = GentError::ArgumentTypeMismatch {
        param: "city".to_string(),
        expected: "String".to_string(),
        got: "Number".to_string(),
        span: Span::new(0, 1),
    };
    assert!(err.to_string().contains("city"));
}

#[test]
fn test_undefined_variable_has_span() {
    let span = Span::new(5, 8);
    let err = GentError::UndefinedVariable {
        name: "foo".to_string(),
        span: span.clone(),
    };
    assert_eq!(err.span(), Some(&span));
}

#[test]
fn test_division_by_zero_has_span() {
    let span = Span::new(10, 11);
    let err = GentError::DivisionByZero { span: span.clone() };
    assert_eq!(err.span(), Some(&span));
}
