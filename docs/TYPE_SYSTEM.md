# Type System Definition

The type system is the foundation of the language. It must be statically typed, sound, and null-safe.

## 1. Soundness Guarantee
If an expression is evaluated to have type `T` at compile-time, it is guaranteed to evaluate to an object of type `T` at runtime.

## 2. Null Safety Rules
- A type `T` cannot contain the value `null`.
- A type `T?` is a union of `T | Null`.
- The compiler enforces explicit checks before `T?` can be used as `T`.

### Smart Casting (Flow-Sensitive Typing)
The type checker analyzes control flow to promote types automatically.
```Viyal
void process(String? input) {
    if (input == null) return;
    // From here on, 'input' is promoted to 'String'
    print(input.length); 
}
```

## 3. Generics
Generics are reified (like Dart/C#), not erased (like Java).
- `List<int>` is distinctly different from `List<String>` at runtime.
- Supports covariance and contravariance constraints where appropriate (`in` and `out` keywords).

## 4. Subtyping
- Nominal subtyping for classes and interfaces.
- `T` is a subtype of `T?`.
- `Bottom` type exists (similar to Dart's `Never` or Rust's `!`) for functions that never return (e.g., throwing unconditionally or infinite loops).
- `Top` type exists (similar to Dart's `Object?` or `Any`).

## 5. Records / Tuples
Anonymous, immutable aggregates of data.
```Viyal
(String, int) getPerson() {
    return ("Raja", 25);
}
```

## 6. Type Aliases
```Viyal
type Json = Map<String, dynamic>;
```
