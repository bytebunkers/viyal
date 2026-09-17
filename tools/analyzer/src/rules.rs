use ast::{Decl, Stmt, Expr};
use crate::AnalyzerWarning;

pub trait AnalyzerRule {
    fn check_decl(&self, _decl: &Decl) -> Option<AnalyzerWarning> { None }
    fn check_stmt(&self, _stmt: &Stmt) -> Option<AnalyzerWarning> { None }
    fn check_expr(&self, _expr: &Expr) -> Option<AnalyzerWarning> { None }
}

pub struct EmptyBlockRule;

impl AnalyzerRule for EmptyBlockRule {
    fn check_stmt(&self, stmt: &Stmt) -> Option<AnalyzerWarning> {
        if let Stmt::Block(stmts) = stmt {
            if stmts.is_empty() {
                // To get the exact span, we'd need Stmt to wrap the block span.
                // Our AST Stmt::Block itself doesn't store a span, the Spanned<Stmt> does.
                // For MVP, we just yield a warning with an empty span (0..0)
                // In a production linter, the visitor would pass Spanned<Stmt>.
                return Some(AnalyzerWarning {
                    message: "Empty block detected. Consider removing it or adding a comment.".to_string(),
                    span: 0..0,
                });
            }
        }
        None
    }
}
