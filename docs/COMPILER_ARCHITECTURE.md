# Compiler Architecture

The Viyal compiler will be written in **Rust** to leverage its performance, memory safety, and mature compiler toolchain (e.g., `logos`, `chumsky`, or custom hand-written parsers).

## 1. Pipeline Overview

```text
Source Code
     ↓ (Lexer)
Tokens
     ↓ (Parser)
AST (Abstract Syntax Tree)
     ↓ (Resolver)
Resolved AST
     ↓ (Type Checker)
Typed AST
     ↓ (Lowering)
HIR (High-Level Intermediate Representation)
     ↓ (Optimization / Lowering)
MIR (Mid-Level Intermediate Representation)
     ↓
 ┌───────────────┬───────────────┐
 │               │               │
Bytecode IR      LLVM IR       WASM IR
 │               │               │
 VM            Native Executable   Browser
```

## 2. Components

### Lexer
- Converts raw source text into a stream of Tokens.
- Handles whitespace, comments, string interpolation boundaries.

### Parser
- Consumes Tokens to produce an Abstract Syntax Tree (AST).
- Recovers from errors gracefully to support IDE features (LSP).

### Resolver & Type Checker
- Binds identifiers to declarations.
- Enforces null safety and type rules.
- Performs type inference.

### HIR & MIR
- High-level concepts (classes, async/await) are lowered into simpler representations (structs, state machines).
- Enables optimizations independent of the final target (Bytecode vs Native).
