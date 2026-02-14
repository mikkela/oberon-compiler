// src/lexer.rs
//
// A small, explicit finite-state lexer that emits tokens for:
// - Number
// - Identifier
// - Keyword (IF)
// - Symbol (< and <=)
//
// Notes:
// - No comments handled here (Oberon nested comments are not regular).
// - Spans are byte offsets [start, end).
//

use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Number(i64),
    Identifier(String),
    KeywordIf,
    Less,      // <
    LessEqual, // <=
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Start,
    Number,
    Ident,
    Lt, // saw '<' and might become <=
}

#[derive(Debug)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,        // byte offset of current cursor
    token_start: usize,
    state: State,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            token_start: 0,
            state: State::Start,
        }
    }

    /// Produce the next token.
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        use State::*;

        loop {
            match self.state {
                Start => {
                    self.skip_whitespace();

                    self.token_start = self.pos;

                    let Some(ch) = self.peek_char() else {
                        return Ok(Token {
                            kind: TokenKind::Eof,
                            span: Span::new(self.pos, self.pos),
                        });
                    };

                    self.state = if ch.is_ascii_digit() {
                        Number
                    } else if is_ident_start(ch) {
                        Ident
                    } else if ch == '<' {
                        Lt
                    } else {
                        // Unknown single char => error (extend with more symbols later)
                        let start = self.pos;
                        self.bump_char();
                        return Err(LexError {
                            message: format!("Unexpected character: {ch:?}"),
                            span: Span::new(start, self.pos),
                        });
                    };
                }

                Number => {
                    // Consume digits
                    while let Some(ch) = self.peek_char() {
                        if ch.is_ascii_digit() {
                            self.bump_char();
                        } else {
                            break;
                        }
                    }

                    let span = Span::new(self.token_start, self.pos);
                    let text = &self.input[span.start..span.end];

                    let value = text.parse::<i64>().map_err(|e| LexError {
                        message: format!("Invalid integer literal: {e}"),
                        span,
                    })?;

                    self.state = Start;
                    return Ok(Token {
                        kind: TokenKind::Number(value),
                        span,
                    });
                }

                Ident => {
                    // Consume ident chars
                    while let Some(ch) = self.peek_char() {
                        if is_ident_continue(ch) {
                            self.bump_char();
                        } else {
                            break;
                        }
                    }

                    let span = Span::new(self.token_start, self.pos);
                    let text = &self.input[span.start..span.end];

                    // Keyword recognition happens when leaving Ident state (classic lexer pattern)
                    let kind = if text.eq_ignore_ascii_case("IF") {
                        TokenKind::KeywordIf
                    } else {
                        TokenKind::Identifier(text.to_string())
                    };

                    self.state = Start;
                    return Ok(Token { kind, span });
                }

                Lt => {
                    // We are sitting at token_start, but haven't consumed '<' yet.
                    self.bump_char(); // consume '<'

                    let kind = if self.peek_char() == Some('=') {
                        self.bump_char();
                        TokenKind::LessEqual
                    } else {
                        TokenKind::Less
                    };

                    let span = Span::new(self.token_start, self.pos);
                    self.state = Start;
                    return Ok(Token { kind, span });
                }
            }
        }
    }

    // ---------- helpers ----------

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_whitespace() {
                self.bump_char();
            } else {
                break;
            }
        }
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}
