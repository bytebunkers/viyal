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

use cranelift_codegen::Context;
use cranelift_codegen::ir::{
    AbiParam, Block, Function, InstBuilder, UserFuncName, Value, condcodes::IntCC, types,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{FuncId, Linkage, Module};

use ast::{BinaryOp, Literal, Type};
use mir::ir::{MirFunction, MirProgram, Operand, Rvalue, Statement, Terminator};

use std::collections::HashMap;

// ─── Type Mapping ────────────────────────────────────────────────────────────

/// Map a Viyal `ast::Type` to a Cranelift scalar type.
/// For the MVP we support `int` (i64), `double`/`float` (f64), and `bool` (i8).
/// Everything else (objects, strings) is represented as a pointer-sized integer.
pub fn viyal_type_to_cl(ty: &Option<Type>) -> Option<types::Type> {
    match ty {
        Some(Type::Named(n, _)) => match n.as_str() {
            "void" | "Void" => None,
            "int" | "Int" => Some(types::I64),
            "double" | "float" | "Float" | "Double" => Some(types::F64),
            "bool" | "Bool" => Some(types::I8),
            _ => Some(types::I64), // All objects treated as opaque pointers for now
        },
        _ => Some(types::I64),
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
    /// Pre-declared FuncRefs for all functions (populated by Pass 1 and `module.declare_func_in_func`).
    /// Used to emit cross-function call instructions.
    func_refs: HashMap<String, cranelift_codegen::ir::FuncRef>,
    /// Access to the full program for struct size/field offset lookups.
    pub mir_program: &'a MirProgram,
}

impl<'a> FunctionTranslator<'a> {
    pub fn new(builder: FunctionBuilder<'a>, func_refs: HashMap<String, cranelift_codegen::ir::FuncRef>, mir_program: &'a MirProgram) -> Self {
        Self {
            builder,
            vars: HashMap::new(),
            blocks: HashMap::new(),
            func_refs,
            mir_program,
        }
    }

    /// Full translation entry point for one `MirFunction`.
    pub fn translate(mut self, mir_fn: &MirFunction) {
        // 1. Declare a Cranelift Variable for every MIR local.
        for (idx, local_decl) in mir_fn.locals.iter().enumerate() {
            let var = Variable::from_u32(idx as u32);
            let cl_type = viyal_type_to_cl(&Some(local_decl.ty.clone())).unwrap_or(types::I64);
            self.builder.declare_var(var, cl_type);
            self.vars.insert(idx, var);
        }

        // 2. Pre-create a Cranelift Block for every MIR basic block.
        for bb in &mir_fn.basic_blocks {
            let block = self.builder.create_block();
            self.blocks.insert(bb.id, block);
        }

        // 3. Append function parameters to the entry block.
        let entry_block = *self
            .blocks
            .get(&0)
            .expect("MIR function has no entry block");
        self.builder
            .append_block_params_for_function_params(entry_block);
        self.builder.switch_to_block(entry_block);

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
            }
            for stmt in &bb.statements {
                self.translate_stmt(stmt, mir_fn);
            }
            self.translate_terminator(&bb.terminator, mir_fn);
        }

        self.builder.seal_all_blocks();
        self.builder.finalize();
    }

    // ─── Statement ───────────────────────────────────────────────────────────

    fn translate_stmt(&mut self, stmt: &Statement, mir_fn: &MirFunction) {
        match stmt {
            Statement::Assign(local, rvalue) => {
                let val = self.translate_rvalue(rvalue, mir_fn);
                let var = Variable::from_u32(local.0 as u32);
                self.builder.def_var(var, val);
            }
        }
    }

    // ─── Rvalue ──────────────────────────────────────────────────────────────

    fn translate_rvalue(&mut self, rvalue: &Rvalue, mir_fn: &MirFunction) -> Value {
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
                    BinaryOp::Eq => {
                        let res = self.builder.ins().icmp(IntCC::Equal, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
                    BinaryOp::NotEq => {
                        let res = self.builder.ins().icmp(IntCC::NotEqual, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
                    BinaryOp::Less => {
                        let res = self.builder.ins().icmp(IntCC::SignedLessThan, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
                    BinaryOp::Greater => {
                        let res = self.builder.ins().icmp(IntCC::SignedGreaterThan, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
                    BinaryOp::LessEq => {
                        let res = self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
                    BinaryOp::GreaterEq => {
                        let res = self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l, r);
                        self.builder.ins().uextend(types::I64, res)
                    }
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

                if let Some(&func_ref) = self.func_refs.get(&callee_name) {
                    let arg_vals: Vec<Value> =
                        args.iter().map(|a| self.translate_operand(a)).collect();
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

            Rvalue::MethodCall(obj, method_name, args) => {
                let class_name = if let Operand::Copy(local) = obj {
                    if let Type::Named(name, _) = &mir_fn.locals[local.0].ty {
                        name.clone()
                    } else {
                        "Main".to_string()
                    }
                } else {
                    "Main".to_string()
                };

                let callee_name = format!("{}::{}", class_name, method_name);
                if let Some(&func_ref) = self.func_refs.get(&callee_name) {
                    let mut arg_vals: Vec<Value> = vec![self.translate_operand(obj)];
                    arg_vals.extend(args.iter().map(|a| self.translate_operand(a)));
                    
                    let call = self.builder.ins().call(func_ref, &arg_vals);
                    let results = self.builder.inst_results(call);
                    if results.is_empty() {
                        self.builder.ins().iconst(types::I64, 0)
                    } else {
                        results[0]
                    }
                } else {
                    self.builder.ins().iconst(types::I64, 0)
                }
            }

            Rvalue::New(class_name, args) => {
                if let Some(class) = self.mir_program.classes.get(class_name) {
                    let size = class.fields.len() as i64 * 8;
                    let size_val = self.builder.ins().iconst(types::I64, size);
                    
                    if let Some(&malloc_ref) = self.func_refs.get("malloc") {
                        let call = self.builder.ins().call(malloc_ref, &[size_val]);
                        let ptr = self.builder.inst_results(call)[0];
                        
                        // Initialize fields with args (if mapped 1:1)
                        for (i, arg) in args.iter().enumerate() {
                            if i < class.fields.len() {
                                let arg_val = self.translate_operand(arg);
                                let offset = (i * 8) as i32;
                                self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), arg_val, ptr, offset);
                            }
                        }
                        ptr
                    } else {
                        self.builder.ins().iconst(types::I64, 0)
                    }
                } else {
                    self.builder.ins().iconst(types::I64, 0)
                }
            }
            Rvalue::PropertyAccess(obj, prop) => {
                let ptr = self.translate_operand(obj);
                
                let class_name = if let Operand::Copy(local) = obj {
                    if let Type::Named(name, _) = &mir_fn.locals[local.0].ty {
                        Some(name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                if let Some(class_name) = class_name {
                    if let Some(class) = self.mir_program.classes.get(&class_name) {
                        if let Some(idx) = class.fields.iter().position(|f| f == prop) {
                            let offset = (idx * 8) as i32;
                            return self.builder.ins().load(types::I64, cranelift_codegen::ir::MemFlags::trusted(), ptr, offset);
                        }
                    }
                }
                
                self.builder.ins().iconst(types::I64, 0)
            }
            Rvalue::PropertyAssign(obj, prop, val_op) => {
                let ptr = self.translate_operand(obj);
                let val = self.translate_operand(val_op);
                
                let class_name = if let Operand::Copy(local) = obj {
                    if let Type::Named(name, _) = &mir_fn.locals[local.0].ty {
                        Some(name.clone())
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                if let Some(class_name) = class_name {
                    if let Some(class) = self.mir_program.classes.get(&class_name) {
                        if let Some(idx) = class.fields.iter().position(|f| f == prop) {
                            let offset = (idx * 8) as i32;
                            self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), val, ptr, offset);
                            return val;
                        }
                    }
                }
                
                self.builder.ins().iconst(types::I64, 0)
            }
            Rvalue::Array(elements) => {
                let len = elements.len() as i64;
                let size_bytes = 8 + len * 8; // 8 bytes for length, plus elements
                if let Some(&malloc_ref) = self.func_refs.get("malloc") {
                    let size_val = self.builder.ins().iconst(types::I64, size_bytes);
                    let call = self.builder.ins().call(malloc_ref, &[size_val]);
                    let ptr = self.builder.inst_results(call)[0];
                    
                    // store length at offset 0
                    let len_val = self.builder.ins().iconst(types::I64, len);
                    self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), len_val, ptr, 0);
                    
                    // store elements
                    for (i, el) in elements.iter().enumerate() {
                        let el_val = self.translate_operand(el);
                        let offset = 8 + (i as i32) * 8;
                        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), el_val, ptr, offset);
                    }
                    ptr
                } else {
                    self.builder.ins().iconst(types::I64, 0)
                }
            }
            Rvalue::Map(pairs) => {
                let len = pairs.len() as i64;
                let size_bytes = 8 + len * 16; // length + (key, value) pairs
                if let Some(&malloc_ref) = self.func_refs.get("malloc") {
                    let size_val = self.builder.ins().iconst(types::I64, size_bytes);
                    let call = self.builder.ins().call(malloc_ref, &[size_val]);
                    let ptr = self.builder.inst_results(call)[0];
                    
                    let len_val = self.builder.ins().iconst(types::I64, len);
                    self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), len_val, ptr, 0);
                    
                    for (i, (k, v)) in pairs.iter().enumerate() {
                        let k_val = self.translate_operand(k);
                        let v_val = self.translate_operand(v);
                        let offset_k = 8 + (i as i32) * 16;
                        let offset_v = 8 + (i as i32) * 16 + 8;
                        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), k_val, ptr, offset_k);
                        self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), v_val, ptr, offset_v);
                    }
                    ptr
                } else {
                    self.builder.ins().iconst(types::I64, 0)
                }
            }
            Rvalue::Index(arr, idx) => {
                let ptr = self.translate_operand(arr);
                let idx_val = self.translate_operand(idx);
                // Compute address: ptr + 8 + idx_val * 8
                // Note: For MVP, we assume it's an Array. A Map would require a linear scan loop in Cranelift.
                let eight = self.builder.ins().iconst(types::I64, 8);
                let offset = self.builder.ins().imul(idx_val, eight);
                let ptr_plus_offset = self.builder.ins().iadd(ptr, offset);
                self.builder.ins().load(types::I64, cranelift_codegen::ir::MemFlags::trusted(), ptr_plus_offset, 8)
            }
            Rvalue::IndexAssign(arr, idx, val_op) => {
                let ptr = self.translate_operand(arr);
                let idx_val = self.translate_operand(idx);
                let val = self.translate_operand(val_op);
                
                let eight = self.builder.ins().iconst(types::I64, 8);
                let offset = self.builder.ins().imul(idx_val, eight);
                let ptr_plus_offset = self.builder.ins().iadd(ptr, offset);
                
                self.builder.ins().store(cranelift_codegen::ir::MemFlags::trusted(), val, ptr_plus_offset, 8);
                val
            }
            Rvalue::Length(arr) => {
                let ptr = self.translate_operand(arr);
                self.builder.ins().load(types::I64, cranelift_codegen::ir::MemFlags::trusted(), ptr, 0)
            }
            // Higher-level rvalues are emitted
            // as zero placeholders for the MVP.
            _ => self.builder.ins().iconst(types::I64, 0),
        }
    }

    fn translate_operand(&mut self, operand: &Operand) -> Value {
        match operand {
            Operand::Constant(lit) => match lit {
                Literal::Integer(i) => self.builder.ins().iconst(types::I64, *i),
                Literal::Float(f) => self.builder.ins().f64const(*f),
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

            Terminator::If {
                cond,
                then_target,
                else_target,
            } => {
                let cond_val = self.translate_operand(cond);
                let then_block = *self
                    .blocks
                    .get(then_target)
                    .expect("If then block not found");
                let else_block = *self
                    .blocks
                    .get(else_target)
                    .expect("If else block not found");
                self.builder
                    .ins()
                    .brif(cond_val, then_block, &[], else_block, &[]);
            }

            Terminator::IfOk {
                val,
                then_target,
                else_target,
            } => {
                let v = self.translate_operand(val);
                let then_block = *self
                    .blocks
                    .get(then_target)
                    .expect("IfOk then block not found");
                let else_block = *self
                    .blocks
                    .get(else_target)
                    .expect("IfOk else block not found");
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
        let cl_ty =
            viyal_type_to_cl(&Some(mir_fn.locals[param_local.0].ty.clone())).unwrap_or(types::I64);
        sig.params.push(AbiParam::new(cl_ty));
    }
    if let Some(ret_ty) = &mir_fn.return_type {
        if let Some(cl_ty) = viyal_type_to_cl(&Some(ret_ty.clone())) {
            sig.returns.push(AbiParam::new(cl_ty));
        }
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
    program: &MirProgram,
) -> Result<FuncId, String> {
    let func_id = *func_ids
        .get(&mir_fn.name)
        .ok_or_else(|| format!("'{}' was not declared in Pass 1", mir_fn.name))?;

    let mut sig = module.make_signature();
    for &param_local in &mir_fn.params {
        let cl_ty =
            viyal_type_to_cl(&Some(mir_fn.locals[param_local.0].ty.clone())).unwrap_or(types::I64);
        sig.params.push(AbiParam::new(cl_ty));
    }
    if let Some(ret_ty) = &mir_fn.return_type {
        if let Some(cl_ty) = viyal_type_to_cl(&Some(ret_ty.clone())) {
            sig.returns.push(AbiParam::new(cl_ty));
        }
    }

    ctx.func = Function::with_name_signature(UserFuncName::user(0, func_id.as_u32()), sig);

    let mut func_refs = HashMap::new();
    for (name, &id) in func_ids {
        let func_ref = module.declare_func_in_func(id, &mut ctx.func);
        func_refs.insert(name.clone(), func_ref);
    }

    {
        let builder = FunctionBuilder::new(&mut ctx.func, fb_ctx);
        let translator = FunctionTranslator::new(builder, func_refs, program);
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
    
    // Builtin: print
    let mut print_sig = module.make_signature();
    print_sig.params.push(cranelift_codegen::ir::AbiParam::new(types::I64));
    print_sig.returns.push(cranelift_codegen::ir::AbiParam::new(types::I64));
    if let Ok(print_id) = module.declare_function("print", cranelift_module::Linkage::Import, &print_sig) {
        func_ids.insert("print".to_string(), print_id);
    }
    // Builtin: malloc
    let mut malloc_sig = module.make_signature();
    malloc_sig.params.push(cranelift_codegen::ir::AbiParam::new(types::I64));
    malloc_sig.returns.push(cranelift_codegen::ir::AbiParam::new(types::I64));
    if let Ok(malloc_id) = module.declare_function("malloc", cranelift_module::Linkage::Import, &malloc_sig) {
        func_ids.insert("malloc".to_string(), malloc_id);
    }

    for (name, mir_fn) in &program.functions {
        let fid = declare_mir_function(module, mir_fn)?;
        func_ids.insert(name.clone(), fid);
    }

    // Pass 2 — compile all bodies (all signatures now known).
    for mir_fn in program.functions.values() {
        compile_mir_function(module, fb_ctx, ctx, mir_fn, &func_ids, program)?;
    }

    Ok(func_ids)
}
