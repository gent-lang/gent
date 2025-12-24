use std::process::ExitCode;

fn main() -> ExitCode {
    println!("GENT v{}", env!("CARGO_PKG_VERSION"));
    println!("A programming language for AI agents");
    ExitCode::SUCCESS
}
