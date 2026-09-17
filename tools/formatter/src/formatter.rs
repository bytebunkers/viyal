use ast::{Program, Decl, Stmt, Expr, Type, Literal, BinaryOp, Method};

pub struct Formatter {
    output: String,
    indent_level: usize,
    needs_indent: bool,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter {
            output: String::new(),
            indent_level: 0,
            needs_indent: true,
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    fn push(&mut self, s: &str) {
        if self.needs_indent {
            self.output.push_str(&"    ".repeat(self.indent_level));
            self.needs_indent = false;
        }
        self.output.push_str(s);
    }

    fn push_line(&mut self, s: &str) {
        self.push(s);
        self.output.push('\n');
        self.needs_indent = true;
    }

    fn newline(&mut self) {
        self.output.push('\n');
        self.needs_indent = true;
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    pub fn format_program(&mut self, program: &Program) {
        for (i, decl) in program.declarations.iter().enumerate() {
            self.format_decl(&decl.node);
            if i < program.declarations.len() - 1 {
                self.newline();
                self.newline();
            }
        }
    }

    fn format_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(method) => {
                self.format_method(method);
            }
            Decl::Class { name, extends_class, implements_interfaces: _, fields, primary_constructor, methods } => {
                self.push(&format!("class {}", name));
                
                if !primary_constructor.is_empty() {
                    self.push("(");
                    for (i, param) in primary_constructor.iter().enumerate() {
                        self.format_type(&param.param_type);
                        self.push(&format!(" {}", param.name));
                        if i < primary_constructor.len() - 1 {
                            self.push(", ");
                        }
                    }
                    self.push(")");
                }
                
                if let Some(ext) = extends_class {
                    self.push(&format!(" extends {}", ext));
                }
                
                self.push_line(" {");
                self.indent();
                for field in fields {
                    if field.is_final {
                        self.push("final ");
                    }
                    self.format_type(&field.field_type);
                    self.push_line(&format!(" {};", field.name));
                }
                if !fields.is_empty() && !methods.is_empty() {
                    self.newline();
                }
                for (i, method) in methods.iter().enumerate() {
                    self.format_method(method);
                    if i < methods.len() - 1 {
                        self.newline();
                        self.newline();
                    }
                }
                self.dedent();
                self.push_line("}");
            }
            Decl::TypeAlias { name, target_type } => {
                self.push(&format!("type {} = ", name));
                self.format_type(target_type);
                self.push_line(";");
            }
        }
    }

    fn format_method(&mut self, method: &Method) {
        if let Some(ret_type) = &method.return_type {
            self.format_type(ret_type);
            self.push(" ");
        }
        self.push(&method.name);
        self.push("(");
        for (i, param) in method.params.iter().enumerate() {
            self.format_type(&param.param_type);
            self.push(&format!(" {}", param.name));
            if i < method.params.len() - 1 {
                self.push(", ");
            }
        }
        self.push(") ");
        if let Stmt::Block(_) = &method.body.node {
            self.format_stmt(&method.body.node);
        } else {
            // Should be a block, but just in case
            self.push_line("{");
            self.indent();
            self.format_stmt(&method.body.node);
            self.dedent();
            self.push_line("}");
        }
    }

    fn format_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.format_expr(&expr.node);
                self.push_line(";");
            }
            Stmt::VarDecl { is_final, type_annot, name, initializer } => {
                if *is_final {
                    self.push("final ");
                } else if type_annot.is_none() {
                    self.push("var ");
                }
                
                if let Some(t) = type_annot {
                    self.format_type(t);
                    self.push(" ");
                }
                
                self.push(name);
                if let Some(expr) = initializer {
                    self.push(" = ");
                    self.format_expr(&expr.node);
                }
                self.push_line(";");
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.push("if (");
                self.format_expr(&condition.node);
                self.push(") ");
                self.format_stmt_as_block(&then_branch.node);
                if let Some(else_b) = else_branch {
                    self.output.pop(); // remove newline from block
                    self.needs_indent = false;
                    self.push(" else ");
                    self.format_stmt_as_block(&else_b.node);
                }
            }
            Stmt::While { condition, body } => {
                self.push("while (");
                self.format_expr(&condition.node);
                self.push(") ");
                self.format_stmt_as_block(&body.node);
            }
            Stmt::ForIn { item_name, iterable, body } => {
                self.push("for ");
                self.push(item_name);
                self.push(" in ");
                self.format_expr(&iterable.node);
                self.push(" ");
                self.format_stmt_as_block(&body.node);
            }
            Stmt::ForRange { item_name, start, end, body } => {
                self.push("for ");
                self.push(item_name);
                self.push(" in ");
                self.format_expr(&start.node);
                self.push("..");
                self.format_expr(&end.node);
                self.push(" ");
                self.format_stmt_as_block(&body.node);
            }
            Stmt::Return(expr) => {
                self.push("return");
                if let Some(e) = expr {
                    self.push(" ");
                    self.format_expr(&e.node);
                }
                self.push_line(";");
            }
            Stmt::Block(stmts) => {
                self.push_line("{");
                self.indent();
                for stmt in stmts {
                    self.format_stmt(&stmt.node);
                }
                self.dedent();
                self.push_line("}");
            }
        }
    }

    fn format_stmt_as_block(&mut self, stmt: &Stmt) {
        if let Stmt::Block(_) = stmt {
            self.format_stmt(stmt);
        } else {
            self.push_line("{");
            self.indent();
            self.format_stmt(stmt);
            self.dedent();
            self.push_line("}");
        }
    }

    fn format_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Integer(i) => self.push(&i.to_string()),
                    Literal::Float(f) => self.push(&f.to_string()),
                    Literal::String(s) => self.push(&format!("\"{}\"", s)),
                    Literal::Boolean(b) => self.push(&b.to_string()),
                    Literal::Null => self.push("null"),
                }
            }
            Expr::Identifier(ident) => {
                self.push(ident);
            }
            Expr::Binary(left, operator, right) => {
                self.format_expr(&left.node);
                self.push(" ");
                self.format_binary_op(operator);
                self.push(" ");
                self.format_expr(&right.node);
            }
            Expr::Call(callee, arguments) => {
                self.format_expr(&callee.node);
                self.push("(");
                for (i, arg) in arguments.iter().enumerate() {
                    self.format_expr(&arg.node);
                    if i < arguments.len() - 1 {
                        self.push(", ");
                    }
                }
                self.push(")");
            }
            Expr::PropertyAccess(object, field) => {
                self.format_expr(&object.node);
                self.push(&format!(".{}", field));
            }
            Expr::SafePropertyAccess(object, field) => {
                self.format_expr(&object.node);
                self.push(&format!("?.{}", field));
            }
            Expr::NullCoalesce(left, right) => {
                self.format_expr(&left.node);
                self.push(" ?? ");
                self.format_expr(&right.node);
            }
            Expr::New(class_name, arguments) => {
                self.push(&format!("new {}(", class_name));
                for (i, arg) in arguments.iter().enumerate() {
                    self.format_expr(&arg.node);
                    if i < arguments.len() - 1 {
                        self.push(", ");
                    }
                }
                self.push(")");
            }
            Expr::PropertyAssign(object, field, value) => {
                self.format_expr(&object.node);
                self.push(&format!(".{} = ", field));
                self.format_expr(&value.node);
            }
            Expr::Array(items) => {
                self.push("[");
                for (i, item) in items.iter().enumerate() {
                    self.format_expr(&item.node);
                    if i < items.len() - 1 {
                        self.push(", ");
                    }
                }
                self.push("]");
            }
            Expr::Index(array, index) => {
                self.format_expr(&array.node);
                self.push("[");
                self.format_expr(&index.node);
                self.push("]");
            }
            Expr::IndexAssign(array, index, value) => {
                self.format_expr(&array.node);
                self.push("[");
                self.format_expr(&index.node);
                self.push("] = ");
                self.format_expr(&value.node);
            }
            Expr::Map(pairs) => {
                self.push("{");
                for (i, (key, value)) in pairs.iter().enumerate() {
                    self.format_expr(&key.node);
                    self.push(": ");
                    self.format_expr(&value.node);
                    if i < pairs.len() - 1 {
                        self.push(", ");
                    }
                }
                self.push("}");
            }
            Expr::This => self.push("this"),
            Expr::Super => self.push("super"),
            Expr::Try(inner) => {
                self.format_expr(&inner.node);
                self.push("?");
            },
            Expr::UnwrapOrElse(inner, block) => {
                self.format_expr(&inner.node);
                self.push(" or ");
                self.format_stmt_as_block(&block.node);
            }
        }
    }

    fn format_type(&mut self, ty: &Type) {
        match ty {
            Type::Named(name) => self.push(name),
            Type::Array(inner) => {
                self.format_type(inner);
                self.push("[]");
            }
            Type::Map(key_ty, val_ty) => {
                self.push("Map<");
                self.format_type(key_ty);
                self.push(", ");
                self.format_type(val_ty);
                self.push(">");
            }
            Type::Nullable(inner) => {
                self.format_type(inner);
                self.push("?");
            }
            Type::Result(inner) => {
                self.format_type(inner);
                self.push("!");
            }
            Type::Option(inner) => {
                self.format_type(inner);
                self.push("?");
            }
        }
    }

    fn format_binary_op(&mut self, op: &BinaryOp) {
        match op {
            BinaryOp::Add => self.push("+"),
            BinaryOp::Sub => self.push("-"),
            BinaryOp::Mul => self.push("*"),
            BinaryOp::Div => self.push("/"),
            BinaryOp::Eq => self.push("=="),
            BinaryOp::NotEq => self.push("!="),
            BinaryOp::Less => self.push("<"),
            BinaryOp::LessEq => self.push("<="),
            BinaryOp::Greater => self.push(">"),
            BinaryOp::GreaterEq => self.push(">="),
            BinaryOp::Assign => self.push("="),
        }
    }
}
