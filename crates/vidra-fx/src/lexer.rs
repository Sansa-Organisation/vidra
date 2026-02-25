use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Effect,
    Let,
    Pipe, // ->

    Identifier(String),
    NumberLiteral(f64),
    ColorLiteral(String),
    StringLiteral(String),

    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Equals,

    Plus,
    Minus,
    Star,
    Slash,

    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Effect => write!(f, "@effect"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Pipe => write!(f, "->"),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::NumberLiteral(n) => write!(f, "{}", n),
            TokenKind::ColorLiteral(c) => write!(f, "#{}", c),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, vidra_core::VidraError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.source.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '/' && self.peek_next() == Some('/') {
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, vidra_core::VidraError> {
        self.skip_whitespace_and_comments();

        let start = self.pos;
        let line = self.line;
        let column = self.column;

        let ch = match self.peek() {
            Some(ch) => ch,
            None => {
                return Ok(Token::new(
                    TokenKind::Eof,
                    Span::new(start, start, line, column),
                ));
            }
        };

        let kind = match ch {
            '{' => {
                self.advance();
                TokenKind::LeftBrace
            }
            '}' => {
                self.advance();
                TokenKind::RightBrace
            }
            '(' => {
                self.advance();
                TokenKind::LeftParen
            }
            ')' => {
                self.advance();
                TokenKind::RightParen
            }
            ',' => {
                self.advance();
                TokenKind::Comma
            }
            ':' => {
                self.advance();
                TokenKind::Colon
            }
            '=' => {
                self.advance();
                TokenKind::Equals
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Pipe
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                TokenKind::Slash
            }
            '@' => {
                self.advance();
                let ident = self.read_while(|c| c.is_alphabetic());
                if ident == "effect" {
                    TokenKind::Effect
                } else {
                    return Err(vidra_core::VidraError::parse(
                        format!("Unknown decorator: @{}", ident),
                        "<input>",
                        line,
                        column,
                    ));
                }
            }
            '#' => {
                self.advance();
                let hex = self.read_while(|c| c.is_ascii_hexdigit());
                if hex.len() != 6 && hex.len() != 8 && hex.len() != 3 {
                    return Err(vidra_core::VidraError::parse(
                        format!("invalid hex color: #{}", hex),
                        "<input>",
                        line,
                        column,
                    ));
                }
                TokenKind::ColorLiteral(hex)
            }
            '"' => {
                self.advance();
                let mut s = String::new();
                while let Some(c) = self.peek() {
                    if c == '"' {
                        self.advance();
                        break;
                    }
                    s.push(c);
                    self.advance();
                }
                TokenKind::StringLiteral(s)
            }
            c if c.is_ascii_digit() || c == '.' => {
                let num_str = self.read_while(|c| c.is_ascii_digit() || c == '.');
                if let Ok(value) = num_str.parse::<f64>() {
                    TokenKind::NumberLiteral(value)
                } else {
                    return Err(vidra_core::VidraError::parse(
                        format!("invalid number: {}", num_str),
                        "<input>",
                        line,
                        column,
                    ));
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let ident = self.read_while(|c| c.is_alphanumeric() || c == '_');
                match ident.as_str() {
                    "let" => TokenKind::Let,
                    _ => TokenKind::Identifier(ident),
                }
            }
            _ => {
                return Err(vidra_core::VidraError::parse(
                    format!("unexpected character: '{}'", ch),
                    "<input>",
                    line,
                    column,
                ));
            }
        };

        Ok(Token::new(kind, Span::new(start, self.pos, line, column)))
    }

    fn read_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }
}
