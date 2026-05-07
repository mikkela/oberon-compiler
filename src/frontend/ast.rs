use crate::frontend::span::{Span, Spanned};

// --------------------------- MODULES ---------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub name: Identifier,
    pub end_name: Identifier,
    pub declarations: Declarations,
    pub stmts: StatementSequence,
    pub span: Span,
}

// --------------------------- DECLARATIONS ---------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct Declarations {
    pub const_declarations: Vec<ConstDeclaration>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConstDeclaration {
    pub ident: IdentifierDef,
    pub value: Expression,
}

impl Spanned for ConstDeclaration {
    fn span(&self) -> Span {
        Span::new(self.ident.span.start, self.value.span().end)
    }
}

// --------------------------- EXPRESSIONS ---------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub first: Expression,
    pub second: Option<Expression>,
    pub span: Span,
}
impl Spanned for Element {
    fn span(&self) -> Span { self.span }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Int { value: i64, span: Span },
    Real   { value: f64, span: Span },
    String { value: String, span: Span },
    Nil    { span: Span },
    False    { span: Span },
    True    { span: Span },
    Set { elements: Vec<Element>, span: Span },
    Designator { designator: Designator, actual_parameters: Option<Vec<Expression>>, span: Span },
    Unary { op: UnaryOperation, operand: Box<Expression>, span: Span },
    Binary { op: BinaryOperation, lhs: Box<Expression>, rhs: Box<Expression>, span: Span },
}

impl Spanned for Expression {
    fn span(&self) -> Span {
        match self {
            Expression::Int { span, .. } => *span,
            Expression::Real { span, .. } => *span,
            Expression::String { span, .. } => *span,
            Expression::Nil { span, .. } => *span,
            Expression::True { span, .. } => *span,
            Expression::False { span, .. } => *span,
            Expression::Set { span, .. } => *span,
            Expression::Designator { span, .. } => *span,
            Expression::Unary { span, .. } => *span,
            Expression::Binary { span, .. } => *span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOperation {
    Not, Plus, Minus
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOperation {
    Addition, Subtraction, Multiplication, Division, Mod, Div, And, Or,
    Eq, Neq, Lt, Le, Gt, Ge, In, Is
}

// --------------------------- STATEMENTS ---------------------------
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assign { target: Designator, value: Expression, span: Span },
}

impl Spanned for Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Assign { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatementSequence {
    pub statements: Vec<Statement>,
    pub span: Span,
}

impl Spanned for StatementSequence {
    fn span(&self) -> Span { self.span }
}

// --------------------------- DESIGNATORS, SELECTORS, IDENTIFIERS ---------------------------
#[derive(Clone, Debug, PartialEq)]
pub struct Designator {
    pub head: QualifiedIdentifier,
    pub selectors: Vec<Selector>,
    pub span: Span,
}

impl Spanned for Designator {
    fn span(&self) -> Span { self.span }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Selector {
    Field(Identifier),          // .x
    Index(Vec<Expression>, Span),     // [e1, e2]
    Deref(Span),                // ^

    TypeGuard(QualifiedIdentifier, Span), // (ident)
}

impl Spanned for Selector {
    fn span(&self) -> Span {
        match self {
            Selector::Field(id) => id.span(),
            Selector::Index(_, span) => *span,
            Selector::Deref(span) => *span,
            Selector::TypeGuard(_, span) => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedIdentifier {
    pub parts: Vec<Identifier>, // len >= 1
}

impl Spanned for QualifiedIdentifier {
    fn span(&self) -> Span {
        let start = self.parts.first().unwrap().span.start;
        let end = self.parts.last().unwrap().span.end;
        Span::new(start, end)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IdentifierDef {
    pub ident: Identifier,
    pub exported: bool, // star
    pub span: Span,
}

impl Spanned for IdentifierDef {
    fn span(&self) -> Span { self.span }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    pub text: String,
    pub span: Span,
}

impl Spanned for Identifier {
    fn span(&self) -> Span { self.span }
}
