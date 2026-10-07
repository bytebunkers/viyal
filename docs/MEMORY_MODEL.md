# Viyal Memory Model

## 1. Object Allocation
Viyal currently implements a custom, heap-based arena allocator for complex data types (`String`, `Array`, `Map`, `Class`, `Instance`, `Function`, `Module`, `NativeFunction`).

- **Allocator**: `GcAllocator<T>` (defined in `runtime/gc/src/allocator.rs`).
- **Handle**: A `GcHandle` (effectively a `usize` index) is returned upon allocation and used universally across the VM to reference objects. This avoids unsafe Rust pointers and enables memory-safe object graphs.

## 2. Garbage Collection (GC)
The system currently uses an un-triggered Mark and Sweep garbage collector.
- **Marking**: The arena supports marking (`allocator.mark(handle)`), but the VM `CallFrame` stack and variable slots are not actively traversed to mark reachable objects during execution.
- **Sweeping**: The `allocator.sweep()` method correctly reclaims un-marked slots for future allocations. However, since the VM does not trigger garbage collection passes, this method is currently bypassed.
- **Effect**: Viyal programs leak memory monotonically. Memory will not be reclaimed until the VM process exits.

## 3. Reference Semantics
Viyal employs shared mutability for reference types (objects, arrays, maps).
- Assignments of objects do not deep copy. Both variables share the same `GcHandle`.
- Pass-by-value is used for primitive types (integers, floats, booleans, null).

## 4. FFI Boundaries
Native functions receive arguments as borrowed `&[Value]` slices. If native functions need to allocate complex Viyal objects to return to the script, they must do so via the provided `&mut VM` parameter, seamlessly tying those allocations to the VM's arena.

## 5. Known Vulnerabilities
- **Memory Leak**: The lack of automatic garbage collection triggering means long-running Viyal processes (e.g. servers) will eventually OOM.
- **Dangling Handles**: Safe by default. Because `GcHandle` is an index, if a bug were to incorrectly sweep an object, subsequent lookups would return `None` (resulting in a VM `RuntimeError` rather than undefined behavior).

## Future Directives (Phase 08+)
To achieve a production-grade runtime, the VM's execution loop must be instrumented to periodically invoke GC. This requires traversing the `VM.stack`, `VM.frames`, and `VM.globals` to accurately mark reachable handles.
