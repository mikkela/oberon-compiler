use std::str::CharIndices;
use thiserror::Error;
use crate::frontend::span::{Position, Span};
use crate::frontend::token::{Token, TokenKind};

#[derive(Debug, Error)]
pub enum LexerError {
    #[error("Unexpected character '{ch}' at {span:?}")]
    UnexpectedCharacter {
        ch: char,
        span: Span,
    },

    #[error("Unterminated string literal at {span:?}")]
    UnterminatedString {
        span: Span,
    },

    #[error("Unterminated comment at {span:?}")]
    UnterminatedComment {
        span: Span,
    },

    #[error("Invalid number literal at {span:?}")]
    InvalidNumber {
        span: Span,
    },

    #[error("Unexpected end of file at {span:?}")]
    UnexpectedEof{
        span: Span,
    }
}

struct Cursor<'a> {
    input: &'a str,
    chars: CharIndices<'a>,
    current: Option<(usize, char)>,
    next: Option<(usize, char)>,
    pos: Position,
}

impl<'a> Cursor<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = input.char_indices();
        let current = chars.next();
        let next = chars.next();

        Self {
            input,
            chars,
            current,
            next,
            pos: Position::initial(),
        }
    }
}

impl<'a> Cursor<'a> {
    fn peek(&self) -> Option<char> {
        self.current.map(|(_, ch)| ch)
    }

    fn peek_next(&self) -> Option<char> {
        self.next.map(|(_, ch)| ch)
    }

    fn position(&self) -> Position {
        self.pos
    }

    fn bump(&mut self) -> Option<char> {
        let (_, ch) = self.current?;

        match ch {
            '\n' => {
                self.pos.offset += ch.len_utf8();
                self.pos.line += 1;
                self.pos.column = 1;
            }
            _ => {
                self.pos.offset += ch.len_utf8();
                self.pos.column += 1;
            }
        }

        self.current = self.next;
        self.next = self.chars.next();

        Some(ch)
    }

    fn take_while<F>(&mut self, mut predicate: F) -> &'a str
    where
        F: FnMut(char) -> bool,
    {
        let start = self.pos.offset;

        while let Some(ch) = self.peek() {
            if predicate(ch) {
                self.bump();
            } else {
                break;
            }
        }

        &self.input[start..self.pos.offset]
    }

    fn slice_from(&self, start: Position) -> &'a str {
        &self.input[start.offset..self.pos.offset]
    }
}

pub struct Lexer<'a> {
    cursor: Cursor<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            cursor: Cursor::new(input),
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace()?;

        let start = self.cursor.position();

        match self.cursor.peek() {
            None => Ok(Token::new(TokenKind::Eof, "", Span::new(start, start))),

            Some(c) if c.is_ascii_alphabetic() => self.lex_identifier_or_keyword(start),
            Some(c) if c.is_ascii_digit() => self.lex_digits(start),
            Some(c) if c == '"' => self.lex_string(start),
            Some(c) if self.is_symbol(c) => self.lex_symbol(start),
            Some(c) =>
                {
                    let start = self.cursor.position();
                    self.cursor.bump();
                    let end = self.cursor.position();
                    Err(LexerError::UnexpectedCharacter {
                        ch: c,
                        span: Span::new(start, end),
                    })
                }
        }
    }

    fn skip_whitespace(&mut self) -> Result<(), LexerError> {
        loop {
            self.cursor.take_while(|c| c.is_ascii_whitespace());

            if !self.starts_comment() {
                break;
            }

            self.skip_comment()?;
        }

        Ok(())
    }

    fn starts_comment(&self) -> bool {
        self.cursor.peek() == Some('(')
            && self.cursor.peek_next() == Some('*')
    }

    fn ends_comment(&self) -> bool {
        self.cursor.peek() == Some('*')
            && self.cursor.peek_next() == Some(')')
    }

    fn skip_comment(&mut self) -> Result<(), LexerError> {
        let start = self.cursor.position();
        self.cursor.bump();
        self.cursor.bump();
        let end = self.cursor.position();

        let mut depth = 1;

        while let Some(_c) = self.cursor.peek() {
            if self.starts_comment() {
                self.cursor.bump();
                self.cursor.bump();
                depth += 1;
            } else if self.ends_comment() {
                self.cursor.bump();
                self.cursor.bump();
                depth -= 1;

                if depth == 0 {
                    return Ok(());
                }
            } else {
                self.cursor.bump();
            }
        }

        Err(LexerError::UnterminatedComment { span: Span::new(start, end) })
    }

    fn lex_digits(&mut self, start: Position) -> Result<Token, LexerError> {
        let mut saw_hex_letter = false;

        while let Some(c) = self.cursor.peek() {
            if c.is_ascii_digit() {
                self.cursor.bump();
            } else if matches!(c, 'A'..='F') {
                saw_hex_letter = true;
                self.cursor.bump();
            } else {
                break;
            }
        }

        match self.cursor.peek() {
            Some('H') => {
                self.cursor.bump();

                Ok(Token::new(
                    TokenKind::Number,
                    self.cursor.slice_from(start),
                    Span::new(start, self.cursor.position()),
                ))
            }

            Some('X') => {
                self.cursor.bump();

                Ok(Token::new(
                    TokenKind::String,
                    self.cursor.slice_from(start),
                    Span::new(start, self.cursor.position()),
                ))
            }

            Some('.') => {
                if saw_hex_letter {
                    return Err(LexerError::InvalidNumber {
                        span: Span::new(start, self.cursor.position()),
                    });
                }

                self.lex_real_after_digits(start)
            }

            _ => {
                if saw_hex_letter {
                    return Err(LexerError::InvalidNumber {
                        span: Span::new(start, self.cursor.position()),
                    });
                }

                Ok(Token::new(
                    TokenKind::Number,
                    self.cursor.slice_from(start),
                    Span::new(start, self.cursor.position()),
                ))
            }
        }
    }

    fn lex_real_after_digits(&mut self, start: Position) -> Result<Token, LexerError> {
        self.cursor.bump(); // consume '.'

        self.cursor.take_while(|c| c.is_ascii_digit());

        if self.cursor.peek() == Some('E') {
            self.cursor.bump();

            if matches!(self.cursor.peek(), Some('+') | Some('-')) {
                self.cursor.bump();
            }

            let exponent_start = self.cursor.position();
            let digits = self.cursor.take_while(|c| c.is_ascii_digit());

            if digits.is_empty() {
                return Err(LexerError::InvalidNumber {
                    span: Span::new(exponent_start, self.cursor.position()),
                });
            }
        }

        Ok(Token::new(
            TokenKind::Number,
            self.cursor.slice_from(start),
            Span::new(start, self.cursor.position()),
        ))
    }

    fn lex_string(&mut self, start: Position) -> Result<Token, LexerError> {
        self.cursor.bump(); // consume opening quote

        while let Some(ch) = self.cursor.peek() {
            if ch == '"' {
                self.cursor.bump(); // consume closing quote
                let span = Span::new(start, self.cursor.position());
                let lexeme = self.cursor.slice_from(start);
                return Ok(Token::new(TokenKind::String, lexeme, span));
            }
            if ch.is_ascii_whitespace() && ch != ' ' && ch != '\t' {
                break;
            }
            self.cursor.bump();
        }

        Err(LexerError::UnterminatedString {
            span: Span::new(start, self.cursor.position()),
        })
    }

    fn lex_symbol(&mut self, start: Position) -> Result<Token, LexerError> {
        let Some(ch) = self.cursor.peek() else {
            return Err(LexerError::UnexpectedEof {
                span: Span::new(start, self.cursor.position()),
            });
        };
        self.cursor.bump();
        let lexeme = self.cursor.slice_from(start);

        match ch {
            '+' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '-' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '*' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '/' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '~' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '&' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '.' =>
                if self.cursor.peek() == Some('.') {
                    self.cursor.bump();
                    let lexeme = self.cursor.slice_from(start);
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                } else {
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                }
            ',' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            ';' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '|' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '(' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '[' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '{' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            ':' =>
                if self.cursor.peek() == Some('=') {
                    self.cursor.bump();
                    let lexeme = self.cursor.slice_from(start);
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                } else {
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                }
            '^' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '=' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '#' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '<' =>
                if self.cursor.peek() == Some('=') {
                    self.cursor.bump();
                    let lexeme = self.cursor.slice_from(start);
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                } else {
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                }
            '>' =>
                if self.cursor.peek() == Some('=') {
                    self.cursor.bump();
                    let lexeme = self.cursor.slice_from(start);
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                } else {
                    Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position())))
                }
            ')' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            ']' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            '}' => Ok(Token::new(TokenKind::OperatorOrDelimiter, lexeme, Span::new(start, self.cursor.position()))),
            _ => Err(LexerError::UnexpectedCharacter {
                ch,
                span: Span::new(start, self.cursor.position()),
            }),
        }

    }

    fn lex_identifier(&mut self, start: Position) -> (&str, Span) {
        self.cursor.bump();
        self.cursor.take_while(|c| c.is_ascii_alphanumeric() );
        let end = self.cursor.position();

        (self.cursor.slice_from(start), Span::new(start, end))
    }

    fn lex_identifier_or_keyword(&mut self, start: Position) -> Result<Token, LexerError> {
        let (lexeme, span) = self.lex_identifier(start);

        let kind = Self::keyword_kind(lexeme)
            .unwrap_or(TokenKind::Identifier);

        Ok(Token::new(kind, lexeme, span))
    }

    fn keyword_kind(s: &str) -> Option<TokenKind> {
        match s {
            "ARRAY" | "BEGIN" | "BY" | "CASE" | "CONST" | "DIV" |
            "DO" | "ELSE" | "ELSIF" | "END" | "FALSE" | "FOR" |
            "IF" | "IMPORT" | "IN" | "IS" | "MOD" | "MODULE" |
            "NIL" | "OF" | "OR" | "POINTER" | "PROCEDURE" |
            "RECORD" | "REPEAT" | "RETURN" | "THEN" | "TO" |
            "TRUE" | "TYPE" | "UNTIL" | "VAR" | "WHILE" => {
                Some(TokenKind::OperatorOrDelimiter)
            }
            _ => None,
        }
    }

    fn is_symbol(&self, c: char) -> bool {
        match c {
            '+' | '-' | '*' | '/' | '~' | '&' | '.' | ',' | ';' | '|' | '(' | '[' | '{' | ':' | '^' | '=' | '#' | '<' | '>' | ')' | ']' | '}' => true,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<TokenKind> {
        let mut lexer = Lexer::new(input);
        let mut result = Vec::new();

        loop {
            let token = lexer.next_token().unwrap();
            let kind = token.kind.clone();

            result.push(kind.clone());

            if kind == TokenKind::Eof {
                break;
            }
        }

        result
    }

    fn lexemes(input: &str) -> Vec<String> {
        let mut lexer = Lexer::new(input);
        let mut result = Vec::new();

        loop {
            let token = lexer.next_token().unwrap();
            result.push(token.lexeme.to_string());

            if token.kind == TokenKind::Eof {
                break;
            }
        }

        result
    }

    #[test]
    fn lexes_identifier() {
        let mut lexer = Lexer::new("foo123");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "foo123");
    }

    #[test]
    fn lexes_keywords() {
        let input = "MODULE BEGIN END VAR PROCEDURE IF THEN ELSE WHILE DO";
        let tokens = kinds(input);

        assert_eq!(
            tokens,
            vec![
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::OperatorOrDelimiter,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_integer_number() {
        let mut lexer = Lexer::new("12345");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Number);
        assert_eq!(token.lexeme, "12345");
    }

    #[test]
    fn lexes_hex_integer_number() {
        let mut lexer = Lexer::new("12AFH");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Number);
        assert_eq!(token.lexeme, "12AFH");
    }

    #[test]
    fn lexes_hex_string() {
        let mut lexer = Lexer::new("12AFX");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "12AFX");
    }

    #[test]
    fn lexes_real_number() {
        let mut lexer = Lexer::new("123.45");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Number);
        assert_eq!(token.lexeme, "123.45");
    }

    #[test]
    fn lexes_real_number_with_scale_factor() {
        let mut lexer = Lexer::new("123.45E-6");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Number);
        assert_eq!(token.lexeme, "123.45E-6");
    }

    #[test]
    fn rejects_hex_digits_without_h_or_x() {
        let mut lexer = Lexer::new("12AF");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::InvalidNumber { .. }));
    }

    #[test]
    fn rejects_real_with_hex_digits() {
        let mut lexer = Lexer::new("12AF.3");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::InvalidNumber { .. }));
    }

    #[test]
    fn rejects_real_scale_factor_without_digits() {
        let mut lexer = Lexer::new("123.45E+");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::InvalidNumber { .. }));
    }

    #[test]
    fn lexes_string_literal() {
        let mut lexer = Lexer::new("\"hello\"");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "\"hello\"");
    }

    #[test]
    fn lexes_string_literal_with_blanks() {
        let mut lexer = Lexer::new("\"\thello \"");

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::String);
        assert_eq!(token.lexeme, "\"\thello \"");
    }

    #[test]
    fn rejects_string_literal_with_newline() {
        let mut lexer = Lexer::new("\"hello\n\"");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::UnterminatedString { .. }));
    }

    #[test]
    fn rejects_unterminated_string_literal() {
        let mut lexer = Lexer::new("\"hello");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::UnterminatedString { .. }));
    }

    #[test]
    fn skips_whitespace() {
        let input = "   \n\t  foo";
        let mut lexer = Lexer::new(input);

        let token = lexer.next_token().unwrap();

        assert_eq!(token.kind, TokenKind::Identifier);
        assert_eq!(token.lexeme, "foo");
    }

    #[test]
    fn lexes_single_char_symbols() {
        let input = "+ - * / ~ & . , ; | ( [ { ^ = # ) ] }";
        let result = lexemes(input);

        assert_eq!(
            result,
            vec![
                "+", "-", "*", "/", "~", "&", ".", ",", ";", "|",
                "(", "[", "{", "^", "=", "#", ")", "]", "}", ""
            ]
        );
    }

    #[test]
    fn lexes_double_char_symbols() {
        let input = ".. := <= >=";
        let result = lexemes(input);

        assert_eq!(result, vec!["..", ":=", "<=", ">=", ""]);
    }

    #[test]
    fn returns_unexpected_character() {
        let mut lexer = Lexer::new("@");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(
            err,
            LexerError::UnexpectedCharacter { ch: '@', .. }
        ));
    }

    #[test]
    fn lexes_small_module() {
        let input = r#"
            MODULE Test;
            VAR x: INTEGER;
            BEGIN
                x := 123;
            END Test.
        "#;

        let result = lexemes(input);

        assert_eq!(
            result,
            vec![
                "MODULE", "Test", ";",
                "VAR", "x", ":", "INTEGER", ";",
                "BEGIN",
                "x", ":=", "123", ";",
                "END", "Test", ".",
                ""
            ]
        );
    }

    #[test]
    fn lexes_comments_are_ignored() {
        let input = r#"
            MODULE Test;
            VAR x: INTEGER;
            BEGIN
                (*x := 123;*)
                x := 127;
            END Test.
        "#;

        let result = lexemes(input);

        assert_eq!(
            result,
            vec![
                "MODULE", "Test", ";",
                "VAR", "x", ":", "INTEGER", ";",
                "BEGIN",
                "x", ":=", "127", ";",
                "END", "Test", ".",
                ""
            ]
        );
    }

    #[test]
    fn lexes_nested_comments_are_ignored() {
        let input = r#"
            MODULE Test;
            VAR x: INTEGER;
            BEGIN
                (* (* x := 123; *) *)
                x := 127;
            END Test.
        "#;

        let result = lexemes(input);

        assert_eq!(
            result,
            vec![
                "MODULE", "Test", ";",
                "VAR", "x", ":", "INTEGER", ";",
                "BEGIN",
                "x", ":=", "127", ";",
                "END", "Test", ".",
                ""
            ]
        );
    }

    #[test]
    fn rejects_unterminated_comment() {
        let mut lexer = Lexer::new("(* hello)");

        let err = lexer.next_token().unwrap_err();

        assert!(matches!(err, LexerError::UnterminatedComment { .. }));
    }
}