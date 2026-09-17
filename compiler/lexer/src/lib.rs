pub mod token;

use logos::Logos;
use token::Token;
use std::ops::Range;

/// Represents a lexed token along with its span in the source code.
#[derive(Debug, PartialEq, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Range<usize>,
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<SpannedToken, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|token_res| {
            match token_res {
                Ok(token) => Ok(SpannedToken {
                    token,
                    span: self.inner.span(),
                }),
                Err(_) => Err(()), // Tokenizing error
            }
        })
    }
}
