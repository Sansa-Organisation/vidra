use std::fmt;

/// Source location for error reporting.
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

/// Token kinds in VidraScript.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Project,
    Scene,
    Layer,
    Text,
    Image,
    Video,
    Audio,
    TTS,
    AutoCaption,
    AiUpscale,
    Shape,
    Solid,
    Animation,
    Import,
    From,
    Component,
    Slot,
    If,
    Else,
    Asset,
    Variant,
    Layout,
    Rules,
    When,
    Aspect,

    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    ColorLiteral(String), // e.g., #FF0000
    DurationLiteral(f64), // stored in seconds (e.g., 5s → 5.0, 500ms → 0.5)

    // Punctuation
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    Comma,
    Colon,
    Dot,
    At,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equals,

    // Special
    Newline,
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Project => write!(f, "project"),
            TokenKind::Scene => write!(f, "scene"),
            TokenKind::Layer => write!(f, "layer"),
            TokenKind::Text => write!(f, "text"),
            TokenKind::Image => write!(f, "image"),
            TokenKind::Video => write!(f, "video"),
            TokenKind::Audio => write!(f, "audio"),
            TokenKind::TTS => write!(f, "tts"),
            TokenKind::AutoCaption => write!(f, "autocaption"),
            TokenKind::AiUpscale => write!(f, "ai_upscale"),
            TokenKind::Shape => write!(f, "shape"),
            TokenKind::Solid => write!(f, "solid"),
            TokenKind::Animation => write!(f, "animation"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::From => write!(f, "from"),
            TokenKind::Component => write!(f, "component"),
            TokenKind::Slot => write!(f, "slot"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::Asset => write!(f, "asset"),
            TokenKind::Variant => write!(f, "variant"),
            TokenKind::Layout => write!(f, "layout"),
            TokenKind::Rules => write!(f, "rules"),
            TokenKind::When => write!(f, "when"),
            TokenKind::Aspect => write!(f, "aspect"),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::NumberLiteral(n) => write!(f, "{}", n),
            TokenKind::ColorLiteral(c) => write!(f, "#{}", c),
            TokenKind::DurationLiteral(d) => {
                if *d < 1.0 {
                    write!(f, "{}ms", d * 1000.0)
                } else {
                    write!(f, "{}s", d)
                }
            }
            TokenKind::LeftBrace => write!(f, "{{"),
            TokenKind::RightBrace => write!(f, "}}"),
            TokenKind::LeftParen => write!(f, "("),
            TokenKind::RightParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Dot => write!(f, "."),
            TokenKind::At => write!(f, "@"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Equals => write!(f, "="),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

/// A token with its kind and source location.
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

/// The VidraScript lexer (tokenizer).
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

    /// Tokenize the entire source into a Vec of tokens.
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

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn next_token(&mut self) -> Result<Token, vidra_core::VidraError> {
        self.skip_whitespace();

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

        // Single-char tokens
        let kind = match ch {
            '\n' => {
                self.advance();
                TokenKind::Newline
            }
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
            '.' => {
                self.advance();
                TokenKind::Dot
            }
            '@' => {
                self.advance();
                TokenKind::At
            }
            '+' => {
                self.advance();
                TokenKind::Plus
            }
            '-' => {
                self.advance();
                TokenKind::Minus
            }
            '*' => {
                self.advance();
                TokenKind::Star
            }
            '/' => {
                self.advance();
                if self.peek() == Some('/') {
                    self.skip_line_comment();
                    return self.next_token();
                }
                TokenKind::Slash
            }
            '=' => {
                self.advance();
                TokenKind::Equals
            }
            '#' => {
                self.advance();
                let hex = self.read_while(|c| c.is_ascii_hexdigit());
                if hex.len() != 6 && hex.len() != 8 {
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
                self.advance(); // consume opening quote
                let mut s = String::new();
                loop {
                    match self.peek() {
                        Some('"') => {
                            self.advance();
                            break;
                        }
                        Some('\\') => {
                            self.advance();
                            match self.advance() {
                                Some('n') => s.push('\n'),
                                Some('t') => s.push('\t'),
                                Some('"') => s.push('"'),
                                Some('\\') => s.push('\\'),
                                Some(c) => s.push(c),
                                None => {
                                    return Err(vidra_core::VidraError::parse(
                                        "unterminated string literal",
                                        "<input>",
                                        line,
                                        column,
                                    ));
                                }
                            }
                        }
                        Some(c) => {
                            self.advance();
                            s.push(c);
                        }
                        None => {
                            return Err(vidra_core::VidraError::parse(
                                "unterminated string literal",
                                "<input>",
                                line,
                                column,
                            ));
                        }
                    }
                }
                TokenKind::StringLiteral(s)
            }
            c if c.is_ascii_digit() => {
                let num_str = self.read_while(|c| c.is_ascii_digit() || c == '.');
                let value: f64 = num_str.parse().map_err(|_| {
                    vidra_core::VidraError::parse(
                        format!("invalid number: {}", num_str),
                        "<input>",
                        line,
                        column,
                    )
                })?;

                // Check for duration suffix
                match self.peek() {
                    Some('s') => {
                        self.advance();
                        TokenKind::DurationLiteral(value)
                    }
                    Some('m') => {
                        self.advance();
                        if self.peek() == Some('s') {
                            self.advance();
                            TokenKind::DurationLiteral(value / 1000.0)
                        } else {
                            // it was just 'm' as part of something else — put back
                            TokenKind::NumberLiteral(value)
                        }
                    }
                    _ => TokenKind::NumberLiteral(value),
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let ident = self.read_while(|c| c.is_alphanumeric() || c == '_' || c == '-');
                match ident.as_str() {
                    "project" => TokenKind::Project,
                    "scene" => TokenKind::Scene,
                    "layer" => TokenKind::Layer,
                    "text" => TokenKind::Text,
                    "image" => TokenKind::Image,
                    "video" => TokenKind::Video,
                    "audio" => TokenKind::Audio,
                    "shape" => TokenKind::Shape,
                    "solid" => TokenKind::Solid,
                    "animation" => TokenKind::Animation,
                    "import" => TokenKind::Import,
                    "from" => TokenKind::From,
                    "component" => TokenKind::Component,
                    "slot" => TokenKind::Slot,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "asset" => TokenKind::Asset,
                    "variant" => TokenKind::Variant,
                    "layout" => TokenKind::Layout,
                    "rules" => TokenKind::Rules,
                    "when" => TokenKind::When,
                    "aspect" => TokenKind::Aspect,
                    "tts" => TokenKind::TTS,
                    "autocaption" => TokenKind::AutoCaption,
                    "ai_upscale" => TokenKind::AiUpscale,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(src: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();
        tokens.into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("project scene layer");
        assert_eq!(tokens[0], TokenKind::Project);
        assert_eq!(tokens[1], TokenKind::Scene);
        assert_eq!(tokens[2], TokenKind::Layer);
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize("\"hello world\"");
        assert_eq!(tokens[0], TokenKind::StringLiteral("hello world".into()));
    }

    #[test]
    fn test_number_literal() {
        let tokens = tokenize("42 3.14");
        assert_eq!(tokens[0], TokenKind::NumberLiteral(42.0));
        assert_eq!(tokens[1], TokenKind::NumberLiteral(3.14));
    }

    #[test]
    fn test_duration_literal() {
        let tokens = tokenize("5s 500ms");
        assert_eq!(tokens[0], TokenKind::DurationLiteral(5.0));
        assert_eq!(tokens[1], TokenKind::DurationLiteral(0.5));
    }

    #[test]
    fn test_color_literal() {
        let tokens = tokenize("#FF0000");
        assert_eq!(tokens[0], TokenKind::ColorLiteral("FF0000".into()));
    }

    #[test]
    fn test_punctuation() {
        let tokens = tokenize("{}(),:.@");
        assert_eq!(tokens[0], TokenKind::LeftBrace);
        assert_eq!(tokens[1], TokenKind::RightBrace);
        assert_eq!(tokens[2], TokenKind::LeftParen);
        assert_eq!(tokens[3], TokenKind::RightParen);
        assert_eq!(tokens[4], TokenKind::Comma);
        assert_eq!(tokens[5], TokenKind::Colon);
        assert_eq!(tokens[6], TokenKind::Dot);
        assert_eq!(tokens[7], TokenKind::At);
    }

    #[test]
    fn test_comments_skipped() {
        let tokens = tokenize("project // this is a comment\nscene");
        assert_eq!(tokens[0], TokenKind::Project);
        assert_eq!(tokens[1], TokenKind::Newline);
        assert_eq!(tokens[2], TokenKind::Scene);
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("my_layer center easeInOut");
        assert_eq!(tokens[0], TokenKind::Identifier("my_layer".into()));
        assert_eq!(tokens[1], TokenKind::Identifier("center".into()));
        assert_eq!(tokens[2], TokenKind::Identifier("easeInOut".into()));
    }

    #[test]
    fn test_full_snippet() {
        let src = r#"project(1920, 1080, 30) {
            scene("intro", 5s) {
                layer("bg") {
                    solid(#000000)
                }
            }
        }"#;
        let tokens = tokenize(src);
        assert_eq!(tokens[0], TokenKind::Project);
        assert_eq!(tokens[1], TokenKind::LeftParen);
        assert!(matches!(tokens.last(), Some(TokenKind::Eof)));
    }

    #[test]
    fn test_invalid_char() {
        let mut lexer = Lexer::new("§");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new("\"hello");
        let result = lexer.tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn test_escape_sequences() {
        let tokens = tokenize(r#""hello\nworld""#);
        assert_eq!(tokens[0], TokenKind::StringLiteral("hello\nworld".into()));
    }
}
