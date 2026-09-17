# Feature Matrix

| Feature | Viyal (Our Language) | Dart | Rust | Kotlin |
|---------|---------------------|------|------|--------|
| **Syntax Style** | C-Style (Strict) | C-Style | Expression C-Style | Expression C-Style |
| **Type System** | Static, Sound | Static, Sound | Static, Sound | Static, Sound |
| **Null Safety** | Built-in, Sound | Built-in, Sound | `Option<T>` | Built-in |
| **Memory Management** | Automatic (GC) | Automatic (GC) | Ownership/Borrowing | Automatic (GC) |
| **Concurrency** | Structured (Async/Tasks) | Unstructured (Futures) | Futures/Async | Coroutines (Structured) |
| **Interfaces** | Explicit (`interface`) | Implicit (Every class) | Traits | Explicit (`interface`) |
| **Data Classes** | Primary Constructors | Records/Macros | `struct` | `data class` |
| **Error Handling** | TBD (Researching) | Unchecked Exceptions | `Result<T, E>` | Unchecked Exceptions |
| **Execution** | VM / JIT / AOT | VM / JIT / AOT | AOT (LLVM) | JVM / Native |
| **Metaprogramming** | Planned (M15) | Macros (WIP) | Procedural Macros | Compiler Plugins / KSP |
| **AI Integration** | Planned (M15) | External Libraries | External Libraries | External Libraries |
