# Viyal Language Status

## Current Progress (As of September 2026)
We have successfully implemented the core foundational layers of the Viyal programming language, and have recently completed massive strides towards a type-safe, error-resilient system inspired by modern languages like V, Go, and Rust.

**Completed Features:**
- **Lexer & Parser:** Full AST generation for classes, methods, variables, math operations, arrays, maps, and control flow logic (`if/else`, `while`).
- **Bytecode Compiler:** Compiles AST into a custom bytecode format. It handles backpatching for jumps, tracks local variables on a stack, and builds method/class chunks.
- **Virtual Machine (VM):** A custom stack-based VM that executes the bytecode.
- **Memory Management:** Unified Garbage Collector (`GcObj`) that manages Classes, Instances, Arrays, Maps, and Functions.
- **Object-Oriented Programming (OOP):** Full support for Classes, Methods, Instances, and Property access/assignment.
- **Control Flow:** Fully working `if/else` branching and `while` loops with nested local scopes.
- **Error Handling (Result/Option):** Implemented modern `try` (`?`) and `UnwrapOrElse` (`or { ... }`) expressions. This replaces traditional exception-handling with a type-safe, explicit Result/Option type system.
- **Immutability by Default:** Enforced the `var` keyword for immutable declarations and introduced `mut` for mutable declarations. Re-assignments and property mutations on immutable variables are strictly rejected by the typechecker.
- **Type Aliasing:** Support for `type AliasName = OriginalType;`, enabling clean, semantic type definitions that resolve natively during compilation.
- **Deep Static Type Checking:** A robust, 2-pass Semantic Analyzer that validates expressions (BinaryOps, Arrays, Maps, Index Assignments) for type consistency and mutability constraints *before* code generation.
- **C Transpilation (MVP):** Added the foundational layers for transpiling the Viyal AST directly to C code, paving the way for native performance.

## Futuristic Tasks (Next Big Things)
To transition Viyal from a prototype into a production-ready "real" programming language, the following major features are pending:

1. **Standard Library & File I/O (fs, http, os)**
   - Expanding the `NativeBinding` system in Rust to expose lower-level modules in lowercase (`fs`, `http`, `os`).
   - Implementing network I/O, file reading/writing, and basic OS bindings.

2. **Map Datatype and JSON**
   - Solidifying the `Map` generic type throughout the toolchain.
   - Introducing native JSON serialization/deserialization utilities into the standard library to interface cleanly with the web.

3. **Advanced "Vlang-Inspired" Features**
   - *Hot Code Reloading:* Dynamically swapping bytecode chunks at runtime in the VM.
   - *Built-in Testing:* Integrating a test runner directly into the `viyal` CLI to support unit-tests seamlessly.

## Next Immediate Goal
Expand the **Standard Library (`fs`, `http`, `os`)** or finalize the **Map Datatype & JSON Utilities** to provide real-world usability and network interaction capabilities.
