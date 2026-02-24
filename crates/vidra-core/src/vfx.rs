//! # VidraFX DSL
//!
//! A TypeScript-like language for writing custom pixel effects in `.vfx` files.
//! VidraFX compiles to WGSL (WebGPU Shading Language) for GPU-accelerated rendering.
//!
//! ## Example `.vfx` file:
//!
//! ```text
//! effect ChromaticAberration {
//!     param intensity: float = 0.5;
//!     param direction: vec2 = vec2(1.0, 0.0);
//!
//!     fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
//!         let offset = direction * intensity * 0.01;
//!         let r = sample(uv + offset).r;
//!         let g = color.g;
//!         let b = sample(uv - offset).b;
//!         return vec4(r, g, b, color.a);
//!     }
//! }
//! ```

use std::fmt;

use crate::VidraError;

// ──────────────────────────────────────────────────────────────────────────────
// Tokens
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum FxToken {
    // Keywords
    Effect,
    Param,
    Fn,
    Let,
    Return,
    If,
    Else,

    // Types
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Bool,

    // Identifiers and literals
    Ident(String),
    NumberLit(f64),
    BoolLit(bool),

    // Punctuation
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Semicolon,
    Colon,
    Comma,
    Equals,
    Arrow, // ->
    Dot,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    EqEq,
    NotEq,
    Bang,
    And,  // &&
    Or,   // ||

    Eof,
}

impl fmt::Display for FxToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxToken::Ident(s) => write!(f, "{}", s),
            FxToken::NumberLit(n) => write!(f, "{}", n),
            FxToken::BoolLit(b) => write!(f, "{}", b),
            _ => write!(f, "{:?}", self),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Lexer
// ──────────────────────────────────────────────────────────────────────────────

pub struct FxLexer {
    chars: Vec<char>,
    pos: usize,
}

impl FxLexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<FxToken>, VidraError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.chars.len() {
                tokens.push(FxToken::Eof);
                break;
            }

            let ch = self.chars[self.pos];
            let token = match ch {
                '{' => { self.pos += 1; FxToken::LeftBrace }
                '}' => { self.pos += 1; FxToken::RightBrace }
                '(' => { self.pos += 1; FxToken::LeftParen }
                ')' => { self.pos += 1; FxToken::RightParen }
                ';' => { self.pos += 1; FxToken::Semicolon }
                ':' => { self.pos += 1; FxToken::Colon }
                ',' => { self.pos += 1; FxToken::Comma }
                '.' => { self.pos += 1; FxToken::Dot }
                '+' => { self.pos += 1; FxToken::Plus }
                '*' => { self.pos += 1; FxToken::Star }
                '/' => { self.pos += 1; FxToken::Slash }
                '-' => {
                    self.pos += 1;
                    if self.peek() == Some('>') {
                        self.pos += 1;
                        FxToken::Arrow
                    } else {
                        FxToken::Minus
                    }
                }
                '=' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        FxToken::EqEq
                    } else {
                        FxToken::Equals
                    }
                }
                '>' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        FxToken::GreaterEq
                    } else {
                        FxToken::Greater
                    }
                }
                '<' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        FxToken::LessEq
                    } else {
                        FxToken::Less
                    }
                }
                '!' => {
                    self.pos += 1;
                    if self.peek() == Some('=') {
                        self.pos += 1;
                        FxToken::NotEq
                    } else {
                        FxToken::Bang
                    }
                }
                '&' => {
                    self.pos += 1;
                    if self.peek() == Some('&') { self.pos += 1; }
                    FxToken::And
                }
                '|' => {
                    self.pos += 1;
                    if self.peek() == Some('|') { self.pos += 1; }
                    FxToken::Or
                }
                c if c.is_ascii_digit() => self.read_number()?,
                c if c.is_ascii_alphabetic() || c == '_' => self.read_ident(),
                _ => {
                    return Err(VidraError::parse(
                        format!("unexpected character: '{}'", ch),
                        "<vfx>",
                        0,
                        self.pos,
                    ));
                }
            };
            tokens.push(token);
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.chars.len() {
            let ch = self.chars[self.pos];
            if ch.is_whitespace() {
                self.pos += 1;
            } else if ch == '/' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '/' {
                // Line comment
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Result<FxToken, VidraError> {
        let start = self.pos;
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_digit() || self.chars[self.pos] == '.') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        let n: f64 = s.parse().map_err(|_| VidraError::parse(
            format!("invalid number: {}", s), "<vfx>", 0, start,
        ))?;
        Ok(FxToken::NumberLit(n))
    }

    fn read_ident(&mut self) -> FxToken {
        let start = self.pos;
        while self.pos < self.chars.len() && (self.chars[self.pos].is_ascii_alphanumeric() || self.chars[self.pos] == '_') {
            self.pos += 1;
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        match s.as_str() {
            "effect" => FxToken::Effect,
            "param" => FxToken::Param,
            "fn" => FxToken::Fn,
            "let" => FxToken::Let,
            "return" => FxToken::Return,
            "if" => FxToken::If,
            "else" => FxToken::Else,
            "float" => FxToken::Float,
            "vec2" => FxToken::Vec2,
            "vec3" => FxToken::Vec3,
            "vec4" => FxToken::Vec4,
            "int" => FxToken::Int,
            "bool" => FxToken::Bool,
            "true" => FxToken::BoolLit(true),
            "false" => FxToken::BoolLit(false),
            _ => FxToken::Ident(s),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// AST
// ──────────────────────────────────────────────────────────────────────────────

/// A complete VidraFX effect definition.
#[derive(Debug, Clone)]
pub struct FxEffect {
    pub name: String,
    pub params: Vec<FxParam>,
    pub functions: Vec<FxFunction>,
}

/// A parameter declaration: `param intensity: float = 0.5;`
#[derive(Debug, Clone)]
pub struct FxParam {
    pub name: String,
    pub ty: FxType,
    pub default: Option<FxExpr>,
}

/// Supported types in VidraFX.
#[derive(Debug, Clone, PartialEq)]
pub enum FxType {
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    Bool,
}

impl fmt::Display for FxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxType::Float => write!(f, "f32"),
            FxType::Vec2 => write!(f, "vec2<f32>"),
            FxType::Vec3 => write!(f, "vec3<f32>"),
            FxType::Vec4 => write!(f, "vec4<f32>"),
            FxType::Int => write!(f, "i32"),
            FxType::Bool => write!(f, "bool"),
        }
    }
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct FxFunction {
    pub name: String,
    pub params: Vec<(String, FxType)>,
    pub return_type: Option<FxType>,
    pub body: Vec<FxStmt>,
}

/// A statement.
#[derive(Debug, Clone)]
pub enum FxStmt {
    Let { name: String, value: FxExpr },
    Return(FxExpr),
    Expr(FxExpr),
    If { condition: FxExpr, then_body: Vec<FxStmt>, else_body: Vec<FxStmt> },
}

/// An expression.
#[derive(Debug, Clone)]
pub enum FxExpr {
    Number(f64),
    Bool(bool),
    Ident(String),
    BinOp { left: Box<FxExpr>, op: FxBinOp, right: Box<FxExpr> },
    UnaryOp { op: FxUnaryOp, expr: Box<FxExpr> },
    Call { func: String, args: Vec<FxExpr> },
    FieldAccess { object: Box<FxExpr>, field: String },
    Constructor { ty: FxType, args: Vec<FxExpr> },
}

#[derive(Debug, Clone, Copy)]
pub enum FxBinOp {
    Add, Sub, Mul, Div,
    Greater, Less, GreaterEq, LessEq,
    Equal, NotEqual,
    And, Or,
}

impl fmt::Display for FxBinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FxBinOp::Add => write!(f, "+"),
            FxBinOp::Sub => write!(f, "-"),
            FxBinOp::Mul => write!(f, "*"),
            FxBinOp::Div => write!(f, "/"),
            FxBinOp::Greater => write!(f, ">"),
            FxBinOp::Less => write!(f, "<"),
            FxBinOp::GreaterEq => write!(f, ">="),
            FxBinOp::LessEq => write!(f, "<="),
            FxBinOp::Equal => write!(f, "=="),
            FxBinOp::NotEqual => write!(f, "!="),
            FxBinOp::And => write!(f, "&&"),
            FxBinOp::Or => write!(f, "||"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FxUnaryOp {
    Neg,
    Not,
}

// ──────────────────────────────────────────────────────────────────────────────
// Parser
// ──────────────────────────────────────────────────────────────────────────────

pub struct FxParser {
    tokens: Vec<FxToken>,
    pos: usize,
}

impl FxParser {
    pub fn new(tokens: Vec<FxToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &FxToken {
        self.tokens.get(self.pos).unwrap_or(&FxToken::Eof)
    }

    fn advance(&mut self) -> FxToken {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(FxToken::Eof);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &FxToken) -> Result<(), VidraError> {
        let t = self.advance();
        if &t == expected {
            Ok(())
        } else {
            Err(VidraError::parse(
                format!("expected {:?}, got {:?}", expected, t),
                "<vfx>", 0, self.pos,
            ))
        }
    }

    /// Parse a complete VidraFX effect.
    pub fn parse(&mut self) -> Result<FxEffect, VidraError> {
        self.expect(&FxToken::Effect)?;
        let name = match self.advance() {
            FxToken::Ident(s) => s,
            t => return Err(VidraError::parse(format!("expected effect name, got {:?}", t), "<vfx>", 0, 0)),
        };
        self.expect(&FxToken::LeftBrace)?;

        let mut params = Vec::new();
        let mut functions = Vec::new();

        while self.peek() != &FxToken::RightBrace && self.peek() != &FxToken::Eof {
            match self.peek() {
                FxToken::Param => {
                    params.push(self.parse_param()?);
                }
                FxToken::Fn => {
                    functions.push(self.parse_function()?);
                }
                _ => {
                    return Err(VidraError::parse(
                        format!("unexpected token in effect body: {:?}", self.peek()),
                        "<vfx>", 0, self.pos,
                    ));
                }
            }
        }

        self.expect(&FxToken::RightBrace)?;

        Ok(FxEffect { name, params, functions })
    }

    fn parse_param(&mut self) -> Result<FxParam, VidraError> {
        self.expect(&FxToken::Param)?;
        let name = match self.advance() {
            FxToken::Ident(s) => s,
            t => return Err(VidraError::parse(format!("expected param name, got {:?}", t), "<vfx>", 0, 0)),
        };
        self.expect(&FxToken::Colon)?;
        let ty = self.parse_type()?;
        let default = if self.peek() == &FxToken::Equals {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(&FxToken::Semicolon)?;
        Ok(FxParam { name, ty, default })
    }

    fn parse_type(&mut self) -> Result<FxType, VidraError> {
        match self.advance() {
            FxToken::Float => Ok(FxType::Float),
            FxToken::Vec2 => Ok(FxType::Vec2),
            FxToken::Vec3 => Ok(FxType::Vec3),
            FxToken::Vec4 => Ok(FxType::Vec4),
            FxToken::Int => Ok(FxType::Int),
            FxToken::Bool => Ok(FxType::Bool),
            t => Err(VidraError::parse(format!("expected type, got {:?}", t), "<vfx>", 0, 0)),
        }
    }

    fn parse_function(&mut self) -> Result<FxFunction, VidraError> {
        self.expect(&FxToken::Fn)?;
        let name = match self.advance() {
            FxToken::Ident(s) => s,
            t => return Err(VidraError::parse(format!("expected function name, got {:?}", t), "<vfx>", 0, 0)),
        };
        self.expect(&FxToken::LeftParen)?;

        let mut params = Vec::new();
        while self.peek() != &FxToken::RightParen && self.peek() != &FxToken::Eof {
            let pname = match self.advance() {
                FxToken::Ident(s) => s,
                t => return Err(VidraError::parse(format!("expected param name, got {:?}", t), "<vfx>", 0, 0)),
            };
            self.expect(&FxToken::Colon)?;
            let ty = self.parse_type()?;
            params.push((pname, ty));
            if self.peek() == &FxToken::Comma {
                self.advance();
            }
        }
        self.expect(&FxToken::RightParen)?;

        let return_type = if self.peek() == &FxToken::Arrow {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(FxFunction { name, params, return_type, body })
    }

    fn parse_block(&mut self) -> Result<Vec<FxStmt>, VidraError> {
        self.expect(&FxToken::LeftBrace)?;
        let mut stmts = Vec::new();
        while self.peek() != &FxToken::RightBrace && self.peek() != &FxToken::Eof {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&FxToken::RightBrace)?;
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<FxStmt, VidraError> {
        match self.peek().clone() {
            FxToken::Let => {
                self.advance();
                let name = match self.advance() {
                    FxToken::Ident(s) => s,
                    t => return Err(VidraError::parse(format!("expected variable name, got {:?}", t), "<vfx>", 0, 0)),
                };
                self.expect(&FxToken::Equals)?;
                let value = self.parse_expr()?;
                self.expect(&FxToken::Semicolon)?;
                Ok(FxStmt::Let { name, value })
            }
            FxToken::Return => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&FxToken::Semicolon)?;
                Ok(FxStmt::Return(expr))
            }
            FxToken::If => {
                self.advance();
                self.expect(&FxToken::LeftParen)?;
                let condition = self.parse_expr()?;
                self.expect(&FxToken::RightParen)?;
                let then_body = self.parse_block()?;
                let else_body = if self.peek() == &FxToken::Else {
                    self.advance();
                    self.parse_block()?
                } else {
                    Vec::new()
                };
                Ok(FxStmt::If { condition, then_body, else_body })
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(&FxToken::Semicolon)?;
                Ok(FxStmt::Expr(expr))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<FxExpr, VidraError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<FxExpr, VidraError> {
        let mut left = self.parse_and()?;
        while self.peek() == &FxToken::Or {
            self.advance();
            let right = self.parse_and()?;
            left = FxExpr::BinOp { left: Box::new(left), op: FxBinOp::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<FxExpr, VidraError> {
        let mut left = self.parse_comparison()?;
        while self.peek() == &FxToken::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = FxExpr::BinOp { left: Box::new(left), op: FxBinOp::And, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<FxExpr, VidraError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                FxToken::Greater => FxBinOp::Greater,
                FxToken::Less => FxBinOp::Less,
                FxToken::GreaterEq => FxBinOp::GreaterEq,
                FxToken::LessEq => FxBinOp::LessEq,
                FxToken::EqEq => FxBinOp::Equal,
                FxToken::NotEq => FxBinOp::NotEqual,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = FxExpr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<FxExpr, VidraError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                FxToken::Plus => FxBinOp::Add,
                FxToken::Minus => FxBinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = FxExpr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<FxExpr, VidraError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                FxToken::Star => FxBinOp::Mul,
                FxToken::Slash => FxBinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = FxExpr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<FxExpr, VidraError> {
        match self.peek() {
            FxToken::Minus => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(FxExpr::UnaryOp { op: FxUnaryOp::Neg, expr: Box::new(expr) })
            }
            FxToken::Bang => {
                self.advance();
                let expr = self.parse_primary()?;
                Ok(FxExpr::UnaryOp { op: FxUnaryOp::Not, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<FxExpr, VidraError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                FxToken::Dot => {
                    self.advance();
                    let field = match self.advance() {
                        FxToken::Ident(s) => s,
                        t => return Err(VidraError::parse(format!("expected field name, got {:?}", t), "<vfx>", 0, 0)),
                    };
                    expr = FxExpr::FieldAccess { object: Box::new(expr), field };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<FxExpr, VidraError> {
        match self.peek().clone() {
            FxToken::NumberLit(n) => {
                self.advance();
                Ok(FxExpr::Number(n))
            }
            FxToken::BoolLit(b) => {
                self.advance();
                Ok(FxExpr::Bool(b))
            }
            FxToken::Vec2 | FxToken::Vec3 | FxToken::Vec4 => {
                let ty = match self.advance() {
                    FxToken::Vec2 => FxType::Vec2,
                    FxToken::Vec3 => FxType::Vec3,
                    FxToken::Vec4 => FxType::Vec4,
                    _ => unreachable!(),
                };
                self.expect(&FxToken::LeftParen)?;
                let mut args = Vec::new();
                while self.peek() != &FxToken::RightParen {
                    args.push(self.parse_expr()?);
                    if self.peek() == &FxToken::Comma { self.advance(); }
                }
                self.expect(&FxToken::RightParen)?;
                Ok(FxExpr::Constructor { ty, args })
            }
            FxToken::Ident(name) => {
                self.advance();
                if self.peek() == &FxToken::LeftParen {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != &FxToken::RightParen && self.peek() != &FxToken::Eof {
                        args.push(self.parse_expr()?);
                        if self.peek() == &FxToken::Comma { self.advance(); }
                    }
                    self.expect(&FxToken::RightParen)?;
                    Ok(FxExpr::Call { func: name, args })
                } else {
                    Ok(FxExpr::Ident(name))
                }
            }
            FxToken::LeftParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&FxToken::RightParen)?;
                Ok(expr)
            }
            t => Err(VidraError::parse(format!("unexpected token in expression: {:?}", t), "<vfx>", 0, 0)),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// WGSL Code Generator
// ──────────────────────────────────────────────────────────────────────────────

/// Compile a VidraFX effect AST into WGSL shader source.
pub fn compile_to_wgsl(effect: &FxEffect) -> String {
    let mut out = String::new();

    // Uniforms struct
    out.push_str("// Auto-generated by VidraFX compiler\n");
    out.push_str("// Effect: ");
    out.push_str(&effect.name);
    out.push('\n');
    out.push('\n');

    // Params become a uniform struct
    if !effect.params.is_empty() {
        out.push_str("struct FxParams {\n");
        for p in &effect.params {
            out.push_str(&format!("    {}: {},\n", p.name, p.ty));
        }
        out.push_str("}\n\n");
        out.push_str("@group(0) @binding(0) var<uniform> params: FxParams;\n");
    }

    // Built-in uniforms
    out.push_str("@group(0) @binding(1) var<uniform> time: f32;\n");
    out.push_str("@group(0) @binding(2) var<uniform> resolution: vec2<f32>;\n");
    out.push_str("@group(0) @binding(3) var input_texture: texture_2d<f32>;\n");
    out.push_str("@group(0) @binding(4) var input_sampler: sampler;\n\n");

    // Helper: sample function
    out.push_str("fn sample(uv: vec2<f32>) -> vec4<f32> {\n");
    out.push_str("    return textureSample(input_texture, input_sampler, uv);\n");
    out.push_str("}\n\n");

    // Emit user functions
    for func in &effect.functions {
        out.push_str(&format!("fn {}(", func.name));
        for (i, (pname, pty)) in func.params.iter().enumerate() {
            if i > 0 { out.push_str(", "); }
            out.push_str(&format!("{}: {}", pname, pty));
        }
        out.push(')');
        if let Some(ref rt) = func.return_type {
            out.push_str(&format!(" -> {}", rt));
        }
        out.push_str(" {\n");
        for stmt in &func.body {
            emit_stmt(&mut out, stmt, 1);
        }
        out.push_str("}\n\n");
    }

    // Fragment shader entry point that calls `apply`
    out.push_str("@fragment\n");
    out.push_str("fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {\n");
    out.push_str("    let color = sample(uv);\n");
    out.push_str("    return apply(uv, color, time);\n");
    out.push_str("}\n");

    out
}

fn emit_stmt(out: &mut String, stmt: &FxStmt, indent: usize) {
    let pad: String = "    ".repeat(indent);
    match stmt {
        FxStmt::Let { name, value } => {
            out.push_str(&format!("{}let {} = {};\n", pad, name, emit_expr(value)));
        }
        FxStmt::Return(expr) => {
            out.push_str(&format!("{}return {};\n", pad, emit_expr(expr)));
        }
        FxStmt::Expr(expr) => {
            out.push_str(&format!("{}{};\n", pad, emit_expr(expr)));
        }
        FxStmt::If { condition, then_body, else_body } => {
            out.push_str(&format!("{}if ({}) {{\n", pad, emit_expr(condition)));
            for s in then_body { emit_stmt(out, s, indent + 1); }
            if !else_body.is_empty() {
                out.push_str(&format!("{}}} else {{\n", pad));
                for s in else_body { emit_stmt(out, s, indent + 1); }
            }
            out.push_str(&format!("{}}}\n", pad));
        }
    }
}

fn emit_expr(expr: &FxExpr) -> String {
    match expr {
        FxExpr::Number(n) => {
            if *n == (*n as i64) as f64 && !n.is_nan() {
                format!("{:.1}", n)
            } else {
                format!("{}", n)
            }
        }
        FxExpr::Bool(b) => format!("{}", b),
        FxExpr::Ident(name) => name.clone(),
        FxExpr::BinOp { left, op, right } => {
            format!("({} {} {})", emit_expr(left), op, emit_expr(right))
        }
        FxExpr::UnaryOp { op, expr } => {
            let op_str = match op {
                FxUnaryOp::Neg => "-",
                FxUnaryOp::Not => "!",
            };
            format!("({}{})", op_str, emit_expr(expr))
        }
        FxExpr::Call { func, args } => {
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}({})", func, args_str.join(", "))
        }
        FxExpr::FieldAccess { object, field } => {
            format!("{}.{}", emit_expr(object), field)
        }
        FxExpr::Constructor { ty, args } => {
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}({})", ty, args_str.join(", "))
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Public API
// ──────────────────────────────────────────────────────────────────────────────

/// Compile a VidraFX source string into WGSL shader code.
pub fn compile(source: &str) -> Result<String, VidraError> {
    let mut lexer = FxLexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = FxParser::new(tokens);
    let effect = parser.parse()?;
    Ok(compile_to_wgsl(&effect))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_basic() {
        let mut lexer = FxLexer::new("effect Blur { param radius: float = 1.0; }");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[0], FxToken::Effect));
        assert!(matches!(tokens[1], FxToken::Ident(_)));
        assert!(matches!(tokens[2], FxToken::LeftBrace));
        assert!(matches!(tokens[3], FxToken::Param));
    }

    #[test]
    fn test_parse_chromatic_aberration() {
        let source = r#"
            effect ChromaticAberration {
                param intensity: float = 0.5;
                param red_offset: float = 0.01;

                fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
                    let offset = vec2(red_offset * intensity, 0.0);
                    let r = sample(uv + offset).r;
                    let g = color.g;
                    let b = sample(uv - offset).b;
                    return vec4(r, g, b, color.a);
                }
            }
        "#;

        let mut lexer = FxLexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = FxParser::new(tokens);
        let effect = parser.parse().unwrap();

        assert_eq!(effect.name, "ChromaticAberration");
        assert_eq!(effect.params.len(), 2);
        assert_eq!(effect.params[0].name, "intensity");
        assert_eq!(effect.params[0].ty, FxType::Float);
        assert_eq!(effect.functions.len(), 1);
        assert_eq!(effect.functions[0].name, "apply");
        assert_eq!(effect.functions[0].params.len(), 3);
        assert!(effect.functions[0].return_type.is_some());
    }

    #[test]
    fn test_compile_to_wgsl() {
        let source = r#"
            effect Vignette {
                param amount: float = 1.0;

                fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
                    let center = vec2(0.5, 0.5);
                    let dist = distance(uv, center);
                    let vignette = 1.0 - dist * amount;
                    return vec4(color.r * vignette, color.g * vignette, color.b * vignette, color.a);
                }
            }
        "#;

        let wgsl = compile(source).unwrap();
        assert!(wgsl.contains("struct FxParams"));
        assert!(wgsl.contains("amount: f32"));
        assert!(wgsl.contains("fn apply"));
        assert!(wgsl.contains("fn fs_main"));
        assert!(wgsl.contains("fn sample"));
    }

    #[test]
    fn test_compile_with_if() {
        let source = r#"
            effect ConditionalBlur {
                param threshold: float = 0.5;

                fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
                    let brightness = color.r + color.g + color.b;
                    if (brightness > threshold) {
                        return color;
                    } else {
                        return vec4(0.0, 0.0, 0.0, 1.0);
                    }
                }
            }
        "#;

        let wgsl = compile(source).unwrap();
        assert!(wgsl.contains("if ("));
        assert!(wgsl.contains("} else {"));
    }

    #[test]
    fn test_compile_no_params() {
        let source = r#"
            effect Grayscale {
                fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
                    let gray = color.r * 0.299 + color.g * 0.587 + color.b * 0.114;
                    return vec4(gray, gray, gray, color.a);
                }
            }
        "#;

        let wgsl = compile(source).unwrap();
        assert!(!wgsl.contains("struct FxParams"));
        assert!(wgsl.contains("fn apply"));
    }

    #[test]
    fn test_full_pipeline() {
        let source = r#"
            effect Pixelate {
                param pixel_size: float = 8.0;

                fn apply(uv: vec2, color: vec4, time: float) -> vec4 {
                    let grid = vec2(pixel_size / resolution.x, pixel_size / resolution.y);
                    let snapped = floor(uv / grid) * grid;
                    return sample(snapped);
                }
            }
        "#;

        let wgsl = compile(source).unwrap();
        // Check the header
        assert!(wgsl.contains("// Effect: Pixelate"));
        // Check params
        assert!(wgsl.contains("pixel_size: f32"));
        // Check the fragment shader entry point
        assert!(wgsl.contains("@fragment"));
        assert!(wgsl.contains("fn fs_main"));
        assert!(wgsl.contains("return apply(uv, color, time)"));
    }
}
