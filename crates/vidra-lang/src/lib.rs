//! # vidra-lang
//!
//! VidraScript parser and compiler.
//! Parses VidraScript source code into an AST, then compiles it to the Vidra IR.

pub mod ast;
pub mod checker;
pub mod compiler;
pub mod lexer;
pub mod parser;
pub mod formatter;
pub mod advanced_anim;

pub use checker::TypeChecker;
pub use compiler::Compiler;
pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;
pub use formatter::Formatter;
