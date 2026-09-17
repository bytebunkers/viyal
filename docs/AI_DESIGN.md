# AI-Native Capabilities (Phase M15)

Viyal is designed primarily as a strict, reliable software engineering language. However, as AI integration becomes a standard part of modern application development, we are exploring native language support for AI features in the later stages (M15).

## 1. The Core Idea
Currently, interacting with Large Language Models (LLMs) requires importing external SDKs, managing fragile JSON strings, and dealing with arbitrary failures at runtime. Viyal aims to provide ergonomic, type-safe primitives for AI operations without turning the language into an esoteric AI-only tool.

## 2. Potential Explorations

### Token-Aware Strings
A string type that natively understands tokenization boundaries (e.g., `TokenString`), making context-window calculations predictable rather than reliant on external C bindings for Tiktoken.

### Model Declarations
```Viyal
model "summarizer" {
    input String
    output Summary
}
```
This syntax could allow the compiler to automatically generate the necessary serialization/deserialization code and API wrappers for specific tasks.

### Structured Outputs
Enforcing schema generation at compile-time. If you request a model to return a `User` object, the Viyal compiler could automatically synthesize the exact JSON schema constraint for the model provider based on the type definition.

## 3. Strict Boundary
These features must **not** compromise the core language. They are strictly exploratory for Phase M15 and will only be adopted if they provide a genuine advantage over standard external libraries (like `langchain` or provider SDKs).
