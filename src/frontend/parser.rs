use crate::frontend::ast::{ConstDeclaration, Declarations, Expression, Identifier, IdentifierDef, Module, StatementSequence};
use crate::frontend::lexer::{Lexer, LexerError};
use thiserror::Error;
use crate::frontend::span::Span;
use crate::frontend::token::{Token, TokenKind};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Unexpected token: '{token:?}'")]
    UnexpectedToken { token: Token },

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error(transparent)]
    Lexer(#[from] LexerError),

    #[error(transparent)]
    InvalidIntegerNumber(#[from] std::num::ParseIntError),

    #[error(transparent)]
    InvalidRealNumber(#[from] std::num::ParseFloatError),
}

pub struct TokenStream<'a> {
    lexer: Lexer<'a>,
    current: Token,
}

impl<'a> TokenStream<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            current: Token::invalid(),
        }
    }

    pub fn current(&self) -> &Token {
        &self.current
    }

    pub fn advance(&mut self) -> Result<(), ParserError> {
        self.current = self.lexer.next_token()?;
        Ok(())
    }
}

pub struct Parser<'a> {
    token_stream: TokenStream<'a>
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self { token_stream: TokenStream::new(lexer) }
    }
}

macro_rules! pred {
    (ASSIGN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "=" };
    (BEGIN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "BEGIN" };
    (CONST) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "CONST" };
    (DOT) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "." };
    (END) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "END" };
    (IDENT) => { |token: &Token| token.kind == TokenKind::Identifier };
    (MODULE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "MODULE" };
    (NUMBER) => { |token: &Token| token.kind == TokenKind::Number };
    (PROCEDURE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "PROCEDURE" };
    (SEMICOLON) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ";" };
    (STRING) => { |token: &Token| token.kind == TokenKind::String };
    (TYPE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "TYPE" };
    (VAR) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "VAR" };
}
impl<'a> Parser<'a> {

    pub fn parse(&mut self) -> Result<Module, ParserError> {
        self.token_stream.advance()?;
        self.parse_module()
    }

    fn parse_module(&mut self)-> Result<Module, ParserError> {
        let start = self.expect(pred!(MODULE))?.span;
        let module_name = self.parse_ident()?;
        self.expect(pred!(SEMICOLON))?;
        let declarations = self.parse_declarations()?;
        self.expect(pred!(END))?;
        let end_name = self.parse_ident()?;
        let end = self.expect(pred!(DOT))?.span;
        Ok(Module {
            name: module_name,
            declarations,
            end_name,
            span: Span::new(start.start, end.end),
            stmts: StatementSequence { statements: vec![], span: Span::default() },
        })
    }

    fn parse_declarations(&mut self) -> Result<Declarations, ParserError> {
        let mut const_declarations = vec![];
        if self.eat(pred!(CONST))?.is_some() {
            const_declarations = self.parse_const_declarations()?;
        }
        Ok(Declarations { const_declarations })
    }

    fn parse_const_declarations(&mut self) -> Result<Vec<ConstDeclaration>, ParserError> {
        let mut result = vec![];
        while self.peek(|t| {
            pred!(TYPE)(t) || pred!(VAR)(t) || pred!(PROCEDURE)(t) || pred!(BEGIN)(t) || pred!(END)(t)
        }).is_none() {
            result.push(self.parse_const_declaration()?);
            self.expect(pred!(SEMICOLON))?;
        }

        Ok(result)
    }

    fn parse_const_declaration(&mut self) -> Result<ConstDeclaration, ParserError> {
        let ident = self.parse_identdef()?;
        self.expect(pred!(ASSIGN))?;
        let value = self.parse_expression()?;

        Ok(ConstDeclaration { ident, value })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_simple_expression()
    }

    fn parse_simple_expression(&mut self) -> Result<Expression, ParserError> {
        self.parse_term()
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        self.parse_factor()
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        if self.peek(pred!(NUMBER)).is_some() {
            self.parse_number()
        }
        else if self.peek(pred!(STRING)).is_some() {
            self.parse_string()
        }
        else {
            Err(ParserError::UnexpectedToken { token: self.token_stream.current().clone() })
        }
    }

    fn parse_number(&mut self) -> Result<Expression, ParserError> {
        let token = self.expect(pred!(NUMBER))?;
        if token.lexeme.ends_with("H") {
            let hex = token.lexeme.strip_suffix("H").unwrap();
            Ok(Expression::Int { value: i64::from_str_radix(hex, 16)?, span: token.span })
        } else if  token.lexeme.contains('.') {
            Ok(Expression::Real{ value: token.lexeme.parse::<f64>()?, span: token.span })
        }
        else {
            Ok(Expression::Int { value: token.lexeme.parse::<i64>()?, span: token.span })
        }
    }

    fn parse_string(&mut self) -> Result<Expression, ParserError> {
        let token = self.expect(pred!(STRING))?;
        if token.lexeme.ends_with("X") {
            let hex = token.lexeme.strip_suffix("X").unwrap();
            let byte_value = u8::from_str_radix(hex, 16)?;
            Ok(Expression::String { value: (byte_value as char).to_string(), span: token.span })
        } else {
            Ok(Expression::String { value:
            token.lexeme.strip_prefix('\"').unwrap().strip_suffix('\"').unwrap().to_string(),
                span: token.span })
        }

    }

    fn parse_identdef(&mut self) -> Result<IdentifierDef, ParserError> {
        Ok(IdentifierDef { ident: self.parse_ident()?, exported: false, span: self.token_stream.current().span })
    }

    fn parse_ident(&mut self) -> Result<Identifier, ParserError> {
        let token = self.expect(pred!(IDENT))?;
        Ok(Identifier { text: token.lexeme, span: token.span })
    }

    fn expect<F>(&mut self, predicate: F) -> Result<Token, ParserError>
    where
        F: Fn(&Token) -> bool,
    {
        let token = self.token_stream.current();

        if predicate(token) {
            let token = token.clone();
            self.token_stream.advance()?;
            Ok(token)
        } else {
            Err(ParserError::UnexpectedToken{token: token.clone()})
        }
    }

    fn eat<F>(&mut self, predicate: F) -> Result<Option<Token>, ParserError>
    where
        F: Fn(&Token) -> bool,
    {
        let token = self.token_stream.current();

        if predicate(token) {
            let token = token.clone();
            self.token_stream.advance()?;
            Ok(Some(token))
        } else {
            Ok(None)
        }
    }

    fn peek<F>(&mut self, predicate: F) -> Option<Token>
    where
        F: Fn(&Token) -> bool,
    {
        let token = self.token_stream.current();

        if predicate(token) {
            Some(token.clone())
        } else {
            None
        }
    }


}

#[cfg(test)]
mod tests {
    use crate::frontend::ast::Module;
    use crate::frontend::lexer::Lexer;
    use crate::frontend::parser::Parser;
    use crate::frontend::span::{Position, Span};
    use crate::frontend::token::TokenKind;


    use crate::frontend::token::Token;

    // ---------- parse ----------
    pub fn parse(module: &str) -> Module {
        let mut p = Parser::new(Lexer::new(module));
        p.parse().unwrap()
    }

    mod expressions {
        use crate::frontend::ast::{ConstDeclaration, Expression};
        use super::*;

        #[test]
        fn parse_integer_const() {
            let module = parse("MODULE m; CONST foo=1987; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Int { value: 1987, .. } = value else { panic!("integer"); };
        }

        #[test]
        fn parse_hex_const() {
            let module = parse("MODULE m; CONST foo=100H; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Int { value: 256, .. } = value else { panic!("hex"); };
        }

        #[test]
        fn parse_real_const() {
            let module = parse("MODULE m; CONST foo=12.3; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Real { value: 12.3, .. } = value else { panic!("real"); };
        }

        #[test]
        fn parse_real_exponent_const() {
            let module = parse("MODULE m; CONST foo=4.567E8; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Real { value: 456700000.0, .. } = value else { panic!("real exponent"); };
        }

        #[test]
        fn parse_string_const() {
            let module = parse("MODULE m; CONST foo=\"OBERON\"; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::String { value, .. } = value else { panic!("string"); };
            assert_eq!(value, "OBERON");
        }

        #[test]
        fn parse_string_character_const() {
            let module = parse("MODULE m; CONST foo=20X; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::String { value, .. } = value else { panic!("character"); };
            assert_eq!(value, " ");
        }
    }
}