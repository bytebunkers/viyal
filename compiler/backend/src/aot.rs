//! AOT (Ahead-of-Time) compilation engine — used by `viyal build --release`.
//!
//! Compiles a `MirProgram` to a native object file (`.o` / `.obj`) on disk
//! using `cranelift-object`, then invokes the system linker to produce a
//! standalone executable. No GCC or LLVM required.

use cranelift_codegen::settings::{self, Configurable};
use cranelift_frontend::FunctionBuilderContext;
use cranelift_module::Module;
use cranelift_native;
use cranelift_object::{ObjectBuilder, ObjectModule};

use mir::ir::MirProgram;

use std::path::Path;
use std::process::Command;

use crate::translator::compile_mir_program;

/// Compile a `MirProgram` to a native executable file.
///
/// This is the `viyal build --release` path:
///   Parse → Typecheck → MIR → Optimizer → **AOT** → object file → linker → exe
///
/// # Parameters
/// - `program`     — The optimized MIR program.
/// - `output_path` — Where to write the final executable (e.g., `./output`).
///
/// # Errors
/// Returns a human-readable error string if compilation or linking fails.
pub fn compile_aot(program: &MirProgram, output_path: &Path) -> Result<(), String> {
    // 1. Configure Cranelift for the current host CPU with optimizations.
    let mut flag_builder = settings::builder();
    // "opt_level speed" → enable Cranelift's own peephole optimizations.
    flag_builder.set("opt_level", "speed").unwrap();
    let isa_builder = cranelift_native::builder().map_err(|e| {
        format!("Failed to detect host ISA for AOT: {}", e)
    })?;
    let isa = isa_builder
        .finish(settings::Flags::new(flag_builder))
        .map_err(|e| format!("Failed to build ISA: {}", e))?;

    // 2. Create the Object module, naming the binary after the output path stem.
    let name = output_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let obj_builder = ObjectBuilder::new(
        isa,
        name.clone(),
        cranelift_module::default_libcall_names(),
    )
    .map_err(|e| format!("Failed to create ObjectBuilder: {}", e))?;
    let mut module = ObjectModule::new(obj_builder);

    // 3. Shared builder contexts.
    let mut ctx = module.make_context();
    let mut fb_ctx = FunctionBuilderContext::new();

    // 4. Compile every MIR function (two-pass: declare then compile).
    compile_mir_program(&mut module, &mut fb_ctx, &mut ctx, program)
        .map_err(|e| format!("[AOT] {}", e))?;

    // 5. Emit the object file bytes and write them to disk.
    let obj_product = module.finish();
    let obj_bytes = obj_product
        .emit()
        .map_err(|e| format!("[AOT] Object emission error: {}", e))?;

    // Write the intermediate `.o` file alongside the output executable.
    let obj_path = output_path.with_extension("o");
    std::fs::write(&obj_path, &obj_bytes)
        .map_err(|e| format!("[AOT] Failed to write object file: {}", e))?;

    // 6. Invoke the system linker to produce the final executable.
    //    We try `cc` first (works on Linux/macOS), then `link.exe` on Windows.
    let exe_path = if cfg!(target_os = "windows") {
        output_path.with_extension("exe")
    } else {
        output_path.to_path_buf()
    };

    let linker_status = link_object_file(&obj_path, &exe_path)?;

    // 7. Clean up the intermediate `.o` file.
    let _ = std::fs::remove_file(&obj_path);

    println!(
        "[AOT] Build successful: {}",
        exe_path.display()
    );

    Ok(())
}

/// Invoke the system C linker to turn an object file into an executable.
///
/// On Windows we try `link.exe` (MSVC). On Unix we try `cc` (usually
/// resolves to clang or gcc). Both are used only for linking (not compiling),
/// so there is no dependency on a full C toolchain—just a linker.
fn link_object_file(obj_path: &Path, out_path: &Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("link.exe")
            .arg(obj_path)
            .arg(format!("/OUT:{}", out_path.display()))
            .arg("/ENTRY:main")
            .arg("/SUBSYSTEM:CONSOLE")
            .status()
            .map_err(|_| {
                "[AOT] Could not find `link.exe`. \
                 Install the MSVC Build Tools or add it to PATH."
                    .to_string()
            })?;
        if !status.success() {
            return Err(format!(
                "[AOT] Linker `link.exe` failed with exit code: {}",
                status.code().unwrap_or(-1)
            ));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let status = Command::new("cc")
            .arg(obj_path)
            .arg("-o")
            .arg(out_path)
            .status()
            .map_err(|_| {
                "[AOT] Could not find a C linker (`cc`). \
                 Please install GCC or Clang."
                    .to_string()
            })?;
        if !status.success() {
            return Err(format!(
                "[AOT] Linker `cc` failed with exit code: {}",
                status.code().unwrap_or(-1)
            ));
        }
    }

    Ok(())
}
