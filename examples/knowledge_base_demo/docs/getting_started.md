# Getting Started with Gent

Gent is a domain-specific language for building AI agents. It provides type safety, radical transparency, and first-class support for agents, tools, and structured outputs.

## Installation

To install Gent, you need Rust installed on your system:

```bash
cargo install gent-lang
```

## Your First Agent

Create a file called `hello.gnt`:

```gent
agent Greeter {
    systemPrompt: "You are a friendly greeter."
    model: "gpt-4o-mini"
}

let response = Greeter.userPrompt("Hello!").run()
println(response)
```

Run it with:

```bash
gent hello.gnt
```

## Environment Setup

Set your OpenAI API key:

```bash
export OPENAI_API_KEY="your-key-here"
```

Or create a `.gent.env` file in your project directory.
