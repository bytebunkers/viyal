use ast::{Decl, Expr, Literal, Program, Stmt, Type};
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
            output: String::from(
                "#include <stdio.h>\n#include <stdint.h>\n#include <stdbool.h>\n#include <stdlib.h>\n\n",
            ),
            indent_level: 0,
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    fn type_to_c(&self, ty: &Type) -> String {
        match ty {
            Type::Named(name, _) => match name.as_str() {
                "int" => "int64_t".to_string(),
                "double" => "double".to_string(),
                "bool" => "bool".to_string(),
                "String" => "const char*".to_string(),
                "void" => "void".to_string(),
                _ => format!("struct {}*", name),
            },
            Type::Array(inner) => format!("{}*", self.type_to_c(inner)),
            _ => "void*".to_string(), // Fallback for unsupported complex types MVP
        }
    }

    fn generate(&mut self, program: &Program) -> Result<(), String> {
        // Forward declare structs
        for decl in &program.declarations {
            if let Decl::Class { name, .. } = &decl.node {
                writeln!(&mut self.output, "struct {};", name).unwrap();
            }
        }
        self.output.push_str("\n");

        for decl in &program.declarations {
            self.visit_decl(&decl.node)?;
        }
        Ok(())
    }

    fn visit_decl(&mut self, decl: &Decl) -> Result<(), String> {
        match decl {
            Decl::Class {
                name,
                fields,
                methods,
                ..
            } => {
                // Generate struct definition
                writeln!(&mut self.output, "struct {} {{", name).unwrap();
                for field in fields {
                    let field_c_type = self.type_to_c(&field.field_type);
                    writeln!(&mut self.output, "    {} {};", field_c_type, field.name).unwrap();
                }

                // Generate function pointers for methods
                for method in methods {
                    if name == "Main" && method.name == "run" {
                        continue;
                    }
                    let ret_type = method
                        .return_type
                        .as_ref()
                        .map_or("void".to_string(), |t| self.type_to_c(t));
                    write!(
                        &mut self.output,
                        "    {} (*{})(struct {}* this",
                        ret_type, method.name, name
                    )
                    .unwrap();
                    for param in &method.params {
                        let param_c_type = self.type_to_c(&param.param_type);
                        write!(&mut self.output, ", {}", param_c_type).unwrap();
                    }
                    self.output.push_str(");\n");
                }
                self.output.push_str("};\n\n");

                for method in methods {
                    let ret_type = method
                        .return_type
                        .as_ref()
                        .map_or("void".to_string(), |t| self.type_to_c(t));

                    if name == "Main" && method.name == "run" {
                        self.output.push_str("int main() {\n");
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.push_indent();
                        self.output.push_str("return 0;\n");
                        self.indent_level -= 1;
                        self.output.push_str("}\n\n");
                    } else {
                        // Generate flattened method name taking `this` pointer
                        write!(
                            &mut self.output,
                            "{} {}_impl_{}(struct {}* this",
                            ret_type, name, method.name, name
                        )
                        .unwrap();
                        for param in &method.params {
                            let param_c_type = self.type_to_c(&param.param_type);
                            write!(&mut self.output, ", {} {}", param_c_type, param.name).unwrap();
                        }
                        self.output.push_str(") {\n");
                        self.indent_level += 1;
                        self.visit_stmt(&method.body.node)?;
                        self.indent_level -= 1;
                        self.output.push_str("}\n\n");
                    }
                }

                // Generate Constructor Allocator (_new)
                if name != "Main" {
                    writeln!(&mut self.output, "struct {0}* {0}_new() {{", name).unwrap();
                    writeln!(
                        &mut self.output,
                        "    struct {0}* obj = calloc(1, sizeof(struct {0}));",
                        name
                    )
                    .unwrap();
                    for method in methods {
                        if method.name == "run" {
                            continue;
                        }
                        writeln!(
                            &mut self.output,
                            "    obj->{1} = {0}_impl_{1};",
                            name, method.name
                        )
                        .unwrap();
                    }
                    writeln!(&mut self.output, "    return obj;\n}}\n").unwrap();
                }
            }
            Decl::Function(method, _) => {
                let ret_type = method
                    .return_type
                    .as_ref()
                    .map_or("void".to_string(), |t| self.type_to_c(t));
                write!(&mut self.output, "{} {}(", ret_type, method.name).unwrap();
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    let param_c_type = self.type_to_c(&param.param_type);
                    write!(&mut self.output, "{} {}", param_c_type, param.name).unwrap();
                }
                self.output.push_str(") {\n");
                self.indent_level += 1;
                self.visit_stmt(&method.body.node)?;
                self.indent_level -= 1;
                self.output.push_str("}\n\n");
            }
            Decl::TypeAlias {
                name, target_type, ..
            } => {
                let c_type = self.type_to_c(target_type);
                writeln!(&mut self.output, "typedef {} {};", c_type, name).unwrap();
            }
            Decl::Import { .. } => {
                // To be implemented
            }
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Block(stmts) => {
                self.push_indent();
                self.output.push_str("{\n");
                self.indent_level += 1;
                for s in stmts {
                    self.visit_stmt(&s.node)?;
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            Stmt::Expr(expr) => {
                self.push_indent();
                self.visit_expr(&expr.node)?;
                self.output.push_str(";\n");
            }
            Stmt::VarDecl {
                type_annot,
                name,
                initializer,
                ..
            } => {
                self.push_indent();
                let mut c_type = "int64_t".to_string(); // default fallback

                if let Some(ty) = type_annot {
                    c_type = self.type_to_c(ty);
                } else if let Some(init) = initializer {
                    match &init.node {
                        Expr::New(class_name, _, _) => c_type = format!("struct {}*", class_name),
                        Expr::Literal(ast::Literal::String(_)) => {
                            c_type = "const char*".to_string()
                        }
                        Expr::Literal(ast::Literal::Float(_)) => c_type = "double".to_string(),
                        Expr::Literal(ast::Literal::Boolean(_)) => c_type = "bool".to_string(),
                        _ => {}
                    }
                }

                write!(&mut self.output, "{} {} = ", c_type, name).unwrap();
                if let Some(init) = initializer {
                    self.visit_expr(&init.node)?;
                } else {
                    self.output.push_str("0");
                }
                self.output.push_str(";\n");
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.push_indent();
                self.output.push_str("if (");
                self.visit_expr(&condition.node)?;
                self.output.push_str(")\n");
                self.visit_stmt(&then_branch.node)?;

                if let Some(else_b) = else_branch {
                    self.push_indent();
                    self.output.push_str("else\n");
                    self.visit_stmt(&else_b.node)?;
                }
            }
            Stmt::While { condition, body } => {
                self.push_indent();
                self.output.push_str("while (");
                self.visit_expr(&condition.node)?;
                self.output.push_str(")\n");
                self.visit_stmt(&body.node)?;
            }
            Stmt::Return(expr_opt) => {
                self.push_indent();
                self.output.push_str("return");
                if let Some(e) = expr_opt {
                    self.output.push_str(" ");
                    self.visit_expr(&e.node)?;
                }
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
            Expr::Literal(Literal::Float(f)) => {
                write!(&mut self.output, "{}", f).unwrap();
            }
            Expr::Literal(Literal::Boolean(b)) => {
                write!(&mut self.output, "{}", if *b { "true" } else { "false" }).unwrap();
            }
            Expr::Literal(Literal::Null) => {
                self.output.push_str("0");
            }
            Expr::Binary(left, op, right) => {
                self.output.push_str("(");
                self.visit_expr(&left.node)?;
                let op_str = match op {
                    ast::BinaryOp::Add => " + ",
                    ast::BinaryOp::Sub => " - ",
                    ast::BinaryOp::Mul => " * ",
                    ast::BinaryOp::Div => " / ",
                    ast::BinaryOp::Eq => " == ",
                    ast::BinaryOp::NotEq => " != ",
                    ast::BinaryOp::Less => " < ",
                    ast::BinaryOp::Greater => " > ",
                    ast::BinaryOp::LessEq => " <= ",
                    ast::BinaryOp::GreaterEq => " >= ",
                    ast::BinaryOp::Assign => " = ",
                };
                self.output.push_str(op_str);
                self.visit_expr(&right.node)?;
                self.output.push_str(")");
            }
            Expr::New(class_name, _, args) => {
                self.output.push_str(&format!("{}_new(", class_name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.visit_expr(&arg.node)?;
                }
                self.output.push_str(")");
            }
            Expr::This => {
                self.output.push_str("this");
            }
            Expr::PropertyAccess(object, property) => {
                self.output.push_str("(");
                self.visit_expr(&object.node)?;
                self.output.push_str(")->");
                self.output.push_str(property);
            }
            Expr::PropertyAssign(object, property, value) => {
                self.output.push_str("(");
                self.visit_expr(&object.node)?;
                self.output.push_str(")->");
                self.output.push_str(property);
                self.output.push_str(" = ");
                self.visit_expr(&value.node)?;
            }
            Expr::Call(callee, _, args) => {
                if let Expr::PropertyAccess(object, property) = &callee.node {
                    self.output.push_str("(");
                    self.visit_expr(&object.node)?;
                    self.output.push_str(")->");
                    self.output.push_str(property);
                    self.output.push_str("(");
                    self.visit_expr(&object.node)?;
                    if !args.is_empty() {
                        self.output.push_str(", ");
                    }
                } else if let Expr::Identifier(id) = &callee.node {
                    if id == "print" {
                        self.output.push_str("printf(");
                        if let Some(arg) = args.first() {
                            if let Expr::Literal(Literal::String(s)) = &arg.node {
                                write!(&mut self.output, "\"{}\\n\"", s).unwrap();
                            } else {
                                self.output.push_str("\"%ld\\n\", ");
                                self.visit_expr(&arg.node)?;
                            }
                        }
                        self.output.push_str(")");
                        return Ok(());
                    }
                    self.output.push_str(id);
                    self.output.push_str("(");
                } else {
                    self.visit_expr(&callee.node)?;
                    self.output.push_str("(");
                }

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
