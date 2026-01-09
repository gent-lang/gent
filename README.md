<p align="center">
  <img src="assets/logo.svg" width="80" height="80" alt="Gent logo">
</p>

# GENT

**A programming language for AI agents**

[![Crates.io](https://img.shields.io/crates/v/gent-lang.svg)](https://crates.io/crates/gent-lang)
[![Downloads](https://img.shields.io/crates/d/gent-lang.svg)](https://crates.io/crates/gent-lang)
[![License](https://img.shields.io/crates/l/gent-lang.svg)](LICENSE)
[![Coverage](assets/coverage.svg)](assets/coverage.svg)

> Write agents in minutes, not hours. Type-safe. Parallel. Observable.

```typescript
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
| **RAG** | Vector DB setup, chunking, retrieval | `knowledge: { source: docs }` |
| **Error handling** | Try/except scattered everywhere | `try { } catch { }` with agent context |

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

```typescript
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

```typescript
agent DataAnalyst {
    systemPrompt: "You analyze data and provide insights."
    model: "gpt-4o-mini"
}

let insight = DataAnalyst.userPrompt("Analyze this: [1,2,3,4,5]").run()
```

### Tools

Type-safe tool definitions that agents can use:

```typescript
tool add(a: number, b: number) -> number {
    return a + b
}

tool multiply(a: number, b: number) -> number {
    return a * b
}

agent MathTutor {
    systemPrompt: "Help with math problems."
    tools: [add, multiply]
    model: "gpt-4o-mini"
}
```

### Built-in RAG

First-class knowledge base support with automatic context injection:

```typescript
// Create and index a knowledge base
let docs = KnowledgeBase("./docs")
docs.index({
    extensions: [".md", ".txt"],
    recursive: true
})

// Agent with automatic RAG
agent DocHelper {
    knowledge: {
        source: docs,
        chunkLimit: 5,
        scoreThreshold: 0.7
    }
    model: "gpt-4o-mini"
    systemPrompt: "Answer based on the documentation."
}

let answer = DocHelper.userPrompt("How do I configure tools?").run()
```

### Structured Output

Get typed responses from agents with compile-time validation:

```typescript
struct Analysis {
    sentiment: string
    confidence: number
    keywords: string[]
}

agent Analyzer {
    systemPrompt: "Analyze text sentiment."
    model: "gpt-4o-mini"
    output: Analysis
    outputRetries: 3
}

let result = Analyzer.userPrompt("I love this product!").run()
// result.sentiment, result.confidence, result.keywords are typed
```

### Parallel Execution

Run multiple agents concurrently with built-in timeout:

```typescript
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

### Enums & Pattern Matching

Define enums with optional data and match on them:

```typescript
enum Status {
    Pending
    Active
    Completed
    Failed(message)
}

let status = Status.Failed("connection timeout")

let message = match status {
    Status.Pending => "Waiting..."
    Status.Active => "Running"
    Status.Completed => "Done!"
    Status.Failed(msg) => "Error: {msg}"
}

// Check variants with .is()
if status.is(Status.Failed) {
    println("Something went wrong")
}
```

### Functions

Define reusable functions with typed parameters:

```typescript
fn formatName(first: string, last: string) -> string {
    return "{first} {last}"
}

fn isPositive(n: number) -> boolean {
    return n > 0
}

let name = formatName("John", "Doe")
```

### Array Methods

Full suite of functional array operations:

```typescript
let numbers = [1, 2, 3, 4, 5]

// Transform with map
let doubled = numbers.map((x) => x * 2)  // [2, 4, 6, 8, 10]

// Filter elements
let evens = numbers.filter((x) => x % 2 == 0)  // [2, 4]

// Reduce to single value
let sum = numbers.reduce((acc, x) => acc + x, 0)  // 15

// Find first match
let firstEven = numbers.find((x) => x % 2 == 0)  // 2

// Chaining
let result = numbers
    .filter((x) => x > 2)
    .map((x) => x * 2)
    .reduce((a, b) => a + b, 0)

// Other methods: indexOf, join, slice, concat, push, pop, length
```

### String Methods

Built-in string manipulation:

```typescript
let text = "  Hello, World!  "

text.trim()              // "Hello, World!"
text.toLowerCase()       // "  hello, world!  "
text.toUpperCase()       // "  HELLO, WORLD!  "
text.split(",")          // ["  Hello", " World!  "]
text.contains("World")   // true
text.startsWith("  H")   // true
text.replace("World", "GENT")  // "  Hello, GENT!  "
text.length()            // 17
```

### Error Handling

Graceful error handling with context:

```typescript
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
cargo install gent-lang
```

Or with Homebrew:

```bash
brew tap gent-lang/tap && brew install gent
```

---

## Quick Start

1. **Create a file** `hello.gnt`:

```typescript
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
| [structured_output.gnt](examples/structured_output.gnt) | Typed agent responses |
| [parallel_agents.gnt](examples/parallel_agents.gnt) | Run agents concurrently |
| [rag_example.gnt](examples/rag_example.gnt) | Knowledge base with auto-RAG |
| [user_tools.gnt](examples/user_tools.gnt) | Define custom tools |
| [enum_examples.gnt](examples/enum_examples.gnt) | Enums and pattern matching |
| [array_examples.gnt](examples/array_examples.gnt) | Array methods and lambdas |
| [functions.gnt](examples/functions.gnt) | Function definitions |
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

```typescript
agent Name {
    systemPrompt: "Instructions for the agent"
    model: "gpt-4o-mini"          // Required: LLM model
    tools: [tool1, tool2]         // Optional: available tools
    output: StructName            // Optional: structured output type
    outputRetries: 3              // Optional: retry on parse failure
    maxSteps: 5                   // Optional: max tool call iterations
    knowledge: {                  // Optional: auto-RAG configuration
        source: knowledgeBase,
        chunkLimit: 5,
        scoreThreshold: 0.7
    }
}
```

### Tool Declaration

```typescript
tool name(param: type, ...) -> returnType {
    return value
}
```

### Function Declaration

```typescript
fn name(param: type, ...) -> returnType {
    return value
}
```

### Enum Declaration

```typescript
enum Name {
    Variant1
    Variant2(field)
    Variant3(a: type, b: type)
}
```

### Parallel Block

```typescript
parallel name {
    agents: [Agent1.userPrompt("..."), Agent2.userPrompt("...")]
    timeout: 30s    // Required: 30s, 2m, 500ms
}

let results = name.run()  // Array of results
```

### Struct Declaration

```typescript
struct Name {
    field1: string
    field2: number
    field3: string[]
}
```

### Control Flow

```typescript
// Conditionals
if condition {
    // ...
}

// Pattern matching
let result = match value {
    Pattern1 => expression1
    Pattern2(x) => expression2
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

### Types

`string`, `number`, `boolean`, `array`, `object`, `any`

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
- [x] Functions
- [x] Enums & pattern matching
- [x] Array methods (map, filter, reduce, find, etc.)
- [x] String methods
- [x] Built-in RAG (knowledge field)
- [ ] Multi-model support (Anthropic, local models)
- [ ] Built-in observability dashboard
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
