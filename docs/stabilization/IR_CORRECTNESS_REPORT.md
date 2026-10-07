# IR Correctness & Analysis Report

## 1. Overview
This report documents Phase 04 stabilization findings regarding the Intermediate Representations (HIR and MIR) in the Viyal compiler. The goal was to audit the transformation pipelines, basic-block CFG structures, and data flow tracking.

## 2. Transformations & Bypasses
The repository architecture specifies two levels of IR: HIR (High-Level) and MIR (Mid-Level).

### HIR (High-level Intermediate Representation)
- **Status**: Non-existent (Stubbed).
- **Finding**: The `compiler/hir` crate contains a single `lib.rs` file marking it as "Planned (v0.2)". It does not perform any desugaring. 

### MIR (Mid-level Intermediate Representation)
- **Transformation**: `ast::Program` -> `mir::ir::MirProgram`.
- **Status**: Partially implemented in `compiler/mir/src/builder.rs`.

### Architecture Bypasses
The vast majority of the compiler backends **bypass MIR completely**.
- `BytecodeCompiler` (used by `viyal run`) operates directly on the AST.
- `Interpreter` (used by evaluation tests) operates directly on the AST.
- `C Codegen` operates directly on the AST.
- **Only Cranelift AOT** uses the MIR, but Cranelift compilation is currently broken and experimental.
- **Impact**: Because the primary backends (Bytecode/Interpreter) consume AST rather than MIR, MIR correctness has historically not been critical to execution, explaining the severe bugs found within it.

## 3. Structural Correctness & Bugs

### Control-Flow Graph (CFG) Construction
- **Branches & Loops**: The `MirBuilder` correctly lowers `if`, `while`, `for ... in`, and `for ... in a..b` constructs into `BasicBlock` structures with valid `Terminator::If` and `Terminator::Goto` nodes. 
- **Tests Added**: Structural invariants for empty functions, variable declarations, and control flow branching were verified and stabilized in `compiler/mir/tests/mir_builder_tests.rs`.

### Variable Resolution & Data Flow (Critical Bugs)
- **Identifier Resolution Bug**: `Expr::Identifier` is hardcoded to emit `Operand::Copy(Local(0))` regardless of scope or variable name (see `builder.rs:308`). Variable tracking environment mapping does not exist.
- **Use-Before-Definition**: The builder performs no validation of variable initialization. 
- **SSA Form**: While `Phi` nodes exist in `ir.rs`, they are never emitted or calculated by the `MirBuilder`.

### Information Loss During Lowering
- **Source Locations (Spans)**: Spans are entirely discarded during AST-to-MIR lowering. `mir::ir` structs (`BasicBlock`, `Statement`, `Rvalue`) contain no location tracking, making it impossible to produce accurate compiler errors during MIR optimization.
- **Type Information**: Types are preserved on `LocalDecl` and function signatures, but dropped on intermediate expressions and `Rvalue`s.
- **Closures**: `Expr::Closure` is not implemented in `MirBuilder` and silently falls back to emitting a `Null` constant.

## 4. Conclusion
The MIR module is structurally sound in CFG allocation but fatally incomplete in identifier resolution, scope tracking, and span retention. Because the functioning backends (Bytecode / Interpreter) bypass MIR, these defects do not break `viyal run`. The MIR pipeline should not be adopted by the Bytecode compiler until these fundamental variable resolution bugs are addressed.
