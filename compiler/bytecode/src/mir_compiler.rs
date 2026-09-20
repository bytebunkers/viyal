use crate::chunk::Chunk;
use crate::opcode::OpCode;
use crate::compiler::CompiledProgram;
use mir::ir::{BasicBlock, MirFunction, MirProgram, Operand, Rvalue, Statement, Terminator, Local};
use interpreter::value::Value;

pub struct MirBytecodeCompiler {
    pub chunk: Chunk,
    pub method_chunks: Vec<Chunk>,
}

impl MirBytecodeCompiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            method_chunks: Vec::new(),
        }
    }

    pub fn compile(mut self, program: &MirProgram) -> Result<CompiledProgram, String> {
        let mut has_main = false;
        
        for (name, func) in &program.functions {
            if name == "main" {
                has_main = true;
            }
            
            let mut func_compiler = MirBytecodeCompiler::new();
            func_compiler.compile_function(func)?;
            
            let compiled_func = func_compiler.compile_finish();
            
            let chunk_idx = self.method_chunks.len();
            self.method_chunks.push(compiled_func.main_chunk);
            self.method_chunks.extend(compiled_func.method_chunks);
            
            let name_idx = self.chunk.add_constant(Value::String(name.clone()));
            self.chunk.write(OpCode::OpFunction(name_idx, chunk_idx, func.params.len() as u8), 1);
        }
        
        if has_main {
            let main_name_idx = self.chunk.add_constant(Value::String("main".to_string()));
            self.chunk.write(OpCode::OpCall(main_name_idx, 0), 1);
            self.chunk.write(OpCode::OpPop, 1);
        }
        
        Ok(self.compile_finish())
    }
    
    fn compile_function(&mut self, func: &MirFunction) -> Result<(), String> {
        // 1. Allocate stack slots for all locals (including temporaries)
        // Note: The VM pushes arguments onto the stack BEFORE calling the function.
        // So the first `func.params.len()` locals are already on the stack!
        let num_locals_to_allocate = func.locals.len().saturating_sub(func.params.len());
        
        for _ in 0..num_locals_to_allocate {
            let null_idx = self.chunk.add_constant(Value::Null);
            self.chunk.write(OpCode::OpConstant(null_idx), 1);
        }
        
        // 2. Compile blocks
        // MIR blocks are linear, but might have arbitrary jumps. We need to track block offsets.
        // For simplicity in MVP, we just emit them in order and patch jumps.
        let mut block_offsets = vec![0; func.basic_blocks.len()];
        let mut pending_jumps: Vec<(usize, usize)> = Vec::new(); // (from_offset, to_block_id)
        
        for block in &func.basic_blocks {
            block_offsets[block.id] = self.chunk.code.len();
            
            for stmt in &block.statements {
                self.compile_statement(stmt)?;
            }
            
            match &block.terminator {
                Terminator::Goto { target } => {
                    let jump_idx = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJump(0), 1); // placeholder
                    pending_jumps.push((jump_idx, *target));
                }
                Terminator::If { cond, then_target, else_target } => {
                    self.compile_operand(cond)?;
                    
                    let false_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJumpIfFalse(0), 1);
                    self.chunk.write(OpCode::OpPop, 1); // pop cond
                    
                    let then_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJump(0), 1); // goto then
                    pending_jumps.push((then_jump, *then_target));
                    
                    // False path
                    self.chunk.patch_jump(false_jump, self.chunk.code.len());
                    self.chunk.write(OpCode::OpPop, 1); // pop cond
                    
                    let else_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJump(0), 1);
                    pending_jumps.push((else_jump, *else_target));
                }
                Terminator::IfOk { val, then_target, else_target } => {
                    self.compile_operand(val)?;
                    
                    let ok_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJumpIfOk(0), 1);
                    self.chunk.write(OpCode::OpPop, 1); // pop err
                    
                    let else_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJump(0), 1);
                    pending_jumps.push((else_jump, *else_target));
                    
                    // Ok path
                    self.chunk.patch_jump(ok_jump, self.chunk.code.len());
                    let then_jump = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJump(0), 1);
                    pending_jumps.push((then_jump, *then_target));
                }
                Terminator::Return { value } => {
                    if let Some(val) = value {
                        self.compile_operand(val)?;
                    } else {
                        let null_idx = self.chunk.add_constant(Value::Null);
                        self.chunk.write(OpCode::OpConstant(null_idx), 1);
                    }
                    self.chunk.write(OpCode::OpReturn, 1);
                }
                Terminator::Unreachable => {
                    self.chunk.write(OpCode::OpReturn, 1); // Safe fallback
                }
            }
        }
        
        // Patch all pending jumps
        for (jump_offset, target_block_id) in pending_jumps {
            let target_address = block_offsets[target_block_id];
            self.chunk.patch_jump(jump_offset, target_address);
        }
        
        Ok(())
    }
    
    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::Assign(local, rvalue) => {
                match rvalue {
                    Rvalue::Use(operand) => {
                        self.compile_operand(operand)?;
                    }
                    Rvalue::BinaryOp(op, left, right) => {
                        self.compile_operand(left)?;
                        self.compile_operand(right)?;
                        match op {
                            ast::BinaryOp::Add => self.chunk.write(OpCode::OpAdd, 1),
                            ast::BinaryOp::Sub => self.chunk.write(OpCode::OpSubtract, 1),
                            ast::BinaryOp::Mul => self.chunk.write(OpCode::OpMultiply, 1),
                            ast::BinaryOp::Div => self.chunk.write(OpCode::OpDivide, 1),
                            ast::BinaryOp::Eq => self.chunk.write(OpCode::OpEqual, 1),
                            ast::BinaryOp::NotEq => self.chunk.write(OpCode::OpNotEqual, 1),
                            ast::BinaryOp::Less => self.chunk.write(OpCode::OpLess, 1),
                            ast::BinaryOp::Greater => self.chunk.write(OpCode::OpGreater, 1),
                            ast::BinaryOp::LessEq => self.chunk.write(OpCode::OpLessEqual, 1),
                            ast::BinaryOp::GreaterEq => self.chunk.write(OpCode::OpGreaterEqual, 1),
                            _ => return Err(format!("Unsupported binary op in MIR: {:?}", op)),
                        }
                    }
                    Rvalue::Call { func, args } => {
                        for arg in args {
                            self.compile_operand(arg)?;
                        }
                        if let Operand::Copy(Local(idx)) = func {
                            // Assume function is a string identifier for now
                            // Wait, if it's a Copy, it means it's a variable reference.
                            // In MVP, we might need a better way to represent function names.
                            return Err("MIR dynamic calls not fully supported yet".to_string());
                        } else if let Operand::Constant(ast::Literal::String(name)) = func {
                            let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                            self.chunk.write(OpCode::OpCall(name_idx, args.len() as u8), 1);
                        }
                    }
                    Rvalue::Array(elements) => {
                        for el in elements {
                            self.compile_operand(el)?;
                        }
                        self.chunk.write(OpCode::OpArray(elements.len()), 1);
                    }
                    Rvalue::Map(pairs) => {
                        for (k, v) in pairs {
                            self.compile_operand(k)?;
                            self.compile_operand(v)?;
                        }
                        self.chunk.write(OpCode::OpMap(pairs.len()), 1);
                    }
                    Rvalue::PropertyAccess(obj, prop) => {
                        self.compile_operand(obj)?;
                        let prop_idx = self.chunk.add_constant(Value::String(prop.clone()));
                        self.chunk.write(OpCode::OpGetProperty(prop_idx), 1);
                    }
                    Rvalue::MethodCall(obj, method, args) => {
                        self.compile_operand(obj)?;
                        for arg in args {
                            self.compile_operand(arg)?;
                        }
                        let meth_idx = self.chunk.add_constant(Value::String(method.clone()));
                        self.chunk.write(OpCode::OpInvoke(meth_idx, args.len() as u8), 1);
                    }
                    Rvalue::Length(obj) => {
                        self.compile_operand(obj)?;
                        self.chunk.write(OpCode::OpLength, 1);
                    }
                    Rvalue::Index(arr, idx) => {
                        self.compile_operand(arr)?;
                        self.compile_operand(idx)?;
                        self.chunk.write(OpCode::OpIndex, 1);
                    }
                    Rvalue::Try(inner) => {
                        self.compile_operand(inner)?;
                        self.chunk.write(OpCode::OpTry, 1);
                    }
                    _ => return Err(format!("Unsupported Rvalue in MIR: {:?}", rvalue)),
                }
                
                // Store result in local
                self.chunk.write(OpCode::OpSetLocal(local.0), 1);
                self.chunk.write(OpCode::OpPop, 1); // OpSetLocal leaves value on stack, so pop it
            }
        }
        Ok(())
    }
    
    fn compile_operand(&mut self, operand: &Operand) -> Result<(), String> {
        match operand {
            Operand::Constant(lit) => {
                let val = match lit {
                    ast::Literal::Integer(i) => Value::Integer(*i),
                    ast::Literal::Float(f) => Value::Float(*f),
                    ast::Literal::String(s) => Value::String(s.clone()),
                    ast::Literal::Boolean(b) => Value::Boolean(*b),
                    ast::Literal::Null => Value::Null,
                };
                let idx = self.chunk.add_constant(val);
                self.chunk.write(OpCode::OpConstant(idx), 1);
            }
            Operand::Copy(local) => {
                self.chunk.write(OpCode::OpGetLocal(local.0), 1);
            }
        }
        Ok(())
    }
    
    pub fn compile_finish(self) -> CompiledProgram {
        CompiledProgram {
            main_chunk: self.chunk,
            method_chunks: self.method_chunks,
        }
    }
}
