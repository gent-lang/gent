//! Error formatting with source context

use super::GentError;

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

    /// Format an error with source context
    pub fn format(&self, error: &GentError) -> String {
        let mut output = String::new();

        // Error header
        let error_msg = error.to_string();
        if self.use_colors {
            output.push_str(&format!("\x1b[31merror:\x1b[0m {}\n", error_msg));
        } else {
            output.push_str(&format!("error: {}\n", error_msg));
        }

        // Source location if available
        if let Some(span) = error.span() {
            let (line, col) = self.line_col(span.start);
            let source_line = self.get_line(span.start);
            let caret_count = (span.end - span.start).max(1);

            // Location line
            if self.use_colors {
                output.push_str(&format!(
                    "  \x1b[36m-->\x1b[0m {}:\x1b[34m{}:{}\x1b[0m\n",
                    self.filename, line, col
                ));
            } else {
                output.push_str(&format!("  --> {}:{}:{}\n", self.filename, line, col));
            }

            // Gutter and source line
            let line_num_width = line.to_string().len();
            output.push_str(&format!("{:width$} |\n", "", width = line_num_width + 1));

            if self.use_colors {
                output.push_str(&format!(
                    "\x1b[34m{:>width$}\x1b[0m | {}\n",
                    line,
                    source_line,
                    width = line_num_width
                ));
            } else {
                output.push_str(&format!("{:>width$} | {}\n", line, source_line, width = line_num_width));
            }

            // Caret line
            let padding = col - 1;
            let carets = "^".repeat(caret_count);
            if self.use_colors {
                output.push_str(&format!(
                    "{:width$} | {:padding$}\x1b[31m{}\x1b[0m\n",
                    "",
                    "",
                    carets,
                    width = line_num_width + 1,
                    padding = padding
                ));
            } else {
                output.push_str(&format!(
                    "{:width$} | {:padding$}{}\n",
                    "",
                    "",
                    carets,
                    width = line_num_width + 1,
                    padding = padding
                ));
            }
        }

        output
    }
}
