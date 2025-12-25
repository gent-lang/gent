//! GENT CLI - A programming language for AI agents

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use gent::config::Config;
use gent::errors::GentError;
use gent::interpreter::evaluate;
use gent::parser::parse;
use gent::runtime::{MockLLMClient, OpenAIClient, ToolRegistry};

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
    let source = fs::read_to_string(&cli.file).map_err(|e| GentError::FileReadError {
        path: cli.file.display().to_string(),
        source: e,
    })?;

    let program = parse(&source)?;
    let mut tools = ToolRegistry::with_builtins();

    if cli.mock {
        let llm = if let Some(response) = &cli.mock_response {
            MockLLMClient::with_response(response)
        } else {
            MockLLMClient::new()
        };
        evaluate(&program, &llm, &mut tools).await?;
    } else {
        let config = Config::load();
        let api_key = config.require_openai_key()?;
        let llm = OpenAIClient::new(api_key.to_string());
        evaluate(&program, &llm, &mut tools).await?;
    }

    Ok(())
}
