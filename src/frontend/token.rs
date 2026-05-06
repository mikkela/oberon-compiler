use crate::frontend::span::{Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier,
    Number,
    String,
    OperatorOrDelimiter,
    Eof,
    Invalid
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

impl Token {
    pub(crate) fn invalid() -> Token {
        Token { kind: TokenKind::Invalid, lexeme: "".to_string(), span: Span::default() }
    }
}

impl Token {
    pub(crate) fn new(kind: TokenKind, lexeme: &str, span: Span) -> Token {
        Token { kind, lexeme: lexeme.to_string(), span }
    }
}