# Viyal Testing Strategy

## 1. Conformance Testing
The language conformance suite (`tests/conformance`) exists to establish a singular, authoritative baseline for Viyal execution behavior.
*   **Methodology**: We execute Viyal source code simultaneously through two independent execution paths:
    1. The AST-walking `Interpreter`.
    2. The Bytecode `VM`.
*   **Validation**: Both implementations must emit identical standard output, successfully execute the script, and return matching exit status. This guarantees execution environment parity.
*   **Coverage**: Variables, Arithmetic, Strings, Functions, Control Flow (If, While), Arrays, Maps, Classes/Objects, and Error Handling (Null Safety, Try/Or blocks).

## 2. Fuzzing
Due to dependency and scope limitations, we utilize a custom fast pseudo-random number generator (`XorShift`) embedded directly in the `tests/conformance/fuzz.rs` module.
*   **Lexer**: Fuzzed with arbitrary byte arrays to guarantee panic-free tokenization.
*   **Parser**: Fuzzed with massive structurally malformed streams to guarantee resilience against infinite loops and recursive stack overflows.
*   **TypeChecker**: Fuzzed with pseudo-valid structural templates containing arbitrary identifiers to test namespace resolution and scoping panics.
*   **Rules**: A fuzz failure is never ignored; the offending seed and bytes must be transcribed into a permanent regression test.

## 3. Tooling Validation
The `tools/` crate tests are largely integration tests validated during Phase 08.
*   **CLI**: Tested for argument parsing, missing file handling, and exit codes.
*   **Formatter**: Tested for idempotence (formatting already formatted code yields no changes).
*   **LSP**: Real-time diagnostic generation on malformed code.

## 4. Property and Regression Testing
When bugs are identified, minimal reproduction cases are established in `tests/regression/`. Tests must cleanly exit with expected failure codes when designed to do so, without crashing the host process.
