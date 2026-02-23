use crate::lexer::Span;

#[derive(Debug, Clone)]
pub struct EffectDef {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub default_value: Option<f64>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let { name: String, value: Expr, span: Span },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Call {
        name: String,
        args: Vec<Arg>,
        span: Span,
    },
    Pipe {
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Ident(String, Span),
    Number(f64, Span),
    ColorHex(String, Span),
    StringLit(String, Span),
    BinOp {
        op: Op,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Call { span, .. } => *span,
            Expr::Pipe { span, .. } => *span,
            Expr::Ident(_, span) => *span,
            Expr::Number(_, span) => *span,
            Expr::ColorHex(_, span) => *span,
            Expr::StringLit(_, span) => *span,
            Expr::BinOp { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}
