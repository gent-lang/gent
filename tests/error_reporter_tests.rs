use gent::errors::{ErrorReporter, GentError, Span};

#[test]
fn test_format_undefined_agent() {
    let source = "run Ghost";
    let reporter = ErrorReporter::new(source, "test.gnt");

    let error = GentError::UndefinedAgent {
        name: "Ghost".to_string(),
        span: Span::new(4, 9),
    };

    let formatted = reporter.format(&error);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("Undefined agent"));
    assert!(formatted.contains("Ghost"));
    assert!(formatted.contains("test.gnt:1:5"));
    assert!(formatted.contains("run Ghost"));
    assert!(formatted.contains("^^^^^"));
}

#[test]
fn test_format_syntax_error() {
    let source = "agent A { prompt: }";
    let reporter = ErrorReporter::new(source, "test.gnt");

    let error = GentError::SyntaxError {
        message: "expected expression".to_string(),
        span: Span::new(18, 19),
    };

    let formatted = reporter.format(&error);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("test.gnt:1:19"));
    assert!(formatted.contains("^"));
}

#[test]
fn test_format_missing_field() {
    let source = "agent Broken {\n    prompt: \"hi\"\n}";
    let reporter = ErrorReporter::new(source, "example.gnt");

    let error = GentError::MissingAgentField {
        agent: "Broken".to_string(),
        field: "model".to_string(),
        span: Span::new(0, 14),
    };

    let formatted = reporter.format(&error);

    assert!(formatted.contains("example.gnt:1:1"));
    assert!(formatted.contains("agent Broken {"));
}

#[test]
fn test_format_error_without_span() {
    let source = "run Agent";
    let reporter = ErrorReporter::new(source, "test.gnt");

    let error = GentError::ApiError {
        message: "rate limit exceeded".to_string(),
    };

    let formatted = reporter.format(&error);

    assert!(formatted.contains("error:"));
    assert!(formatted.contains("rate limit exceeded"));
    // Should not contain source context
    assert!(!formatted.contains("-->"));
}

#[test]
fn test_format_multiline_source() {
    let source = "agent A {\n    prompt: \"x\"\n    model: \"y\"\n}\nrun B";
    let reporter = ErrorReporter::new(source, "test.gnt");

    let error = GentError::UndefinedAgent {
        name: "B".to_string(),
        span: Span::new(47, 48), // Position of 'B' in "run B"
    };

    let formatted = reporter.format(&error);

    assert!(formatted.contains("test.gnt:5:5"));
    assert!(formatted.contains("run B"));
}

#[test]
fn test_no_colors_when_disabled() {
    let source = "run Ghost";
    let mut reporter = ErrorReporter::new(source, "test.gnt");
    reporter.use_colors = false;

    let error = GentError::UndefinedAgent {
        name: "Ghost".to_string(),
        span: Span::new(4, 9),
    };

    let formatted = reporter.format(&error);

    // Should not contain ANSI escape codes
    assert!(!formatted.contains("\x1b["));
}
