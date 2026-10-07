# Project Status

## Current Phase
Phase 09 — Fuzzing and Conformance (Completed)

## Completed
- Defined M0 implementation plan
- Created foundational project structure and documents
- Froze initial MVP specification
- Created repository structure (sub-directories)
- Initialized Rust workspace
- Implemented Lexer (M1)
- Wrote and passed Lexer tests (M1)
- Defined AST structure (M2)
- Implemented recursive descent parser (M2)
- Wrote and passed Parser tests (M2)
- Implemented tree-walking interpreter (M3)
- Implemented runtime value types and environment (M3)
- Executed Viyal programs end-to-end (M3)
- Implemented type resolution and variable scoping (M4)
- Enforced type checking rules with custom TypeErrors (M4)
- Verified TypeChecker via automated tests (M4)
- Added Lexer support for OOP and Null Safety tokens (M5)
- Upgraded AST to support fields, inheritance, and nullable types (M5)
- Rewrote parser to handle complex expressions and OOP constructs (M5)
- Verified workspace builds and tests pass (M5)
- Defined bytecode instruction set architecture (ISA) (M6)
- Implemented BytecodeCompiler to lower AST to Chunk (M6)
- Built high-performance Stack-Based Virtual Machine (M6)
- Verified VM execution with automated tests (M6)
- Add `HashSet` of breakpoints and `InterpretResult::Breakpoint` to `vm/src/vm.rs`.
- Expose introspection methods (`step`, `ip`, `stack`) on `VM`.
- Add `vm` and `bytecode` dependencies to `tools/debugger/Cargo.toml`.
- Implement `Debugger` struct and REPL in `tools/debugger/src/lib.rs`.
- Add `debugger` dependency to `tools/cli/Cargo.toml`.
- Add `viyal debug` command to `tools/cli/src/main.rs`.
- Test the debugger using `hello.vy`.d to support running without arguments in `tools/cli/src/main.rs`.
- Verify `viyal new` and `viyal run` locally. (M8)
- Registered `Math` and `IO` modules as native FFI bindings in the VM (M8)
- Verified standard library registration and execution (M8)
- Integrated `lsp-server` and `lsp-types` to build a synchronous LSP (M9)
- Hooked up `publishDiagnostics` to Viyal's Lexer, Parser, and TypeChecker (M9)
- Successfully compiled the Language Server binary (M9)
- Built `formatter` crate to standardize code layout (M10)
- Implemented an AST-traversing pretty-printer for classes and expressions (M10)
- Verified whitespace normalization and string generation via automated tests (M10)
- Built `analyzer` crate for AST-based static analysis and linting (M11)
- Implemented `AnalyzerRule` framework and `EmptyBlockRule` (M11)
- Verified analyzer warning generation via automated tests (M11)
- **Phase 00**: Completed comprehensive repository audit.
- **Phase 00**: Executed baseline tests and recorded results.
- **Phase 00**: Mapped compiler architecture and created audit documents.
- **Phase 01**: Inventoried parser, AST, typechecker, and VM.
- **Phase 01**: Created `LANGUAGE_SPEC.md` tracking lexical rules and grammar.
- **Phase 01**: Created `TYPE_SYSTEM.md` tracking the typechecker's nominal types and limitations.
- **Phase 01**: Created `ERROR_MODEL.md` detailing the Error type and Try/or operators.
- **Phase 01**: Created `MODULE_SYSTEM.md` identifying the state of native and script imports.
- **Phase 01**: Documented semantic gaps and missing implementations in `SEMANTIC_GAPS.md`.
- **Phase 02**: Expanded `lexer_tests.rs` with extensive token, span, unicode, and error coverage.
- **Phase 02**: Expanded `parser_tests.rs` with AST, negative testing, and error recovery cases.
- **Phase 02**: Added regression tests for missing precedence and top-level limitations.
- **Phase 02**: Generated `LEXER_PARSER_REPORT.md` documenting frontend robustness and defects.
- **Phase 03**: Resolved race conditions in multi-threaded `resolver_tests.rs` and thoroughly verified imports/mangling logic.
- **Phase 03**: Wrote extensive type checker tests validating assignment, function calls, shadowing, and return types.
- **Phase 03**: Documented gaps in semantic checking (e.g. ignored `if` statements, unvalidated return types).
- **Phase 03**: Generated `SEMANTIC_ANALYSIS_REPORT.md` documenting Phase 03 findings.
- **Phase 04**: Audited HIR and MIR, discovering HIR is a planned stub and MIR is bypassed by the primary AST-based Bytecode compiler.
- **Phase 04**: Added MIR control-flow graph tests to `mir_builder_tests.rs` verifying `BasicBlock` and `Terminator` emission for branches and loops.
- **Phase 04**: Identified critical bugs in MIR lowering (e.g. all identifiers map to `Local(0)` and spans are discarded).
- **Phase 04**: Documented findings in `IR_CORRECTNESS_REPORT.md`.
- **Phase 05**: Identified integer overflow panics in SCCP and constant folding, fixing them to return top-level Lattice/None and defer to VM runtime.
- **Phase 05**: Fixed critical correctness bug in GVN matching uninitialized state across independent branches by narrowing it to Local Value Numbering.
- **Phase 05**: Added execution tests utilizing Cranelift-JIT backend to cross-verify optimized vs unoptimized output.
- **Phase 05**: Documented optimizer invariants and semantic mapping constraints in `OPTIMIZER_REPORT.md`.
- **Phase 06**: Inventoried supported execution backends and documented the capability matrix.
- **Phase 06**: Implemented shared conformance suite for testing across VM and Interpreter.
- **Phase 06**: Identified language spec mismatch for map literals and resolved `Result`/`Option` type parsing syntax.
- **Phase 06**: Documented findings in `EXECUTION_CONSISTENCY_REPORT.md`.
- **Phase 07**: Audited GC and VM allocator, discovering arena allocation without `sweep` execution.
- **Phase 07**: Implemented and executed memory allocation stress testing (1,000,000 arrays).
- **Phase 07**: Verified FFI boundary safety and `NativeFunction` error propagation.
- **Phase 07**: Confirmed lack of unsafe Rust blocks and theoretical immunity to data races.
- **Phase 07**: Generated `MEMORY_MODEL.md`, `EXECUTION_MODEL.md`, and `RUNTIME_SAFETY_REPORT.md`.
- **Phase 08**: Verified CLI operations, handling of missing files, and diagnostic error output formatting.
- **Phase 08**: Identified and resolved a defect in the `test` command where standalone top-level functions were skipped.
- **Phase 08**: Verified LSP `TextDocumentSyncKind::FULL` initialization and live diagnostics piping.
- **Phase 08**: Audited the MVP interactive debugger `repl` commands.
- **Phase 08**: Generated `TOOLING_REPORT.md`.
- **Phase 09**: Added `tests/conformance/src/fuzz.rs` to generate continuous pseudo-random malformed byte streams to stress lexer and parser.
- **Phase 09**: Successfully ran fuzzer boundaries without provoking Rust panics or memory segfaults.
- **Phase 09**: Generated `TESTING_STRATEGY.md` and `CONFORMANCE_REPORT.md`.

## Existing Failures
- `cargo run --bin viyal -- build hello.vyl`: Fails due to missing `gcc` and incomplete C codegen.
- `cargo run --bin viyal -- build hello.vyl --release`: Fails due to Cranelift verifier error in AOT backend.
- `cargo run --bin viyal -- test math_test.vyl`: 1 test (`test_failure`) explicitly fails (this is expected behavior as the test checks assert failure).
- `conformance` tests involving `test_arithmetic`, `test_functions`, etc., panic when run via optimization pipeline because `MirBytecodeCompiler` does not support dynamic calls (documented in optimization architecture).

## Current Architecture
The compiler features a bifurcated backend:
1. **Frontend**: Lexer -> Parser -> AST -> TypeChecker -> Resolver
2. **VM Interpretation**: AST -> BytecodeCompiler -> Chunk -> VM (Functional)
3. **MIR VM**: AST -> MirBuilder -> Optimizer -> MirBytecodeCompiler -> Chunk -> VM (Experimental)
4. **C Codegen**: AST -> generate_c -> gcc -> Executable (Stubbed)
5. **AOT**: AST -> MirBuilder -> Optimizer -> compile_aot -> Linker -> Executable (Broken)

## Files Changed (Phase 05)
- Created: `docs/stabilization/OPTIMIZER_REPORT.md`
- Created: `compiler/optimizer/tests/execution_tests.rs`
- Modified: `compiler/optimizer/src/sccp.rs`
- Modified: `compiler/optimizer/src/constant_folding.rs`
- Modified: `compiler/optimizer/src/gvn.rs`
- Modified: `compiler/optimizer/Cargo.toml`
- Updated: `task.md`

## Remaining Tasks
- None for Phase 05.

## Exact Verification Commands and Results
- `cargo test` -> Passed (17 tests).
- `cargo build` -> Passed (with unused variable warnings).
- `cargo clippy -- -D warnings` -> Failed.
- `cargo fmt --check` -> Failed.
- `cargo run --bin viyal -- run hello.vyl` -> Passed (Prints: "Hello from Viyal CLI!").
- `cargo run --bin viyal -- test math_test.vyl` -> 3 passed, 1 failed (expected).
- `cargo run --bin viyal -- build hello.vyl` -> Failed (GCC not found).
- `cargo run --bin viyal -- build hello.vyl --release` -> Failed (Cranelift Verifier error).

## Next Recommended Task
- **Phase 06**: Proceed to stabilize and verify the Interpreter and Runtime execution backends.

## Architecture Decisions
- **Implementation Language**: Rust (chosen for compiler ecosystem, performance, memory safety).
- **Phased Execution**: Lexer -> Parser -> Interpreter -> Bytecode VM -> Native (LLVM/Cranelift).
- **Concurrency**: First-class structured concurrency with async/await (details in CONCURRENCY.md).
- **Syntax**: Strictly leaning toward C-style syntax (like Dart/Java).

## Language Decisions
- **Null Safety**: Sound null safety as a core language design feature.
- **Typing**: Statically typed, sound, generic, object-oriented.
- **AI Features**: Treated as exploratory (Phase M15).

## Compiler Decisions
- **Lexer/Parser**: Hand-written (or using Rust parser libraries) to emit AST.
- **Type Checker**: Emits HIR (High-level IR) after resolution and type inference.

## Runtime Decisions
- **Memory**: Automatic memory management (GC by default) — precise model TBD.

## Research Questions
- LLVM vs Cranelift for native backend.
- Generational GC vs Concurrent GC implementation in Rust.

## Future Features
- WebAssembly Backend (M13).
- Mobile/Desktop Ecosystem (M14).
- Advanced Metaprogramming/Compile-time evaluation (M15).
- AI-Native APIs (M15).
