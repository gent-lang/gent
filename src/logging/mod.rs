//! Logging infrastructure for GENT
//!
//! Provides structured logging with levels, colored output, and timing support.

use std::fmt;
use std::io::{self, Write};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Instant;

/// Log levels in order of verbosity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trace" => Ok(LogLevel::Trace),
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            "off" | "none" => Ok(LogLevel::Off),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

impl LogLevel {
    fn color_code(&self) -> &'static str {
        match self {
            LogLevel::Trace => "\x1b[90m", // Gray
            LogLevel::Debug => "\x1b[36m", // Cyan
            LogLevel::Info => "\x1b[32m",  // Green
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Error => "\x1b[31m", // Red
            LogLevel::Off => "",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO ",
            LogLevel::Warn => "WARN ",
            LogLevel::Error => "ERROR",
            LogLevel::Off => "",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label().trim())
    }
}

/// Logger trait for abstracting log output
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, target: &str, message: &str);
    fn log_with_duration(&self, level: LogLevel, target: &str, message: &str, duration_ms: u64);
    fn is_enabled(&self, level: LogLevel) -> bool;
    fn level(&self) -> LogLevel;
}

/// Main logger implementation with colored pretty output
pub struct GentLogger {
    level: LogLevel,
    writer: Mutex<Box<dyn Write + Send>>,
    use_colors: bool,
}

impl GentLogger {
    /// Create a new logger with the given level, writing to stderr
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            writer: Mutex::new(Box::new(io::stderr())),
            use_colors: atty::is(atty::Stream::Stderr),
        }
    }

    /// Create a logger that writes to a custom writer
    pub fn with_writer(level: LogLevel, writer: Box<dyn Write + Send>) -> Self {
        Self {
            level,
            writer: Mutex::new(writer),
            use_colors: false,
        }
    }

    fn format_message(
        &self,
        level: LogLevel,
        target: &str,
        message: &str,
        duration_ms: Option<u64>,
    ) -> String {
        let reset = "\x1b[0m";
        let dim = "\x1b[90m";

        if self.use_colors {
            let mut output = format!(
                "{}{}{} {}{}{} {}",
                level.color_code(),
                level.label(),
                reset,
                dim,
                target,
                reset,
                message,
            );

            if let Some(ms) = duration_ms {
                output.push_str(&format!(" {}({}ms){}", dim, ms, reset));
            }
            output
        } else {
            let mut output = format!("{} {} {}", level.label(), target, message);
            if let Some(ms) = duration_ms {
                output.push_str(&format!(" ({}ms)", ms));
            }
            output
        }
    }
}

impl Logger for GentLogger {
    fn log(&self, level: LogLevel, target: &str, message: &str) {
        if level >= self.level {
            let formatted = self.format_message(level, target, message, None);
            if let Ok(mut writer) = self.writer.lock() {
                let _ = writeln!(writer, "{}", formatted);
            }
        }
    }

    fn log_with_duration(&self, level: LogLevel, target: &str, message: &str, duration_ms: u64) {
        if level >= self.level {
            let formatted = self.format_message(level, target, message, Some(duration_ms));
            if let Ok(mut writer) = self.writer.lock() {
                let _ = writeln!(writer, "{}", formatted);
            }
        }
    }

    fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.level
    }

    fn level(&self) -> LogLevel {
        self.level
    }
}

/// A no-op logger that discards all messages
pub struct NullLogger;

impl Logger for NullLogger {
    fn log(&self, _level: LogLevel, _target: &str, _message: &str) {}
    fn log_with_duration(
        &self,
        _level: LogLevel,
        _target: &str,
        _message: &str,
        _duration_ms: u64,
    ) {
    }
    fn is_enabled(&self, _level: LogLevel) -> bool {
        false
    }
    fn level(&self) -> LogLevel {
        LogLevel::Off
    }
}

/// Timer for measuring operation duration
/// Logs when dropped
pub struct Timer<'a> {
    start: Instant,
    name: String,
    target: String,
    level: LogLevel,
    logger: &'a dyn Logger,
}

impl<'a> Timer<'a> {
    pub fn new(
        name: impl Into<String>,
        target: impl Into<String>,
        level: LogLevel,
        logger: &'a dyn Logger,
    ) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
            target: target.into(),
            level,
            logger,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        let ms = self.elapsed_ms();
        self.logger.log_with_duration(
            self.level,
            &self.target,
            &format!("{} completed", self.name),
            ms,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Trace < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
        assert!(LogLevel::Error < LogLevel::Off);
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!("debug".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("DEBUG".parse::<LogLevel>(), Ok(LogLevel::Debug));
        assert_eq!("info".parse::<LogLevel>(), Ok(LogLevel::Info));
        assert_eq!("warn".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert_eq!("warning".parse::<LogLevel>(), Ok(LogLevel::Warn));
        assert!("invalid".parse::<LogLevel>().is_err());
    }

    #[test]
    fn test_logger_is_enabled() {
        let logger = GentLogger::new(LogLevel::Info);
        assert!(!logger.is_enabled(LogLevel::Debug));
        assert!(logger.is_enabled(LogLevel::Info));
        assert!(logger.is_enabled(LogLevel::Warn));
        assert!(logger.is_enabled(LogLevel::Error));
    }

    #[test]
    fn test_null_logger() {
        let logger = NullLogger;
        assert!(!logger.is_enabled(LogLevel::Error));
        // Should not panic
        logger.log(LogLevel::Error, "test", "message");
    }

    #[test]
    fn test_logger_captures_output() {
        use std::sync::Arc;

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();

        let writer = Box::new(TestWriter {
            buffer: buffer_clone,
        });
        let logger = GentLogger::with_writer(LogLevel::Debug, writer);

        logger.log(LogLevel::Info, "test", "hello world");

        let output = buffer.lock().unwrap();
        let output_str = String::from_utf8_lossy(&output);
        assert!(output_str.contains("INFO"));
        assert!(output_str.contains("test"));
        assert!(output_str.contains("hello world"));
    }

    struct TestWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl Write for TestWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
}
