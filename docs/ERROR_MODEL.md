# Viyal Error Model

This document outlines the error handling and propagation model implemented in Viyal.

## 1. The `Error` Type

Errors in Viyal are first-class values represented by the `Error` type (and internally by `Value::Error(String)` in the VM). 
Errors can be created manually using the built-in `error(msg)` function.

## 2. Try Operator (`?`)

The `?` operator is used to propagate errors and nulls.
*   **Syntax:** `expr?`
*   **Typechecker:** The target expression must be of type `Result<T>` or `Option<T>`. The expression evaluates to `T`.
*   **VM Behavior:** At runtime, the `OpTry` instruction checks the top of the stack. If the value is `Value::Error` or `Value::Null`, it immediately pops the current call frame and returns that error/null to the caller. Otherwise, execution continues.

## 3. Unwrap Or Else (`or`)

The `or` operator provides a fallback mechanism.
*   **Syntax:** `expr or { stmt }`
*   **Typechecker:** The target expression must be `Result<T>` or `Option<T>`. The block must return a compatible type or diverge (e.g., panic/return).
*   **VM Behavior:** Uses `OpJumpIfOk`. If the expression evaluates to a successful value, it jumps over the fallback block. If it is an `Error` or `Null`, it executes the fallback block.

## 4. Runtime Errors (Panics)

The VM can encounter unrecoverable states (e.g., type mismatches not caught by the typechecker, out-of-bounds array access, missing methods).
These result in an `InterpretResult::RuntimeError` halting the VM execution entirely. They are not catchable within the Viyal language itself using `try/catch` (which does not exist in the language).
