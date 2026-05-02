use crate::frontend::span::{Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier,
    IntegerNumber,
    RealNumber,
    String,
    OperatorOrDelimiter,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub lexeme: &'a str,
    pub span: Span,
}

impl<'a> Token<'a> {
    pub(crate) fn new(kind: TokenKind, lexeme: &str, span: Span) -> Token {
        Token { kind, lexeme, span }
    }
}