# Metaprogramming & Compile-Time Evaluation

## 1. The Problem
Developers often need to generate code to avoid boilerplate (e.g., JSON serialization, database ORMs, routing tables).
Dart struggled with this for years, relying on `build_runner` (a slow, external text-based generation tool) before attempting to build native macros.

## 2. Viyal's Approach (Phase M15)
Viyal will adopt a deterministic, robust compile-time code generation model.

### Compile-Time Reflection
Instead of relying strictly on runtime reflection (which harms tree-shaking and AOT compilation), Viyal will allow querying type information at compile-time.

### Macros
```Viyal
@Serializable
class User {
    String name;
    int age;
}
```
The `@Serializable` annotation will trigger a macro during compilation.
Unlike text-based generation, this macro will operate on the AST/HIR and inject new AST nodes directly into the compiler pipeline.

## 3. Evaluation Environment
Macros must run inside a sandboxed evaluation environment during compilation (e.g., a small WASM runtime or internal interpreter) to ensure they are deterministic and fast.

## 4. Constant Evaluation
Functions can be marked as `const` or `comptime` to force evaluation during compilation.
```Viyal
comptime int computeHash() {
    return 42; // Evaluated by the compiler
}
```
