# Runtime and Memory Model

## 1. Execution Architecture
Viyal will support multiple execution targets to balance development speed with production performance.

### Phase 1: Interpreter
- Direct evaluation of the AST.
- Extremely fast to build and iterate on compiler features.
- Used strictly for bootstrapping and early development.

### Phase 2: Bytecode VM
- AST compiled to Bytecode IR.
- Custom register or stack-based virtual machine (TBD).
- Fast compilation times, suitable for standard development iteration.

### Phase 3: Native Compilation (AOT)
- MIR compiled via LLVM or Cranelift to native machine code.
- Optimized for startup time and execution speed.
- Target for production deployments.

## 2. Memory Management (Garbage Collection)
Viyal relies on automatic memory management. We will not use Rust-style manual lifetimes or borrow checking, as it conflicts with the "Dart-like simplicity" goal.

### GC Design Research
- **Generational GC**: Most objects die young. A nursery for new allocations and a tenured space for long-lived objects.
- **Concurrent/Incremental GC**: To avoid long pause times (crucial for UI apps and low-latency servers).
- **Value Types**: Implementing structs/records that allocate on the stack to reduce GC pressure.

## 3. FFI (Foreign Function Interface)
- Seamless integration with C ABI.
- Essential for accessing system libraries and avoiding reinventing every wheel in the ecosystem.
- Unsafe operations must be explicitly wrapped in `unsafe` blocks or boundaries.
