use crate::chunk::Chunk;
use crate::opcode::OpCode;
use ast::{Expr, Literal, Program, Spanned, Stmt, BinaryOp, Decl};
use interpreter::value::Value;

pub struct Local {
    name: String,
    depth: usize,
}

pub struct CompiledProgram {
    pub main_chunk: Chunk,
    pub method_chunks: Vec<Chunk>,
}

pub struct BytecodeCompiler {
    pub chunk: Chunk,
    pub method_chunks: Vec<Chunk>,
    locals: Vec<Local>,
    scope_depth: usize,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            method_chunks: Vec::new(),
            locals: Vec::new(),
            scope_depth: 0,
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.chunk.write(OpCode::OpPop, 1);
                self.locals.pop();
            } else {
                break;
            }
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn compile(mut self, program: &Program) -> Result<CompiledProgram, String> {
        // Compile all class declarations
        let mut has_main_func = false;

        for decl in &program.declarations {
            if let Decl::Class { name, methods, .. } = &decl.node {
                let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write(OpCode::OpClass(name_idx), 1);
                
                // Compile methods
                for method in methods {
                    let mut method_compiler = BytecodeCompiler::new();
                    method_compiler.begin_scope();
                    // Add 'this' (implicit local 0)
                    method_compiler.locals.push(Local { name: "this".to_string(), depth: method_compiler.scope_depth });
                    
                    for param in &method.params {
                        method_compiler.locals.push(Local { name: param.name.clone(), depth: method_compiler.scope_depth });
                    }
                    
                    if let Stmt::Block(stmts) = &method.body.node {
                        for stmt in stmts {
                            method_compiler.compile_stmt(stmt)?;
                        }
                    }
                    method_compiler.end_scope();
                    let method_prog = method_compiler.compile_finish();
                    
                    let chunk_idx = self.method_chunks.len();
                    self.method_chunks.push(method_prog.main_chunk);
                    self.method_chunks.extend(method_prog.method_chunks);
                    
                    let meth_name_idx = self.chunk.add_constant(Value::String(method.name.clone()));
                    self.chunk.write(OpCode::OpMethod(meth_name_idx, chunk_idx, method.params.len() as u8), 1);
                }
            } else if let Decl::Function(method) = &decl.node {
                if method.name == "main" {
                    has_main_func = true;
                }
                let mut method_compiler = BytecodeCompiler::new();
                method_compiler.begin_scope();
                
                for param in &method.params {
                    method_compiler.locals.push(Local { name: param.name.clone(), depth: method_compiler.scope_depth });
                }
                
                if let Stmt::Block(stmts) = &method.body.node {
                    for stmt in stmts {
                        method_compiler.compile_stmt(stmt)?;
                    }
                }
                method_compiler.end_scope();
                let method_prog = method_compiler.compile_finish();
                
                let chunk_idx = self.method_chunks.len();
                self.method_chunks.push(method_prog.main_chunk);
                self.method_chunks.extend(method_prog.method_chunks);
                
                let meth_name_idx = self.chunk.add_constant(Value::String(method.name.clone()));
                self.chunk.write(OpCode::OpFunction(meth_name_idx, chunk_idx, method.params.len() as u8), 1);
            }
        }
        
        if has_main_func {
            let main_name_idx = self.chunk.add_constant(Value::String("main".to_string()));
            self.chunk.write(OpCode::OpCall(main_name_idx, 0), 1);
            self.chunk.write(OpCode::OpPop, 1);
        } else {
            // Find Main.run to execute it.
            // For MVP: We just inject a `new Main().run()` at the end of the global script.
            let main_name_idx = self.chunk.add_constant(Value::String("Main".to_string()));
            self.chunk.write(OpCode::OpConstruct(main_name_idx, 0), 1);
            let run_name_idx = self.chunk.add_constant(Value::String("run".to_string()));
            self.chunk.write(OpCode::OpInvoke(run_name_idx, 0), 1);
            self.chunk.write(OpCode::OpPop, 1);
        }
        
        Ok(self.compile_finish())
    }
    
    pub fn compile_stmt(&mut self, stmt: &Spanned<Stmt>) -> Result<(), String> {
        match &stmt.node {
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
                self.chunk.write(OpCode::OpPop, 1);
            }
            Stmt::VarDecl { name, initializer, .. } => {
                if let Some(init) = initializer {
                    self.compile_expr(init)?;
                } else {
                    let idx = self.chunk.add_constant(Value::Null);
                    self.chunk.write(OpCode::OpConstant(idx), 1);
                }
                self.locals.push(Local { name: name.clone(), depth: self.scope_depth });
            }
            Stmt::Block(stmts) => {
                self.begin_scope();
                for s in stmts {
                    self.compile_stmt(s)?;
                }
                self.end_scope();
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expr(condition)?;
                let then_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfFalse(0), 1); // placeholder
                self.chunk.write(OpCode::OpPop, 1); // pop condition
                
                self.compile_stmt(then_branch)?;
                
                let else_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJump(0), 1); // placeholder
                
                self.chunk.patch_jump(then_jump, self.chunk.code.len());
                self.chunk.write(OpCode::OpPop, 1); // pop condition on false path
                
                if let Some(else_b) = else_branch {
                    self.compile_stmt(else_b)?;
                }
                
                self.chunk.patch_jump(else_jump, self.chunk.code.len());
            }
            Stmt::While { condition, body } => {
                let loop_start = self.chunk.code.len();
                self.compile_expr(condition)?;
                
                let exit_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfFalse(0), 1);
                self.chunk.write(OpCode::OpPop, 1); // pop condition
                
                self.compile_stmt(body)?;
                self.chunk.write(OpCode::OpJump(loop_start), 1);
                
                self.chunk.patch_jump(exit_jump, self.chunk.code.len());
                self.chunk.write(OpCode::OpPop, 1); // pop condition on exit
            }
            Stmt::ForRange { item_name, start, end, body } => {
                self.begin_scope();
                
                self.compile_expr(start)?;
                self.locals.push(Local { name: item_name.clone(), depth: self.scope_depth });
                
                self.compile_expr(end)?;
                self.locals.push(Local { name: "__end".to_string(), depth: self.scope_depth });
                
                let loop_start = self.chunk.code.len();
                
                let item_idx = self.resolve_local(item_name).unwrap();
                self.chunk.write(OpCode::OpGetLocal(item_idx), 1);
                
                let end_idx = self.resolve_local("__end").unwrap();
                self.chunk.write(OpCode::OpGetLocal(end_idx), 1);
                
                self.chunk.write(OpCode::OpLess, 1);
                
                let exit_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfFalse(0), 1);
                self.chunk.write(OpCode::OpPop, 1); 
                
                self.compile_stmt(body)?;
                
                self.chunk.write(OpCode::OpGetLocal(item_idx), 1);
                let one_idx = self.chunk.add_constant(Value::Integer(1));
                self.chunk.write(OpCode::OpConstant(one_idx), 1);
                self.chunk.write(OpCode::OpAdd, 1);
                self.chunk.write(OpCode::OpSetLocal(item_idx), 1);
                self.chunk.write(OpCode::OpPop, 1);
                
                self.chunk.write(OpCode::OpJump(loop_start), 1);
                
                self.chunk.patch_jump(exit_jump, self.chunk.code.len());
                self.chunk.write(OpCode::OpPop, 1);
                
                self.end_scope();
            }
            Stmt::ForIn { item_name, iterable, body } => {
                self.begin_scope();
                
                self.compile_expr(iterable)?;
                self.locals.push(Local { name: "__iterable".to_string(), depth: self.scope_depth });
                
                let zero_idx = self.chunk.add_constant(Value::Integer(0));
                self.chunk.write(OpCode::OpConstant(zero_idx), 1);
                self.locals.push(Local { name: "__index".to_string(), depth: self.scope_depth });
                
                let null_idx = self.chunk.add_constant(Value::Null);
                self.chunk.write(OpCode::OpConstant(null_idx), 1);
                self.locals.push(Local { name: item_name.clone(), depth: self.scope_depth });
                
                let loop_start = self.chunk.code.len();
                
                let index_idx = self.resolve_local("__index").unwrap();
                self.chunk.write(OpCode::OpGetLocal(index_idx), 1);
                
                let iterable_idx = self.resolve_local("__iterable").unwrap();
                self.chunk.write(OpCode::OpGetLocal(iterable_idx), 1);
                self.chunk.write(OpCode::OpLength, 1);
                
                self.chunk.write(OpCode::OpLess, 1);
                
                let exit_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfFalse(0), 1);
                self.chunk.write(OpCode::OpPop, 1);
                
                // item = __iterable[__index]
                self.chunk.write(OpCode::OpGetLocal(iterable_idx), 1);
                self.chunk.write(OpCode::OpGetLocal(index_idx), 1);
                self.chunk.write(OpCode::OpIndex, 1);
                let item_idx = self.resolve_local(item_name).unwrap();
                self.chunk.write(OpCode::OpSetLocal(item_idx), 1);
                self.chunk.write(OpCode::OpPop, 1);
                
                self.compile_stmt(body)?;
                
                // __index = __index + 1
                self.chunk.write(OpCode::OpGetLocal(index_idx), 1);
                let one_idx = self.chunk.add_constant(Value::Integer(1));
                self.chunk.write(OpCode::OpConstant(one_idx), 1);
                self.chunk.write(OpCode::OpAdd, 1);
                self.chunk.write(OpCode::OpSetLocal(index_idx), 1);
                self.chunk.write(OpCode::OpPop, 1);
                
                self.chunk.write(OpCode::OpJump(loop_start), 1);
                
                self.chunk.patch_jump(exit_jump, self.chunk.code.len());
                self.chunk.write(OpCode::OpPop, 1);
                
                self.end_scope();
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expr(e)?;
                } else {
                    let idx = self.chunk.add_constant(Value::Null);
                    self.chunk.write(OpCode::OpConstant(idx), 1);
                }
                self.chunk.write(OpCode::OpReturn, 1);
            }
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Spanned<Expr>) -> Result<(), String> {
        let line = 1;
        match &expr.node {
            Expr::Literal(lit) => {
                let val = match lit {
                    Literal::Integer(i) => Value::Integer(*i),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Boolean(b) => Value::Boolean(*b),
                    Literal::Null => Value::Null,
                };
                let idx = self.chunk.add_constant(val);
                self.chunk.write(OpCode::OpConstant(idx), line);
            },
            Expr::Identifier(name) => {
                if let Some(local_idx) = self.resolve_local(name) {
                    self.chunk.write(OpCode::OpGetLocal(local_idx), line);
                } else {
                    let idx = self.chunk.add_constant(Value::String(name.clone()));
                    self.chunk.write(OpCode::OpGetGlobal(idx), line);
                }
            },
            Expr::Binary(left, op, right) => {
                if let BinaryOp::Assign = op {
                    if let Expr::Identifier(name) = &left.node {
                        self.compile_expr(right)?;
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.chunk.write(OpCode::OpSetLocal(local_idx), line);
                        } else {
                            let idx = self.chunk.add_constant(Value::String(name.clone()));
                            self.chunk.write(OpCode::OpSetGlobal(idx), line);
                        }
                        return Ok(());
                    } else {
                        return Err("Invalid assignment target".into());
                    }
                }
                
                self.compile_expr(left)?;
                self.compile_expr(right)?;
                match op {
                    BinaryOp::Add => self.chunk.write(OpCode::OpAdd, line),
                    BinaryOp::Sub => self.chunk.write(OpCode::OpSubtract, line),
                    BinaryOp::Mul => self.chunk.write(OpCode::OpMultiply, line),
                    BinaryOp::Div => self.chunk.write(OpCode::OpDivide, line),
                    BinaryOp::Eq => self.chunk.write(OpCode::OpEqual, line),
                    BinaryOp::NotEq => self.chunk.write(OpCode::OpNotEqual, line),
                    BinaryOp::Less => self.chunk.write(OpCode::OpLess, line),
                    BinaryOp::Greater => self.chunk.write(OpCode::OpGreater, line),
                    BinaryOp::LessEq => self.chunk.write(OpCode::OpLessEqual, line),
                    BinaryOp::GreaterEq => self.chunk.write(OpCode::OpGreaterEqual, line),
                    _ => return Err("Unsupported binary op".into()),
                }
            },
            Expr::Call(target, args) => {
                if let Expr::Identifier(id) = &target.node {
                    if id == "print" {
                        if let Some(arg) = args.first() {
                            self.compile_expr(arg)?;
                            self.chunk.write(OpCode::OpPrint, line);
                        }
                    } else if id == "error" {
                        if let Some(arg) = args.first() {
                            self.compile_expr(arg)?;
                            self.chunk.write(OpCode::OpError, line);
                        }
                    } else {
                        for arg in args {
                            self.compile_expr(arg)?;
                        }
                        let name_idx = self.chunk.add_constant(Value::String(id.clone()));
                        self.chunk.write(OpCode::OpCall(name_idx, args.len() as u8), line);
                    }
                } else if let Expr::PropertyAccess(obj, method_name) = &target.node {
                    self.compile_expr(obj)?;
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    let name_idx = self.chunk.add_constant(Value::String(method_name.clone()));
                    self.chunk.write(OpCode::OpInvoke(name_idx, args.len() as u8), line);
                }
            }
            Expr::New(class_name, args) => {
                for arg in args {
                    self.compile_expr(arg)?;
                }
                let name_idx = self.chunk.add_constant(Value::String(class_name.clone()));
                self.chunk.write(OpCode::OpConstruct(name_idx, args.len() as u8), line);
            }
            Expr::Array(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.chunk.write(OpCode::OpArray(items.len()), line);
            }
            Expr::Map(pairs) => {
                for (key, value) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(value)?;
                }
                self.chunk.write(OpCode::OpMap(pairs.len()), line);
            }
            Expr::Index(array, index) => {
                self.compile_expr(array)?;
                self.compile_expr(index)?;
                self.chunk.write(OpCode::OpIndex, line);
            }
            Expr::IndexAssign(array, index, value) => {
                self.compile_expr(array)?;
                self.compile_expr(index)?;
                self.compile_expr(value)?;
                self.chunk.write(OpCode::OpIndexSet, line);
            }
            Expr::Try(inner) => {
                self.compile_expr(inner)?;
                self.chunk.write(OpCode::OpTry, line);
            }
            Expr::UnwrapOrElse(inner, block) => {
                self.compile_expr(inner)?;
                let ok_jump = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfOk(0), line); // placeholder jumps if OK
                self.chunk.write(OpCode::OpPop, line); // pop error if we didn't jump
                
                self.compile_stmt(block)?;
                // the block must return or panic, but if it doesn't we might need a fallback.
                
                self.chunk.patch_jump(ok_jump, self.chunk.code.len());
            }
            Expr::This => {
                if let Some(local_idx) = self.resolve_local(&"this".to_string()) {
                    self.chunk.write(OpCode::OpGetLocal(local_idx), line);
                } else {
                    return Err("Cannot use 'this' outside of a class".into());
                }
            }
            Expr::PropertyAccess(_obj, method_name) => {
                // If it's used as an expression (not a call), we don't have a specific opcode right now?
                // Wait, if it's a property access not in a call, we return an error for now since we only support method calls.
                return Err(format!("Property access without call is not supported yet: {:?}", method_name));
            }
            _ => return Err(format!("Unsupported expr: {:?}", expr.node)),
        }
        Ok(())
    }
    
    pub fn compile_finish(mut self) -> CompiledProgram {
        let null_idx = self.chunk.add_constant(Value::Null);
        self.chunk.write(OpCode::OpConstant(null_idx), 1);
        self.chunk.write(OpCode::OpReturn, 1);
        CompiledProgram {
            main_chunk: self.chunk,
            method_chunks: self.method_chunks,
        }
    }
}
