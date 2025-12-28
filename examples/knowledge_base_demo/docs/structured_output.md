# Structured Output in Gent

Gent supports structured output, allowing agents to return typed JSON responses instead of free-form text.

## Inline Output Schema

Define the output schema directly in the agent:

```gent
agent Classifier {
    systemPrompt: "Classify the input text."
    model: "gpt-4o-mini"
    output: {
        category: string,
        confidence: number,
        keywords: string[]
    }
}

let result = Classifier.userPrompt("AI is transforming healthcare").run()
// result is a JSON object with category, confidence, and keywords
```

## Using Structs

Define reusable output types with structs:

```gent
struct Analysis {
    sentiment: string
    score: number
    summary: string
}

agent Analyzer {
    systemPrompt: "Analyze the sentiment of the text."
    model: "gpt-4o-mini"
    output: Analysis
}
```

## Validation and Retries

Gent automatically validates the LLM's output against the schema. If validation fails, it will retry up to 3 times by default.

Configure retry behavior:

```gent
agent StrictClassifier {
    systemPrompt: "Classify precisely."
    model: "gpt-4o"
    output: { category: string }
    outputRetries: 5
    retryPrompt: "Please respond with valid JSON matching the schema."
}
```

## Nested Types

Structured outputs support nested objects and arrays:

```gent
struct Person {
    name: string
    age: number
    address: {
        street: string
        city: string
        country: string
    }
    hobbies: string[]
}
```

## Best Practices

1. Keep schemas simple and focused
2. Use descriptive field names
3. Prefer specific types over generic ones
4. Test with edge cases
5. Consider using enums for categorical values
