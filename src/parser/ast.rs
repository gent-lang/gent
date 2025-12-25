//! Abstract Syntax Tree definitions for GENT

use crate::Span;

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not, // !
    Neg, // -
}

/// Type names in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Field types for structured output schemas
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array(Box<FieldType>),
    Object(Vec<StructField>),
    Named(String), // reference to a struct
}

/// A field in a struct or inline object type
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: FieldType,
    pub span: Span,
}

/// Struct declaration: `struct Name { fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
    pub span: Span,
}

/// Output type specification (inline or named)
#[derive(Debug, Clone, PartialEq)]
pub enum OutputType {
    Inline(Vec<StructField>),
    Named(String),
}

/// A complete GENT program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    AgentDecl(AgentDecl),
    ToolDecl(ToolDecl),
    StructDecl(StructDecl),
    LetStmt(LetStmt),
}

/// An agent declaration: `agent Name { fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDecl {
    pub name: String,
    pub fields: Vec<AgentField>,
    pub tools: Vec<String>, // Tool names from `use` statements
    pub output: Option<OutputType>,
    pub span: Span,
}

/// A field in an agent declaration: `name: value`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentField {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// Tool declaration: `tool name(params) -> return_type { body }`
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeName>,
    pub body: Block,
    pub span: Span,
}

/// Parameter in a tool declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_name: TypeName,
    pub span: Span,
}

/// An expression in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// String literal: `"hello"`
    String(String, Span),
    /// Number literal: `42` or `3.14`
    Number(f64, Span),
    /// Boolean literal: `true` or `false`
    Boolean(bool, Span),
    /// Identifier reference: `varName`
    Identifier(String, Span),
    /// Null literal: `null`
    Null(Span),
    /// Array literal: `[1, 2, 3]`
    Array(Vec<Expression>, Span),
    /// Object literal: `{key: value}`
    Object(Vec<(String, Expression)>, Span),
    /// Binary operation: `a + b`
    Binary(BinaryOp, Box<Expression>, Box<Expression>, Span),
    /// Unary operation: `-x`, `!x`
    Unary(UnaryOp, Box<Expression>, Span),
    /// Function call: `foo(args)`
    Call(Box<Expression>, Vec<Expression>, Span),
    /// Member access: `obj.prop`
    Member(Box<Expression>, String, Span),
    /// Index access: `arr[0]`
    Index(Box<Expression>, Box<Expression>, Span),
}

impl Expression {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expression::String(_, span) => span,
            Expression::Number(_, span) => span,
            Expression::Boolean(_, span) => span,
            Expression::Identifier(_, span) => span,
            Expression::Null(span) => span,
            Expression::Array(_, span) => span,
            Expression::Object(_, span) => span,
            Expression::Binary(_, _, _, span) => span,
            Expression::Unary(_, _, span) => span,
            Expression::Call(_, _, span) => span,
            Expression::Member(_, _, span) => span,
            Expression::Index(_, _, span) => span,
        }
    }
}

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<BlockStmt>,
    pub span: Span,
}

/// A statement within a block
#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    Let(LetStmt),
    Return(ReturnStmt),
    If(IfStmt),
    Expr(Expression),
}

/// Let statement: `let x = expr`
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// Return statement: `return expr?`
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub span: Span,
}

/// If statement: `if cond { ... } else { ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}
