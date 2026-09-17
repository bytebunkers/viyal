# Project Status

## Current Phase
M0 — Research and Language Architecture

## Completed
- Defined M0 implementation plan
- Created foundational project structure and documents
- Froze initial MVP specification
- Created repository structure (sub-directories)
- Initialized Rust workspace
- Implemented Lexer (M1)
- Wrote and passed Lexer tests (M1)
- Defined AST structure (M2)
- Implemented recursive descent parser (M2)
- Wrote and passed Parser tests (M2)
- Implemented tree-walking interpreter (M3)
- Implemented runtime value types and environment (M3)
- Executed Viyal programs end-to-end (M3)
- Implemented type resolution and variable scoping (M4)
- Enforced type checking rules with custom TypeErrors (M4)
- Verified TypeChecker via automated tests (M4)
- Added Lexer support for OOP and Null Safety tokens (M5)
- Upgraded AST to support fields, inheritance, and nullable types (M5)
- Rewrote parser to handle complex expressions and OOP constructs (M5)
- Verified workspace builds and tests pass (M5)
- Defined bytecode instruction set architecture (ISA) (M6)
- Implemented BytecodeCompiler to lower AST to Chunk (M6)
- Built high-performance Stack-Based Virtual Machine (M6)
- Verified VM execution with automated tests (M6)
- [x] Add `HashSet` of breakpoints and `InterpretResult::Breakpoint` to `vm/src/vm.rs`.
- [x] Expose introspection methods (`step`, `ip`, `stack`) on `VM`.
- [x] Add `vm` and `bytecode` dependencies to `tools/debugger/Cargo.toml`.
- [x] Implement `Debugger` struct and REPL in `tools/debugger/src/lib.rs`.
- [x] Add `debugger` dependency to `tools/cli/Cargo.toml`.
- [x] Add `viyal debug` command to `tools/cli/src/main.rs`.
- [x] Test the debugger using `hello.vy`.d to support running without arguments in `tools/cli/src/main.rs`.
- [x] Verify `viyal new` and `viyal run` locally. (M8)
- Registered `Math` and `IO` modules as native FFI bindings in the VM (M8)
- Verified standard library registration and execution (M8)
- Integrated `lsp-server` and `lsp-types` to build a synchronous LSP (M9)
- Hooked up `publishDiagnostics` to Viyal's Lexer, Parser, and TypeChecker (M9)
- Successfully compiled the Language Server binary (M9)
- Built `formatter` crate to standardize code layout (M10)
- Implemented an AST-traversing pretty-printer for classes and expressions (M10)
- Verified whitespace normalization and string generation via automated tests (M10)
- Built `analyzer` crate for AST-based static analysis and linting (M11)
- Implemented `AnalyzerRule` framework and `EmptyBlockRule` (M11)
- Verified analyzer warning generation via automated tests (M11)

## In Progress
- Waiting for direction on next milestone (M12: CLI / Developer Tooling)

## Blocked
- None

## Next Tasks
- 1. Await user confirmation for M12.

## Architecture Decisions
- **Implementation Language**: Rust (chosen for compiler ecosystem, performance, memory safety).
- **Phased Execution**: Lexer -> Parser -> Interpreter -> Bytecode VM -> Native (LLVM/Cranelift).
- **Concurrency**: First-class structured concurrency with async/await (details in CONCURRENCY.md).
- **Syntax**: Strictly leaning toward C-style syntax (like Dart/Java).

## Language Decisions
- **Null Safety**: Sound null safety as a core language design feature.
- **Typing**: Statically typed, sound, generic, object-oriented.
- **AI Features**: Treated as exploratory (Phase M15).

## Compiler Decisions
- **Lexer/Parser**: Hand-written (or using Rust parser libraries) to emit AST.
- **Type Checker**: Emits HIR (High-level IR) after resolution and type inference.

## Runtime Decisions
- **Memory**: Automatic memory management (GC by default) — precise model TBD.

## Known Bugs
- N/A

## Technical Debt
- N/A

## Benchmarks
- N/A

## Research Questions
- LLVM vs Cranelift for native backend.
- Generational GC vs Concurrent GC implementation in Rust.

## Future Features
- WebAssembly Backend (M13).
- Mobile/Desktop Ecosystem (M14).
- Advanced Metaprogramming/Compile-time evaluation (M15).
- AI-Native APIs (M15).

## Last Completed Milestone
None (Currently in M0)
