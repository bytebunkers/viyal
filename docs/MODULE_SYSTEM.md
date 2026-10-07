# Viyal Module System

This document describes the state of the Viyal module system.

## 1. Syntax

Viyal defines syntax for importing and exporting items:
*   **Export:** Functions, classes, and type aliases can be prefixed with the `export` keyword.
*   **Import:** `import { item1, item2 as alias } from "path";`

## 2. Compiler Implementation Status

*   **Parser:** Successfully parses `import` and `export` statements into the AST (`Decl::Import`, and `is_exported` flags).
*   **Typechecker:** 
    *   Exports are recorded but not enforced across file boundaries.
    *   Imports are explicitly marked as "To be implemented" and are currently skipped during typechecking.
*   **Bytecode Compiler:** Does not support compiling imports. It assumes all classes and functions are available in a single global compilation unit.

## 3. Native Modules

The VM supports injecting native modules written in Rust into the global scope.
*   When a native module is registered (e.g., `math`, `fs`), it is instantiated as a `Value::Object(ModuleObj)` in the global variables map.
*   Native functions attached to these modules can be invoked using standard property access and invocation (`math.pow(2, 3)`).
*   The typechecker recognizes a hardcoded list of standard library modules (`math`, `fs`, `os`, `time`, `http`, `json`, `path`, `process`, `str`) and assigns them the `Module` type.

## Conclusion

The module system for user-defined Viyal files is unimplemented. All Viyal code must currently reside in a single file or be concatenated before compilation. Only Rust-injected native modules are fully supported.
