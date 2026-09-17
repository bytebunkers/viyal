use ast::{Program, Decl, Stmt, Expr, Literal, Type};
use std::fmt::Write;

pub fn generate_c(program: &Program) -> Result<String, String> {
    let mut generator = CGenerator::new();
    generator.generate(program)?;
    Ok(generator.output)
}

struct CGenerator {
    output: String,
    indent_level: usize,
}

impl CGenerator {
    fn new() -> Self {
        Self {
            output: String::from("#include <stdio.h>\n#include <stdint.h>\n\n"),
            indent_level: 0,
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn generate(&mut self, program: &Program) -> Result<(), String> {
        for decl in &program.declarations {
            self.visit_decl(&decl.node)?;
        }
        Ok(())
    }

    fn visit_decl(&mut self, decl: &Decl) -> Result<(), String> {
        match decl {
            Decl::Class { name, methods, .. } => {
                // For MVP, we don't fully generate structs, just top level functions.
                // Especially handling the Main class.
                for method in methods {
                    // Standard C int main() for entrypoint
                    if name == "Main" && method.name == "run" {
                        self.output.push_str("int main() {\n");
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.push_indent();
                        self.output.push_str("return 0;\n");
                        self.indent_level -= 1;
                        self.output.push_str("}\n");
                    } else {
                        // MVP: Just emit as standalone C function
                        let ret_type = match &method.return_type {
                            Some(Type::Named(n)) if n == "void" => "void",
                            Some(Type::Named(n)) if n == "int" => "int64_t",
                            Some(Type::Named(n)) if n == "String" => "const char*",
                            _ => "void",
                        };
                        writeln!(&mut self.output, "{} {}() {{", ret_type, method.name).unwrap();
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.indent_level -= 1;
                        self.output.push_str("}\n");
                    }
                }
            }
            Decl::Function(method) => {
                // MVP
                writeln!(&mut self.output, "void {}() {{", method.name).unwrap();
                self.indent_level += 1;
                self.visit_stmt(&method.body.node)?;
                self.indent_level -= 1;
                self.output.push_str("}\n");
            }
            Decl::TypeAlias { name, target_type } => {
                // C typedef representation
                let c_type = match target_type {
                    Type::Named(n) if n == "int" => "int64_t",
                    Type::Named(n) if n == "double" => "double",
                    Type::Named(n) if n == "bool" => "bool",
                    Type::Named(n) if n == "String" => "const char*",
                    _ => "void*",
                };
                writeln!(&mut self.output, "typedef {} {};", c_type, name).unwrap();
            }
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.visit_stmt(&s.node)?;
                }
            }
            Stmt::Expr(expr) => {
                self.push_indent();
                self.visit_expr(&expr.node)?;
                self.output.push_str(";\n");
            }
            _ => {
                self.push_indent();
                self.output.push_str("// Unimplemented Statement\n");
            }
        }
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Literal(Literal::String(s)) => {
                write!(&mut self.output, "\"{}\"", s).unwrap();
            }
            Expr::Literal(Literal::Integer(i)) => {
                write!(&mut self.output, "{}", i).unwrap();
            }
            Expr::Call(callee, args) => {
                if let Expr::Identifier(id) = &callee.node {
                    if id == "print" {
                        // MVP string printing
                        self.output.push_str("printf(");
                        if let Some(arg) = args.first() {
                            if let Expr::Literal(Literal::String(s)) = &arg.node {
                                write!(&mut self.output, "\"{}\\n\"", s).unwrap();
                            } else {
                                self.output.push_str("\"%d\\n\", ");
                                self.visit_expr(&arg.node)?;
                            }
                        }
                        self.output.push_str(")");
                        return Ok(());
                    }
                    self.output.push_str(id);
                } else {
                    self.visit_expr(&callee.node)?;
                }
                
                self.output.push_str("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.visit_expr(&arg.node)?;
                }
                self.output.push_str(")");
            }
            Expr::Identifier(id) => {
                self.output.push_str(id);
            }
            _ => {
                self.output.push_str("/* Unimplemented Expr */");
            }
        }
        Ok(())
    }
}
