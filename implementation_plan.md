# Vlang Inspiration Plan

Based on the research into Vlang (vlang.io), I have identified several key features that we can integrate into **Viyal** to make it a significantly better programming language.

## Proposed Changes to the Roadmap & Spec

### 1. Immutability by Default
Like Vlang and Rust, variables and structs should be immutable by default. We should introduce a `mut` keyword. This makes the language safer and helps the compiler optimize.
- **[MODIFY]** `docs/TYPE_SYSTEM.md`: Update rules to enforce immutability.
- **[MODIFY]** `compiler/parser/src/lib.rs` & `compiler/ast/src/lib.rs`: Update `VarDecl` to parse `let mut x = ...` instead of assuming variables are mutable.

### 2. Built-in Testing Framework
Vlang provides a native `test_` function convention. We can integrate a testing framework directly into the `viyal` CLI tool.
- **[MODIFY]** `docs/ROADMAP.md`: Update M12 (Tooling) to explicitly include a Built-in Testing Framework (`viyal test`).

### 3. Hot Code Reloading
Vlang supports hot code reloading, allowing changes to reflect instantly without losing state. Since Viyal uses a custom Bytecode VM (M6), we can implement hot-swapping of bytecode chunks at runtime!
- **[MODIFY]** `docs/ROADMAP.md`: Add a new milestone **M17 — Hot Code Reloading** for the VM.

### 4. Option/Result Types instead of Exceptions
Vlang enforces error handling by returning `Option/Result` and avoiding hidden exceptions. 
- **[MODIFY]** `docs/TYPE_SYSTEM.md`: Document `Result<T, E>` as the primary error handling mechanism instead of `try/catch`.

## Open Questions

> [!IMPORTANT]
> **User Feedback Required**
> Do you approve of these proposed language design changes? Specifically:
> 1. Do you want to adopt **immutability by default** with a `mut` keyword?
> 2. Do you agree with using **Result/Option types** instead of `try/catch` exceptions?
> 3. Should I go ahead and update the `ROADMAP.md` and `TYPE_SYSTEM.md` documents to reflect this?
