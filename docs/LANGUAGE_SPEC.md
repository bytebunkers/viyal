# Viyal Language Specification

This document details the implemented semantics of the Viyal language (v0.1.0) based on the Phase 00 and Phase 01 codebase audit. Features described here are supported by the existing AST, Parser, Typechecker, Bytecode Compiler, and VM.

## 1. Lexical Rules

*   **Whitespace:** Spaces, tabs (`\t`), newlines (`\n`), form feeds (`\f`), and carriage returns (`\r`) are ignored.
*   **Comments:** 
    *   Single-line: `// ...` to the end of the line.
    *   Block: `/* ... */` (nested block comments are not supported).
*   **Identifiers:** Begin with a letter or underscore, followed by letters, digits, or underscores (`[a-zA-Z_][a-zA-Z0-9_]*`).
*   **Keywords:** `class`, `extends`, `implements`, `new`, `this`, `super`, `interface`, `void`, `final`, `var`, `mut`, `if`, `else`, `for`, `while`, `do`, `switch`, `return`, `async`, `await`, `true`, `false`, `null`, `in`, `type`, `match`, `import`, `export`, `from`, `as`.
*   **Literals:**
    *   **Integer:** Sequence of digits (`[0-9]+`), parsed as 64-bit signed integer.
    *   **Float:** Digits with a decimal point (`[0-9]+\.[0-9]+`), parsed as 64-bit float.
    *   **String:** Double-quoted strings (`"..."`). String interpolation is not implemented.
    *   **Boolean:** `true` and `false`.
    *   **Null:** `null`.

## 2. Variables and Declarations

Variables are declared using `var` or `mut`:
*   `var name = expr;` declares an **immutable** (final) variable.
*   `mut name = expr;` declares a **mutable** variable.

The parser infers type from the initializer. If there is no initializer, the type defaults to `void` (or `null` in the VM). Type annotations are parsed but checked against the inferred type.

## 3. Operators

*   **Arithmetic:** `+`, `-`, `*`, `/` (Supported on integers; `+` also supported on floats).
*   **Comparison:** `==`, `!=`, `<`, `>`, `<=`, `>=`.
*   **Logical:** Not fully implemented as short-circuit operators in bytecode.
*   **Assignment:** `=` (reassignment is verified for mutability during typechecking).
*   **Property Access:** `.` (e.g., `obj.field`).
*   **Safe Property Access:** `?.` (Parsed, but evaluates to `Any` during typechecking and lacks specific VM behavior in MVP).
*   **Null Coalescing:** `??` (Parsed, evaluates right hand side unconditionally at runtime).
*   **Error Handling:** `?` (Try operator), `or` (Unwrap or else).

## 4. Control Flow

### If / Else
Standard `if (condition) stmt [else stmt]` structure. Conditions must evaluate to a boolean type in the typechecker, though the VM treats anything other than `false` or `null` as truthy.

### Loops
*   **While loop:** `while (condition) stmt`.
*   **For-Range loop:** `for item in start..end stmt`. `start` and `end` must be integers. The loop is exclusive of `end`.
*   **For-In loop:** `for item in iterable stmt`. Supported over arrays and strings.

### Match Expression
`match target { pat => expr, ... }`.
*   Matches evaluate to a value.
*   Patterns can be literals (integers, strings, booleans), identifiers, or the catch-all `_`.
*   **Note:** In the current MVP, identifier bindings do not bind local variables in the VM; they act similarly to `_`. A catch-all `_ => expr` arm is required by the typechecker.

## 5. Functions

Top-level functions are defined as:
```viyal
Type functionName(Type1 param1, Type2 param2) { ... }
```
Functions can be marked `export`. Type parameters (`<T>`) are parsed but aliased to `Any` during typechecking.

## 6. Classes

Classes are defined with `class`. They support:
*   **Primary Constructor:** `class Name(Type param1) { ... }`.
*   **Generics:** Parsed but treated as `Any`.
*   **Inheritance:** `extends SuperClass` (Parsed, but runtime method resolution doesn't traverse superclasses).
*   **Interfaces:** `implements Interface1, Interface2` (Parsed only).
*   **Fields:** `Type fieldName;` (All fields default to mutable in the AST parser).
*   **Methods:** `Type methodName() { ... }`.

Instantiation uses `new`: `new ClassName(args)`.

## 7. Arrays and Maps

*   **Arrays:** `[expr1, expr2]`. Zero-indexed. Elements must share a common type (or evaluate to `Any`).
*   **Maps:** `{key1: val1, key2: val2}`. Keys must be Strings.

## 8. Type Aliases
`type Alias = Type;`
Registers a type alias during the typechecking pre-pass.
