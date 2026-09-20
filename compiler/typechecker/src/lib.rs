pub mod error;
pub mod env;

use ast::*;
use env::TypeEnv;
use error::TypeError;

pub struct TypeChecker {
    pub env: TypeEnv,
    pub current_return_type: Option<Type>,
    pub current_class_name: Option<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            env: TypeEnv::new(),
            current_return_type: None,
            current_class_name: None,
        }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        // ── Pre-pass: register all TypeAlias declarations first ───────────────
        // Class fields, function parameters, and return types may all reference
        // type aliases, so we resolve them before the main declaration pass.
        for decl in &program.declarations {
            if let Decl::TypeAlias { name, target_type, .. } = &decl.node {
                let resolved = self.env.resolve_type(target_type);
                self.env.type_aliases.insert(name.clone(), resolved);
            }
        }

        // ── Declaration Pass: register class and function signatures ──────────
        for decl in &program.declarations {
            if let Decl::Class { name, methods, is_exported: _, .. } = &decl.node {
                let mut class_sig = env::ClassSignature {
                    methods: std::collections::HashMap::new(),
                };
                for method in methods {
                    let sig = env::MethodSignature {
                        params: method.params.iter().map(|p| p.param_type.clone()).collect(),
                        return_type: method.return_type.clone(),
                    };
                    class_sig.methods.insert(method.name.clone(), sig);
                }
                self.env.classes.insert(name.clone(), class_sig);
            } else if let Decl::Function(method, _) = &decl.node {
                let sig = env::MethodSignature {
                    params: method.params.iter().map(|p| p.param_type.clone()).collect(),
                    return_type: method.return_type.clone(),
                };
                self.env.functions.insert(method.name.clone(), sig);
            }
        }

        // ── Verification Pass ─────────────────────────────────────────────────
        for decl in &program.declarations {
            self.check_decl(&decl.node)?;
        }
        Ok(())
    }

    fn types_match(&self, expected: &Type, found: &Type) -> bool {
        if expected == found {
            return true;
        }
        if found == &Type::Named("null".to_string(), Vec::new()) {
            return true;
        }
        if found == &Type::Named("Any".to_string(), Vec::new()) || expected == &Type::Named("Any".to_string(), Vec::new()) {
            return true;
        }
        if let Type::Named(name, _) = expected {
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                return true;
            }
        }
        if let Type::Named(name, _) = found {
            if name.len() == 1 && name.chars().next().unwrap().is_uppercase() {
                return true;
            }
        }
        false
    }

    fn check_decl(&mut self, decl: &Decl) -> Result<(), TypeError> {
        match decl {
            Decl::Class { name, methods, primary_constructor, type_params, .. } => {
                let prev_class = self.current_class_name.clone();
                self.current_class_name = Some(name.clone());
                for method in methods {
                    self.env.enter_scope();
                    
                    let prev_return = self.current_return_type.clone();
                    self.current_return_type = method.return_type.clone();
                    
                    for type_param in type_params {
                        self.env.type_aliases.insert(type_param.clone(), Type::Named("Any".to_string(), Vec::new()));
                    }
                    for type_param in &method.type_params {
                        self.env.type_aliases.insert(type_param.clone(), Type::Named("Any".to_string(), Vec::new()));
                    }
                    
                    // Add class fields to scope
                    for param in primary_constructor {
                        self.env.define(param.name.clone(), param.param_type.clone(), false);
                    }
                    
                    // Add method parameters to scope
                    for param in &method.params {
                        self.env.define(param.name.clone(), param.param_type.clone(), false);
                    }
                    
                    self.check_stmt(&method.body)?;
                    
                    self.current_return_type = prev_return;
                    self.env.exit_scope();
                }
                self.current_class_name = prev_class;
            },
            Decl::Function(method, _) => {
                self.env.enter_scope();
                let prev_return = self.current_return_type.clone();
                self.current_return_type = method.return_type.clone();
                for type_param in &method.type_params {
                    self.env.type_aliases.insert(type_param.clone(), Type::Named("Any".to_string(), Vec::new()));
                }
                for param in &method.params {
                    self.env.define(param.name.clone(), param.param_type.clone(), false);
                }
                self.check_stmt(&method.body)?;
                self.current_return_type = prev_return;
                self.env.exit_scope();
            }
            Decl::TypeAlias { .. } => {
                // Already registered in check_program's pre-pass.
                // Nothing further to verify here for the MVP.
            }
            Decl::Import { .. } => {
                // To be implemented
            }
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Spanned<Stmt>) -> Result<(), TypeError> {
        match &stmt.node {
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
            },
            Stmt::Block(stmts) => {
                self.env.enter_scope();
                for s in stmts {
                    self.check_stmt(s)?;
                }
                self.env.exit_scope();
            },
            Stmt::VarDecl { is_final, type_annot, name, initializer, .. } => {
                let inferred_type = if let Some(init) = initializer {
                    self.check_expr(init)?
                } else {
                    Type::Named("void".to_string(), Vec::new())
                };

                let final_type = if let Some(annot) = type_annot {
                    let annot = self.env.resolve_type(annot);
                    if inferred_type != Type::Named("void".to_string(), Vec::new()) && annot != inferred_type {
                        return Err(TypeError {
                            message: "Type mismatch in declaration".to_string(),
                            expected: Some(annot.clone()),
                            found: Some(inferred_type),
                            span: stmt.span.clone(),
                        });
                    }
                    annot
                } else {
                    inferred_type
                };

                self.env.define(name.clone(), final_type, !*is_final);
            },
            Stmt::Return(expr) => {
                let return_expr_type = if let Some(e) = expr {
                    self.check_expr(e)?
                } else {
                    Type::Named("void".to_string(), Vec::new())
                };
                
                let expected_type = self.current_return_type.clone().unwrap_or(Type::Named("void".to_string(), Vec::new()));
                
                let is_valid = if self.types_match(&expected_type, &return_expr_type) {
                    true
                } else {
                    match &expected_type {
                        Type::Result(inner) => {
                            if return_expr_type == **inner || return_expr_type == Type::Named("Error".to_string(), Vec::new()) {
                                true
                            } else {
                                false
                            }
                        },
                        Type::Option(inner) => {
                            if return_expr_type == **inner {
                                true
                            } else {
                                false
                            }
                        },
                        _ => false,
                    }
                };

                if !is_valid {
                     return Err(TypeError {
                         message: "Return type mismatch".to_string(),
                         expected: Some(expected_type),
                         found: Some(return_expr_type),
                         span: stmt.span.clone(),
                     });
                }
            },
            Stmt::While { condition, body } => {
                let cond_ty = self.check_expr(condition)?;
                if cond_ty != Type::Named("bool".to_string(), Vec::new()) {
                    return Err(TypeError {
                        message: "While condition must be a boolean".to_string(),
                        expected: Some(Type::Named("bool".to_string(), Vec::new())),
                        found: Some(cond_ty),
                        span: condition.span.clone(),
                    });
                }
                self.check_stmt(body)?;
            },
            Stmt::ForIn { item_name, iterable, body } => {
                let iter_ty = self.check_expr(iterable)?;
                let item_ty = match iter_ty {
                    Type::Array(inner) => *inner,
                    Type::Named(n, _) if n == "String" => Type::Named("String".to_string(), Vec::new()),
                    _ => Type::Named("Any".to_string(), Vec::new()), // Fallback for dynamic types
                };
                self.env.enter_scope();
                self.env.define(item_name.clone(), item_ty, true);
                self.check_stmt(body)?;
                self.env.exit_scope();
            },
            Stmt::ForRange { item_name, start, end, body } => {
                let start_ty = self.check_expr(start)?;
                let end_ty = self.check_expr(end)?;
                if start_ty != Type::Named("int".to_string(), Vec::new()) || end_ty != Type::Named("int".to_string(), Vec::new()) {
                    return Err(TypeError {
                        message: "Range bounds must be integers".to_string(),
                        expected: Some(Type::Named("int".to_string(), Vec::new())),
                        found: None,
                        span: start.span.start..end.span.end,
                    });
                }
                self.env.enter_scope();
                self.env.define(item_name.clone(), Type::Named("int".to_string(), Vec::new()), true);
                self.check_stmt(body)?;
                self.env.exit_scope();
            },
            _ => {}
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Spanned<Expr>) -> Result<Type, TypeError> {
        match &expr.node {
            Expr::Literal(lit) => {
                let ty = match lit {
                    Literal::String(_) => "String",
                    Literal::Integer(_) => "int",
                    Literal::Float(_) => "double",
                    Literal::Boolean(_) => "bool",
                    Literal::Null => "null",
                };
                Ok(Type::Named(ty.to_string(), Vec::new()))
            },
            Expr::Identifier(name) => {
                if let Some(ty) = self.env.lookup(name) {
                    Ok(ty)
                } else {
                    // Temporarily allow 'print' and stdlib modules for MVP tests
                    if name == "print" || name == "assert" || name == "assertEq" {
                        return Ok(Type::Named("void".to_string(), Vec::new()));
                    }
                    if ["math", "fs", "os", "time", "http", "json", "path", "process", "str"].contains(&name.as_str()) {
                        return Ok(Type::Named("Module".to_string(), Vec::new()));
                    }
                    Err(TypeError {
                        message: format!("Undefined variable '{}'", name),
                        expected: None,
                        found: None,
                        span: expr.span.clone(),
                    })
                }
            },
            Expr::Call(target, _, args) => {
                if let Expr::PropertyAccess(obj, method_name) = &target.node {
                    let obj_type = self.check_expr(obj)?;
                    if let Type::Named(class_name, _) = obj_type {
                        if class_name == "Module" || class_name == "Any" {
                            for arg in args {
                                self.check_expr(arg)?;
                            }
                            // Module and Any functions can return anything, MVP: Any
                            return Ok(Type::Named("Any".to_string(), Vec::new()));
                        }
                        let method_sig_opt = self.env.classes.get(&class_name)
                            .and_then(|class_sig| class_sig.methods.get(method_name).cloned());
                            
                        if let Some(method_sig) = method_sig_opt {
                            if args.len() != method_sig.params.len() {
                                return Err(TypeError {
                                    message: format!("Method {} expects {} arguments, got {}", method_name, method_sig.params.len(), args.len()),
                                    expected: None,
                                    found: None,
                                    span: expr.span.clone(),
                                });
                            }
                            for (i, arg) in args.iter().enumerate() {
                                let arg_type = self.check_expr(arg)?;
                                if !self.types_match(&method_sig.params[i], &arg_type) {
                                    return Err(TypeError {
                                        message: format!("Argument {} type mismatch", i),
                                        expected: Some(method_sig.params[i].clone()),
                                        found: Some(arg_type),
                                        span: arg.span.clone(),
                                    });
                                }
                            }
                            return Ok(method_sig.return_type.clone().unwrap_or(Type::Named("void".to_string(), Vec::new())));
                        } else {
                            return Err(TypeError {
                                message: format!("Method {} not found in class {}", method_name, class_name),
                                expected: None,
                                found: None,
                                span: expr.span.clone(),
                            });
                        }
                    }
                } else if let Expr::Identifier(name) = &target.node {
                    if name == "error" {
                        // special built-in
                        let _ = self.check_expr(&args[0])?;
                        return Ok(Type::Named("Error".to_string(), Vec::new()));
                    }
                    if name == "print" {
                        let _ = self.check_expr(&args[0])?;
                        return Ok(Type::Named("void".to_string(), Vec::new()));
                    }
                    if name == "assert" {
                        if args.len() != 1 {
                            return Err(TypeError {
                                message: "assert expects 1 argument".to_string(),
                                expected: None, found: None, span: expr.span.clone(),
                            });
                        }
                        let _ = self.check_expr(&args[0])?;
                        return Ok(Type::Named("void".to_string(), Vec::new()));
                    }
                    if name == "assertEq" {
                        if args.len() != 2 {
                            return Err(TypeError {
                                message: "assertEq expects 2 arguments".to_string(),
                                expected: None, found: None, span: expr.span.clone(),
                            });
                        }
                        let _ = self.check_expr(&args[0])?;
                        let _ = self.check_expr(&args[1])?;
                        return Ok(Type::Named("void".to_string(), Vec::new()));
                    }
                    if let Some(method_sig) = self.env.functions.get(name).cloned() {
                        if args.len() != method_sig.params.len() {
                            return Err(TypeError {
                                message: format!("Function {} expects {} arguments, got {}", name, method_sig.params.len(), args.len()),
                                expected: None,
                                found: None,
                                span: expr.span.clone(),
                            });
                        }
                        for (i, arg) in args.iter().enumerate() {
                            let arg_type = self.check_expr(arg)?;
                            if !self.types_match(&method_sig.params[i], &arg_type) {
                                return Err(TypeError {
                                    message: format!("Argument {} type mismatch", i),
                                    expected: Some(method_sig.params[i].clone()),
                                    found: Some(arg_type),
                                    span: arg.span.clone(),
                                });
                            }
                        }
                        return Ok(method_sig.return_type.unwrap_or(Type::Named("void".to_string(), Vec::new())));
                    }
                }
                
                let target_type = self.check_expr(target)?;
                for arg in args {
                    self.check_expr(arg)?;
                }
                
                
                if target_type == Type::Named("Any".to_string(), Vec::new()) {
                    Ok(Type::Named("Any".to_string(), Vec::new()))
                } else {
                    Ok(Type::Named("void".to_string(), Vec::new()))
                }
            },
            Expr::Binary(left, op, right) => {
                let left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;
                
                if *op == BinaryOp::Assign {
                    if let Expr::Identifier(ref name) = left.node {
                        if let Some((_, is_mut)) = self.env.lookup_full(name) {
                            if !is_mut {
                                return Err(TypeError {
                                    message: format!("Cannot reassign immutable variable '{}'. Use 'mut {}' instead of 'var {}'.", name, name, name),
                                    expected: None,
                                    found: None,
                                    span: left.span.clone(),
                                });
                            }
                        }
                    }
                }
                
                if !self.types_match(&left_ty, &right_ty) {
                    return Err(TypeError {
                        message: format!("Type mismatch in binary operation {:?}", op),
                        expected: Some(left_ty),
                        found: Some(right_ty),
                        span: expr.span.clone(),
                    });
                }
                
                match op {
                    BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessEq | BinaryOp::GreaterEq => {
                        Ok(Type::Named("bool".to_string(), Vec::new()))
                    },
                    _ => Ok(left_ty)
                }
            }
            Expr::New(class_name, _, args) => {
                // Should check if class exists and verify constructor arguments
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(Type::Named(class_name.clone(), Vec::new()))
            },
            Expr::PropertyAccess(target, _field) | Expr::SafePropertyAccess(target, _field) => {
                let _target_ty = self.check_expr(target)?;
                // Mock return type for MVP: Any
                Ok(Type::Named("Any".to_string(), Vec::new()))
            },
            Expr::PropertyAssign(target, _field, value) => {
                if let Expr::Identifier(ref name) = target.node {
                    if let Some((_, is_mut)) = self.env.lookup_full(name) {
                        if !is_mut {
                            return Err(TypeError {
                                message: format!("Cannot mutate property of immutable variable '{}'. Use 'mut {}' to allow mutation.", name, name),
                                expected: None,
                                found: None,
                                span: target.span.clone(),
                            });
                        }
                    }
                }
                let _target_ty = self.check_expr(target)?;
                let value_ty = self.check_expr(value)?;
                Ok(value_ty)
            },
            Expr::NullCoalesce(left, right) => {
                let _left_ty = self.check_expr(left)?;
                let right_ty = self.check_expr(right)?;
                Ok(right_ty)
            },
            Expr::Array(items) => {
                if items.is_empty() {
                    return Ok(Type::Array(Box::new(Type::Named("void".to_string(), Vec::new())))); // Default empty array type
                }
                let mut common_ty = None;
                for item in items {
                    let ty = self.check_expr(item)?;
                    if let Some(ref c_ty) = common_ty {
                        if ty != *c_ty && ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError {
                                message: "Array elements must have the same type".to_string(),
                                expected: Some(c_ty.clone()),
                                found: Some(ty),
                                span: expr.span.clone(),
                            });
                        }
                    } else {
                        common_ty = Some(ty);
                    }
                }
                let final_ty = common_ty.unwrap_or(Type::Named("void".to_string(), Vec::new()));
                Ok(Type::Array(Box::new(final_ty)))
            },
            Expr::Map(pairs) => {
                if pairs.is_empty() {
                    return Ok(Type::Map(Box::new(Type::Named("void".to_string(), Vec::new())), Box::new(Type::Named("void".to_string(), Vec::new()))));
                }
                let mut common_key_ty = None;
                let mut common_val_ty = None;
                for (k, v) in pairs {
                    let k_ty = self.check_expr(k)?;
                    let v_ty = self.check_expr(v)?;
                    if let Some(ref c_k) = common_key_ty {
                        if k_ty != *c_k && k_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Map keys must have same type".to_string(), expected: Some(c_k.clone()), found: Some(k_ty), span: k.span.clone() });
                        }
                    } else { common_key_ty = Some(k_ty); }
                    
                    if let Some(ref c_v) = common_val_ty {
                        if v_ty != *c_v && v_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Map values must have same type".to_string(), expected: Some(c_v.clone()), found: Some(v_ty), span: v.span.clone() });
                        }
                    } else { common_val_ty = Some(v_ty); }
                }
                let final_key = common_key_ty.unwrap_or(Type::Named("void".to_string(), Vec::new()));
                let final_val = common_val_ty.unwrap_or(Type::Named("void".to_string(), Vec::new()));
                Ok(Type::Map(Box::new(final_key), Box::new(final_val)))
            },
            Expr::Index(collection, index) => {
                let coll_ty = self.check_expr(collection)?;
                let index_ty = self.check_expr(index)?;
                
                match coll_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Named("int".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Array index must be an int".to_string(), expected: Some(Type::Named("int".to_string(), Vec::new())), found: Some(index_ty), span: index.span.clone() });
                        }
                        Ok(*inner)
                    },
                    Type::Map(key_ty, val_ty) => {
                        if index_ty != *key_ty && index_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Map key type mismatch".to_string(), expected: Some(*key_ty), found: Some(index_ty), span: index.span.clone() });
                        }
                        Ok(*val_ty)
                    },
                    Type::Named(name, _) if name == "String" => {
                        if index_ty != Type::Named("int".to_string(), Vec::new()) {
                            return Err(TypeError { message: "String index must be an int".to_string(), expected: Some(Type::Named("int".to_string(), Vec::new())), found: Some(index_ty), span: index.span.clone() });
                        }
                        Ok(Type::Named("String".to_string(), Vec::new()))
                    },
                    Type::Named(name, _) if name == "Any" => {
                        Ok(Type::Named("Any".to_string(), Vec::new()))
                    },
                    _ => Err(TypeError {
                        message: "Cannot index a non-array/map/string type".to_string(),
                        expected: None,
                        found: Some(coll_ty),
                        span: collection.span.clone(),
                    })
                }
            },
            Expr::IndexAssign(collection, index, value) => {
                if let Expr::Identifier(ref name) = collection.node {
                    if let Some((_, is_mut)) = self.env.lookup_full(name) {
                        if !is_mut {
                            return Err(TypeError {
                                message: format!("Cannot mutate index of immutable variable '{}'. Use 'mut {}' to allow mutation.", name, name),
                                expected: None,
                                found: None,
                                span: collection.span.clone(),
                            });
                        }
                    }
                }
                let coll_ty = self.check_expr(collection)?;
                let index_ty = self.check_expr(index)?;
                let value_ty = self.check_expr(value)?;
                
                match coll_ty {
                    Type::Array(inner) => {
                        if index_ty != Type::Named("int".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Array index must be an int".to_string(), expected: Some(Type::Named("int".to_string(), Vec::new())), found: Some(index_ty), span: index.span.clone() });
                        }
                        if value_ty != *inner && value_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Type mismatch in array assignment".to_string(), expected: Some(*inner), found: Some(value_ty), span: value.span.clone() });
                        }
                        Ok(value_ty)
                    },
                    Type::Map(key_ty, val_ty) => {
                        if index_ty != *key_ty && index_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Map key type mismatch".to_string(), expected: Some(*key_ty), found: Some(index_ty), span: index.span.clone() });
                        }
                        if value_ty != *val_ty && value_ty != Type::Named("null".to_string(), Vec::new()) {
                            return Err(TypeError { message: "Map value type mismatch".to_string(), expected: Some(*val_ty), found: Some(value_ty), span: value.span.clone() });
                        }
                        Ok(value_ty)
                    },
                    Type::Named(name, _) if name == "Any" => {
                        Ok(value_ty)
                    },
                    _ => Err(TypeError {
                        message: "Cannot assign to index of non-array/map type".to_string(),
                        expected: None,
                        found: Some(coll_ty),
                        span: collection.span.clone(),
                    })
                }
            },
            Expr::Try(inner) => {
                let inner_ty = self.check_expr(inner)?;
                match inner_ty {
                    Type::Result(t) => Ok(*t),
                    Type::Option(t) => Ok(*t),
                    _ => {
                        Err(TypeError {
                            message: "Cannot use '?' on a type that is not a Result or Option".to_string(),
                            expected: None,
                            found: Some(inner_ty),
                            span: expr.span.clone(),
                        })
                    }
                }
            },
            Expr::UnwrapOrElse(inner, block) => {
                let inner_ty = self.check_expr(inner)?;
                self.check_stmt(block)?; // we assume block returns or panics for MVP
                match inner_ty {
                    Type::Result(t) => Ok(*t),
                    Type::Option(t) => Ok(*t),
                    _ => {
                        Err(TypeError {
                            message: "Cannot use 'or' on a type that is not a Result or Option".to_string(),
                            expected: None,
                            found: Some(inner_ty),
                            span: expr.span.clone(),
                        })
                    }
                }
            },
            Expr::This => {
                if let Some(class_name) = &self.current_class_name {
                    Ok(Type::Named(class_name.clone(), Vec::new()))
                } else {
                    Ok(Type::Named("this".to_string(), Vec::new()))
                }
            },
            Expr::Super => Ok(Type::Named("super".to_string(), Vec::new())),
            Expr::Match(target, arms) => {
                let target_ty = self.check_expr(target)?;
                
                if arms.is_empty() {
                    return Err(TypeError {
                        message: "Match expression must have at least one arm".to_string(),
                        expected: None,
                        found: None,
                        span: expr.span.clone(),
                    });
                }
                
                let mut has_catch_all = false;
                let mut common_ret_ty: Option<Type> = None;
                
                for (pat, arm_expr) in arms {
                    // MVP: We do not check pattern exhaustiveness rigorously unless it's a catch-all.
                    // If pattern is Identifier and it's not "_", it could be a variable binding, but we haven't scoped it.
                    // Actually, if it's Identifier, we just treat it as a binding (but we won't add to scope yet to keep it simple, or we can).
                    if let ast::MatchPattern::CatchAll = pat {
                        has_catch_all = true;
                    }
                    
                    let arm_ty = self.check_expr(arm_expr)?;
                    if let Some(ref c_ty) = common_ret_ty {
                        if *c_ty != arm_ty {
                            return Err(TypeError {
                                message: format!("Match arms have incompatible return types. Expected {:?}, found {:?}", c_ty, arm_ty),
                                expected: Some(c_ty.clone()),
                                found: Some(arm_ty),
                                span: arm_expr.span.clone(),
                            });
                        }
                    } else {
                        common_ret_ty = Some(arm_ty);
                    }
                }
                
                if !has_catch_all {
                    return Err(TypeError {
                        message: "Match expression must have a catch-all `_ => ...` arm to guarantee exhaustiveness".to_string(),
                        expected: None,
                        found: None,
                        span: expr.span.clone(),
                    });
                }
                
                Ok(common_ret_ty.unwrap())
            }
        }
    }
}
