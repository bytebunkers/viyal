# Repository Audit

## 1. Overview
This audit examines the Viyal language repository at Phase 00. The objective is to evaluate current implementations, identify architectural branches, and locate broken or incomplete features.

## 2. Architecture Mapping
The compilation and execution pipelines diverge into multiple paths after parsing and type checking.

### Frontend
1. **Lexer/Parser**: Source code -> `ast::Program`. (`compiler/lexer`, `compiler/parser`)
2. **Type Checking**: AST-based static typing. (`compiler/typechecker`)
3. **Resolver**: Links multiple files, produces a unified `ast::Program`. (`compiler/resolver`)

### Execution Paths
* **Path A: VM Interpretation (Default)**
  - Flow: AST -> `bytecode::compiler::BytecodeCompiler` -> `bytecode::Chunk` -> `vm::vm::VM`
  - Status: Mostly functional. Supported by `viyal run` and `viyal test`.
* **Path B: C Code Generation (Default Build)**
  - Flow: AST -> `codegen::generate_c` -> C Source Code (`.viyal_out.c`) -> `gcc` -> Executable
  - Status: Severely stubbed. Hardcoded to print strings and integers. Fails entirely if `gcc` is missing.
* **Path C: MIR & Cranelift AOT (`--release`)**
  - Flow: AST -> `mir::builder::MirBuilder` -> `mir::ir::MirProgram` -> `optimizer::optimize` -> `backend::compile_aot` (Cranelift) -> System Linker -> Executable
  - Status: Broken. Cranelift verifier throws errors when compiling `hello.vyl`.
* **Path D: MIR VM Interpretation (`--optimize`)**
  - Flow: AST -> MIR -> Optimizer -> `bytecode::mir_compiler::MirBytecodeCompiler` -> `bytecode::Chunk` -> `vm::vm::VM`
  - Status: Present, but experimental/unstable compared to default VM path.

### Tooling
* **CLI (`tools/cli`)**: Main driver.
* **LSP (`tools/lsp`)**: Basic LSP server connected to parser/typechecker diagnostics.
* **Formatter (`tools/formatter`)**: AST-based pretty printer.
* **Analyzer (`tools/analyzer`)**: Basic static linting.
* **Debugger (`tools/debugger`)**: Basic REPL stepping over the VM.

## 3. Findings

### Finding 1: Broken Cranelift AOT Backend
* **Severity**: P1
* **Evidence**: `compiler/backend/src/aot.rs`. Running `cargo run --bin viyal -- build hello.vyl --release` panics with "AOT build error: [AOT] Failed to define 'Main::run': Compilation error: Verifier errors".
* **Current Behavior**: Cranelift verifier fails during AOT build.
* **Dependencies**: Needs valid MIR generation and correct translation to Cranelift IR.
* **Next Action**: Debug MIR translation in `compiler/backend/src/translator.rs`.

### Finding 2: Stubbed C Code Generation
* **Severity**: P2
* **Evidence**: `compiler/codegen/src/lib.rs`.
* **Current Behavior**: Hardcodes `print` function calls and relies on an external `gcc` installation. Completely unsupported statements are silently ignored.
* **Dependencies**: N/A.
* **Next Action**: Either deprecate the C codegen path in favor of Cranelift or fully implement AST to C lowering.

### Finding 3: Clippy Warnings and Errors
* **Severity**: P2
* **Evidence**: `runtime/gc/src/allocator.rs` (clippy error for missing Default).
* **Current Behavior**: `cargo clippy -- -D warnings` fails in the `gc` crate due to `new_without_default`. Several other crates emit unused imports and variables.
* **Dependencies**: N/A.
* **Next Action**: Fix the `Default` trait implementation in `gc/src/allocator.rs` and clean up unused imports.

### Finding 4: Unformatted Code
* **Severity**: P3
* **Evidence**: `vm/src/vm.rs` and `vm/tests/vm_tests.rs`.
* **Current Behavior**: `cargo fmt --check` fails.
* **Dependencies**: N/A.
* **Next Action**: Run `cargo fmt` across the workspace.

### Finding 5: Flawed Test Discovery
* **Severity**: P2
* **Evidence**: `tools/cli/src/main.rs:134`.
* **Current Behavior**: The `viyal test` command explicitly looks for methods starting with `test_` inside `class` declarations. Standalone functions like `void main()` in `test_for.vyl` are completely skipped.
* **Dependencies**: Test runner framework.
* **Next Action**: Update the test runner to warn if a file contains no classes, or support standalone test functions.
