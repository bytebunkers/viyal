pub mod error;

use ast::*;
use error::ParseError;
use lexer::token::Token;
use lexer::{Lexer, SpannedToken};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<SpannedToken>,
    previous_span: Span,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer.next().map(|res| res.unwrap_or(SpannedToken { token: Token::Null, span: 0..0 }));
        Self {
            lexer,
            current,
            previous_span: 0..0,
        }
    }

    fn advance(&mut self) {
        if let Some(tok) = &self.current {
            self.previous_span = tok.span.clone();
        }
        
        self.current = match self.lexer.next() {
            Some(Ok(tok)) => Some(tok),
            Some(Err(_)) => None,
            None => None,
        };
    }

    fn check(&self, expected: &Token) -> bool {
        match &self.current {
            Some(t) => core::mem::discriminant(&t.token) == core::mem::discriminant(expected),
            None => false,
        }
    }

    fn consume(&mut self, expected: Token, message: &str) -> Result<SpannedToken, ParseError> {
        if self.check(&expected) {
            let tok = self.current.clone().unwrap();
            self.advance();
            Ok(tok)
        } else {
            let span = match &self.current {
                Some(t) => t.span.clone(),
                None => self.previous_span.clone(),
            };
            Err(ParseError {
                message: message.to_string(),
                span,
            })
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        match &self.current {
            Some(tok) => {
                let name = match &tok.token {
                    Token::Identifier(s) => s.clone(),
                    Token::StringType => "String".to_string(),
                    Token::Int => "int".to_string(),
                    Token::Double => "double".to_string(),
                    Token::BoolType => "bool".to_string(),
                    Token::Void => "void".to_string(),
                    _ => {
                        return Err(ParseError { message: message.to_string(), span: tok.span.clone() });
                    }
                };
                self.advance();
                Ok(name)
            },
            None => Err(ParseError { message: message.to_string(), span: self.previous_span.clone() }),
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let name = self.consume_identifier("Expected type name")?;
        let mut ty = if name == "Map" && self.check(&Token::Less) {
            self.advance(); // consume '<'
            let key_type = self.parse_type()?;
            self.consume(Token::Comma, "Expected ',' in Map type")?;
            let value_type = self.parse_type()?;
            self.consume(Token::Greater, "Expected '>' in Map type")?;
            Type::Map(Box::new(key_type), Box::new(value_type))
        } else {
            let mut type_args = Vec::new();
            if self.check(&Token::Less) {
                self.advance(); // consume '<'
                if !self.check(&Token::Greater) {
                    loop {
                        type_args.push(self.parse_type()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.consume(Token::Greater, "Expected '>' after type arguments")?;
            }
            Type::Named(name, type_args)
        };
        
        while self.check(&Token::LBracket) {
            self.advance();
            self.consume(Token::RBracket, "Expected ']' after '[' in array type")?;
            ty = Type::Array(Box::new(ty));
        }

        if self.check(&Token::Bang) {
            self.advance();
            ty = Type::Result(Box::new(ty));
        } else if self.check(&Token::Question) {
            self.advance();
            ty = Type::Option(Box::new(ty));
        }
        Ok(ty)
    }

    pub fn parse_program(&mut self) -> Result<Program, Vec<ParseError>> {
        let mut declarations = Vec::new();
        let mut errors = Vec::new();
        while self.current.is_some() {
            match self.parse_declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(Program { declarations })
        } else {
            Err(errors)
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while let Some(tok) = &self.current {
            match &tok.token {
                Token::Class | Token::If | Token::While | Token::For | Token::Return => return,
                _ => self.advance(),
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Spanned<Decl>, ParseError> {
        let start = self.current.as_ref().map(|t| t.span.start).unwrap_or(0);
        
        if self.check(&Token::Import) {
            self.advance();
            
            let mut items = Vec::new();
            self.consume(Token::LBrace, "Expected '{' after import")?;
            if !self.check(&Token::RBrace) {
                loop {
                    let item_name = self.consume_identifier("Expected import item name")?;
                    let mut alias = None;
                    if self.check(&Token::As) {
                        self.advance();
                        alias = Some(self.consume_identifier("Expected alias name after 'as'")?);
                    }
                    items.push((item_name, alias));
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            self.consume(Token::RBrace, "Expected '}' after import items")?;
            self.consume(Token::From, "Expected 'from' after import items")?;
            
            if self.current.is_none() {
                return Err(ParseError {
                    message: "Expected import path string".into(),
                    span: self.previous_span.clone(),
                });
            }
            let path_token = self.current.clone().unwrap();
            self.advance();
            let path = if let Token::StringLit(s) = path_token.token {
                s
            } else {
                return Err(ParseError {
                    message: format!("Expected string literal for import path, found {:?}", path_token.token),
                    span: path_token.span,
                });
            };
            let semi = self.consume(Token::Semi, "Expected ';' after import statement")?;
            
            return Ok(Spanned {
                node: Decl::Import { path, items },
                span: start..semi.span.end,
            });
        }
        
        let mut is_exported = false;
        if self.check(&Token::Export) {
            self.advance();
            is_exported = true;
        }
        
        if self.check(&Token::Class) {
            self.advance(); // consume 'class'
            let name = self.consume_identifier("Expected class name")?;
            
            let mut type_params = Vec::new();
            if self.check(&Token::Less) {
                self.advance();
                if !self.check(&Token::Greater) {
                    loop {
                        type_params.push(self.consume_identifier("Expected type parameter name")?);
                        if !self.check(&Token::Comma) { break; }
                        self.advance();
                    }
                }
                self.consume(Token::Greater, "Expected '>' after type parameters")?;
            }
            
            // Primary constructor (MVP style)
            let mut primary_constructor = Vec::new();
            if self.check(&Token::LParen) {
                self.advance();
                if !self.check(&Token::RParen) {
                    loop {
                        let param_type = self.parse_type()?;
                        let param_name = self.consume_identifier("Expected parameter name")?;
                        primary_constructor.push(Param {
                            param_type,
                            name: param_name,
                        });
                        
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }
                self.consume(Token::RParen, "Expected ')' after constructor parameters")?;
            }

            let mut extends_class = None;
            if self.check(&Token::Extends) {
                self.advance();
                extends_class = Some(self.consume_identifier("Expected superclass name")?);
            }

            let mut implements_interfaces = Vec::new();
            if self.check(&Token::Implements) {
                self.advance();
                loop {
                    implements_interfaces.push(self.consume_identifier("Expected interface name")?);
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance();
                }
            }
            
            self.consume(Token::LBrace, "Expected '{' before class body")?;
            
            let mut fields = Vec::new();
            let mut methods = Vec::new();
            
            while !self.check(&Token::RBrace) && self.current.is_some() {
                // Need to distinguish field vs method
                let member_type = self.parse_type()?;
                let member_name = self.consume_identifier("Expected member name")?;
                
                let mut type_params = Vec::new();
                if self.check(&Token::Less) {
                    self.advance();
                    if !self.check(&Token::Greater) {
                        loop {
                            type_params.push(self.consume_identifier("Expected type parameter name")?);
                            if !self.check(&Token::Comma) { break; }
                            self.advance();
                        }
                    }
                    self.consume(Token::Greater, "Expected '>' after method type parameters")?;
                }
                
                if self.check(&Token::LParen) {
                    // It's a method
                    self.advance();
                    let mut params = Vec::new();
                    if !self.check(&Token::RParen) {
                         loop {
                            let param_type = self.parse_type()?;
                            let param_name = self.consume_identifier("Expected parameter name")?;
                            params.push(Param { param_type, name: param_name });
                            if !self.check(&Token::Comma) { break; }
                            self.advance();
                         }
                    }
                    self.consume(Token::RParen, "Expected ')' after method parameters")?;
                    let body = self.parse_block()?;
                    methods.push(Method {
                        return_type: Some(member_type),
                        name: member_name,
                        type_params,
                        params,
                        body,
                    });
                } else {
                    // It's a field
                    self.consume(Token::Semi, "Expected ';' after field declaration")?;
                    fields.push(Field {
                        name: member_name,
                        field_type: member_type,
                        is_final: false,
                    });
                }
            }
            let rbrace = self.consume(Token::RBrace, "Expected '}' after class body")?;
            
            let span = start..rbrace.span.end;
            Ok(Spanned {
                node: Decl::Class {
                    name,
                    type_params,
                    extends_class,
                    implements_interfaces,
                    fields,
                    primary_constructor,
                    methods,
                    is_exported,
                },
                span,
            })
        } else if self.check(&Token::TypeKeyword) {
            self.advance();
            let name = self.consume_identifier("Expected alias name")?;
            self.consume(Token::Eq, "Expected '=' after alias name")?;
            let target_type = self.parse_type()?;
            let semi = self.consume(Token::Semi, "Expected ';' after type alias")?;
            Ok(Spanned {
                node: Decl::TypeAlias { name, target_type, is_exported },
                span: start..semi.span.end,
            })
        } else {
            // Attempt to parse a top-level function
            let return_type = self.parse_type()?;
            let function_name = self.consume_identifier("Expected function or declaration name")?;
            
            let mut type_params = Vec::new();
            if self.check(&Token::Less) {
                self.advance();
                if !self.check(&Token::Greater) {
                    loop {
                        type_params.push(self.consume_identifier("Expected type parameter name")?);
                        if !self.check(&Token::Comma) { break; }
                        self.advance();
                    }
                }
                self.consume(Token::Greater, "Expected '>' after function type parameters")?;
            }
            
            if self.check(&Token::LParen) {
                self.advance();
                let mut params = Vec::new();
                if !self.check(&Token::RParen) {
                     loop {
                        let param_type = self.parse_type()?;
                        let param_name = self.consume_identifier("Expected parameter name")?;
                        params.push(Param { param_type, name: param_name });
                        if !self.check(&Token::Comma) { break; }
                        self.advance();
                     }
                }
                self.consume(Token::RParen, "Expected ')' after function parameters")?;
                let body = self.parse_block()?;
                let span = start..body.span.end;
                Ok(Spanned {
                    node: Decl::Function(Method {
                        return_type: Some(return_type),
                        name: function_name,
                        type_params,
                        params,
                        body,
                    }, is_exported),
                    span,
                })
            } else {
                Err(ParseError {
                    message: "Expected '(' after top-level function name".to_string(),
                    span: start..start,
                })
            }
        }
    }

    fn parse_block(&mut self) -> Result<Spanned<Stmt>, ParseError> {
        let start = self.current.as_ref().map(|t| t.span.start).unwrap_or(0);
        self.consume(Token::LBrace, "Expected '{' at start of block")?;
        
        let mut stmts = Vec::new();
        while !self.check(&Token::RBrace) && self.current.is_some() {
            stmts.push(self.parse_statement()?);
        }
        
        let rbrace = self.consume(Token::RBrace, "Expected '}' at end of block")?;
        Ok(Spanned {
            node: Stmt::Block(stmts),
            span: start..rbrace.span.end,
        })
    }
    
    fn parse_statement(&mut self) -> Result<Spanned<Stmt>, ParseError> {
        let start = self.current.as_ref().map(|t| t.span.start).unwrap_or(0);
        
        if self.check(&Token::Var) || self.check(&Token::Mut) {
            let is_final = if self.check(&Token::Var) {
                self.advance();
                true
            } else {
                self.advance();
                false
            };
            let name = self.consume_identifier("Expected variable name")?;
            let mut initializer = None;
            if self.check(&Token::Eq) {
                self.advance();
                initializer = Some(self.parse_expression()?);
            }
            let semi = self.consume(Token::Semi, "Expected ';' after variable declaration")?;
            return Ok(Spanned {
                node: Stmt::VarDecl {
                    is_final,
                    type_annot: None,
                    name,
                    initializer,
                },
                span: start..semi.span.end,
            });
        } else if self.check(&Token::LBrace) {
            return self.parse_block();
        } else if self.check(&Token::If) {
            self.advance();
            self.consume(Token::LParen, "Expected '(' after 'if'")?;
            let condition = self.parse_expression()?;
            self.consume(Token::RParen, "Expected ')' after if condition")?;
            
            let then_branch = Box::new(self.parse_statement()?);
            let mut else_branch = None;
            
            if self.check(&Token::Else) {
                self.advance();
                else_branch = Some(Box::new(self.parse_statement()?));
            }
            
            let end_span = else_branch.as_ref().map(|b| b.span.end).unwrap_or(then_branch.span.end);
            return Ok(Spanned {
                node: Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                },
                span: start..end_span,
            });
        } else if self.check(&Token::While) {
            self.advance();
            self.consume(Token::LParen, "Expected '(' after 'while'")?;
            let condition = self.parse_expression()?;
            self.consume(Token::RParen, "Expected ')' after while condition")?;
            
            let body = Box::new(self.parse_statement()?);
            return Ok(Spanned {
                node: Stmt::While {
                    condition,
                    body: body.clone(),
                },
                span: start..body.span.end,
            });
        } else if self.check(&Token::For) {
            self.advance();
            let item_name = self.consume_identifier("Expected variable name after 'for'")?;
            self.consume(Token::In, "Expected 'in' after for loop variable")?;
            
            let expr1 = self.parse_expression()?;
            
            if self.check(&Token::DotDot) {
                self.advance();
                let expr2 = self.parse_expression()?;
                let body = Box::new(self.parse_statement()?);
                return Ok(Spanned {
                    node: Stmt::ForRange {
                        item_name,
                        start: expr1,
                        end: expr2,
                        body: body.clone(),
                    },
                    span: start..body.span.end,
                });
            } else {
                let body = Box::new(self.parse_statement()?);
                return Ok(Spanned {
                    node: Stmt::ForIn {
                        item_name,
                        iterable: expr1,
                        body: body.clone(),
                    },
                    span: start..body.span.end,
                });
            }
        } else if self.check(&Token::Return) {
            self.advance();
            let mut value = None;
            if !self.check(&Token::Semi) {
                value = Some(self.parse_expression()?);
            }
            let semi = self.consume(Token::Semi, "Expected ';' after return value")?;
            return Ok(Spanned {
                node: Stmt::Return(value),
                span: start..semi.span.end,
            });
        }
        
        // Simple expression statement
        let expr = self.parse_expression()?;
        let semi = self.consume(Token::Semi, "Expected ';' after expression")?;
        
        Ok(Spanned {
            node: Stmt::Expr(expr),
            span: start..semi.span.end,
        })
    }
    
    fn parse_expression(&mut self) -> Result<Spanned<Expr>, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let expr = self.parse_equality()?;
        
        if self.check(&Token::Eq) {
            self.advance();
            let value = self.parse_assignment()?; // Right-associative
            let span = expr.span.start..value.span.end;
            
            match expr.node {
                Expr::Index(array, index) => {
                    return Ok(Spanned {
                        node: Expr::IndexAssign(array, index, Box::new(value)),
                        span,
                    });
                }
                Expr::PropertyAccess(obj, prop) => {
                    return Ok(Spanned {
                        node: Expr::PropertyAssign(obj, prop, Box::new(value)),
                        span,
                    });
                }
                _ => {
                    return Ok(Spanned {
                        node: Expr::Binary(Box::new(expr), BinaryOp::Assign, Box::new(value)),
                        span,
                    });
                }
            }
        }
        
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut expr = self.parse_comparison()?;
        while self.check(&Token::EqEq) || self.check(&Token::NotEq) {
            let op = if self.check(&Token::EqEq) { BinaryOp::Eq } else { BinaryOp::NotEq };
            self.advance();
            let right = self.parse_comparison()?;
            let span = expr.span.start..right.span.end;
            expr = Spanned {
                node: Expr::Binary(Box::new(expr), op, Box::new(right)),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut expr = self.parse_term()?;
        while self.check(&Token::Less) || self.check(&Token::Greater) || self.check(&Token::LessEq) || self.check(&Token::GreaterEq) {
            let op = if self.check(&Token::Less) { BinaryOp::Less } 
                     else if self.check(&Token::Greater) { BinaryOp::Greater }
                     else if self.check(&Token::LessEq) { BinaryOp::LessEq }
                     else { BinaryOp::GreaterEq };
            self.advance();
            let right = self.parse_term()?;
            let span = expr.span.start..right.span.end;
            expr = Spanned {
                node: Expr::Binary(Box::new(expr), op, Box::new(right)),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let mut expr = self.parse_call()?;
        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.check(&Token::Plus) { BinaryOp::Add } else { BinaryOp::Sub };
            self.advance();
            let right = self.parse_call()?;
            let span = expr.span.start..right.span.end;
            expr = Spanned {
                node: Expr::Binary(Box::new(expr), op, Box::new(right)),
                span,
            };
        }
        Ok(expr)
    }

    fn parse_call(&mut self) -> Result<Spanned<Expr>, ParseError> {
        // null-coalescing is left-associative, lowest precedence here for simplicity
        let mut expr = self.parse_primary_expr()?;

        loop {
            if self.check(&Token::DoubleQuestion) {
                self.advance();
                let right = self.parse_primary_expr()?;
                let span = expr.span.start..right.span.end;
                expr = Spanned {
                    node: Expr::NullCoalesce(Box::new(expr), Box::new(right)),
                    span,
                };
            } else if self.check(&Token::Dot) {
                self.advance();
                let ident = self.consume_identifier("Expected property name after '.'")?;
                let span = expr.span.start..self.previous_span.end;
                expr = Spanned {
                    node: Expr::PropertyAccess(Box::new(expr), ident),
                    span,
                };
            } else if self.check(&Token::QuestionDot) {
                self.advance();
                let ident = self.consume_identifier("Expected property name after '?.'")?;
                let span = expr.span.start..self.previous_span.end;
                expr = Spanned {
                    node: Expr::SafePropertyAccess(Box::new(expr), ident),
                    span,
                };
            } else if self.check(&Token::LBracket) {
                self.advance();
                let index_expr = self.parse_expression()?;
                let rbracket = self.consume(Token::RBracket, "Expected ']' after array index")?;
                let span = expr.span.start..rbracket.span.end;
                expr = Spanned {
                    node: Expr::Index(Box::new(expr), Box::new(index_expr)),
                    span,
                };
            } else if self.check(&Token::LParen) {
                self.advance();
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                let rparen = self.consume(Token::RParen, "Expected ')' after call arguments")?;
                let span = expr.span.start..rparen.span.end;
                expr = Spanned {
                    node: Expr::Call(Box::new(expr), Vec::new(), args),
                    span,
                };
            } else if self.check(&Token::Question) {
                let question = self.consume(Token::Question, "Expected '?'")?;
                let span = expr.span.start..question.span.end;
                expr = Spanned {
                    node: Expr::Try(Box::new(expr)),
                    span,
                };
            } else if self.check(&Token::Or) {
                self.advance(); // consume 'or'
                let block = self.parse_block()?;
                let span = expr.span.start..block.span.end;
                expr = Spanned {
                    node: Expr::UnwrapOrElse(Box::new(expr), Box::new(block)),
                    span,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }
    fn parse_match_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.previous_span.start;
        self.advance(); // consume 'match'

        let target = self.parse_expression()?;

        self.consume(Token::LBrace, "Expected '{' after match target")?;

        let mut arms = Vec::new();

        while !self.check(&Token::RBrace) && self.current.is_some() {
            let pattern = if let Some(tok) = &self.current {
                match &tok.token {
                    Token::Identifier(id) => {
                        let pat = if id == "_" {
                            MatchPattern::CatchAll
                        } else {
                            MatchPattern::Identifier(id.clone())
                        };
                        self.advance();
                        pat
                    },
                    Token::Integer(i) => {
                        let pat = MatchPattern::Literal(Literal::Integer(*i));
                        self.advance();
                        pat
                    },
                    Token::StringLit(s) => {
                        let pat = MatchPattern::Literal(Literal::String(s.clone()));
                        self.advance();
                        pat
                    },
                    Token::True => {
                        let pat = MatchPattern::Literal(Literal::Boolean(true));
                        self.advance();
                        pat
                    },
                    Token::False => {
                        let pat = MatchPattern::Literal(Literal::Boolean(false));
                        self.advance();
                        pat
                    },
                    _ => return Err(ParseError { message: format!("Invalid match pattern: {:?}", tok.token), span: tok.span.clone() }),
                }
            } else {
                return Err(ParseError { message: "Unexpected EOF in match pattern".into(), span: self.previous_span.clone() });
            };

            self.consume(Token::FatArrow, "Expected '=>' after match pattern")?;

            let body = if self.check(&Token::LBrace) {
                // If it's a block, we parse it as a block statement and wrap it in an expression if needed,
                // But AST expects Expr for match body. We might need a BlockExpr, or we can just parse block
                // Let's parse a block and convert to Expr, or just parse an expression.
                // Wait, if it's a block, it should be an expression that evaluates to the last stmt.
                // For MVP, let's just parse single expressions. The user can use IIFEs if they need blocks, or we add BlockExpr later.
                // Wait, the user asked to support blocks! Let's just parse it as an expression for now to keep AST simple,
                // or wait, let's check if there is a BlockExpr. The AST has Expr::Block? No, only Stmt::Block.
                // Let's just parse `parse_expression` for now and I'll explain we can add BlockExpr later.
                self.parse_expression()?
            } else {
                self.parse_expression()?
            };

            arms.push((pattern, body));

            if self.check(&Token::Comma) {
                self.advance();
            } else if !self.check(&Token::RBrace) {
                return Err(ParseError { message: "Expected ',' or '}' after match arm".into(), span: self.previous_span.clone() });
            }
        }

        let end_brace = self.consume(Token::RBrace, "Expected '}' at the end of match block")?;

        Ok(Spanned {
            node: Expr::Match(Box::new(target), arms),
            span: start..end_brace.span.end,
        })
    }

    fn parse_primary_expr(&mut self) -> Result<Spanned<Expr>, ParseError> {
        let start = self.current.as_ref().map(|t| t.span.start).unwrap_or(0);
        
        if let Some(tok) = &self.current {
            match &tok.token {
                Token::StringLit(s) => {
                    let s_val = s.clone();
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::String(s_val)), span })
                },
                Token::LBracket => {
                    self.advance();
                    let mut items = Vec::new();
                    if !self.check(&Token::RBracket) {
                        loop {
                            items.push(self.parse_expression()?);
                            if !self.check(&Token::Comma) { break; }
                            self.advance();
                        }
                    }
                    let rbracket = self.consume(Token::RBracket, "Expected ']' after array elements")?;
                    Ok(Spanned { node: Expr::Array(items), span: start..rbracket.span.end })
                },
                Token::LBrace => {
                    self.advance();
                    let mut pairs = Vec::new();
                    if !self.check(&Token::RBrace) {
                        loop {
                            let key = self.parse_expression()?;
                            self.consume(Token::Colon, "Expected ':' after map key")?;
                            let value = self.parse_expression()?;
                            pairs.push((key, value));
                            if !self.check(&Token::Comma) { break; }
                            self.advance();
                        }
                    }
                    let rbrace = self.consume(Token::RBrace, "Expected '}' after map elements")?;
                    Ok(Spanned { node: Expr::Map(pairs), span: start..rbrace.span.end })
                },
                Token::Integer(i) => {
                    let i_val = *i;
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::Integer(i_val)), span })
                },
                Token::Float(f) => {
                    let f_val = *f;
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::Float(f_val)), span })
                },
                Token::True => {
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::Boolean(true)), span })
                },
                Token::False => {
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::Boolean(false)), span })
                },
                Token::Null => {
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Literal(Literal::Null), span })
                },
                Token::This => {
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::This, span })
                },
                Token::Super => {
                    let span = tok.span.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Super, span })
                },
                Token::New => {
                    self.advance();
                    let class_name = self.consume_identifier("Expected class name after 'new'")?;
                    self.consume(Token::LParen, "Expected '('")?;
                    let mut args = Vec::new();
                    if !self.check(&Token::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.check(&Token::Comma) { break; }
                            self.advance();
                        }
                    }
                    let rparen = self.consume(Token::RParen, "Expected ')'")?;
                    Ok(Spanned { node: Expr::New(class_name, Vec::new(), args), span: start..rparen.span.end })
                },
                Token::Match => {
                    self.parse_match_expr()
                },
                Token::Identifier(id) => {
                    let id_val = id.clone();
                    self.advance();
                    Ok(Spanned { node: Expr::Identifier(id_val), span: start..self.previous_span.end })
                },
                _ => Err(ParseError { message: format!("Expected expression, found {:?}", tok), span: start..start }),
            }
        } else {
            Err(ParseError { message: "Unexpected EOF".to_string(), span: start..start })
        }
    }
}
