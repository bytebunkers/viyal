# Concurrency Design

Concurrency is a major differentiation point from Dart. While Dart relies heavily on single-threaded event loops (Isolates) and unstructured `Future`s, Viyal aims for **Structured Concurrency**.

## 1. The Problem with Unstructured Concurrency
In traditional async models (like Dart or JavaScript), a `Future` or `Promise` can be fired off and forgotten. If it fails, the error might be silently swallowed or crash the event loop globally. Cancelling a deep tree of async operations is notoriously difficult.

## 2. Structured Concurrency
Viyal adopts structured concurrency concepts (inspired by Swift and Kotlin). Async tasks are strictly bound to a hierarchical scope.

```Viyal
async function fetchData() {
    final result = await network.get();
    return result;
}

// Conceptual syntax for structured concurrency
scope {
    task {
        fetchUsers();
    }

    task {
        fetchProducts();
    }
} // Scope blocks until all child tasks complete or fail.
```

## 3. Guarantees
1. **Cancellation**: If a parent scope is cancelled, all child tasks are automatically cancelled.
2. **Error Propagation**: If a child task fails, the error bubbles up to the parent scope, cancelling siblings automatically to prevent wasted work.
3. **No Leaks**: A task cannot outlive its parent scope.

## 4. Execution Model
- **Tasks**: Lightweight threads (green threads/coroutines) multiplexed over OS threads.
- **Async/Await**: Standard syntax for awaiting the result of an asynchronous computation.
- **Channels**: Go-style channels for safe communication between concurrent tasks without shared mutable state.

## 5. Research Areas
- How to balance an Event Loop (for UI work) with Thread Pools (for CPU-bound work) transparently.
- Integrating Isolates (memory-isolated actors) for massive parallelism without GC pauses affecting the main thread.
