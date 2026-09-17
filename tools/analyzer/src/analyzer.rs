use ast::{Program, Decl, Stmt, Expr};
use crate::AnalyzerWarning;
use crate::rules::{AnalyzerRule, EmptyBlockRule};

pub struct Analyzer {
    rules: Vec<Box<dyn AnalyzerRule>>,
    warnings: Vec<AnalyzerWarning>,
}

impl Analyzer {
    pub fn new() -> Self {
        let mut analyzer = Analyzer {
            rules: Vec::new(),
            warnings: Vec::new(),
        };
        // Register default rules
        analyzer.rules.push(Box::new(EmptyBlockRule));
        analyzer
    }

    pub fn analyze_program(&mut self, program: &Program) -> Vec<AnalyzerWarning> {
        self.warnings.clear();
        for decl in &program.declarations {
            self.visit_decl(&decl.node);
        }
        self.warnings.clone()
    }

    fn visit_decl(&mut self, decl: &Decl) {
        for rule in &self.rules {
            if let Some(warning) = rule.check_decl(decl) {
                self.warnings.push(warning);
            }
        }
        
        match decl {
            Decl::Function(method) => {
                self.visit_stmt(&method.body.node);
            }
            Decl::Class { methods, .. } => {
                for method in methods {
                    self.visit_stmt(&method.body.node);
                }
            }
            Decl::TypeAlias { .. } => {
                // Currently no expressions inside type aliases to analyze
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        for rule in &self.rules {
            if let Some(warning) = rule.check_stmt(stmt) {
                self.warnings.push(warning);
            }
        }

        match stmt {
            Stmt::Expr(expr) => self.visit_expr(&expr.node),
            Stmt::VarDecl { initializer, .. } => {
                if let Some(init) = initializer {
                    self.visit_expr(&init.node);
                }
            }
            Stmt::Block(stmts) => {
                for s in stmts {
                    self.visit_stmt(&s.node);
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.visit_expr(&condition.node);
                self.visit_stmt(&then_branch.node);
                if let Some(e) = else_branch {
                    self.visit_stmt(&e.node);
                }
            }
            Stmt::While { condition, body } => {
                self.visit_expr(&condition.node);
                self.visit_stmt(&body.node);
            }
            Stmt::ForIn { iterable, body, .. } => {
                self.visit_expr(&iterable.node);
                self.visit_stmt(&body.node);
            }
            Stmt::ForRange { start, end, body, .. } => {
                self.visit_expr(&start.node);
                self.visit_expr(&end.node);
                self.visit_stmt(&body.node);
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.visit_expr(&e.node);
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        for rule in &self.rules {
            if let Some(warning) = rule.check_expr(expr) {
                self.warnings.push(warning);
            }
        }

        match expr {
            Expr::Literal(_) | Expr::Identifier(_) | Expr::This | Expr::Super => {}
            Expr::Binary(left, _, right) => {
                self.visit_expr(&left.node);
                self.visit_expr(&right.node);
            }
            Expr::Call(callee, args) => {
                self.visit_expr(&callee.node);
                for arg in args {
                    self.visit_expr(&arg.node);
                }
            }
            Expr::PropertyAccess(obj, _) | Expr::SafePropertyAccess(obj, _) => {
                self.visit_expr(&obj.node);
            }
            Expr::NullCoalesce(left, right) => {
                self.visit_expr(&left.node);
                self.visit_expr(&right.node);
            }
            Expr::New(_, args) => {
                for arg in args {
                    self.visit_expr(&arg.node);
                }
            }
            Expr::PropertyAssign(obj, _, val) => {
                self.visit_expr(&obj.node);
                self.visit_expr(&val.node);
            }
            Expr::Array(items) => {
                for item in items {
                    self.visit_expr(&item.node);
                }
            }
            Expr::Index(array, index) => {
                self.visit_expr(&array.node);
                self.visit_expr(&index.node);
            }
            Expr::IndexAssign(array, index, val) => {
                self.visit_expr(&array.node);
                self.visit_expr(&index.node);
                self.visit_expr(&val.node);
            }
            Expr::Map(pairs) => {
                for (key, val) in pairs {
                    self.visit_expr(&key.node);
                    self.visit_expr(&val.node);
                }
            }
            Expr::Try(inner) => {
                self.visit_expr(&inner.node);
            }
            Expr::UnwrapOrElse(inner, block) => {
                self.visit_expr(&inner.node);
                self.visit_stmt(&block.node);
            }
        }
    }
}
