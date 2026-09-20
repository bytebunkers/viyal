//! Viyal native backend — dual-mode compiler backend.
//!
//! Exposes two public compilation modes:
//!
//! | Mode | CLI command            | Engine              | Optimization |
//! |------|------------------------|---------------------|--------------|
//! | JIT  | `viyal run`            | `cranelift-jit`     | None (fast)  |
//! | AOT  | `viyal build --release`| `cranelift-object`  | Speed        |
//!
//! Both modes share the exact same [`translator`] module for MIR → Cranelift IR
//! translation, ensuring identical semantics and a single code path to maintain.

pub mod aot;
pub mod jit;
pub mod translator;

pub use aot::compile_aot;
pub use jit::execute_jit;

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cranelift_codegen::settings::Configurable;

    use ast::{BinaryOp, Literal, Type};
    use mir::ir::{
        BasicBlock, Local, LocalDecl, MirFunction, MirProgram, Operand, Rvalue, Statement,
        Terminator,
    };
    use std::collections::HashMap;

    /// Build a minimal `MirProgram` containing a single `main` function
    /// that computes `10 + 20` and returns the result.
    ///
    /// MIR layout:
    /// ```
    /// fn main() -> int {
    ///     bb0:
    ///         _0 = 10 + 20   // Statement::Assign
    ///         return _0      // Terminator::Return
    /// }
    /// ```
    fn make_add_program() -> MirProgram {
        let mut program = MirProgram::default();

        let mut func = MirFunction::new("main".to_string(), Some(Type::Named("int".to_string(), vec![])));

        // Local 0: the return value slot.
        func.locals.push(LocalDecl {
            ty: Type::Named("int".to_string(), vec![]),
            name: Some("_0".to_string()),
            is_mut: true,
        });

        // Single basic block.
        let bb = BasicBlock {
            id: 0,
            phis: vec![],
            statements: vec![
                // _0 = 10 + 20
                Statement::Assign(
                    Local(0),
                    Rvalue::BinaryOp(
                        BinaryOp::Add,
                        Operand::Constant(Literal::Integer(10)),
                        Operand::Constant(Literal::Integer(20)),
                    ),
                ),
            ],
            terminator: Terminator::Return {
                value: Some(Operand::Copy(Local(0))),
            },
        };

        func.basic_blocks.push(bb);
        program.functions.insert("main".to_string(), func);
        program
    }

    #[test]
    fn test_jit_add() {
        let program = make_add_program();
        let result = execute_jit(&program).expect("JIT execution failed");
        assert_eq!(result, 30, "Expected 10 + 20 = 30 via JIT");
    }

    #[test]
    fn test_aot_emits_object_file() {
        let program = make_add_program();
        let tmp_dir = std::env::temp_dir();
        let out_path = tmp_dir.join("viyal_aot_test_output");

        // We only check that the object-file emission step succeeds.
        // Full linking requires a system linker, so we stop just before that.
        // This validates the Cranelift compilation pipeline.
        use cranelift_codegen::{settings};
        use cranelift_codegen::settings::Configurable;
        use cranelift_frontend::FunctionBuilderContext;
        use cranelift_module::Module;
        use cranelift_object::{ObjectBuilder, ObjectModule};
        use cranelift_native;
        use crate::translator::compile_mir_program;

        let mut flag_builder = settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        let isa = cranelift_native::builder()
            .unwrap()
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let obj_builder = ObjectBuilder::new(
            isa,
            "test".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let mut module = ObjectModule::new(obj_builder);
        let mut ctx = module.make_context();
        let mut fb_ctx = FunctionBuilderContext::new();

        compile_mir_program(&mut module, &mut fb_ctx, &mut ctx, &program)
            .expect("AOT compilation failed");

        let product = module.finish();
        let bytes = product.emit().expect("Object file emission failed");
        assert!(!bytes.is_empty(), "Object file should not be empty");
    }
}
