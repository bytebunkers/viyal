# Known Issues

This document tracks identified bugs, limitations, and areas of missing functionality in the Viyal project at Phase 00.

## 1. Compiler Backend
* **Cranelift AOT Backend is Broken (P1)**: Compiling via `viyal build --release` produces Cranelift verifier errors during function definition. Found in `compiler/backend/src/aot.rs` and `compiler/backend/src/translator.rs`.
* **C Code Generation is Stubbed (P2)**: The legacy C codegen path (`codegen::generate_c`) handles only basic literals and `print`. Most AST nodes are ignored or cause failures. Requires GCC installed on the host.

## 2. Tooling and CLI
* **Test Runner Discovery is Limited (P2)**: The `viyal test` runner only searches for methods prefixed with `test_` within a `class`. Top-level functions or standalone scripts are silently ignored (e.g., `test_for.vyl`).
* **Clippy Lints are Failing (P2)**: `cargo clippy` fails due to `clippy::new-without-default` in `gc/src/allocator.rs`, along with widespread unused import/variable warnings in `optimizer`, `mir`, `bytecode`, and `backend`.
* **Workspace Formatting (P3)**: Some workspace files (`vm/src/vm.rs`, `vm/tests/vm_tests.rs`) fail `cargo fmt --check`.

## 3. Runtime & VM
* **Partial MIR Interpretation**: Running with `--optimize` uses the MIR-to-Chunk compiler (`mir_compiler.rs`) which has incomplete handling for some MIR structures, evidenced by unused indices/variables in bytecode generation.
