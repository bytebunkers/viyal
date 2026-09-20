use crate::ir::*;
use ast::{Decl, Expr, Program, Spanned, Stmt, Type, MatchPattern};
use std::collections::HashMap;

pub struct MirBuilder {
    program: MirProgram,
    current_func: Option<MirFunction>,
    current_block: usize,
}

impl MirBuilder {
    pub fn new() -> Self {
        Self {
            program: MirProgram::default(),
            current_func: None,
            current_block: 0,
        }
    }

    pub fn build(mut self, ast_program: &Program) -> MirProgram {
        for decl in &ast_program.declarations {
            self.visit_decl(&decl.node);
        }
        self.program
    }

    fn visit_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(method, _) => {
                let mut func = MirFunction::new(method.name.clone(), method.return_type.clone());
                
                // Add params as locals
                for param in &method.params {
                    let local_idx = func.locals.len();
                    func.locals.push(LocalDecl {
                        ty: param.param_type.clone(),
                        name: Some(param.name.clone()),
                        is_mut: false,
                    });
                    func.params.push(Local(local_idx));
                }
                
                // Start with a basic block
                let start_block = BasicBlock {
                    id: 0,
                    phis: Vec::new(),
                    statements: Vec::new(),
                    terminator: Terminator::Unreachable,
                };
                func.basic_blocks.push(start_block);
                
                self.current_func = Some(func);
                self.current_block = 0;
                
                self.visit_stmt(&method.body.node);
                
                // Ensure the last block has a terminator
                let mut func = self.current_func.take().unwrap();
                let last_block = &mut func.basic_blocks[self.current_block];
                if matches!(last_block.terminator, Terminator::Unreachable) {
                    last_block.terminator = Terminator::Return { value: None };
                }
                
                self.program.functions.insert(func.name.clone(), func);
            }
            Decl::TypeAlias { .. } | Decl::Import { .. } => {}
            Decl::Class { name, methods, .. } => {
                // To keep it simple for now, we mangle method names as ClassName::MethodName
                for method in methods {
                    let mut func = MirFunction::new(format!("{}::{}", name, method.name), method.return_type.clone());
                    
                    // Add 'this' param
                    let this_idx = func.locals.len();
                    func.locals.push(LocalDecl {
                        ty: Type::Named(name.clone(), Vec::new()),
                        name: Some("this".to_string()),
                        is_mut: false,
                    });
                    func.params.push(Local(this_idx));
                    
                    for param in &method.params {
                        let local_idx = func.locals.len();
                        func.locals.push(LocalDecl {
                            ty: param.param_type.clone(),
                            name: Some(param.name.clone()),
                            is_mut: false,
                        });
                        func.params.push(Local(local_idx));
                    }
                    
                    let start_block = BasicBlock {
                        id: 0,
                        phis: Vec::new(),
                        statements: Vec::new(),
                        terminator: Terminator::Unreachable,
                    };
                    func.basic_blocks.push(start_block);
                    
                    self.current_func = Some(func);
                    self.current_block = 0;
                    
                    self.visit_stmt(&method.body.node);
                    
                    let mut func = self.current_func.take().unwrap();
                    let last_block = &mut func.basic_blocks[self.current_block];
                    if matches!(last_block.terminator, Terminator::Unreachable) {
                        last_block.terminator = Terminator::Return { value: None };
                    }
                    
                    self.program.functions.insert(func.name.clone(), func);
                }
            }
            _ => {} // Skip TypeAlias for now
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                let _ = self.visit_expr(&expr.node);
            }
            Stmt::VarDecl { name, type_annot, initializer, .. } => {
                let ty = type_annot.clone().unwrap_or(Type::Named("Any".to_string(), Vec::new())); // Fallback
                let local = self.add_local(ty, Some(name.clone()));
                
                if let Some(init) = initializer {
                    let operand = self.visit_expr(&init.node);
                    self.add_statement(Statement::Assign(local, Rvalue::Use(operand)));
                }
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.visit_stmt(&s.node);
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond_op = self.visit_expr(&condition.node);
                
                let then_block_id = self.new_block();
                let else_block_id = if else_branch.is_some() { self.new_block() } else { 0 }; // 0 is placeholder
                let merge_block_id = self.new_block();
                
                let actual_else = if else_branch.is_some() { else_block_id } else { merge_block_id };
                
                self.set_terminator(Terminator::If {
                    cond: cond_op,
                    then_target: then_block_id,
                    else_target: actual_else,
                });
                
                self.current_block = then_block_id;
                self.visit_stmt(&then_branch.node);
                self.set_terminator(Terminator::Goto { target: merge_block_id });
                
                if let Some(else_br) = else_branch {
                    self.current_block = else_block_id;
                    self.visit_stmt(&else_br.node);
                    self.set_terminator(Terminator::Goto { target: merge_block_id });
                }
                
                self.current_block = merge_block_id;
            }
            Stmt::Return(opt_expr) => {
                let ret_val = if let Some(expr) = opt_expr {
                    Some(self.visit_expr(&expr.node))
                } else {
                    None
                };
                self.set_terminator(Terminator::Return { value: ret_val });
                // We create a new unreachable block just in case there are statements after return
                let new_block = self.new_block();
                self.current_block = new_block;
            }
            Stmt::While { condition, body } => {
                let cond_block_id = self.new_block();
                let body_block_id = self.new_block();
                let exit_block_id = self.new_block();
                
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                self.current_block = cond_block_id;
                let cond_op = self.visit_expr(&condition.node);
                self.set_terminator(Terminator::If {
                    cond: cond_op,
                    then_target: body_block_id,
                    else_target: exit_block_id,
                });
                
                self.current_block = body_block_id;
                self.visit_stmt(&body.node);
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                self.current_block = exit_block_id;
            }
            Stmt::ForIn { item_name, iterable, body } => {
                let iterable_op = self.visit_expr(&iterable.node);
                let iterable_local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(iterable_local, Rvalue::Use(iterable_op)));
                
                let index_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), Some("__index".to_string()));
                self.add_statement(Statement::Assign(index_local, Rvalue::Use(Operand::Constant(ast::Literal::Integer(0)))));
                
                let length_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), Some("__length".to_string()));
                self.add_statement(Statement::Assign(length_local, Rvalue::Length(Operand::Copy(iterable_local))));
                
                let cond_block_id = self.new_block();
                let body_block_id = self.new_block();
                let exit_block_id = self.new_block();
                
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                // cond_block
                self.current_block = cond_block_id;
                let cond_local = self.add_local(Type::Named("Bool".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(cond_local, Rvalue::BinaryOp(
                    ast::BinaryOp::Less,
                    Operand::Copy(index_local),
                    Operand::Copy(length_local)
                )));
                self.set_terminator(Terminator::If {
                    cond: Operand::Copy(cond_local),
                    then_target: body_block_id,
                    else_target: exit_block_id,
                });
                
                // body_block
                self.current_block = body_block_id;
                let item_local = self.add_local(Type::Named("Any".to_string(), Vec::new()), Some(item_name.clone()));
                self.add_statement(Statement::Assign(item_local, Rvalue::Index(
                    Operand::Copy(iterable_local),
                    Operand::Copy(index_local)
                )));
                
                self.visit_stmt(&body.node);
                
                // index = index + 1
                let incremented_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(incremented_local, Rvalue::BinaryOp(
                    ast::BinaryOp::Add,
                    Operand::Copy(index_local),
                    Operand::Constant(ast::Literal::Integer(1))
                )));
                self.add_statement(Statement::Assign(index_local, Rvalue::Use(Operand::Copy(incremented_local))));
                
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                // exit
                self.current_block = exit_block_id;
            }
            Stmt::ForRange { item_name, start, end, body, .. } => {
                let start_op = self.visit_expr(&start.node);
                let end_op = self.visit_expr(&end.node);
                
                let index_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), Some(item_name.clone()));
                self.add_statement(Statement::Assign(index_local, Rvalue::Use(start_op)));
                
                let end_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), Some("__end".to_string()));
                self.add_statement(Statement::Assign(end_local, Rvalue::Use(end_op)));
                
                let cond_block_id = self.new_block();
                let body_block_id = self.new_block();
                let exit_block_id = self.new_block();
                
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                // cond
                self.current_block = cond_block_id;
                let cond_local = self.add_local(Type::Named("Bool".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(cond_local, Rvalue::BinaryOp(
                    ast::BinaryOp::Less,
                    Operand::Copy(index_local),
                    Operand::Copy(end_local)
                )));
                self.set_terminator(Terminator::If {
                    cond: Operand::Copy(cond_local),
                    then_target: body_block_id,
                    else_target: exit_block_id,
                });
                
                // body
                self.current_block = body_block_id;
                self.visit_stmt(&body.node);
                
                // index = index + 1
                let incremented_local = self.add_local(Type::Named("Int".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(incremented_local, Rvalue::BinaryOp(
                    ast::BinaryOp::Add,
                    Operand::Copy(index_local),
                    Operand::Constant(ast::Literal::Integer(1))
                )));
                self.add_statement(Statement::Assign(index_local, Rvalue::Use(Operand::Copy(incremented_local))));
                
                self.set_terminator(Terminator::Goto { target: cond_block_id });
                
                // exit
                self.current_block = exit_block_id;
            }
            _ => {} // Fallback for any other statement
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> Operand {
        match expr {
            Expr::Literal(lit) => Operand::Constant(lit.clone()),
            Expr::Identifier(name) => {
                // TODO: Need a real environment to map names to Locals.
                // For now, this is a placeholder. A real implementation needs scope tracking.
                Operand::Copy(Local(0)) // BUG: Placeholder
            }
            Expr::Binary(left, op, right) => {
                let l_op = self.visit_expr(&left.node);
                let r_op = self.visit_expr(&right.node);
                
                let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(local, Rvalue::BinaryOp(op.clone(), l_op, r_op)));
                Operand::Copy(local)
            }
            Expr::Call(callee, _, args) => {
                if let Expr::PropertyAccess(obj, method_name) = &callee.node {
                    let obj_op = self.visit_expr(&obj.node);
                    let mut arg_ops = Vec::new();
                    for arg in args {
                        arg_ops.push(self.visit_expr(&arg.node));
                    }
                    let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                    self.add_statement(Statement::Assign(local, Rvalue::MethodCall(obj_op, method_name.clone(), arg_ops)));
                    Operand::Copy(local)
                } else {
                    let func_op = self.visit_expr(&callee.node);
                    let mut arg_ops = Vec::new();
                    for arg in args {
                        arg_ops.push(self.visit_expr(&arg.node));
                    }
                    
                    let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                    self.add_statement(Statement::Assign(local, Rvalue::Call { func: func_op, args: arg_ops }));
                    Operand::Copy(local)
                }
            }
            Expr::Array(elements) => {
                let mut ops = Vec::new();
                for el in elements {
                    ops.push(self.visit_expr(&el.node));
                }
                let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(local, Rvalue::Array(ops)));
                Operand::Copy(local)
            }
            Expr::Map(pairs) => {
                let mut ops = Vec::new();
                for (k, v) in pairs {
                    let k_op = self.visit_expr(&k.node);
                    let v_op = self.visit_expr(&v.node);
                    ops.push((k_op, v_op));
                }
                let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(local, Rvalue::Map(ops)));
                Operand::Copy(local)
            }
            Expr::PropertyAccess(obj, prop) => {
                let obj_op = self.visit_expr(&obj.node);
                let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(local, Rvalue::PropertyAccess(obj_op, prop.clone())));
                Operand::Copy(local)
            }
            Expr::Match(subject, arms) => {
                let subject_op = self.visit_expr(&subject.node);
                let result_local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                let merge_block_id = self.new_block();

                for (pattern, expr) in arms {
                    let test_block_id = self.current_block;
                    let body_block_id = self.new_block();
                    let next_test_block_id = self.new_block();

                    match pattern {
                        MatchPattern::CatchAll => {
                            self.set_terminator(Terminator::Goto { target: body_block_id });
                        }
                        MatchPattern::Literal(lit) => {
                            let cond_local = self.add_local(Type::Named("Bool".to_string(), Vec::new()), None);
                            self.add_statement(Statement::Assign(
                                cond_local,
                                Rvalue::BinaryOp(ast::BinaryOp::Eq, subject_op.clone(), Operand::Constant(lit.clone()))
                            ));
                            
                            self.set_terminator(Terminator::If {
                                cond: Operand::Copy(cond_local),
                                then_target: body_block_id,
                                else_target: next_test_block_id,
                            });
                        }
                        MatchPattern::Identifier(name) => {
                            // Map the identifier to a new local, copy the subject into it
                            let bound_local = self.add_local(Type::Named("Any".to_string(), Vec::new()), Some(name.clone()));
                            self.add_statement(Statement::Assign(bound_local, Rvalue::Use(subject_op.clone())));
                            self.set_terminator(Terminator::Goto { target: body_block_id });
                        }
                    }

                    // Build the body block
                    self.current_block = body_block_id;
                    let body_val = self.visit_expr(&expr.node);
                    self.add_statement(Statement::Assign(result_local, Rvalue::Use(body_val)));
                    self.set_terminator(Terminator::Goto { target: merge_block_id });

                    // Set up for next iteration
                    self.current_block = next_test_block_id;
                }

                // If match is exhaustive, the final next_test_block_id is unreachable.
                self.current_block = merge_block_id;
                Operand::Copy(result_local)
            }
            Expr::Try(inner) => {
                let inner_op = self.visit_expr(&inner.node);
                let local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                self.add_statement(Statement::Assign(local, Rvalue::Try(inner_op)));
                Operand::Copy(local)
            }
            Expr::UnwrapOrElse(inner, block) => {
                let inner_op = self.visit_expr(&inner.node);
                let result_local = self.add_local(Type::Named("Any".to_string(), Vec::new()), None);
                
                let ok_block_id = self.new_block();
                let err_block_id = self.new_block();
                let merge_block_id = self.new_block();
                
                self.set_terminator(Terminator::IfOk {
                    val: inner_op.clone(),
                    then_target: ok_block_id,
                    else_target: err_block_id,
                });
                
                // OK path: just assign the result
                self.current_block = ok_block_id;
                self.add_statement(Statement::Assign(result_local, Rvalue::Use(inner_op)));
                self.set_terminator(Terminator::Goto { target: merge_block_id });
                
                // ERR path: execute block (which might return/throw, or evaluate to a value)
                self.current_block = err_block_id;
                self.visit_stmt(&block.node);
                // Viyal blocks in UnwrapOrElse are technically Statements, but they act like expressions.
                // For MVP, we'll assign Null to result_local if it reaches here, though it likely returns.
                self.add_statement(Statement::Assign(result_local, Rvalue::Use(Operand::Constant(ast::Literal::Null))));
                self.set_terminator(Terminator::Goto { target: merge_block_id });
                
                self.current_block = merge_block_id;
                Operand::Copy(result_local)
            }
            _ => {
                // Fallback placeholder
                Operand::Constant(ast::Literal::Null)
            }
        }
    }
    
    // --- Helpers ---
    
    fn add_local(&mut self, ty: Type, name: Option<String>) -> Local {
        let func = self.current_func.as_mut().unwrap();
        let idx = func.locals.len();
        func.locals.push(LocalDecl { ty, name, is_mut: true });
        Local(idx)
    }
    
    fn add_statement(&mut self, stmt: Statement) {
        let func = self.current_func.as_mut().unwrap();
        func.basic_blocks[self.current_block].statements.push(stmt);
    }
    
    fn set_terminator(&mut self, term: Terminator) {
        let func = self.current_func.as_mut().unwrap();
        // Only set if it's currently Unreachable (so we don't overwrite a Return)
        if matches!(func.basic_blocks[self.current_block].terminator, Terminator::Unreachable) {
            func.basic_blocks[self.current_block].terminator = term;
        }
    }
    
    fn new_block(&mut self) -> usize {
        let func = self.current_func.as_mut().unwrap();
        let id = func.basic_blocks.len();
        func.basic_blocks.push(BasicBlock {
            id,
            phis: Vec::new(),
            statements: Vec::new(),
            terminator: Terminator::Unreachable, // Placeholder
        });
        id
    }
}
