use crate::ast::*;
use crate::lexer::{Span, Token, TokenKind};
use vidra_core::VidraError;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, source: &'a str) -> Self {
        Self {
            tokens,
            pos: 0,
            source,
        }
    }

    pub fn parse(&mut self) -> Result<EffectDef, VidraError> {
        self.parse_effect_def()
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, VidraError> {
        let token = self.advance().ok_or_else(|| {
            VidraError::parse("Unexpected end of input", self.source, 0, 0)
        })?;
        if token.kind == kind {
            Ok(token)
        } else {
            Err(VidraError::parse(
                format!("Expected {}, found {:?}", kind, token.kind),
                self.source,
                token.span.line,
                token.span.column,
            ))
        }
    }

    fn parse_effect_def(&mut self) -> Result<EffectDef, VidraError> {
        let start_span = self.expect(TokenKind::Effect)?.span;
        
        let name_token = self.advance().ok_or_else(|| {
            VidraError::parse("Expected effect name", self.source, start_span.line, start_span.column)
        })?;
        
        let name = if let TokenKind::Identifier(ref name) = name_token.kind {
            name.clone()
        } else {
            return Err(VidraError::parse(
                format!("Expected effect name, found {:?}", name_token.kind),
                self.source,
                name_token.span.line,
                name_token.span.column,
            ));
        };

        let mut params = Vec::new();
        if let Some(tok) = self.peek() {
            if tok.kind == TokenKind::LeftParen {
                self.advance();
                while let Some(tok) = self.peek() {
                    if tok.kind == TokenKind::RightParen {
                        break;
                    }
                    params.push(self.parse_param()?);
                    if let Some(next) = self.peek() {
                        if next.kind == TokenKind::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RightParen)?;
            }
        }

        self.expect(TokenKind::LeftBrace)?;
        let mut body = Vec::new();
        while let Some(tok) = self.peek() {
            if tok.kind == TokenKind::RightBrace {
                break;
            }
            body.push(self.parse_statement()?);
        }
        let end_span = self.expect(TokenKind::RightBrace)?.span;

        Ok(EffectDef {
            name,
            params,
            body,
            span: Span::new(start_span.start, end_span.end, start_span.line, start_span.column),
        })
    }

    fn parse_param(&mut self) -> Result<Param, VidraError> {
        let start_tok = self.advance().unwrap();
        let name = if let TokenKind::Identifier(ref name) = start_tok.kind {
            name.clone()
        } else {
            return Err(VidraError::parse("Expected parameter name", self.source, start_tok.span.line, start_tok.span.column));
        };

        let mut default_value = None;
        if let Some(tok) = self.peek() {
            if tok.kind == TokenKind::Colon {
                self.advance();
                let val_tok = self.advance().unwrap();
                if let TokenKind::NumberLiteral(val) = val_tok.kind {
                    default_value = Some(val);
                } else {
                    return Err(VidraError::parse("Expected number literal for default param", self.source, val_tok.span.line, val_tok.span.column));
                }
            }
        }

        Ok(Param {
            name,
            default_value,
            span: start_tok.span,
        })
    }

    fn parse_statement(&mut self) -> Result<Statement, VidraError> {
        let tok = self.peek().unwrap();
        if tok.kind == TokenKind::Let {
            let start_span = self.advance().unwrap().span;
            let name_tok = self.advance().unwrap();
            let name = if let TokenKind::Identifier(ref name) = name_tok.kind {
                name.clone()
            } else {
                return Err(VidraError::parse("Expected identifier after let", self.source, name_tok.span.line, name_tok.span.column));
            };
            self.expect(TokenKind::Equals)?;
            let value = self.parse_expr()?;
            let span = Span::new(start_span.start, value.span().end, start_span.line, start_span.column);
            Ok(Statement::Let { name, value, span })
        } else {
            let expr = self.parse_expr()?;
            Ok(Statement::Expr(expr))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, VidraError> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr, VidraError> {
        let mut left = self.parse_additive()?;
        
        while let Some(tok) = self.peek() {
            if tok.kind == TokenKind::Pipe {
                self.advance();
                let right = self.parse_additive()?;
                let span = Span::new(left.span().start, right.span().end, left.span().line, left.span().column);
                left = Expr::Pipe {
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            } else {
                break;
            }
        }
        
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, VidraError> {
        let mut left = self.parse_multiplicative()?;
        
        while let Some(tok) = self.peek() {
            let op = match tok.kind {
                TokenKind::Plus => Op::Add,
                TokenKind::Minus => Op::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = Span::new(left.span().start, right.span().end, left.span().line, left.span().column);
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, VidraError> {
        let mut left = self.parse_atom()?;
        
        while let Some(tok) = self.peek() {
            let op = match tok.kind {
                TokenKind::Star => Op::Mul,
                TokenKind::Slash => Op::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_atom()?;
            let span = Span::new(left.span().start, right.span().end, left.span().line, left.span().column);
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right), span };
        }
        
        Ok(left)
    }

    fn parse_atom(&mut self) -> Result<Expr, VidraError> {
        let tok = self.advance().ok_or_else(|| VidraError::parse("Unexpected EOF", self.source, 0, 0))?;
        
        match &tok.kind {
            TokenKind::Identifier(name) => {
                let mut span = tok.span;
                if let Some(next) = self.peek() {
                    if next.kind == TokenKind::LeftParen {
                        self.advance();
                        let mut args = Vec::new();
                        while let Some(inner) = self.peek() {
                            if inner.kind == TokenKind::RightParen {
                                break;
                            }
                            args.push(self.parse_arg()?);
                            if let Some(comma) = self.peek() {
                                if comma.kind == TokenKind::Comma {
                                    self.advance();
                                } else {
                                    break;
                                }
                            }
                        }
                        let end_span = self.expect(TokenKind::RightParen)?.span;
                        span = Span::new(span.start, end_span.end, span.line, span.column);
                        return Ok(Expr::Call { name: name.clone(), args, span });
                    }
                }
                
                Ok(Expr::Ident(name.clone(), span))
            }
            TokenKind::NumberLiteral(val) => Ok(Expr::Number(*val, tok.span)),
            TokenKind::ColorLiteral(val) => Ok(Expr::ColorHex(val.clone(), tok.span)),
            TokenKind::StringLiteral(val) => Ok(Expr::StringLit(val.clone(), tok.span)),
            TokenKind::LeftParen => {
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }
            _ => Err(VidraError::parse(format!("Unexpected token: {:?}", tok.kind), self.source, tok.span.line, tok.span.column)),
        }
    }

    fn parse_arg(&mut self) -> Result<Arg, VidraError> {
        let _start_pos = self.pos;
        
        let mut name = None;
        if let Some(Token { kind: TokenKind::Identifier(n), .. }) = self.peek() {
            if let Some(Token { kind: TokenKind::Colon, .. }) = self.tokens.get(self.pos + 1) {
                name = Some(n.clone());
                self.advance();
                self.advance();
            }
        }
        
        let value = self.parse_expr()?;
        let span = Span::new(value.span().start, value.span().end, value.span().line, value.span().column);
        
        Ok(Arg { name, value, span })
    }
}
