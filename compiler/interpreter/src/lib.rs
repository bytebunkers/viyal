pub mod value;
pub mod environment;

use ast::*;
use environment::Environment;
use std::fmt;
use value::Value;

#[derive(Debug)]
pub struct RuntimeError(pub String);

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for RuntimeError {}

pub struct Interpreter<'a> {
    pub env: Environment,
    output: &'a mut dyn std::fmt::Write,
}

impl<'a> Interpreter<'a> {
    pub fn new(output: &'a mut dyn std::fmt::Write) -> Self {
        Self {
            env: Environment::new(),
            output,
        }
    }

    pub fn interpret(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for decl in &program.declarations {
            self.execute_decl(&decl.node)?;
        }
        Ok(())
    }

    fn execute_decl(&mut self, decl: &Decl) -> Result<(), RuntimeError> {
        match decl {
            Decl::Class { methods, .. } => {
                // For MVP, we'll just automatically execute methods in the class to simulate execution.
                for method in methods {
                    self.execute_stmt(&method.body.node)?;
                }
            },
            Decl::Function(method) => {
                self.execute_stmt(&method.body.node)?;
            }
            Decl::TypeAlias { .. } => {
                // No runtime effect for aliases in the simple tree-walk interpreter
            }
        }
        Ok(())
    }

    fn execute_stmt(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expr(expr) => {
                self.evaluate(&expr.node)?;
            },
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.execute_stmt(&s.node)?;
                }
            },
            Stmt::VarDecl { .. } => {
                // Not fully implemented for MVP
            },
            _ => return Err(RuntimeError("Unsupported statement".to_string())),
        }
        Ok(())
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::String(s) => Ok(Value::String(s.clone())),
                    Literal::Integer(i) => Ok(Value::Integer(*i)),
                    Literal::Float(f) => Ok(Value::Float(*f)),
                    Literal::Boolean(b) => Ok(Value::Boolean(*b)),
                    Literal::Null => Ok(Value::Null),
                }
            },
            Expr::Call(target, args) => {
                // Hardcode built-in 'print' function
                if let Expr::Identifier(ref id) = target.node {
                    if id == "print" {
                        if let Some(arg) = args.first() {
                            let val = self.evaluate(&arg.node)?;
                            writeln!(self.output, "{}", val)
                                .map_err(|_| RuntimeError("IO Error".to_string()))?;
                        }
                        return Ok(Value::Void);
                    }
                }
                Err(RuntimeError("Unsupported call".to_string()))
            },
            Expr::Identifier(name) => {
                if let Some(val) = self.env.get(name) {
                    Ok(val)
                } else {
                    Err(RuntimeError(format!("Undefined variable '{}'", name)))
                }
            },
            Expr::New(_class_name, _args) => {
                // Mock object creation
                Ok(Value::Void)
            },
            Expr::PropertyAccess(_target, _field) | Expr::SafePropertyAccess(_target, _field) => {
                Ok(Value::Void)
            },
            Expr::NullCoalesce(left, right) => {
                let left_val = self.evaluate(&left.node)?;
                if left_val == Value::Null {
                    self.evaluate(&right.node)
                } else {
                    Ok(left_val)
                }
            },
            Expr::This | Expr::Super => Ok(Value::Void),
            _ => Err(RuntimeError("Unsupported expression".to_string())),
        }
    }
}
