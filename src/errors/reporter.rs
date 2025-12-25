//! Error formatting with source context

use super::{GentError, Span};

/// Formats errors with source context
pub struct ErrorReporter<'a> {
    source: &'a str,
    filename: &'a str,
    use_colors: bool,
}

impl<'a> ErrorReporter<'a> {
    /// Create a new error reporter
    pub fn new(source: &'a str, filename: &'a str) -> Self {
        Self {
            source,
            filename,
            use_colors: atty::is(atty::Stream::Stderr),
        }
    }

    /// Calculate line and column from byte offset
    fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    /// Get the source line containing the given offset
    fn get_line(&self, offset: usize) -> &str {
        let start = self.source[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let end = self.source[offset..]
            .find('\n')
            .map(|i| offset + i)
            .unwrap_or(self.source.len());
        &self.source[start..end]
    }
}
