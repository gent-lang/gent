# Tools in Gent

Tools extend agent capabilities by allowing them to perform actions like fetching web pages, reading files, or calling APIs.

## Built-in Tools

Gent provides several built-in tools:

### web_fetch
Fetches content from a URL.

```gent
agent WebResearcher {
    tools: [web_fetch]
    systemPrompt: "Research topics by fetching web pages."
    model: "gpt-4o"
}
```

### read_file
Reads content from a local file.

```gent
agent FileReader {
    tools: [read_file]
    systemPrompt: "Read and analyze files."
    model: "gpt-4o-mini"
}
```

### write_file
Writes content to a local file.

```gent
agent FileWriter {
    tools: [write_file]
    systemPrompt: "Create and write files."
    model: "gpt-4o-mini"
}
```

### json_parse
Parses JSON strings into objects.

## Custom Tools

Define custom tools using the `tool` keyword:

```gent
tool calculator(expression: string) -> string {
    // Tool implementation
    return eval(expression)
}

agent MathHelper {
    tools: [calculator]
    systemPrompt: "Help with math using the calculator."
    model: "gpt-4o-mini"
}
```

## Knowledge Base as Tool

A KnowledgeBase can be used as a tool for manual search:

```gent
let docs = KnowledgeBase("./docs")
docs.index({ extensions: [".md"] })

agent DocSearcher {
    tools: [docs]
    systemPrompt: "Search the documentation when needed."
    model: "gpt-4o"
}
```

## Multiple Tools

Combine multiple tools for complex agents:

```gent
agent SuperAgent {
    tools: [web_fetch, read_file, write_file, docs]
    systemPrompt: "You have access to web, files, and documentation."
    model: "gpt-4o"
}
```

## Tool Execution Flow

1. Agent receives user prompt
2. LLM decides which tool(s) to use
3. Gent executes the tool and returns results
4. LLM processes results and may call more tools
5. Process repeats until LLM provides final answer
6. Maximum iterations controlled by `maxSteps`
