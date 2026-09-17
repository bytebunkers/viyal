# Dart Comparison and Evaluation

This document classifies major Dart capabilities into: **Adopt**, **Modify**, **Replace**, **Reject**, or **Research opportunity**.

## 1. Syntax (C-Style)
* **Decision**: **Adopt**
* **Why**: Familiarity for millions of developers. Reduces learning curve.
* **Modifications**: Remove legacy cruft (like optional semicolons ambiguity, if any). Enforce strict formatting.

## 2. Sound Null Safety
* **Decision**: **Adopt (and strengthen)**
* **Why**: Eliminates NullPointerExceptions at runtime.
* **Modifications**: Make flow-sensitive type promotion even smarter (e.g., smart casts in logical operators).

## 3. Type Inference
* **Decision**: **Adopt**
* **Why**: Reduces verbosity (`var x = 10;` instead of `int x = 10;`).
* **Limitations**: Inference should not cross function boundaries for return types to maintain readable APIs.

## 4. Classes, Mixins, Interfaces
* **Decision**: **Modify**
* **Why**: Dart's implicit interfaces (every class is an interface) can lead to fragile code.
* **Modifications**: Require explicit `interface` declarations. Retain Mixins but with stricter conflict resolution rules.

## 5. Extension Methods/Types
* **Decision**: **Adopt**
* **Why**: Allows adding functionality to closed types without wrapper classes.

## 6. Records and Patterns
* **Decision**: **Adopt**
* **Why**: Essential for returning multiple values and destructuring data concisely.

## 7. Primary Constructors
* **Decision**: **Adopt** (Recently added to Dart as well)
* **Why**: Reduces boilerplate in data classes.

## 8. Async/Await and Futures
* **Decision**: **Replace (with Structured Concurrency)**
* **Why**: Dart's unstructured Futures can lead to unhandled errors and abandoned async tasks.
* **Modifications**: Move towards structured concurrency (like Swift or Kotlin coroutines) to guarantee cancellation and lifecycle management.

## 9. Isolates
* **Decision**: **Modify/Research**
* **Why**: Actor-model message passing is great for avoiding shared-state mutations, but the serialization cost between Isolates in Dart can be high.
* **Research**: Investigate lightweight threads/channels combined with isolated memory arenas.

## 10. VM, AOT, and JIT
* **Decision**: **Adopt**
* **Why**: JIT/VM provides hot-reload and fast dev cycles. AOT provides fast startup and performance. We will build an Interpreter -> Bytecode VM -> AOT pipeline.

## 11. Macros/Metaprogramming
* **Decision**: **Research / Modify**
* **Why**: Dart's macro system has been heavily delayed and went through many iterations (`build_runner` -> static metaprogramming).
* **Research**: Look at Rust's procedural macros or C#'s source generators for a deterministic, fast compile-time code generation model.

## 12. Error Handling (Exceptions vs Results)
* **Decision**: **Research**
* **Why**: Dart uses unchecked exceptions (`try`/`catch`). This is convenient but hides control flow.
* **Research**: Evaluate a `Result<T, E>` based approach vs checked exceptions vs Swift-style typed throws.

## 13. Package Ecosystem & Build System
* **Decision**: **Adopt**
* **Why**: A unified `pub`-like system is vastly superior to fragmented package managers (e.g., npm/yarn/pnpm or C++ CMake/vcpkg).
