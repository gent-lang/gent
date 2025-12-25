//! GENT - A programming language for AI agents

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use gent::errors::GentError;
use gent::interpreter::evaluate;
use gent::parser::parse;
use gent::runtime::MockLLMClient;

/// GENT - A programming language for AI agents
#[derive(Parser, Debug)]
#[command(name = "gent")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the .gnt file to execute
    file: PathBuf,

    /// Use mock LLM (for testing)
    #[arg(long, default_value = "true")]
    mock: bool,

    /// Custom mock response
    #[arg(long)]
    mock_response: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(&cli).await {
        eprintln!("Error: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

async fn run(cli: &Cli) -> Result<(), GentError> {
    // Read the source file
    let source = fs::read_to_string(&cli.file).map_err(|e| GentError::FileReadError {
        path: cli.file.display().to_string(),
        source: e,
    })?;

    // Parse the source
    let program = parse(&source)?;

    // Create LLM client
    let llm = if let Some(response) = &cli.mock_response {
        MockLLMClient::with_response(response)
    } else {
        MockLLMClient::new()
    };

    // Evaluate the program
    evaluate(&program, &llm).await?;

    Ok(())
}
