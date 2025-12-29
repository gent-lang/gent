//! GENT CLI - A programming language for AI agents

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use gent::config::Config;
use gent::errors::{ErrorReporter, GentError};
use gent::interpreter::evaluate_with_output;
use gent::logging::{GentLogger, LogLevel, Logger};
use gent::parser::parse;
use gent::runtime::ToolRegistry;

#[derive(Parser, Debug)]
#[command(name = "gent")]
#[command(author, version, about = "A programming language for AI agents", long_about = None)]
struct Cli {
    /// Path to the .gnt file to execute
    file: PathBuf,

    /// Use mock LLM (for testing)
    #[arg(long)]
    mock: bool,

    /// Custom mock response
    #[arg(long)]
    mock_response: Option<String>,

    /// Log level: trace, debug, info, warn, error, off
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Verbose mode (-v = debug, -vv = trace)
    #[arg(short, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short, long)]
    quiet: bool,
}

impl Cli {
    fn effective_log_level(&self) -> LogLevel {
        if self.quiet {
            return LogLevel::Error;
        }

        match self.verbose {
            0 => self.log_level.parse().unwrap_or(LogLevel::Info),
            1 => LogLevel::Debug,
            _ => LogLevel::Trace,
        }
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let log_level = cli.effective_log_level();
    let logger: Arc<dyn Logger> = Arc::new(GentLogger::new(log_level));

    // Load source first so we can use it for error reporting
    let filename = cli.file.display().to_string();
    let source = match fs::read_to_string(&cli.file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: Could not read file '{}': {}", filename, e);
            return ExitCode::FAILURE;
        }
    };

    let reporter = ErrorReporter::new(&source, &filename);

    if let Err(e) = run(&cli, &source, logger.as_ref()).await {
        eprint!("{}", reporter.format(&e));
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run(cli: &Cli, source: &str, logger: &dyn Logger) -> Result<(), GentError> {
    logger.log(
        LogLevel::Debug,
        "cli",
        &format!("Parsing {} bytes", source.len()),
    );
    let program = parse(source)?;
    logger.log(
        LogLevel::Debug,
        "cli",
        &format!("Parsed {} statements", program.statements.len()),
    );

    let mut tools = ToolRegistry::with_builtins();

    // Load config and apply CLI overrides
    let mut config = Config::load();
    if cli.mock {
        logger.log(LogLevel::Info, "cli", "Using mock LLM");
        config.mock_mode = true;
        config.mock_response = cli.mock_response.clone();
    } else {
        logger.log(LogLevel::Debug, "cli", "Using real LLM (provider determined by model)");
    }

    let outputs = evaluate_with_output(&program, &config, &mut tools, logger).await?;

    // Print outputs
    for output in outputs {
        println!("{}", output);
    }

    Ok(())
}
