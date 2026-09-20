use ast::Span;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Diagnostic)]
#[error("Parse Error: {message}")]
#[diagnostic(code(viyal::syntax_error))]
pub struct ParseError {
    pub message: String,
    
    #[label("here")]
    pub span: Span,
}
