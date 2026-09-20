//! JIT execution engine — used by `viyal run` (debug mode).
//!
//! Compiles every function in the `MirProgram` into in-memory machine code
//! using `cranelift-jit`, then immediately calls the `main` entry point.
//! Because no object files or linker calls are needed, startup is near-instant.

use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
use cranelift_native;

use mir::ir::MirProgram;

use crate::translator::compile_mir_program;

/// Run a `MirProgram` directly in memory using Cranelift JIT.
///
/// This is the `viyal run` fast path:
///   Parse → Typecheck → MIR → **JIT** → execute
///
/// # Errors
/// Returns a human-readable error string if compilation or execution fails.
pub fn execute_jit(program: &MirProgram) -> Result<i64, String> {
    // 1. Configure Cranelift for the current host CPU.
    //    `cranelift_native::builder()` detects the host ISA automatically
    //    (x86_64, AArch64, etc.) and enables all available CPU features.
    let mut flag_builder = settings::builder();
    // "opt_level none" → skip optimizations for maximum JIT compilation speed.
    flag_builder.set("opt_level", "none").unwrap();
    let isa_builder = cranelift_native::builder().map_err(|e| {
        format!("Failed to detect host ISA for JIT: {}", e)
    })?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| format!("Failed to build ISA: {}", e))?;

    // 2. Create the JIT module.
    let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
    let mut module = JITModule::new(builder);

    // 3. Shared builder contexts (re-used across all functions for efficiency).
    let mut ctx = module.make_context();
    let mut fb_ctx = FunctionBuilderContext::new();

    // 4. Compile every MIR function (two-pass: declare then compile).
    compile_mir_program(&mut module, &mut fb_ctx, &mut ctx, program)
        .map_err(|e| format!("[JIT] {}", e))?;

    // 5. Finalize — this performs relocation and patches all call-site addresses.
    module.finalize_definitions().map_err(|e| format!("[JIT] Finalize error: {}", e))?;

    // 6. Retrieve a raw function pointer for the `main` entry point.
    //    We look for the Viyal convention: a top-level function named `main`.
    //    Verify it exists in the MIR program first.
    let _ = program
        .functions
        .get("main")
        .ok_or_else(|| "[JIT] No `main` function found in MIR program".to_string())?;

    let func_id = module
        .get_name("main")
        .ok_or_else(|| "[JIT] `main` was not exported from the JIT module".to_string())?;

    let func_id = match func_id {
        cranelift_module::FuncOrDataId::Func(id) => id,
        _ => return Err("[JIT] `main` is not a function".to_string()),
    };

    let raw_fn_ptr = module.get_finalized_function(func_id);

    // SAFETY: We trust Cranelift to produce valid code for the signature we
    // declared. The `main` function takes no arguments and returns `i64`.
    let main_fn: unsafe extern "C" fn() -> i64 = unsafe { std::mem::transmute(raw_fn_ptr) };
    let result = unsafe { main_fn() };

    Ok(result)
}
