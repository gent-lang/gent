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
