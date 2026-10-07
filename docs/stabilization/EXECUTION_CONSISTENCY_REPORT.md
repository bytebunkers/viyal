# Phase 06: Execution Consistency Report

## Supported Execution Backends & Capability Matrix

The Viyal language execution pipeline currently supports multiple execution paths:
1. **Interpreter**: AST-walking interpreter (used mostly for rapid validation).
2. **Bytecode VM**: Compiles AST directly to custom Viyal bytecode and executes it in a stack machine.
3. **MIR JIT/AOT** (Experimental): Translates AST to MIR, optimizes it, and then compiles to Cranelift IR for JIT execution or AOT object generation.
4. **MIR VM** (Experimental): Compiles MIR to Viyal Bytecode. Currently heavily lacking feature support.

### Capability Matrix

| Feature / Construct | Interpreter | Bytecode VM | MIR JIT/AOT | MIR VM |
| :--- | :--- | :--- | :--- | :--- |
| **Arithmetic** (+, -, *, /) | Supported | Supported | Supported | Supported |
| **Variables** (var, mut) | Supported | Supported | Supported | Supported |
| **Control Flow** (if, while) | Supported | Supported | Supported | Supported |
| **For Loops** (range, in) | Supported | Supported | Unsupported | Unsupported |
| **Match Expressions** | Supported | Supported | Unsupported | Unsupported |
| **Functions** | Supported | Supported | Static Only | Unsupported |
| **Strings** | Supported | Supported | Unsupported | Unsupported |
| **Arrays & Maps** | Supported | Supported | Unsupported | Unsupported |
| **Classes & Methods** | Supported | Supported | Unsupported | Unsupported |
| **Error Handling** (?, or) | Supported | Supported | Unsupported | Unsupported |
| **Dynamic Calls** (print) | Supported | Supported | Unsupported | Unsupported |

*Note: The MIR backends (JIT/AOT and MIR VM) are in experimental stages and only support a narrow subset of language features (primarily basic arithmetic and control flow). They cannot yet handle heap allocations, objects, collections, or dynamic print operations.*

## Conformance Testing

To ensure consistency across execution environments, we've established a shared conformance suite in `tests/conformance/src/lib.rs`. The suite validates that valid Viyal programs produce identical observable outputs and exit behavior across the supported backends.

The conformance tests cover:
- Arithmetic operations (`test_arithmetic`)
- String literals and output (`test_string`)
- Functions (`test_functions`)
- Control flow (`test_control_flow`)
- Arrays and Map Collections (`test_arrays`, `test_maps`)
- Classes and field assignment (`test_classes_and_fields`)
- Error handling with `error()`, the `!` type suffix, and `or` fallback (`test_error_handling`)

Because the `Bytecode VM` and `Interpreter` represent the current golden standard for feature completeness, they are the primary targets of the conformance tests. The MIR backends correctly fail with clear compiler errors when attempting to compile unsupported features rather than silently corrupting output.

## Known Discrepancies & Discovered Issues

- **Type Parsing for Errors**: `Result<T>` and `Option<T>` must be specified using `T!` and `T?` respectively in Viyal (e.g. `int!`), not with generic-like syntax.
- **Map Literals**: Map literal syntax uses braces `{ "key": val }`, not brackets `[ "key": val ]` as originally stated in the spec. The `LANGUAGE_SPEC.md` has been corrected to reflect the actual parser implementation.
- **Backend Outputs**: JIT execution returns a single integer value from the compiled `main` function but lacks support for string serialization and standard output mapping (`print`).
