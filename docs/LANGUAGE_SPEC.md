# Initial MVP Specification (Phase M0)

This is the authoritative MVP specification for the language.

## 1. Syntax Overview
The language uses a strict C-style syntax with expression-oriented capabilities where appropriate, maintaining a balance between familiarity and modern ergonomics.

```Viyal
class Person(String name, int age) {
    void greet() {
        print("Hello $name");
    }
}

void main() {
    final person = Person("Raja", 25);
    person.greet();
}
```

## 2. Variables and Declarations
- `final`: Immutable reference (value cannot be reassigned).
- `var`: Mutable reference (type inferred).
- Type annotations can precede the variable name.

```Viyal
String name = "Raja";
var age = 25;
final score = 98.5;
```

## 3. Core Types
- `int`: 64-bit signed integer.
- `double`: 64-bit floating point.
- `bool`: `true` or `false`.
- `String`: UTF-8 encoded string.
- Collections: `List<T>`, `Set<T>`, `Map<K, V>`.

## 4. Null Safety
Null safety is baked into the type system. Types are non-nullable by default.
- `T` is non-nullable.
- `T?` is nullable.

```Viyal
String name = "Raja"; // Valid
String? nickname = null; // Valid
// String invalid = null; // Compiler Error
```
Control flow analysis performs smart casting:
```Viyal
if (nickname != null) {
    print(nickname.length); // nickname is treated as String here
}
```

## 5. Functions
Functions support positional, named, default, and optional parameters.
```Viyal
int add(int a, int b) {
    return a + b;
}

final multiply = (int a, int b) => a * b;
```

## 6. Object-Oriented Features
- Classes with primary constructors (e.g., `class Point(int x, int y)`).
- Interfaces declared explicitly (`interface Logger { ... }`).
- Abstract classes for partial implementations.
- No implicit interfaces (unlike Dart).

## 7. Control Flow
Standard C-style constructs:
- `if` / `else`
- `for` (C-style and `for-in`)
- `while` / `do-while`
- `switch` (exhaustive by default on enums and sealed classes)

## 8. Exclusions for MVP
The MVP (M0-M6) will *not* include:
- Advanced metaprogramming (macros)
- AI-native syntax (e.g., `model` declarations) - Deferred to Phase M15.
- Complex trait systems (using simpler interfaces/mixins initially).
