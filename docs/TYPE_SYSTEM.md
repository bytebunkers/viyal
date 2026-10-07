# Viyal Type System

This document specifies the Viyal type system as it is currently implemented in the Phase 01 codebase.

## 1. Built-in Types

*   `int`: 64-bit signed integer.
*   `double`: 64-bit floating-point number.
*   `bool`: Boolean (`true` or `false`).
*   `String`: Text string.
*   `void`: Represents the absence of a value (e.g., function return types without a return statement).
*   `null`: Represents a null literal.
*   `Any`: A wildcard type. Variables or functions typed as `Any` bypass strict type checks.
*   `Error`: Represents a runtime error value.

## 2. Compound Types

*   **Array (`Type[]`):** A collection of elements of a uniform type.
*   **Map (`Map<K, V>`):** A key-value collection. Keys must currently be `String` at runtime.
*   **Nullable (`Type?`):** Represents a type that can be either `Type` or `null`.
*   **Result (`Result<Type>`):** Represents either a successful value of `Type` or an `Error`.
*   **Option (`Option<Type>`):** Represents either a value of `Type` or absence.

## 3. Type Checking Rules

The typechecker implements a static, nominal type system with specific compatibility rules:

*   **Identity:** A type matches itself.
*   **Nullability:** `null` is compatible with any type during equality checks (currently overly permissive).
*   **Any:** `Any` is compatible with all types.
*   **Class Types:** Checked nominally. A named type starting with an uppercase letter is assumed to be a valid class/object type and is currently allowed to bypass some strict resolution if not found, or matches any other uppercase type (due to MVP constraints).

## 4. Generics

Generics are supported in syntax for classes (`class Foo<T>`), methods (`T method<T>()`), and type parameters (`Map<String, int>`).
**Current Limitation:** During typechecking, generic parameters (`<T>`) are aliased to `Any`. True parametric polymorphism and monomorphization are not yet implemented.

## 5. Type Inference

Type inference is minimal:
*   Variables declared with an initializer (`var x = 1;`) infer their type from the expression.
*   Variables without an initializer (`var x;`) infer `void`.
*   If a type annotation is provided (`Type x = expr;`), the inferred type of `expr` must match `Type`.

## 6. Known MVP Behaviors

*   Native modules (e.g., `math`, `fs`, `os`) evaluate to a built-in `Module` type.
*   Property access on objects or modules currently evaluates to `Any`.
*   Indexing an array with an `int` returns the inner array type. Indexing a map returns the inner value type.
