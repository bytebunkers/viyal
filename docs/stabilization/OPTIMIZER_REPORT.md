# Viyal Optimizer Correctness Report

## Overview
This report documents the current status, preconditions, invariants, and testing of optimization passes in the Viyal compiler pipeline, particularly focusing on how edge cases such as integer overflow, division by zero, floating point edge cases, and side effects are handled.

## Optimization Pipeline
The optimization pipeline consists of the following sequence:
1. **SSA Construction (`ssa.rs`)**: Transforms the MIR into Static Single Assignment form to simplify dataflow analysis.
2. **SCCP (`sccp.rs`)**: Sparse Conditional Constant Propagation.
3. **Constant Folding (`constant_folding.rs`)**: Folds operations on constant literals.
4. **GVN (`gvn.rs`)**: Global Value Numbering / Common Subexpression Elimination.
5. **De-SSA (`de_ssa.rs`)**: Reverts SSA form to standard MIR, inserting necessary copies.
6. **Dead Code Elimination (`dce.rs`)**: Removes unreachable blocks from the control flow graph.

## Pass Inventory & Invariants

### 1. SCCP (Sparse Conditional Constant Propagation)
*   **Status**: Active and integrated.
*   **Preconditions**: MIR must be in valid SSA form.
*   **Transformations**: Optimistically propagates constant values through PHI nodes and conditional branches. Eliminates unreachable branches internally and sets unreachable blocks to evaluate to `Lattice::Bottom`.
*   **Invariants**: Replaces `Operand::Copy` instances with `Operand::Constant` if the value is proven constant. Rewrites branch terminators to direct `Goto`s when conditions are statically known.
*   **Edge Case Handling (Fixed)**: Previously, integer operations were evaluated using standard wrapping semantics, which could silently corrupt runtime execution. It has been updated to use `checked_` math operations. Integer overflow now evaluates to `Lattice::Top` to prevent silent corruption at compile time and leave overflow handling to the VM.

### 2. Constant Folding
*   **Status**: Active and integrated.
*   **Preconditions**: Operates on standard MIR. Usually runs after SCCP.
*   **Transformations**: Replaces side-effect-free binary operations containing two `Operand::Constant` values with a single `Operand::Constant` result.
*   **Invariants**: Only deterministic mathematical and logical operations are folded.
*   **Edge Case Handling (Fixed)**: Previously panicked on overflow during compile time. It has been updated to use `checked_` operations, falling back to runtime execution on overflow or division by zero, thereby preserving language semantics and preventing compiler crashes. Floating-point behavior conforms to IEEE-754 semantics.

### 3. GVN / CSE (Global Value Numbering)
*   **Status**: Active and integrated (Restricted).
*   **Preconditions**: MIR must be in SSA form.
*   **Transformations**: Identifies duplicate computations of side-effect-free RValues and replaces subsequent instances with copies of the originally computed `Local`.
*   **Invariants**: Only "pure" expressions are hoisted. Expressions with potential side-effects (e.g., allocations, function calls) are ignored.
*   **Edge Case Handling (Fixed)**: The previous implementation performed unsafe global matching across independent basic blocks without consulting a Dominator Tree, which led to incorrect substitution across unexecuted branches. It has been restricted to Local Value Numbering (scoping the tracking table to a single basic block) to ensure correctness while maintaining safety. 

### 4. DCE (Dead Code Elimination)
*   **Status**: Active and integrated.
*   **Preconditions**: Operates on whole MIR program.
*   **Transformations**: Removes basic blocks that are completely unreachable from the entry block (block 0).
*   **Invariants**: Does not alter the internal statements or terminators of any reachable block. Ensures basic blocks are safely disconnected from the CFG.

## Known Limitations and Unspecified Semantics
As per the current language specification:
1. **Integer Overflow**: The behavior of integer overflow is left unspecified in the VM (it maps to Rust's native arithmetic which panics in debug mode and wraps in release). The optimizer avoids making assumptions and preserves the original operations when overflow is detected during constant folding.
2. **Floating-point Edge Cases**: Floating-point operations strictly evaluate to standard `NaN` and `inf` at compile-time as supported by the underlying host system.
3. **Array/String Indexing Side-effects**: Array bounds checking is inherently a side-effect (capable of halting the VM with a `RuntimeError`). Current GVN does not eliminate bounds-checking since it avoids cross-block hoisting. Future global CSE passes must track execution safety boundaries to avoid removing observable panics.

## Testing Status
All passes have been verified through:
*   Unit tests explicitly verifying transformation invariants.
*   Negative tests preventing constant-folding bugs (e.g. division by zero, integer overflow).
*   Regression tests addressing cross-block interference in GVN.
*   Integration checks comparing full Unoptimized VM execution versus Optimized VM execution in `run_conformance`.
