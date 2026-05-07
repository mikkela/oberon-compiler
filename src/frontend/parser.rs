use crate::frontend::ast::{BinaryOperation, ConstDeclaration, Declarations, Designator, Element, Expression, FPSection, FieldList, FormalParameters, FormalType, Identifier, IdentifierDef, Module, QualifiedIdentifier, Selector, StatementSequence, Type, TypeDeclaration, UnaryOperation};
use crate::frontend::lexer::{Lexer, LexerError};
use crate::frontend::span::{Span, Spanned};
use crate::frontend::token::{Token, TokenKind};
use std::collections::VecDeque;
use thiserror::Error;

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
    lookahead: VecDeque<Token>,
}

impl<'a> TokenStream<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            current: Token::invalid(),
            lookahead: VecDeque::new(),
        }
    }

    pub fn current(&self) -> &Token {
        &self.current
    }

    pub fn peek_n(&mut self, n: usize) -> Vec<&Token> {
        while self.lookahead.len() < n {
            match self.lexer.next_token() {
                Ok(token) => self.lookahead.push_back(token),
                Err(_) => break,
            }
        }

        self.lookahead.iter().take(n).collect()
    }

    pub fn advance(&mut self) -> Result<(), ParserError> {
        if let Some(next) = self.lookahead.pop_front() {
            self.current = next;
        } else {
            self.current = self.lexer.next_token()?;
        }
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
    (AMPERSAND) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "&" };
    (ARRAY) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "ARRAY" };
    (ASSIGN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "=" };
    (BEGIN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "BEGIN" };
    (CARET) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "^" };
    (COLON) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ":" };
    (COMMA) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "," };
    (CONST) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "CONST" };
    (DIV) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "DIV" };
    (DOT) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "." };
    (DOTDOT) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ".." };
    (END) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "END" };
    (EQUAL) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "=" };
    (FALSE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "FALSE" };
    (GREATER) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ">" };
    (GREATEREQUAL) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ">=" };
    (IDENT) => { |token: &Token| token.kind == TokenKind::Identifier };
    (IN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "IN" };
    (IS) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "IS" };
    (LBRACKET) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "[" };
    (LCURLY) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "{" };
    (LESS) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "<" };
    (LESSEQUAL) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "<=" };
    (LPAREN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "(" };
    (MINUS) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "-" };
    (MOD) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "MOD" };
    (MODULE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "MODULE" };
    (NIL) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "NIL" };
    (NONEQUAL) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "#" };
    (NUMBER) => { |token: &Token| token.kind == TokenKind::Number };
    (OF) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "OF" };
    (OR) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "OR" };
    (PLUS) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "+" };
    (POINTER) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "POINTER" };
    (PROCEDURE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "PROCEDURE" };
    (RBRACKET) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "]" };
    (RCURLY) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "}" };
    (RECORD) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "RECORD" };
    (RPAREN) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ")" };
    (SEMICOLON) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == ";" };
    (SLASH) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "/" };
    (STAR) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "*" };
    (STRING) => { |token: &Token| token.kind == TokenKind::String };
    (TILDE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "~" };
    (TO) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "TO" };
    (TRUE) => { |token: &Token| token.kind == TokenKind::OperatorOrDelimiter && token.lexeme == "TRUE" };
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
        let mut type_declarations = vec![];
        if self.eat(pred!(TYPE))?.is_some() {
            type_declarations = self.parse_type_declarations()?;
        }
        Ok(Declarations { const_declarations, type_declarations })
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

    fn parse_type_declarations(&mut self) -> Result<Vec<TypeDeclaration>, ParserError> {
        let mut result = vec![];
        while self.peek(|t| {
            pred!(VAR)(t) || pred!(PROCEDURE)(t) || pred!(BEGIN)(t) || pred!(END)(t)
        }).is_none() {
            result.push(self.parse_type_declaration()?);
            self.expect(pred!(SEMICOLON))?;
        }

        Ok(result)
    }

    fn parse_type_declaration(&mut self) -> Result<TypeDeclaration, ParserError> {
        let ident = self.parse_identdef()?;
        self.expect(pred!(ASSIGN))?;
        let ty = self.parse_type()?;

        Ok(TypeDeclaration { ident, ty })
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>, ParserError> {
        let mut result = vec![self.parse_expression()?];
        while self.eat(pred!(COMMA))?.is_some() {
            result.push(self.parse_expression()?);
        }
        Ok(result)
    }

    fn parse_binary_operator(token: Token) -> BinaryOperation {
        match token.lexeme.as_str() {
            "=" => BinaryOperation::Eq,
            "#" => BinaryOperation::Neq,
            "<" => BinaryOperation::Lt,
            "<=" => BinaryOperation::Le,
            ">" => BinaryOperation::Gt,
            ">=" => BinaryOperation::Ge,
            "*" => BinaryOperation::Multiplication,
            "/" => BinaryOperation::Division,
            "+" => BinaryOperation::Addition,
            "-" => BinaryOperation::Subtraction,
            "MOD" => BinaryOperation::Mod,
            "DIV" => BinaryOperation::Div,
            "&" => BinaryOperation::And,
            "OR" => BinaryOperation::Or,
            "IN" => BinaryOperation::In,
            "IS" => BinaryOperation::Is,
            _ => panic!("Invalid binary operator: {}", token.lexeme),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        let simple_expression = self.parse_simple_expression()?;
        if let Some(token) = self.eat(|t| pred!(EQUAL)(t)
            || pred!(NONEQUAL)(t)
            || pred!(LESS)(t)
            || pred!(LESSEQUAL)(t)
            || pred!(GREATER)(t)
            || pred!(GREATEREQUAL)(t)
            || pred!(IN)(t)
            || pred!(IS)(t)
        )? {
            let rhs = self.parse_simple_expression()?;
            let span = Span::new(simple_expression.span().start, rhs.span().end);
            Ok(Expression::Binary {
                op: Self::parse_binary_operator(token),
                lhs: Box::new(simple_expression), rhs: Box::new(rhs), span })
        }
        else {
            Ok(simple_expression)
        }

    }

    fn parse_simple_expression(&mut self) -> Result<Expression, ParserError> {
        let sign_token = self.eat(|t| pred!(PLUS)(t) || pred!(MINUS)(t))?;

        let mut expr = self.parse_term()?;

        if let Some(token) = sign_token {
            let span = Span::new(token.span.start, expr.span().end);
            let op = if pred!(PLUS)(&token) { UnaryOperation::Plus } else { UnaryOperation::Minus };
            expr = Expression::Unary { op, operand: Box::new(expr), span };
        }

        if let Some(token) = self.eat(|t| pred!(PLUS)(t) || pred!(MINUS)(t) || pred!(OR)(t))? {
            let rhs = self.parse_term()?;
            let span = Span::new(expr.span().start, rhs.span().end);

            expr = Expression::Binary {
                op: Self::parse_binary_operator(token),
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span,
            };
        }

        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expression, ParserError> {
        let factor = self.parse_factor()?;
        if let Some(token) = self.eat(|t| pred!(STAR)(t)
            || pred!(SLASH)(t)
            || pred!(MOD)(t)
            || pred!(DIV)(t)
            || pred!(AMPERSAND)(t)
        )? {
            let rhs = self.parse_factor()?;
            let span = Span::new(factor.span().start, rhs.span().end);
            Ok(Expression::Binary {
                op: Self::parse_binary_operator(token),
                lhs: Box::new(factor), rhs: Box::new(rhs), span })
        }
        else {
            Ok(factor)
        }
    }

    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        if self.peek(pred!(NUMBER)).is_some() {
            self.parse_number()
        }
        else if self.peek(pred!(STRING)).is_some() {
            self.parse_string()
        }
        else if let Some(token) = self.eat(pred!(NIL))? {
            Ok(Expression::Nil { span: token.span })
        }
        else if let Some(token) = self.eat(pred!(TRUE))? {
            Ok(Expression::True { span: token.span })
        }
        else if let Some(token) = self.eat(pred!(FALSE))? {
            Ok(Expression::False { span: token.span })
        }
        else if self.peek(pred!(LCURLY)).is_some() {
            self.parse_set()
        }
        else if let Some(start) = self.peek(pred!(IDENT)) {
            let designator = self.parse_designator()?;
            let actual_parameters =
                if self.peek(pred!(LPAREN)).is_some() {
                    Some(self.parse_actual_parameters()?)
                } else {
                    None
                };

            let end = self.token_stream.current();
            Ok(Expression::Designator { designator, actual_parameters, span: Span::new(start.span.start, end.span.end) })
        }
        else if self.eat(pred!(LPAREN))?.is_some() {
            let expr = self.parse_expression()?;
            self.expect(pred!(RPAREN))?;
            Ok(expr)
        }
        else if let Some(token) = self.eat(pred!(TILDE))? {
            let operand = self.parse_factor()?;
            Ok(Expression::Unary { op: UnaryOperation::Not, operand: Box::new(operand), span: token.span })
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

    fn parse_set(&mut self) -> Result<Expression, ParserError> {
        let start =self.expect(pred!(LCURLY))?;
        let mut elements = vec![];
        while self.peek(pred!(RCURLY)).is_none() {
            if !elements.is_empty() {
                self.expect(pred!(COMMA))?;
            }
            elements.push(self.parse_element()?);
        }
        let end = self.expect(pred!(RCURLY))?;
        Ok(Expression::Set { elements, span: Span::new(start.span.start, end.span.end) })
    }

    fn parse_element(&mut self) -> Result<Element, ParserError> {
        let first = self.parse_expression()?;
        if self.eat(pred!(DOTDOT))?.is_some() {
            let second = self.parse_expression()?;
            let span = Span::new(first.span().start, second.span().end);
            Ok(Element { first, second: Some(second), span })
        } else {
            let span = first.span();
            Ok(Element { first, second: None, span })
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParserError> {
        if self.peek(pred!(IDENT)).is_some() {
            Ok(Type::Named { name: self.parse_qualident()? })
        } else if let Some(start) = self.eat(pred!(ARRAY))? {
            let lengths = self.parse_lengths()?;
            self.expect(pred!(OF))?;
            let element = Box::new(self.parse_type()?);
            let span = Span::new(start.span.start, self.token_stream.current().span.end);
            Ok(Type::Array { lengths, element, span })
        } else if let Some(start) = self.eat(pred!(RECORD))? {
            let base = self.parse_base_type()?;
            let field_lists = self.parse_field_lists()?;
            self.expect(pred!(END))?;
            let span = Span::new(start.span.start, self.token_stream.current().span.end);
            Ok(Type::Record { base, field_lists, span })
        } else if let Some(start) = self.eat(pred!(POINTER))? {
            self.expect(pred!(TO))?;
            let pointee = self.parse_type()?;
            let span = Span::new(start.span.start, pointee.span().end);
            Ok(Type::Pointer { pointee: Box::new(pointee), span })
        } else if let Some(start) = self.eat(pred!(PROCEDURE))? {
            let params = self.parse_formal_parameters()?;
            let span = Span::new(start.span.start, self.token_stream.current().span.end);
            Ok(Type::Procedure { params , span })
        }
        else {
            Err(ParserError::UnexpectedToken { token: self.token_stream.current().clone() })
        }
    }

    fn parse_lengths(&mut self) -> Result<Vec<Expression>, ParserError> {
        let mut result = vec![self.parse_expression()?];
        while self.eat(pred!(COMMA))?.is_some() {
            result.push(self.parse_expression()?);
        }
        Ok(result)
    }

    fn parse_base_type(&mut self) -> Result<Option<QualifiedIdentifier>, ParserError> {
        if self.eat(pred!(LPAREN))?.is_some() {
            let element = self.parse_qualident()?;
            self.eat(pred!(RPAREN))?;
            Ok(Some(element))
        } else {
            Ok(None)
        }
    }

    fn parse_field_lists(&mut self) -> Result<Vec<FieldList>, ParserError> {
        let mut result = vec![];
        while self.peek(pred!(END)).is_none() {
            if !result.is_empty() {
                self.expect(pred!(SEMICOLON))?;
            }
            result.push(self.parse_field_list()?);
        }
        Ok(result)
    }

    fn parse_field_list(&mut self) -> Result<FieldList, ParserError> {
        let fields = self.parse_identdef_list()?;
        self.expect(pred!(COLON))?;
        let ty = self.parse_type()?;
        Ok(FieldList{ fields, ty })
    }

    fn parse_formal_parameters(&mut self) -> Result<Option<FormalParameters>, ParserError> {
        let start_span = self.expect(pred!(LPAREN))?.span;
        let sections = self.parse_fp_sections()?;
        let mut end_span = self.expect(pred!(RPAREN))?.span;
        let return_type =
            if self.eat(pred!(COLON))?.is_some() {
                let qualident = self.parse_qualident()?;
                end_span = qualident.span();
                Some(qualident)
            } else { None };

        if sections.is_empty() && return_type.is_none() {
            Ok (None)
        } else {
            let span = Span::new(start_span.start, end_span.end);
            Ok(Some(FormalParameters { sections, return_type, span }))
        }
    }

    fn parse_fp_sections(&mut self) -> Result<Vec<FPSection>, ParserError> {
        let mut result = vec![];
        while self.peek(pred!(RPAREN)).is_none() {
            if !result.is_empty() { self.expect(pred!(SEMICOLON))?; }
            result.push(self.parse_fp_section()?);
        }
        Ok(result)
    }

    fn parse_fp_section(&mut self) -> Result<FPSection, ParserError> {
        let start_span = self.token_stream.current.span;
        let by_ref = self.eat(pred!(VAR))?.is_some();
        let names = self.parse_ident_list()?;
        self.expect(pred!(COLON));
        let ty = self.parse_formal_type()?;
        let span = Span::new(start_span.start, ty.span.end);
        Ok(FPSection{ names, by_ref, ty, span })
    }

    fn parse_formal_type(&mut self) -> Result<FormalType, ParserError> {
        let start_span = self.token_stream.current.span;
        let mut open_arrays = 0;
        while self.eat(pred!(ARRAY))?.is_some() {
            self.expect(pred!(OF))?;
            open_arrays += 1;
        }
        let base = self.parse_qualident()?;
        let span = Span::new(start_span.start, base.span().end);
        Ok(FormalType{ open_arrays, base, span })
    }

    fn parse_designator(&mut self) -> Result<Designator, ParserError> {
        let start = self.token_stream.current().span;
        let head = self.parse_qualident()?;
        let selectors = self.parse_selectors()?;
        let end = self.token_stream.current().span;
        Ok(Designator{ head, selectors, span: Span::new(start.start, end.end)})
    }

    fn parse_qualident(&mut self) -> Result<QualifiedIdentifier, ParserError> {
        let mut parts = vec![self.parse_ident()?];
        if self.eat(pred!(DOT))?.is_some() {
            parts.push(self.parse_ident()?);
        }
        Ok(QualifiedIdentifier{ parts })
    }

    fn parse_selectors(&mut self) -> Result<Vec<Selector>, ParserError> {
        let mut result = vec![];
        while self.peek(|t| pred!(DOT)(t)
            || pred!(LBRACKET)(t)
            || pred!(CARET)(t)
        ).is_some() || self.type_guard_selector() {
            result.push(self.parse_selector()?);
        }
        Ok(result)
    }

    fn type_guard_selector(&mut self) -> bool {
        if !pred!(LPAREN)(self.token_stream.current()) {
            return false;
        }
        match self.token_stream.peek_n(4).as_slice() {
            [t0, t1, ..] if pred!(IDENT)(t0) && pred!(RPAREN)(t1) => true,

            [t0, t1, t2, t3]
            if pred!(IDENT)(t0) && pred!(DOT)(t1) && pred!(IDENT)(t2) && pred!(RPAREN)(t3) => true,

            _ => false,
        }
    }

    fn parse_selector(&mut self) -> Result<Selector, ParserError> {
        if self.eat(pred!(DOT))?.is_some() {
            Ok(Selector::Field(self.parse_ident()?))
        } else if let Some(start) =self.eat(pred!(LBRACKET))? {
            let index = self.parse_expression_list()?;
            let end = self.expect(pred!(RBRACKET))?;
            Ok(Selector::Index(index, Span::new(start.span.start, end.span.end)))
        } else if let Some(token) =self.eat(pred!(CARET))? {
            Ok(Selector::Deref(token.span))
        } else if let Some(start) =self.eat(pred!(LPAREN))? {
            let guard = self.parse_qualident()?;
            let end =self.expect(pred!(RPAREN))?;
            Ok(Selector::TypeGuard(guard, Span::new(start.span.start, end.span.end)))
        }
        else {
            Err(ParserError::UnexpectedToken { token: self.token_stream.current().clone() })
        }
    }

    fn parse_actual_parameters(&mut self) -> Result<Vec<Expression>, ParserError> {
        self.expect(pred!(LPAREN))?;
        if self.eat(pred!(RPAREN))?.is_some() {
            return Ok(vec![]);
        }
        let result = self.parse_expression_list()?;
        self.expect(pred!(RPAREN))?;
        Ok(result)
    }

    fn parse_identdef_list(&mut self) -> Result<Vec<IdentifierDef>, ParserError> {
        let mut result = vec![self.parse_identdef()?];
        while self.eat(pred!(COMMA))?.is_some() {
            result.push(self.parse_identdef()?);
        }
        Ok(result)
    }

    fn parse_identdef(&mut self) -> Result<IdentifierDef, ParserError> {
        let ident = self.parse_ident()?;
        let star = self.eat(pred!(STAR))?;
        if star.is_some() {
            let star = star.unwrap();
            let span = Span::new(ident.span.start, star.span.end);
            Ok(IdentifierDef {
                ident,
                exported: true,
                span,
            })
        } else {
            let span = ident.span;
            Ok(IdentifierDef {
                ident,
                exported: false,
                span,
            })
        }
    }

    fn parse_ident_list(&mut self) -> Result<Vec<Identifier>, ParserError> {
        let mut result = vec![self.parse_ident()?];
        while self.eat(pred!(COMMA))?.is_some() {
            result.push(self.parse_ident()?);
        }
        Ok(result)
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


    // ---------- parse ----------
    pub fn parse(module: &str) -> Module {
        let mut p = Parser::new(Lexer::new(module));
        p.parse().unwrap()
    }

    mod expressions {
        use super::*;
        use crate::frontend::ast::{BinaryOperation, ConstDeclaration, Expression, Selector, UnaryOperation};

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

        #[test]
        fn parse_nil() {
            let module = parse("MODULE m; CONST foo=NIL; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Nil {  .. } = value else { panic!("NIL"); };
        }

        #[test]
        fn parse_true() {
            let module = parse("MODULE m; CONST foo=TRUE; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::True {  .. } = value else { panic!("TRUE"); };
        }

        #[test]
        fn parse_false() {
            let module = parse("MODULE m; CONST foo=FALSE; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::False {  .. } = value else { panic!("FALSE"); };
        }

        #[test]
        fn parse_empty_set() {
            let module = parse("MODULE m; CONST foo={}; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Set { elements, .. } = value else { panic!("Empty Set"); };
            assert_eq!(elements.len(), 0);
        }

        #[test]
        fn parse_single_element_set() {
            let module = parse("MODULE m; CONST foo={TRUE}; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Set { elements, .. } = value else { panic!("Single Element Set"); };
            assert_eq!(elements.len(), 1);
        }

        #[test]
        fn parse_multiple_elements_set() {
            let module = parse("MODULE m; CONST foo={TRUE, FALSE}; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Set { elements, .. } = value else { panic!("Single Element Set"); };
            assert_eq!(elements.len(), 2);
        }

        #[test]
        fn parse_spanned_element_set() {
            let module = parse("MODULE m; CONST foo={ 1 .. 5 }; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Set { elements, .. } = value else { panic!("Single Element Set"); };
            assert_eq!(elements.len(), 1);
        }

        #[test]
        fn parse_simple_designator() {
            let module = parse("MODULE m; CONST foo=bar; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Simple Designator"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 0);
        }

        #[test]
        fn parse_compound_designator() {
            let module = parse("MODULE m; CONST foo=bar.baz; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Compound Designator"); };
            assert_eq!(designator.head.parts.len(), 2);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.head.parts[1].text, "baz");
            assert_eq!(designator.selectors.len(), 0);
        }

        #[test]
        fn parse_compound_designator_with_field_selector() {
            let module = parse("MODULE m; CONST foo=bar.baz.fez; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Compound Designator with field selector"); };
            assert_eq!(designator.head.parts.len(), 2);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.head.parts[1].text, "baz");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::Field (field) = selector else { panic!("Compound Designator with field selector and field"); };
            assert_eq!(field.text, "fez");
            assert_eq!(designator.head.parts.len(), 2);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.head.parts[1].text, "baz");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::Field (field) = selector else { panic!("Compound Designator with field selector and field"); };
            assert_eq!(field.text, "fez");
        }

        #[test]
        fn parse_simple_designator_with_single_index() {
            let module = parse("MODULE m; CONST foo=bar[1]; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Simple Designator with single index"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::Index (index, _) = selector else { panic!("Simple Designator with single index expression"); };
            assert_eq!(index.len(), 1);
            let value = &index[0];
            let Expression::Int { value: 1, .. } = value else { panic!("Simple Designator with single index expression value"); };
        }

        #[test]
        fn parse_simple_designator_with_multiple_indeces() {
            let module = parse("MODULE m; CONST foo=bar[1, 2, 3]; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Simple Designator with single index"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::Index (index, _) = selector else { panic!("Simple Designator with single index expression"); };
            assert_eq!(index.len(), 3);
            let mut value = &index[0];
            let Expression::Int { value: 1, .. } = value else { panic!("Simple Designator with single index expression value 1"); };
            value = &index[1];
            let Expression::Int { value: 2, .. } = value else { panic!("Simple Designator with single index expression value 2"); };
            value = &index[2];
            let Expression::Int { value: 3, .. } = value else { panic!("Simple Designator with single index expression value 3"); };
        }

        #[test]
        fn parse_simple_designator_with_caret_selector() {
            let module = parse("MODULE m; CONST foo=bar^; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Designator with caret selector"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::Deref (..) = selector else { panic!("Designator with caret selector and selector"); };
        }

        #[test]
        fn parse_simple_designator_with_simple_type_guard_selector() {
            let module = parse("MODULE m; CONST foo=bar(baz); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Designator with simple type guard selector"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::TypeGuard (type_guard, ..) = selector else { panic!("Designator with simple type guard selector and selector"); };
            assert_eq!(type_guard.parts.len(), 1);
            assert_eq!(type_guard.parts[0].text, "baz");
        }

        #[test]
        fn parse_simple_designator_with_compound_type_guard_selector() {
            let module = parse("MODULE m; CONST foo=bar(baz.fez); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, ..} = value else { panic!("Designator with compound type guard selector"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::TypeGuard (type_guard, ..) = selector else { panic!("Designator with compound type guard selector and selector"); };
            assert_eq!(type_guard.parts.len(), 2);
            assert_eq!(type_guard.parts[0].text, "baz");
            assert_eq!(type_guard.parts[1].text, "fez");
        }

        #[test]
        fn parse_simple_designator_with_empty_argumentsl() {
            let module = parse("MODULE m; CONST foo=bar(); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Simple designator with empty arguments"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 0);
            assert_eq!(actual_parameters.is_some(), true);
            let parameters = actual_parameters.clone().unwrap();
            assert_eq!(parameters.len(), 0);
        }

        #[test]
        fn parse_simple_designator_with_argumentsl() {
            let module = parse("MODULE m; CONST foo=bar(FALSE, TRUE); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Simple designator with arguments"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 0);
            assert_eq!(actual_parameters.is_some(), true);
            let parameters = actual_parameters.clone().unwrap();
            assert_eq!(parameters.len(), 2);
            let mut value = &parameters[0];
            let Expression::False {  .. } = value else { panic!("FALSE"); };
            value = &parameters[1];
            let Expression::True {  .. } = value else { panic!("TRUE"); };
        }

        #[test]
        fn parse_compound_designator_with_empty_argumentsl() {
            let module = parse("MODULE m; CONST foo=bar.baz(); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Compound designator with empty arguments"); };
            assert_eq!(designator.head.parts.len(), 2);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.head.parts[1].text, "baz");
            assert_eq!(designator.selectors.len(), 0);
            assert_eq!(actual_parameters.is_some(), true);
            let parameters = actual_parameters.clone().unwrap();
            assert_eq!(parameters.len(), 0);
        }

        #[test]
        fn parse_simple_designator_with_typeguard_and_empty_argumentsl() {
            let module = parse("MODULE m; CONST foo=bar(baz)(); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Compound designator with empty arguments"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::TypeGuard (type_guard, ..) = selector else { panic!("Designator with simple type guard selector and selector"); };
            assert_eq!(type_guard.parts.len(), 1);
            assert_eq!(type_guard.parts[0].text, "baz");
            assert_eq!(actual_parameters.is_some(), true);
            let parameters = actual_parameters.clone().unwrap();
            assert_eq!(parameters.len(), 0);
        }

        #[test]
        fn parse_simple_designator_with_multiple_typeguards() {
            let module = parse("MODULE m; CONST foo=bar(baz)(fez); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Compound designator with empty arguments"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 2);
            let selector = &designator.selectors[1];
            let Selector::TypeGuard (type_guard, ..) = selector else { panic!("Designator with simple type guard selector and selector"); };
            assert_eq!(type_guard.parts.len(), 1);
            assert_eq!(type_guard.parts[0].text, "fez");
            assert_eq!(actual_parameters.is_some(), false);
        }

        #[test]
        fn parse_simple_designator_with_type_guard_and_arguments() {
            let module = parse("MODULE m; CONST foo=bar(baz)(fez, 2); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Designator { designator, actual_parameters, ..} = value else { panic!("Compound designator with empty arguments"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "bar");
            assert_eq!(designator.selectors.len(), 1);
            let selector = &designator.selectors[0];
            let Selector::TypeGuard (type_guard, ..) = selector else { panic!("Designator with simple type guard selector and selector"); };
            assert_eq!(type_guard.parts.len(), 1);
            assert_eq!(type_guard.parts[0].text, "baz");
            assert_eq!(actual_parameters.is_some(), true);
            let parameters = actual_parameters.clone().unwrap();
            assert_eq!(parameters.len(), 2);
            let mut value = &parameters[0];
            let Expression::Designator { designator, .. } = value else { panic!("Designator"); };
            assert_eq!(designator.head.parts.len(), 1);
            assert_eq!(designator.head.parts[0].text, "fez");
            value = &parameters[1];
            let Expression::Int { value: 2, .. } = value else { panic!("TRUE"); };
        }

        #[test]
        fn parse_parenthesis() {
            let module = parse("MODULE m; CONST foo=(FALSE); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::False {  .. } = value else { panic!("FALSE"); };
        }

        #[test]
        fn parse_tilde() {
            let module = parse("MODULE m; CONST foo=~FALSE; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Unary { op: UnaryOperation::Not, operand,  .. } = value else { panic!("TILDE"); };
            let Expression::False {  .. } = &**operand else { panic!("TILDE"); };
        }

        #[test]
        fn parse_plus() {
            let module = parse("MODULE m; CONST foo=+2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Unary { op: UnaryOperation::Plus, operand,  .. } = value else { panic!("TILDE"); };
            let Expression::Int { value: 2,  .. } = &**operand else { panic!("PLUS"); };
        }

        #[test]
        fn parse_minus() {
            let module = parse("MODULE m; CONST foo= -2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Unary { op: UnaryOperation::Minus, operand,  .. } = value else { panic!("TILDE"); };
            let Expression::Int { value: 2,  .. } = &**operand else { panic!("MINUS"); };
        }

        #[test]
        fn parse_multiplication() {
            let module = parse("MODULE m; CONST foo=1*2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Multiplication, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_division() {
            let module = parse("MODULE m; CONST foo=1/2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Division, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }
        #[test]
        fn parse_addition() {
            let module = parse("MODULE m; CONST foo=1+2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Addition, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_subtraction() {
            let module = parse("MODULE m; CONST foo=1-2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Subtraction, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_and() {
            let module = parse("MODULE m; CONST foo=1 & 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::And, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_or() {
            let module = parse("MODULE m; CONST foo=1 OR 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Or, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_div() {
            let module = parse("MODULE m; CONST foo=1 DIV 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Div, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_mod() {
            let module = parse("MODULE m; CONST foo=1 MOD 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Mod, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_combined_sign_and_addition() {
            let module = parse("MODULE m; CONST foo=+1 + (-2); END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Addition, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Unary { op: UnaryOperation::Plus, operand,  .. } = &**lhs else { panic!("+1"); };
            let Expression::Int { value: 1,  .. } = &**operand else { panic!("1"); };
            let Expression::Unary { op: UnaryOperation::Minus, operand,  .. } = &**rhs else { panic!("-2"); };
            let Expression::Int { value: 2,  .. } = &**operand else { panic!("-2"); };
        }

        #[test]
        fn parse_equal() {
            let module = parse("MODULE m; CONST foo=1 = 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Eq, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_not_equal() {
            let module = parse("MODULE m; CONST foo=1 # 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Neq, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_less_than() {
            let module = parse("MODULE m; CONST foo=1 < 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Lt, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_less_than_equal() {
            let module = parse("MODULE m; CONST foo=1 <= 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Le, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_greater_than() {
            let module = parse("MODULE m; CONST foo=1 > 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Gt, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_greater_than_equal() {
            let module = parse("MODULE m; CONST foo=1 >= 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Ge, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_in_equal() {
            let module = parse("MODULE m; CONST foo=1 IN 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::In, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }

        #[test]
        fn parse_is_equal() {
            let module = parse("MODULE m; CONST foo=1 IS 2; END m .");
            let decls = module.declarations;
            let ConstDeclaration { value, .. } = &decls.const_declarations[0];
            let Expression::Binary { op: BinaryOperation::Is, lhs, rhs,  .. } = value else { panic!("Multiplication"); };
            let Expression::Int { value: 1,  .. } = &**lhs else { panic!("1"); };
            let Expression::Int { value: 2,  .. } = &**rhs else { panic!("2"); };
        }
    }

    mod declarations {
        use super::*;
        use crate::frontend::ast::ConstDeclaration;
        #[test]
        fn parse_const_declaration() {
            let module = parse("MODULE m; CONST foo=1987; END m .");
            let decls = module.declarations;
            let ConstDeclaration { ident, .. } = &decls.const_declarations[0];
            assert_eq!(ident.ident.text, "foo");
        }

        #[test]
        fn parse_const_declarations() {
            let module = parse("MODULE m; CONST foo=1987; bar=24; END m .");
            let decls = module.declarations;
            let ConstDeclaration { ident, .. } = &decls.const_declarations[0];
            assert_eq!(ident.ident.text, "foo");
            let ConstDeclaration { ident, .. } = &decls.const_declarations[1];
            assert_eq!(ident.ident.text, "bar");
        }
    }

    mod types {
        use crate::frontend::ast::{Expression, FPSection, FieldList, FormalType, IdentifierDef, Type, TypeDeclaration};
        use crate::frontend::parser::tests::parse;

        #[test]
        fn parse_named_type() {
            let module = parse("MODULE m; TYPE foo=bar; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Named { name } = ty else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "bar");
        }

        #[test]
        fn parse_single_dimension_array_type() {
            let module = parse("MODULE m; TYPE foo=ARRAY 2 OF bar; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Array { lengths, element, .. } = ty else { panic!("Array type"); };
            assert_eq!(lengths.len(), 1);
            let Expression::Int { value: 2, .. } = &lengths[0] else { panic!("Array Length"); };
            let Type::Named { name } = &**element else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "bar");
        }

        #[test]
        fn parse_multiple_dimensions_array_type() {
            let module = parse("MODULE m; TYPE foo=ARRAY 2, 5 OF bar; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Array { lengths, element, .. } = ty else { panic!("Array type"); };
            assert_eq!(lengths.len(), 2);
            let Expression::Int { value: 2, .. } = &lengths[0] else { panic!("Array Length"); };
            let Expression::Int { value: 5, .. } = &lengths[1] else { panic!("Array Length"); };
            let Type::Named { name } = &**element else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "bar");
        }

        #[test]
        fn parse_simplest_record_type() {
            let module = parse("MODULE m; TYPE foo=RECORD END; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Record { base, field_lists, .. } = ty else { panic!("Record type"); };
            assert!(base.is_none());
            assert!(field_lists.is_empty());
        }

        #[test]
        fn parse_single_field_list_record_type() {
            let module = parse("MODULE m; TYPE foo=RECORD bar*: baz END; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Record { base, field_lists, .. } = ty else { panic!("Record type"); };
            assert!(base.is_none());
            assert_eq!(field_lists.len(), 1);
            let FieldList { fields, ty, .. } = &field_lists[0];
            assert_eq!(fields.len(), 1);
            let IdentifierDef { ident, exported: true, .. } = &fields[0] else { panic!("Field"); };
            assert_eq!(ident.text, "bar");
            let Type::Named { name } = ty else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "baz");
        }

        #[test]
        fn parse_multi_field_lists_record_type() {
            let module = parse("MODULE m; TYPE foo=RECORD bar*: baz; fez, guz: hez END; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Record { base, field_lists, .. } = ty else { panic!("Record type"); };
            assert!(base.is_none());
            assert_eq!(field_lists.len(), 2);
            let FieldList { fields, ty, .. } = &field_lists[0];
            assert_eq!(fields.len(), 1);
            let IdentifierDef { ident, exported: true, .. } = &fields[0] else { panic!("Field"); };
            assert_eq!(ident.text, "bar");
            let Type::Named { name } = ty else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "baz");
            let FieldList { fields, ty, .. } = &field_lists[1];
            assert_eq!(fields.len(), 2);
            let IdentifierDef { ident, exported: false, .. } = &fields[0] else { panic!("Field"); };
            assert_eq!(ident.text, "fez");
            let IdentifierDef { ident, exported: false, .. } = &fields[1] else { panic!("Field"); };
            assert_eq!(ident.text, "guz");
            let Type::Named { name } = ty else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "hez");
        }

        #[test]
        fn parse_pointer_type() {
            let module = parse("MODULE m; TYPE foo=POINTER TO bar; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Pointer { pointee, .. } = ty else { panic!("Pointer type"); };
            let Type::Named { name, .. } = &**pointee else { panic!("Named type"); };
            assert_eq!(name.parts.len(), 1);
            assert_eq!(name.parts[0].text, "bar");
        }

        #[test]
        fn parse_simplest_procedure_type() {
            let module = parse("MODULE m; TYPE foo=PROCEDURE(); END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Procedure { params: None, .. } = ty else { panic!("Procedure type"); };

        }

        #[test]
        fn parse_simple_procedure_with_return_type() {
            let module = parse("MODULE m; TYPE foo=PROCEDURE(): baz; END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Procedure { params: Some(params), .. } = ty else { panic!("Procedure type"); };

            assert_eq!(params.sections.len(), 0);
            assert!(params.return_type.is_some());
            let return_type = params.return_type.clone().unwrap();
            assert_eq!(return_type.parts.len(), 1);
            assert_eq!(return_type.parts[0].text, "baz");
        }

        #[test]
        fn parse_simpel_formal_parameter_procedure_type() {
            let module = parse("MODULE m; TYPE foo=PROCEDURE(bar: baz); END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Procedure { params: Some(params), .. } = ty else { panic!("Procedure type"); };

            assert_eq!(params.sections.len(), 1);
            let FPSection { by_ref: false, names, ty, .. } = &params.sections[0] else { panic!("FPSection"); };
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].text, "bar");
            let FormalType { open_arrays: 0, base, .. } = ty else { panic!("FormalType"); };
            assert_eq!(base.parts.len(), 1);
            assert_eq!(base.parts[0].text, "baz");
        }

        #[test]
        fn parse_multi_parameter_formal_parameter_procedure_type() {
            let module = parse("MODULE m; TYPE foo=PROCEDURE(bar: ARRAY OF ARRAY OF baz; VAR fez, cuz: hez); END m .");
            let decls = module.declarations;
            let TypeDeclaration { ty, .. } = &decls.type_declarations[0];
            let Type::Procedure { params: Some(params), .. } = ty else { panic!("Procedure type"); };

            assert_eq!(params.sections.len(), 2);
            let FPSection { by_ref: false, names, ty, .. } = &params.sections[0] else { panic!("FPSection"); };
            assert_eq!(names.len(), 1);
            assert_eq!(names[0].text, "bar");
            let FormalType { open_arrays: 2, base, .. } = ty else { panic!("Named type"); };
            assert_eq!(base.parts.len(), 1);
            assert_eq!(base.parts[0].text, "baz");

            let FPSection { by_ref: true, names, ty, .. } = &params.sections[1] else { panic!("FPSection"); };
            assert_eq!(names.len(), 2);
            assert_eq!(names[0].text, "fez");
            assert_eq!(names[1].text, "cuz");
            let FormalType { open_arrays: 0, base, .. } = ty else { panic!("Named type"); };
            assert_eq!(base.parts.len(), 1);
            assert_eq!(base.parts[0].text, "hez");
        }
    }
}