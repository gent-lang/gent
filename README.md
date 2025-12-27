# GENT

**A programming language for AI agents**

[![Crates.io](https://img.shields.io/crates/v/gent.svg)](https://crates.io/crates/gent)
[![Downloads](https://img.shields.io/crates/d/gent.svg)](https://crates.io/crates/gent)
[![License](https://img.shields.io/crates/l/gent.svg)](LICENSE)

> Write agents in minutes, not hours. Type-safe. Parallel. Observable.

```gent
agent Researcher {
    systemPrompt: "You research topics thoroughly and cite sources."
    model: "gpt-4o-mini"
}

let findings = Researcher.userPrompt("quantum computing 2025").run()
println(findings)
```

---

## Why GENT?

AI agent frameworks require endless boilerplate. GENT is a dedicated language that makes agents first-class citizens.

| | Python + LangChain | GENT |
|---|---|---|
| **Define an agent** | 15+ lines of imports and setup | 4 lines |
| **Parallel execution** | Manual async/await, error handling | `parallel { }` block |
| **Structured output** | Runtime schema validation | Compile-time type checking |
| **Tool definition** | Decorators, docstrings, type hints | `tool greet(name: string) -> string` |
| **Error handling** | Try/except scattered everywhere | `try { } catch { }` with agent context |
| **Learning curve** | Days to weeks | Hours |

### The Problem

```python
# LangChain: 20+ lines before you even start
from langchain.agents import create_react_agent
from langchain.prompts import ChatPromptTemplate
from langchain_openai import ChatOpenAI
from langchain.tools import tool

llm = ChatOpenAI(model="gpt-4")
prompt = ChatPromptTemplate.from_messages([...])
# ... more setup ...
```

### The Solution

```gent
// GENT: Just describe what you want
agent Helper {
    systemPrompt: "You help users with their questions."
    model: "gpt-4"
}

let answer = Helper.userPrompt("What is 2+2?").run()
```

---

## Features

### Agents

Declarative agent definitions with all configuration in one place:

```gent
agent DataAnalyst {
    systemPrompt: "You analyze data and provide insights."
    model: "gpt-4o-mini"
    maxSteps: 10
}

let insight = DataAnalyst.userPrompt("Analyze this: [1,2,3,4,5]").run()
```

### Tools

Type-safe tool definitions that agents can use:

```gent
tool calculate(expression: string) -> number {
    // Tool implementation
    return eval(expression)
}

tool fetchWeather(city: string) -> string {
    return http_get("https://api.weather.com/{city}")
}

agent Assistant {
    systemPrompt: "You help with calculations and weather."
    use calculate, fetchWeather
    model: "gpt-4o-mini"
}
```

### Parallel Execution

Run multiple agents concurrently with built-in timeout:

```gent
agent WebSearcher { systemPrompt: "Search the web." model: "gpt-4o-mini" }
agent NewsAnalyst { systemPrompt: "Analyze news." model: "gpt-4o-mini" }
agent AcademicSearcher { systemPrompt: "Find papers." model: "gpt-4o-mini" }

parallel research {
    agents: [
        WebSearcher.userPrompt("AI trends 2025"),
        NewsAnalyst.userPrompt("AI news today"),
        AcademicSearcher.userPrompt("latest AI papers")
    ]
    timeout: 30s
}

let results = research.run()  // Returns array of all results
```

### Structured Output

Get typed responses from agents with compile-time validation:

```gent
struct Analysis {
    sentiment: string
    confidence: number
    keywords: string[]
}

agent Analyzer {
    systemPrompt: "Analyze text sentiment."
    model: "gpt-4o-mini"
    output: Analysis
}

let result = Analyzer.userPrompt("I love this product!").run()
// result.sentiment, result.confidence, result.keywords are typed
```

### Agent Chaining

Compose agents naturally with variables:

```gent
agent Summarizer {
    systemPrompt: "Summarize in 2 sentences."
    model: "gpt-4o-mini"
}

agent Translator {
    systemPrompt: "Translate to French."
    model: "gpt-4o-mini"
}

let summary = Summarizer.userPrompt(longDocument).run()
let french = Translator.userPrompt(summary).run()
```

### Error Handling

Graceful error handling with context:

```gent
try {
    let result = RiskyAgent.run()
    println("Success: {result}")
} catch error {
    println("Failed: {error}")
}
```

---

## Installation

```bash
cargo install gent
```

Or with Homebrew:

```bash
brew tap gent-lang/tap && brew install gent
```

---

## Quick Start

1. **Create a file** `hello.gnt`:

```gent
agent Greeter {
    systemPrompt: "You are friendly and cheerful."
    model: "gpt-4o-mini"
}

let greeting = Greeter.userPrompt("Say hello!").run()
println(greeting)
```

2. **Set your API key**:

```bash
export OPENAI_API_KEY="your-key-here"
```

3. **Run it**:

```bash
gent hello.gnt
```

---

## Examples

| Example | Description |
|---------|-------------|
| [hello.gnt](examples/hello.gnt) | Minimal agent example |
| [chaining.gnt](examples/chaining.gnt) | Chain multiple agents |
| [parallel_agents.gnt](examples/parallel_agents.gnt) | Run agents concurrently |
| [structured_output.gnt](examples/structured_output.gnt) | Typed agent responses |
| [user_tools.gnt](examples/user_tools.gnt) | Define custom tools |
| [try_catch.gnt](examples/try_catch.gnt) | Error handling |

Run any example:

```bash
gent examples/hello.gnt
```

Run with mock LLM (for testing):

```bash
gent --mock examples/hello.gnt
```

---

## Language Reference

### Agent Declaration

```gent
agent Name {
    systemPrompt: "Instructions for the agent"
    model: "gpt-4o-mini"          // Required: LLM model
    maxSteps: 5                   // Optional: max tool calls
    output: StructName            // Optional: structured output type
    outputRetries: 3              // Optional: retry on parse failure
    use tool1, tool2              // Optional: available tools
}
```

### Tool Declaration

```gent
tool name(param: type, ...) -> returnType {
    // Implementation
    return value
}
```

**Types**: `string`, `number`, `boolean`, `array`, `object`, `any`

### Parallel Block

```gent
parallel name {
    agents: [Agent1.userPrompt("..."), Agent2.userPrompt("...")]
    timeout: 30s    // Required: 30s, 2m, 500ms
}

let results = name.run()  // Array of results
```

### Struct Declaration

```gent
struct Name {
    field1: string
    field2: number
    field3: string[]
}
```

### Control Flow

```gent
// Conditionals
if condition {
    // ...
}

// Loops
for item in items {
    // ...
}

while condition {
    // ...
}

// Error handling
try {
    // ...
} catch error {
    // ...
}
```

---

## Design Philosophy

GENT follows these principles, in order of priority:

1. **Agent-first** — Agents are the primary abstraction, not an afterthought bolted onto a general-purpose language.

2. **Fail fast** — Catch errors at parse time, not in production. Type mismatches, missing fields, and invalid configurations are caught before your code runs.

3. **Minimal boilerplate** — If you're writing the same thing twice, the language should handle it. No decorators, no factory patterns, no dependency injection.

4. **Observable by default** — Tracing, logging, and debugging are built into the language, not libraries you have to configure.

5. **One way to do things** — Reduce cognitive load. There's one way to define an agent, one way to run it, one way to handle errors.

---

## Status

GENT is in **alpha**. The language is functional but:

- Syntax may change between versions
- Some features are still being developed
- Not recommended for production use yet

### Roadmap

- [x] Agent declarations
- [x] Tool definitions
- [x] Structured output
- [x] Parallel execution
- [x] Error handling (try/catch)
- [x] Control flow (if/for/while)
- [ ] Memory/context persistence
- [ ] Multi-model support
- [ ] Built-in observability
- [ ] Package system
- [ ] LSP for editor support

---

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Running Examples

```bash
cargo run -- examples/hello.gnt
cargo run -- --mock examples/hello.gnt  # Mock LLM mode
```

---

## Contributing

Contributions are welcome! Whether it's:

- Bug reports
- Feature requests
- Documentation improvements
- Code contributions

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## License

MIT License. See [LICENSE](LICENSE) for details.

---

## Acknowledgments

GENT draws inspiration from:

- [Gleam](https://gleam.run/) — Friendly, type-safe language design
- [Pkl](https://pkl-lang.org/) — Configuration as code
- [Dhall](https://dhall-lang.org/) — Principled language design
- [Jason](https://jason-lang.github.io/) — Agent-oriented programming research

Built with Rust, powered by [pest](https://pest.rs/) parser.
