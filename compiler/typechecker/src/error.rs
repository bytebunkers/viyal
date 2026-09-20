use ast::{Span, Type};
use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Diagnostic)]
#[error("Type Error: {message}")]
#[diagnostic(code(viyal::type_error))]
pub struct TypeError {
    pub message: String,
    
    #[help]
    pub expected: Option<Type>,
    
    #[help]
    pub found: Option<Type>,
    
    #[label("here")]
    pub span: Span,
}
