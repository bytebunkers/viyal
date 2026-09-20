//! MIR → Cranelift IR translator.
//!
//! This is the core of the dual-mode backend. It is generic over `M: Module`,
//! meaning the exact same translation logic is reused for both:
//!   - `JITModule`  (debug / `viyal run`)
//!   - `ObjectModule` (release / `viyal build --release`)
//!
//! ## Two-pass compilation
//!
//! To support cross-function calls, compilation is split into two passes:
//!
//! **Pass 1 — `declare_mir_function`**: Registers every function's signature with
//! the module. No IR is generated. All functions are visible to each other after
//! this pass completes.
//!
//! **Pass 2 — `compile_mir_function`**: Generates Cranelift IR for the function
//! body, using the pre-declared `FuncId` map to resolve call targets.

use cranelift_codegen::ir::{
    condcodes::IntCC, types, AbiParam, Block, Function, InstBuilder, UserFuncName, Value,
};
use cranelift_codegen::Context;
use cranelift_codegen::settings::Configurable;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};

use ast::{BinaryOp, Literal, Type};
use mir::ir::{MirFunction, MirProgram, Operand, Rvalue, Statement, Terminator};

use std::collections::HashMap;

// ─── Type Mapping ────────────────────────────────────────────────────────────

/// Map a Viyal `ast::Type` to a Cranelift scalar type.
/// For the MVP we support `int` (i64), `double`/`float` (f64), and `bool` (i8).
/// Everything else (objects, strings) is represented as a pointer-sized integer.
pub fn viyal_type_to_cl(ty: &Option<Type>) -> types::Type {
    match ty {
        Some(Type::Named(n, _)) => match n.as_str() {
            "int" | "Int" => types::I64,
            "double" | "float" | "Float" | "Double" => types::F64,
            "bool" | "Bool" => types::I8,
            _ => types::I64, // All objects treated as opaque pointers for now
        },
        _ => types::I64,
    }
}

// ─── FunctionTranslator ───────────────────────────────────────────────────────

/// Translates a single `MirFunction` into Cranelift instructions.
pub struct FunctionTranslator<'a> {
    pub builder: FunctionBuilder<'a>,
    /// Maps `mir::Local` index → Cranelift `Variable`.
    vars: HashMap<usize, Variable>,
    /// Maps `mir::BasicBlock` id → Cranelift `Block`.
    blocks: HashMap<usize, Block>,
    /// Pre-declared FuncIds for all functions (populated by Pass 1).
    /// Used to emit cross-function call instructions.
    func_ids: HashMap<String, FuncId>,
}

impl<'a> FunctionTranslator<'a> {
    pub fn new(builder: FunctionBuilder<'a>, func_ids: HashMap<String, FuncId>) -> Self {
        Self {
            builder,
            vars: HashMap::new(),
            blocks: HashMap::new(),
            func_ids,
        }
    }

    /// Full translation entry point for one `MirFunction`.
    pub fn translate(mut self, mir_fn: &MirFunction) {
        // 1. Declare a Cranelift Variable for every MIR local.
        for (idx, local_decl) in mir_fn.locals.iter().enumerate() {
            let var = Variable::from_u32(idx as u32);
            let cl_type = viyal_type_to_cl(&Some(local_decl.ty.clone()));
            self.builder.declare_var(var, cl_type);
            self.vars.insert(idx, var);
        }

        // 2. Pre-create a Cranelift Block for every MIR basic block.
        for bb in &mir_fn.basic_blocks {
            let block = self.builder.create_block();
            self.blocks.insert(bb.id, block);
        }

        // 3. Append function parameters to the entry block.
        let entry_block = *self.blocks.get(&0).expect("MIR function has no entry block");
        self.builder.append_block_params_for_function_params(entry_block);
        self.builder.switch_to_block(entry_block);
        self.builder.seal_block(entry_block);

        // Initialize param locals from entry block params.
        for (param_idx, &local) in mir_fn.params.iter().enumerate() {
            let val = self.builder.block_params(entry_block)[param_idx];
            let var = Variable::from_u32(local.0 as u32);
            self.builder.def_var(var, val);
        }

        // 4. Translate each basic block.
        let bbs = mir_fn.basic_blocks.clone();
        for bb in &bbs {
            let cl_block = *self.blocks.get(&bb.id).expect("Block not pre-created");
            if bb.id != 0 {
                self.builder.switch_to_block(cl_block);
                self.builder.seal_block(cl_block);
            }
            for stmt in &bb.statements {
                self.translate_stmt(stmt);
            }
            self.translate_terminator(&bb.terminator, mir_fn);
        }

        self.builder.finalize();
    }

    // ─── Statement ───────────────────────────────────────────────────────────

    fn translate_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign(local, rvalue) => {
                let val = self.translate_rvalue(rvalue);
                let var = Variable::from_u32(local.0 as u32);
                self.builder.def_var(var, val);
            }
        }
    }

    // ─── Rvalue ──────────────────────────────────────────────────────────────

    fn translate_rvalue(&mut self, rvalue: &Rvalue) -> Value {
        match rvalue {
            Rvalue::Use(operand) => self.translate_operand(operand),

            Rvalue::BinaryOp(op, lhs, rhs) => {
                let l = self.translate_operand(lhs);
                let r = self.translate_operand(rhs);
                match op {
                    BinaryOp::Add => self.builder.ins().iadd(l, r),
                    BinaryOp::Sub => self.builder.ins().isub(l, r),
                    BinaryOp::Mul => self.builder.ins().imul(l, r),
                    BinaryOp::Div => self.builder.ins().sdiv(l, r),
                    BinaryOp::Eq      => self.builder.ins().icmp(IntCC::Equal, l, r),
                    BinaryOp::NotEq   => self.builder.ins().icmp(IntCC::NotEqual, l, r),
                    BinaryOp::Less    => self.builder.ins().icmp(IntCC::SignedLessThan, l, r),
                    BinaryOp::Greater => self.builder.ins().icmp(IntCC::SignedGreaterThan, l, r),
                    BinaryOp::LessEq  => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r),
                    BinaryOp::GreaterEq => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r),
                    BinaryOp::Assign => r,
                }
            }

            // ── Cross-function call ───────────────────────────────────────────
            //
            // `Rvalue::Call { func: Operand::Constant(Literal::String(name)), args }`
            // is emitted by the MIR builder for direct named function calls.
            // We look up the pre-declared FuncId, build an ExtFuncData entry, then
            // emit a `call` instruction.
            Rvalue::Call { func, args } => {
                // Resolve callee name from the operand.
                let callee_name = match func {
                    Operand::Constant(Literal::String(s)) => s.clone(),
                    Operand::Copy(local) => {
                        // Indirect call — not yet supported; emit safe zero placeholder.
                        let _ = self.translate_operand(&Operand::Copy(*local));
                        return self.builder.ins().iconst(types::I64, 0);
                    }
                    _ => return self.builder.ins().iconst(types::I64, 0),
                };

                if let Some(&func_id) = self.func_ids.get(&callee_name) {
                    // For MVP we just use the caller's signature as a template for the callee.
                    // A full implementation would look up the exact callee signature.
                    let sig = self.builder.func.signature.clone();
                    let sig_ref = self.builder.import_signature(sig);

                    let func_ref = self.builder.func.dfg.ext_funcs.push(
                        cranelift_codegen::ir::ExtFuncData {
                            name: cranelift_codegen::ir::ExternalName::user(
                                cranelift_codegen::ir::UserExternalNameRef::from_u32(func_id.as_u32()),
                            ),
                            signature: sig_ref,
                            colocated: true,
                        }
                    );
                    let arg_vals: Vec<Value> = args.iter()
                        .map(|a| self.translate_operand(a))
                        .collect();
                    let call = self.builder.ins().call(func_ref, &arg_vals);
                    let results = self.builder.inst_results(call);
                    if results.is_empty() {
                        self.builder.ins().iconst(types::I64, 0)
                    } else {
                        results[0]
                    }
                } else {
                    // Unknown function — safe zero placeholder.
                    // Should not occur after a successful typecheck pass.
                    self.builder.ins().iconst(types::I64, 0)
                }
            }

            // Higher-level rvalues (Array, Map, New, MethodCall, etc.) are emitted
            // as zero placeholders for the MVP — will be runtime-call-lowered in v0.2.
            _ => self.builder.ins().iconst(types::I64, 0),
        }
    }

    fn translate_operand(&mut self, operand: &Operand) -> Value {
        match operand {
            Operand::Constant(lit) => match lit {
                Literal::Integer(i) => self.builder.ins().iconst(types::I64, *i),
                Literal::Float(f)   => self.builder.ins().f64const(*f),
                Literal::Boolean(b) => self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 }),
                Literal::String(_) | Literal::Null => self.builder.ins().iconst(types::I64, 0),
            },
            Operand::Copy(local) => {
                let var = Variable::from_u32(local.0 as u32);
                self.builder.use_var(var)
            }
        }
    }

    // ─── Terminator ──────────────────────────────────────────────────────────

    fn translate_terminator(&mut self, term: &Terminator, _mir_fn: &MirFunction) {
        match term {
            Terminator::Return { value } => {
                let ret_vals: &[Value] = if let Some(operand) = value {
                    let v = self.translate_operand(operand);
                    &[v][..]
                } else {
                    &[]
                };
                self.builder.ins().return_(ret_vals);
            }

            Terminator::Goto { target } => {
                let target_block = *self.blocks.get(target).expect("Goto target not found");
                self.builder.ins().jump(target_block, &[]);
            }

            Terminator::If { cond, then_target, else_target } => {
                let cond_val = self.translate_operand(cond);
                let then_block = *self.blocks.get(then_target).expect("If then block not found");
                let else_block = *self.blocks.get(else_target).expect("If else block not found");
                self.builder.ins().brif(cond_val, then_block, &[], else_block, &[]);
            }

            Terminator::IfOk { val, then_target, else_target } => {
                let v = self.translate_operand(val);
                let then_block = *self.blocks.get(then_target).expect("IfOk then block not found");
                let else_block = *self.blocks.get(else_target).expect("IfOk else block not found");
                self.builder.ins().brif(v, then_block, &[], else_block, &[]);
            }

            Terminator::Unreachable => {
                let trap = cranelift_codegen::ir::TrapCode::unwrap_user(1);
                self.builder.ins().trap(trap);
            }
        }
    }
}

// ─── Two-Pass Compile API ─────────────────────────────────────────────────────

/// **Pass 1** — Register a function's signature in the module without generating IR.
///
/// Call for every function in the program *before* any `compile_mir_function` call.
pub fn declare_mir_function<M: Module>(
    module: &mut M,
    mir_fn: &MirFunction,
) -> Result<FuncId, String> {
    let mut sig = module.make_signature();
    for &param_local in &mir_fn.params {
        let cl_ty = viyal_type_to_cl(&Some(mir_fn.locals[param_local.0].ty.clone()));
        sig.params.push(AbiParam::new(cl_ty));
    }
    if let Some(ret_ty) = &mir_fn.return_type {
        sig.returns.push(AbiParam::new(viyal_type_to_cl(&Some(ret_ty.clone()))));
    }
    module
        .declare_function(&mir_fn.name, Linkage::Export, &sig)
        .map_err(|e| format!("Failed to declare '{}': {}", mir_fn.name, e))
}

/// **Pass 2** — Compile a function body.
///
/// Requires the `func_ids` map built by Pass 1 so cross-function call targets
/// can be resolved inside `FunctionTranslator`.
pub fn compile_mir_function<M: Module>(
    module: &mut M,
    fb_ctx: &mut FunctionBuilderContext,
    ctx: &mut Context,
    mir_fn: &MirFunction,
    func_ids: &HashMap<String, FuncId>,
) -> Result<FuncId, String> {
    let func_id = *func_ids
        .get(&mir_fn.name)
        .ok_or_else(|| format!("'{}' was not declared in Pass 1", mir_fn.name))?;

    let mut sig = module.make_signature();
    for &param_local in &mir_fn.params {
        let cl_ty = viyal_type_to_cl(&Some(mir_fn.locals[param_local.0].ty.clone()));
        sig.params.push(AbiParam::new(cl_ty));
    }
    if let Some(ret_ty) = &mir_fn.return_type {
        sig.returns.push(AbiParam::new(viyal_type_to_cl(&Some(ret_ty.clone()))));
    }

    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);

    {
        let builder = FunctionBuilder::new(&mut ctx.func, fb_ctx);
        let translator = FunctionTranslator::new(builder, func_ids.clone());
        translator.translate(mir_fn);
    }

    module
        .define_function(func_id, ctx)
        .map_err(|e| format!("Failed to define '{}': {}", mir_fn.name, e))?;

    ctx.clear();
    Ok(func_id)
}

/// Convenience: run both passes for an entire `MirProgram`.
///
/// Called by `jit.rs` and `aot.rs`.
pub fn compile_mir_program<M: Module>(
    module: &mut M,
    fb_ctx: &mut FunctionBuilderContext,
    ctx: &mut Context,
    program: &MirProgram,
) -> Result<HashMap<String, FuncId>, String> {
    // Pass 1 — declare all signatures.
    let mut func_ids: HashMap<String, FuncId> = HashMap::new();
    for (name, mir_fn) in &program.functions {
        let fid = declare_mir_function(module, mir_fn)?;
        func_ids.insert(name.clone(), fid);
    }

    // Pass 2 — compile all bodies (all signatures now known).
    for mir_fn in program.functions.values() {
        compile_mir_function(module, fb_ctx, ctx, mir_fn, &func_ids)?;
    }

    Ok(func_ids)
}
