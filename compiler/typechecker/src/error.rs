use ast::{Span, Type};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct TypeError {
    pub message: String,
    pub expected: Option<Type>,
    pub found: Option<Type>,
    pub span: Span,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Type Error: {} at {}..{}", self.message, self.span.start, self.span.end)?;
        if let Some(exp) = &self.expected {
            write!(f, ", Expected: {:?}", exp)?;
        }
        if let Some(fnd) = &self.found {
            write!(f, ", Found: {:?}", fnd)?;
        }
        Ok(())
    }
}
impl std::error::Error for TypeError {}
