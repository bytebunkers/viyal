# Viyal Programming Language (Placeholder Name)

Viyal is a modern, general-purpose programming language inspired by the strengths of Dart, designed for serious, production-oriented software engineering. 

> "Dart-like simplicity with stronger systems capabilities, first-class concurrency, powerful compile-time programming, predictable performance, and modern AI/software-development capabilities."

## Core Philosophy
Viyal aims to provide a Dart-like developer experience:
- Readable C-style syntax
- Sound static typing and type inference
- Object-oriented programming with generics
- First-class concurrency (async/await, structured concurrency)
- Fast development cycle

However, Viyal is **not** a Dart clone. It introduces carefully designed improvements in concurrency, compile-time programming, and systems capabilities.

## Target Use Cases
- Backend services and CLI applications
- Desktop, Mobile, and Web applications
- Data-processing and general software development

## Development Milestones
This project is currently in **Phase M0 (Research)**. The development roadmap is strictly phased:
1. Lexer & Parser
2. Interpreter
3. Static Type Checker
4. Bytecode VM
5. Native Backend (AOT)
6. Ecosystem (Package manager, LSP, Formatter)

Please see `docs/ROADMAP.md` for a complete breakdown of all 15 milestones.

## Documentation
All foundational design documentation can be found in the `docs/` directory. Check `docs/VISION.md` to understand the core motivations.

## Implementation
The Viyal compiler and runtime will be implemented in **Rust** to leverage its performance, memory safety, and mature compiler ecosystem.
