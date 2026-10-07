# Semantic Gaps & Unresolved Decisions (Phase 01)

This document tracks language features that are partially implemented, mocked, or missing, creating gaps between the parser, typechecker, and VM.

## 1. Partially Implemented Features

*   **Generics (`<T>`):** Successfully parsed in classes and functions, but the typechecker simply aliases them to `Any`. There is no monomorphization or type checking of generic parameters.
*   **Inheritance & Interfaces:** `extends` and `implements` are parsed. However, the typechecker does not verify interface compliance, and the VM's method dispatcher (`OpInvoke`) does not traverse up a class hierarchy to find inherited methods.
*   **Match Identifier Bindings:** In `match` expressions, arms like `id => expr` are parsed. However, the bytecode compiler treats this as a catch-all without actually binding `id` to the matched value in the local scope.
*   **Safe Property Access (`?.`):** Parsed into the AST, but there is no specific bytecode generation for it. It would likely behave as a regular property access (which panics on `null`) if it were compiled.
*   **Module System:** `import` is parsed but ignored by both the typechecker and bytecode compiler. Only native injected modules work.

## 2. Missing Features

*   **Logical Operators:** The lexer lacks `&&` (AND) and `||` (OR) operators. The `or` keyword is exclusively used for the `UnwrapOrElse` expression (`expr or { ... }`).
*   **String Interpolation:** Not supported by the lexer.
*   **Constructors:** While `class Name(Type param)` parses, the VM's `OpConstruct` merely allocates an empty instance and does not actually invoke a constructor function to initialize fields based on parameters.
*   **Closures:** Anonymous functions or closures are not defined in the syntax or VM.

## 3. Unresolved Decisions

1.  **Type Inference:** Should `var x;` default to `void`, `Any`, or `null` type? Currently, it defaults to `void`.
2.  **Immutability:** `var` defaults to immutable (`is_final = true`) in the parser, while `mut` is mutable. However, class fields are always parsed as mutable (`is_final = false`). Should fields inherit a mutability keyword?
3.  **Error Handling vs Exceptions:** Viyal uses `?` and `or` for explicit error propagation. However, native methods and VM instructions (like missing method or out-of-bounds array access) cause unrecoverable VM panics. A decision is needed on whether user-code can catch VM-level panics.
