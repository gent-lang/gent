# Agents in Gent

Agents are the core building blocks of Gent programs. An agent wraps an LLM with a system prompt and optional configuration.

## Basic Agent Definition

```gent
agent MyAgent {
    systemPrompt: "You are a helpful assistant."
    model: "gpt-4o-mini"
}
```

## Agent Fields

### systemPrompt (required)
The system prompt that defines the agent's behavior and personality.

### model (optional)
The LLM model to use. Defaults to "gpt-4o-mini". Options include:
- "gpt-4o" - Most capable model
- "gpt-4o-mini" - Fast and cost-effective
- "gpt-4-turbo" - Good balance of speed and capability

### userPrompt (optional)
A default user prompt. Can also be set dynamically with `.userPrompt()`.

### maxSteps (optional)
Maximum number of tool-calling iterations. Default is 10.

## Running Agents

```gent
// Simple run
let result = MyAgent.run()

// With custom user prompt
let result = MyAgent.userPrompt("What is 2+2?").run()

// Chain multiple calls
let agent = MyAgent.userPrompt("Hello")
let response = agent.run()
```

## Agent with Tools

Agents can use tools to perform actions:

```gent
agent Researcher {
    tools: [web_fetch, read_file]
    systemPrompt: "You research topics using available tools."
    model: "gpt-4o"
}
```

## Agent with Knowledge Base

Agents can automatically receive context from a knowledge base:

```gent
let docs = KnowledgeBase("./docs")
docs.index({ extensions: [".md"] })

agent DocHelper {
    knowledge: {
        source: docs,
        chunkLimit: 5,
        scoreThreshold: 0.5
    }
    systemPrompt: "Answer questions using the provided context."
    model: "gpt-4o-mini"
}
```
