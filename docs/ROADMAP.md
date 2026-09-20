# Project Roadmap

The development of the Viyal language follows strict milestone-based quality gates. We do not advance to the next milestone until the current one is complete, tested, and documented.

## M0 — Research and Architecture (Current)
- Establish documentation, architecture, and feature matrices.
- Define MVP specification.

## M1 — Lexer
- Tokenize source code.
- Handle keywords, operators, comments, and identifiers.
- Deliverable: Robust lexer with extensive tests.

## M2 — Parser
- Convert tokens into an Abstract Syntax Tree (AST).
- Handle parsing of expressions, classes, functions, and control flow.

## M3 — Interpreter
- Execute the AST directly.
- Capable of running basic programs without compilation.

## M4 — Static Type Checker
- Resolve variables, enforce null safety, check types.
- Deliverable: High-quality compiler error reporting.

## M5 — Object System
- Classes, interfaces, inheritance, constructors.

## M6 — Bytecode VM
- Compile AST -> Bytecode IR.
- Custom VM implementation.

## M7 — Garbage Collector
- Integrate memory management into the VM.

## M8 — Async/Concurrency
- Implement structured concurrency, tasks, and async/await syntax.

## M9 — Standard Library
- Core, math, IO, network, collections.

## M10 — Native Backend
- Compile MIR to native machine code via LLVM/Cranelift.

## M11 — Package Manager
- `Viyal pub` equivalent. Dependency resolution and semantic versioning.

## M12 — Tooling
- Formatter, Analyzer, LSP (Language Server Protocol).

## M13 — Web/WASM
- WebAssembly backend compilation target.

## M14 — Declarative Cross-Platform UI Framework
- Develop a declarative, reactive UI framework (Viyal UI).
- Support Mobile (iOS/Android), Desktop (Win/Mac/Linux), and Web (WASM/WebGL).
- Use a high-performance rendering engine (e.g., Skia) for pixel-perfect consistency.
- Integrate Hot Reloading into the UI widget tree.

## M15 — Advanced Language Features
- Macros, AI-Native APIs, Metaprogramming.
