# Language Vision

## The Mission
To design a modern, general-purpose programming language with the following philosophy:

> "Dart-like simplicity with stronger systems capabilities, first-class concurrency, powerful compile-time programming, predictable performance, and modern AI/software-development capabilities."

## Core Characteristics

The language must be:
- **Statically Typed & Sound**: Catch errors at compile-time.
- **Null-Safe**: Nullability is part of the type system (`String` vs `String?`), not an afterthought.
- **Object-Oriented**: Classes, interfaces, mixins, and generics.
- **Garbage Collected**: Automatic memory management by default for developer productivity.
- **Async-First**: Deeply integrated concurrency models that reduce race conditions and simplify I/O.
- **Cross-Platform**: Compile to Native, VM Bytecode, and eventually WebAssembly.
- **Tooling-Friendly**: Built from day one for LSPs, formatters, and static analyzers.
- **Package-Oriented**: A unified, deterministic package manager out-of-the-box.
- **Familiar**: Strictly C-style syntax that feels immediately familiar to developers of Dart, Java, C#, and TypeScript.

## Target Domains

The language is designed to be suitable for:
1. **CLI applications**: Fast startup times and native compilation.
2. **Backend services**: High-concurrency async runtimes.
3. **Desktop applications**: Native UI bindings and FFI.
4. **Mobile applications**: Fast execution and predictable UI rendering cycles.
5. **Web applications**: WebAssembly compilation.
6. **Data-processing applications**: Predictable performance and fast memory allocation.
7. **AI applications**: Exploring future API ergonomics for language-model integration.

## Design Constraints

1. **Do not clone blindly**: Features exist because they solve a problem, not because another language has them.
2. **Predictable Performance**: Hidden costs should be minimized. The developer should understand when allocations happen.
3. **Developer Experience (DX) over Purity**: Pragmatism is favored over academic purity, provided it doesn't break soundness.
