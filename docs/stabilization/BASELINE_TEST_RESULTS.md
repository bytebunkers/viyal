# Baseline Test Results

## Workspace Checks

| Command | Status | Notes |
|---|---|---|
| `cargo test` | Passed | 17 tests executed and passed across workspace crates. |
| `cargo build` | Passed | Builds successfully with some unused variable/import warnings. |
| `cargo clippy -- -D warnings` | Failed | Fails in `runtime/gc/src/allocator.rs` due to `clippy::new-without-default`. |
| `cargo fmt --check` | Failed | Unformatted files in `vm/src/vm.rs` and `vm/tests/vm_tests.rs`. |

## CLI Commands

| Command | Target | Status | Notes |
|---|---|---|---|
| `viyal run` | `hello.vyl` | Passed | Executes VM successfully. |
| `viyal test` | `math_test.vyl` | Failed | Discovered 4 tests. 3 passed. 1 explicitly failed (`test_failure`). |
| `viyal test` | `test_for.vyl` | Skipped | Reports "No tests found". Test runner only looks inside classes. |
| `viyal build` | `hello.vyl` | Failed | Attempts to invoke `gcc` which is not found. C codegen is heavily stubbed. |
| `viyal build --release` | `hello.vyl` | Failed | AOT build error: `[AOT] Failed to define 'Main::run': Compilation error: Verifier errors`. |
