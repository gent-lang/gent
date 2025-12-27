# Contributing to GENT

Thank you for your interest in contributing to GENT! This document provides guidelines and information for contributors.

## Getting Started

1. **Fork the repository** and clone it locally
2. **Install Rust** (1.70 or later recommended)
3. **Build the project**: `cargo build`
4. **Run tests**: `cargo test`

## Development Workflow

### Running the Compiler

```bash
# Run a GENT file
cargo run -- examples/hello.gnt

# Run with mock LLM (no API calls)
cargo run -- --mock examples/hello.gnt
```

### Project Structure

```
gent/
├── src/
│   ├── lexer/          # Grammar and tokenization
│   │   └── grammar.pest
│   ├── parser/         # AST and parsing
│   ├── interpreter/    # Runtime evaluation
│   ├── runtime/        # Agent execution, LLM clients
│   └── errors/         # Error types
├── tests/              # Integration and unit tests
├── examples/           # Example GENT programs
└── docs/               # Documentation
```

### Making Changes

1. **Create a branch** for your changes
2. **Write tests** for new functionality
3. **Run the test suite**: `cargo test`
4. **Format your code**: `cargo fmt`
5. **Check for warnings**: `cargo clippy`

## Types of Contributions

### Bug Reports

- Use GitHub Issues
- Include a minimal reproduction case
- Include the GENT version and Rust version
- Describe expected vs actual behavior

### Feature Requests

- Open a GitHub Issue for discussion first
- Explain the use case
- Consider how it fits with GENT's design philosophy

### Code Contributions

1. **Small fixes** (typos, documentation) can be submitted directly
2. **New features** should be discussed in an issue first
3. **Breaking changes** require discussion and approval

### Documentation

- Fix typos or unclear explanations
- Add examples
- Improve the README

## Code Style

- Follow Rust conventions
- Use `cargo fmt` before committing
- Keep functions focused and small
- Write descriptive commit messages

## Commit Messages

Use clear, descriptive commit messages:

```
Add parallel execution timeout handling

- Add ParallelTimeout error type
- Implement tokio::time::timeout wrapper
- Add tests for timeout behavior
```

## Testing

- Add tests for new functionality
- Run the full test suite before submitting
- Tests should be deterministic

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

## Questions?

- Open a GitHub Issue for questions
- Check existing issues first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
