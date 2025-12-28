# Knowledge Base Demo

This example demonstrates how to use the KnowledgeBase feature in Gent for building RAG (Retrieval-Augmented Generation) powered agents.

## What's Included

```
knowledge_base_demo/
├── main.gnt          # Main demo script
├── README.md         # This file
└── docs/             # Sample documentation to index
    ├── getting_started.md
    ├── agents.md
    ├── structured_output.md
    └── tools.md
```

## Running the Demo

```bash
# From the project root
cargo run -- examples/knowledge_base_demo/main.gnt

# Or with mock LLM (no API key needed)
cargo run -- examples/knowledge_base_demo/main.gnt --mock
```

## Key Concepts

### 1. Creating a Knowledge Base

```gent
let docs = KnowledgeBase("./path/to/docs")
let chunkCount = docs.index({
    extensions: [".md", ".txt"],
    recursive: true
})
```

### 2. Auto-RAG with `knowledge:` field

The `knowledge:` field automatically injects relevant context into the system prompt:

```gent
agent DocAssistant {
    knowledge: {
        source: docs,        // Required: the KnowledgeBase
        chunkLimit: 3,       // Optional: max chunks (default: 3)
        scoreThreshold: 0.3  // Optional: min relevance (default: 0.5)
    }
    systemPrompt: "Answer questions using the provided context."
    model: "gpt-4o-mini"
}
```

### 3. Tool-based RAG with `tools:` field

For more control, use the KB as a tool:

```gent
agent ResearchAgent {
    tools: [docs]
    systemPrompt: "Search the docs when you need information."
    model: "gpt-4o"
}
```

## When to Use Each Approach

| Feature | Auto-RAG (`knowledge:`) | Tool-based (`tools:`) |
|---------|------------------------|----------------------|
| Context injection | Automatic | Agent decides |
| Latency | Lower (no tool call) | Higher (extra round-trip) |
| Control | Less flexible | More flexible |
| Best for | Q&A, chatbots | Research, complex tasks |
