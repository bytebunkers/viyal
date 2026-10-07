# Phase 07: Runtime Safety & Memory Report

## Objective
Assess the memory safety, resource lifecycle, concurrency primitives, and FFI boundaries of the Viyal runtime.

## 1. Object Graphs and Memory Lifecycle
Viyal employs a custom Mark and Sweep GC (`GcAllocator`) built purely in safe Rust. 
*   **Object Graph Allocation**: Objects are successfully allocated and returned as handles (`GcHandle`) when initializing variables, creating arrays, or declaring classes. The stress test correctly allocates over 1,000,000 array references back-to-back without producing an Out-Of-Memory (OOM) panic within standard usage limits.
*   **Reclamation Gap**: As documented in the `MEMORY_MODEL.md`, while the arena logic is implemented, the VM runtime lacks the plumbing to actively execute `mark` and `sweep` phases. Thus, **Viyal processes monotonically leak memory during their lifetime**.
*   **Recommendation**: This is a known technical debt item. To proceed past basic scripting into long-running daemon execution, Phase 08 or M12 must instrument the VM's main loop to periodically trace the `CallFrame` stack.

## 2. Resource Exhaustion and Stress Tests
I executed a looping allocation stress test:
```viyal
class Main { 
    void run() { 
        mut i = 0; 
        while (i < 1000000) { 
            var arr = [1, 2, 3]; 
            i = i + 1; 
        } 
        print("done"); 
    } 
}
```
**Result**: The test completed successfully. Rust's backing allocator handles the monotonic growth gracefully for short scripts, but the theoretical risk of memory exhaustion remains for infinite loops.

## 3. Concurrency and Async Systems
*   **Findings**: `runtime/concurrency` and `runtime/async` crates are placeholder scaffolding.
*   **Safety Status**: Inherently thread-safe due to the absence of threading implementation. There are no data races or deadlocks possible in Viyal right now.

## 4. Unsafe Rust Usage
*   **Findings**: The VM and GC implementations strictly utilize safe Rust. Handled-based arenas completely side-step raw pointers, `unsafe {}` blocks, and manual lifetime tracking. 
*   **Safety Status**: The implementation is resilient against use-after-free, double-free, and invalid pointer dereferencing. If the GC were to prematurely free a handle (which is currently impossible since sweep isn't called), the `get_obj` method would safely return `None` rather than a segfault.

## 5. FFI and Standard Library Bindings
Native functions are represented by `NativeFunctionObj`, passing `&mut VM` and `&[Value]` slice.
*   **Safety**: Validated. The VM intercepts all errors safely (`Result<Value, String>`). Attempting to call undefined native libraries results in an immediate VM halt rather than undefined behavior.

## Conclusion
Viyal's memory model prioritizes crash-safety and sandbox-safety over absolute efficiency. While memory is monotonically leaked (a critical functional defect for servers), there are no memory safety vulnerabilities (segfaults, data races, buffer overflows). Theoretical risks are purely constrained to resource exhaustion (OOM).
