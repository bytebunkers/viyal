# Intermediate Representation (IR) Design

To decouple the front-end (syntax/type checking) from the back-end (code generation), the compiler uses a series of Intermediate Representations.

## 1. AST (Abstract Syntax Tree)
- Closest to the source code.
- Contains syntax information, line/column metadata (for error reporting).
- Highly language-specific.

## 2. HIR (High-Level IR)
- Desugared version of AST.
- E.g., `for` loops might be lowered into `while` loops or basic blocks with branches.
- Type information is fully resolved and attached to nodes.
- Closures and captures are explicitly resolved.

## 3. MIR (Mid-Level IR)
- Control Flow Graph (CFG) based.
- Consists of Basic Blocks, instructions, and jumps.
- Perfect for optimization passes:
  - Dead code elimination
  - Constant folding
  - Inlining
- Async/await state machines are fully expanded at this level.

## 4. Bytecode IR
- Specific to the Viyal Virtual Machine.
- Stack-based or Register-based (TBD).
- Designed for fast interpretation.

## 5. Native IR
- Translation from MIR into LLVM IR (or Cranelift).
- Hooks into advanced back-end optimizations.
